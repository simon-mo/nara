use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server as HyperServer};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

enum Command {
    Increment,
}

pub struct Server {
    pub local_port: u16,
    pub batch_size: u64,
}

impl Server {
    async fn batcher_task(
        self: Arc<Server>,
        mut batcher_receiver: mpsc::UnboundedReceiver<(Command, oneshot::Sender<u64>)>,
    ) {
        let mut batch_id: u64 = 0;
        let mut curr_batch_size: u64 = 0;
        let mut waiters: Vec<oneshot::Sender<u64>> = vec![];

        while let Some((cmd, resp)) = batcher_receiver.recv().await {
            match cmd {
                Command::Increment => {
                    curr_batch_size += 1;
                    waiters.push(resp);
                    if curr_batch_size == self.batch_size {
                        while let Some(w) = waiters.pop() {
                            w.send(batch_id).unwrap();
                        }
                        batch_id += 1;
                        curr_batch_size = 0;
                    }
                }
            }
        }
    }

    pub async fn serve(self, notify_event: oneshot::Sender<bool>) {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.local_port));

        let (batcher_sender, batcher_receiver) =
            mpsc::unbounded_channel::<(Command, oneshot::Sender<u64>)>();

        let self_ref = Arc::new(self);
        let cloned_ref = self_ref.clone();
        let handle = tokio::spawn(async move { cloned_ref.batcher_task(batcher_receiver).await });

        let make_svc = make_service_fn(|_conn| {
            let batcher_sender = batcher_sender.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
                    let batcher_sender = batcher_sender.clone();
                    async move {
                        let (req_tx, resp_rx) = oneshot::channel::<u64>();
                        batcher_sender.send((Command::Increment, req_tx));
                        let res = resp_rx.await.unwrap();
                        println!("We are batch {:#?}", res);
                        Ok::<_, Infallible>(Response::new(Body::from("Hello, World")))
                    }
                }))
            }
        });

        let server = HyperServer::bind(&addr).serve(make_svc);
        notify_event.send(true);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
        drop(handle);
    }
}
