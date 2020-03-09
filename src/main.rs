use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
// use std::time::Duration;
// use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};
// use tokio::time::delay_for;

enum Command {
    Increment,
    // Other commands can be added here
}

#[tokio::main]
async fn main() {
    // let mut rt = Runtime::new().unwrap();

    // rt.block_on(async {

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<(Command, oneshot::Sender<u64>)>(100);

    const batch_size: u64 = 2;

    tokio::spawn(async move {
        let mut batch_id: u64 = 0;
        let mut curr_batch_size: u64 = 0;
        let mut waiters: Vec<oneshot::Sender<u64>> = vec![];

        while let Some((cmd, resp)) = cmd_rx.recv().await {
            match cmd {
                Command::Increment => {
                    curr_batch_size += 1;
                    waiters.push(resp);
                    if curr_batch_size == batch_size {
                        while let Some(w) = waiters.pop() {
                            w.send(batch_id).unwrap();
                        }
                        batch_id += 1;
                        curr_batch_size = 0;
                    }
                }
            }
        }
    });

    let make_svc = make_service_fn(|_conn| {
        let mut cmd_tx = cmd_tx.clone();

        async move {
            let (req_tx, resp_rx) = oneshot::channel::<u64>();
            cmd_tx
                .send((Command::Increment, req_tx))
                .await
                .ok()
                .unwrap();
            let res = resp_rx.await.unwrap();
            println!("res {:#?}", res);
            Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
                async move { Ok::<_, Infallible>(Response::new(Body::from("Hello, World"))) }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    // tokio::spawn(server);

    // delay_for(Duration::from_secs(100)).await;
    // })

    // .await;
    // println!("Slept 100s");

    // Run this server for... forever!

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
