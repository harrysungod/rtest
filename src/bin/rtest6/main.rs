use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod request_generated;

async fn handle(req: Request<Body>, mut tx: Sender<Vec<u8>>) -> Result<Response<Body>, Error> {
    let test_data = vec![1, 2, 3];

    tx.send(test_data)
        .await
        .expect("unable to write to channel");

    let resp_message = "OK\n";
    Ok(Response::new(resp_message.into()))
}

async fn receiver(mut rx: Receiver<Vec<u8>>) {
    println!("Starting receiver");

    while let Some(finished_data) = rx.recv().await {
        println!("Got {} bytes", finished_data.len());
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(100);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tokio::spawn(async move { receiver(rx).await });

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
            Ok::<_, Error>(service_fn(move |req: Request<Body>| {

                handle(req, tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
