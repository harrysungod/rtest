use hyper::body;
use hyper::body::Buf;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::cell::RefCell;
use std::thread;
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod request_generated;

async fn handle(
    req: Request<Body>,
    tx: &Sender<Box<flatbuffers::FlatBufferBuilder>>,
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

    let finished_data = builder.finished_data();

    let resp_message = format!("Marshalled to {} bytes\n", finished_data.len());

    Ok(Response::new(resp_message.into()))
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (
        Sender<Box<flatbuffers::FlatBufferBuilder>>,
        Receiver<Box<flatbuffers::FlatBufferBuilder>>,
    ) = mpsc::channel(100);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(|req: Request<Body>| handle(req, &tx)))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
