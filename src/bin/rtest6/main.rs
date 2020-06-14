use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::marker::PhantomData;
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlatBufferBuilder<'fbb> {
    owned_buf: Vec<u8>,
    head: usize,
    _phantom: PhantomData<&'fbb ()>,
}

impl<'fbb> FlatBufferBuilder<'fbb> {
    /// Create a FlatBufferBuilder that is ready for writing.
    pub fn new() -> Self {
        Self::new_with_capacity(0)
    }

    /// Create a FlatBufferBuilder that is ready for writing, with a
    /// ready-to-use capacity of the provided size.
    ///
    /// The maximum valid value is `FLATBUFFERS_MAX_BUFFER_SIZE`.
    pub fn new_with_capacity(size: usize) -> Self {
        FlatBufferBuilder {
            owned_buf: vec![0u8; size],
            head: size,
            _phantom: PhantomData,
        }
    }
}
async fn handle(
    req: Request<Body>,
    mut tx: Sender<Box<FlatBufferBuilder<'_>>>,
) -> Result<Response<Body>, Infallible> {
    let mut builder = Box::new(FlatBufferBuilder::new_with_capacity(4096));

    /*
    let id = builder.create_string("");
    let method = builder.create_string(req.method().as_str());
    let uri = builder.create_string(&req.uri().to_string());
    // figure out how to read raw header bytes
    let headers = builder.create_string("");

    let body = hyper::body::to_bytes::<Body>(req.into_body())
        .await
        .expect("Reading body failed");

    let body = builder.create_vector::<u8>(body.bytes());

    let buf = request_generated::fbr::Request::create(
        &mut builder,
        &request_generated::fbr::RequestArgs {
            id: Some(id),
            method: Some(method),
            body: Some(body),
            headers: Some(headers),
            uri: Some(uri),
        },
    );

    builder.finish(buf, None);
    */

    tx.send(builder).await.expect("Sending failed");

    // let finished_data = builder.finished_data();
    // let resp_message = format!("Marshalled to {} bytes\n", finished_data.len());
    let resp_message = "OK";

    Ok(Response::new(resp_message.into()))
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (
        Sender<Box<FlatBufferBuilder>>,
        Receiver<Box<FlatBufferBuilder>>,
    ) = mpsc::channel(100);
    // let tx = Box::leak(Box::new(tx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let make_svc = make_service_fn(|_conn| {
        // we must clone the 'tx' to be something owned by the closure
        // the new `tx` will be tied to the scope of the closure and not to
        // caller, `main`. This must be outside out `async` block below.
        // that is it must be done *now*, not in future.
        let tx = tx.clone();

        // tx is now a separate clone for each instance of http-connection

        async /* move */ { // move keyword seems optional here - find out why

            // move keyword is very much required in the closure below
            // this function is called for each request. Needs a separate tx clone.
            //
            // `move` keywords moves `tx` to inside closure. without it, 
            // subsequent clones can't be made out of a reference that has disappeared
            //
            // Still a bit confused, but this is all I know at this point.
            // `move` is required here, but why wasn't it required
            // at ..... make_service_fn(|_conn|... closure..
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {

                handle(req, tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
