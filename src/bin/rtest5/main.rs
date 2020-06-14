use flatbuffers::FlatBufferBuilder;
use hyper::body;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::{convert::Infallible, net::SocketAddr};
use tokio::fs;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod request_generated;

use hyper::body::Buf;
use tokio::io::AsyncWriteExt;

async fn handle(
    req: Request<Body>,
    mut tx: Sender<Box<flatbuffers::FlatBufferBuilder<'_>>>,
) -> Result<Response<Body>, Infallible> {
    let mut builder = Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096));

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

    tx.send(builder).await.expect("unable to write to channel");

    /*
    let finished_data = builder.finished_data();
    let resp_message = format!("Marshalled to {} bytes\n", finished_data.len());
    */
    let resp_message = "OK";
    Ok(Response::new(resp_message.into()))
}

async fn recorder(file_name: String, mut rx: Receiver<Box<flatbuffers::FlatBufferBuilder<'_>>>) {
    println!("Starting recorder");

    let mut file = tokio::fs::File::create(file_name)
        .await
        .expect("Unable to create file");

    let mut total_received = 0;
    let mut total_size = 0;

    while let Some(builder) = rx.recv().await {
        let raw_bytes = builder.finished_data();
        file.write(raw_bytes).await.expect("write failed");
        total_received += 1;
        total_size += raw_bytes.len();
        if total_received % 1000 == 0 {
            println!("Saved {} requests", total_received);
        }
        if total_size % 100000 == 0 {
            println!("Saved {} bytes", total_size);
        }
    }

    println!("Sender queueing finished");
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (
        Sender<Box<FlatBufferBuilder>>,
        Receiver<Box<FlatBufferBuilder>>,
    ) = mpsc::channel(100);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tokio::spawn(async move { recorder(String::from("foo.data"), rx) });

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

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
            // `move` is required here, it won't compile without it (even if you
            // add `move` to async block-start above, but why wasn't it required
            // at ..... make_service_fn(|_conn|... closure..above?
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
