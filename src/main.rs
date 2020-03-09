use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

enum Command {
    Increment,
}

fn main() {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(async {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        const BATCH_SIZE: u64 = 2;

        let (cmd_tx, mut cmd_rx) = mpsc::channel::<(Command, oneshot::Sender<u64>)>(100);

        tokio::spawn(async move {
            let mut batch_id: u64 = 0;
            let mut curr_batch_size: u64 = 0;
            let mut waiters: Vec<oneshot::Sender<u64>> = vec![];

            while let Some((cmd, resp)) = cmd_rx.recv().await {
                match cmd {
                    Command::Increment => {
                        curr_batch_size += 1;
                        waiters.push(resp);
                        if curr_batch_size == BATCH_SIZE {
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
            let cmd_tx = cmd_tx.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
                    let mut cmd_tx = cmd_tx.clone();
                    async move {
                        let (req_tx, resp_rx) = oneshot::channel::<u64>();
                        cmd_tx
                            .send((Command::Increment, req_tx))
                            .await
                            .ok()
                            .unwrap();
                        let res = resp_rx.await.unwrap();
                        println!("We are batch {:#?}", res);
                        Ok::<_, Infallible>(Response::new(Body::from("Hello, World")))
                    }
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });
}
