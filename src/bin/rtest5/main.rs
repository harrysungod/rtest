use crossbeam::channel::{Receiver, Sender};
use flatbuffers::FlatBufferBuilder;
use hyper::body;
use hyper::body::Buf;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lz4;
use object_pool::Pool;
use std::io::Write;
use std::sync::mpsc;
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::fs;
use tokio::io::AsyncWrite;
use tokio::task;

#[macro_use]
extern crate lazy_static;

mod request_generated;

/*
lazy_static! {
    static ref POOL: Arc<Pool<Box<flatbuffers::FlatBufferBuilder<'static>>>> =
        Arc::new(Pool::new(1000, || {
            Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096))
        }));
}
*/

async fn handle(
    req: Request<Body>,
    mut tx: Sender<Vec<u8>>,
    // pool: Arc<Pool<Box<flatbuffers::FlatBufferBuilder<'static>>>>,
) -> Result<Response<Body>, Infallible> {
    let mut builder = Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096));
    // let mut builder = POOL.try_pull().expect("unable to get item from pool");
    //builder.reset();

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
    let finished_bytes_vec = builder.finished_data().to_vec().clone();
    tx.send(finished_bytes_vec)
        .expect("unable to write to channel");

    /*
    let finished_data = builder.finished_data();
    let resp_message = format!("Marshalled to {} bytes\n", finished_data.len());
    */
    let resp_message = "OK\n";
    Ok(Response::new(resp_message.into()))
}

fn recorder(file_name: String, mut rx: Receiver<Vec<u8>>) {
    println!("Starting recorder");

    // let mut file = std::fs::File::create(file_name).expect("Unable to create file");

    let mut file = std::fs::File::create(file_name).expect("Unable to create file");

    let mut encoder = lz4::EncoderBuilder::new()
        .level(0)
        .build(file)
        .expect("Unable to init lz4");

    let mut total_received: i32 = 0;
    let mut total_size = 0;

    while let Ok(finished_data) = rx.recv() {
        encoder
            .write(finished_data.as_slice())
            .expect("write failed");

        total_received += 1;
        total_size += finished_data.len();
        if total_received % 100000 == 0 {
            println!("Saved {} requests", total_received);
        }
        if total_size % 10000000 == 0 {
            println!("Saved {} bytes", total_size);
        }
    }

    println!("Recorder thread finished");
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = crossbeam::channel::bounded(100);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    for i in 0..5 {
        let rx = rx.clone();
        let file_name = format!("foo{}.data", i);
        task::spawn_blocking(move || recorder(file_name, rx));
    }

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let make_svc = make_service_fn(|_conn| {
        // we must clone the 'tx' to be something owned by the closure
        // the new `tx` will be tied to the scope of the closure and not to
        // caller, `main`. This must be outside out `async` block below.
        // that is it must be done *now*, not in future.
        let tx = tx.clone();
        // tx is now a separate clone for each instance of http-connection
        //let pool: Arc<Pool<Box<flatbuffers::FlatBufferBuilder<'_>>>> =
        //    Arc::new(Pool::new(100, || {
        //        Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096))
        //    }));
        async /* move */ { // move keyword seems optional here - find out why

            // move keyword is very much required in the closure below
            // this function is called for each request. Needs a separate tx clone.
            //
            // `move` keywords moves `tx` to inside closure. without it, 
            // subsequent clones can't be made out of a reference that has disappeared
            //
            // Still a bit confused, but this is all I know at this point.
            // `move` is required here, but why wasn't it required
            // at ..... make_service_fn(|_conn|... closure..above
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {

                // handle(req, tx.clone(), pool.clone())
                handle(req, tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
