use async_timer::oneshot::{Oneshot, Timer};
use hyper::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

use std::time::Duration;

// async fn serve(started_event: oneshot::Sender<bool>) {
//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

//     const BATCH_SIZE: u64 = 1;

//     let (cmd_tx, mut cmd_rx) = mpsc::channel::<(Command, oneshot::Sender<u64>)>(1000);

//     tokio::spawn(async move {
//         let mut batch_id: u64 = 0;
//         let mut curr_batch_size: u64 = 0;
//         let mut waiters: Vec<oneshot::Sender<u64>> = vec![];

//         while let Some((cmd, resp)) = cmd_rx.recv().await {
//             match cmd {
//                 Command::Increment => {
//                     curr_batch_size += 1;
//                     waiters.push(resp);
//                     if curr_batch_size == BATCH_SIZE {
//                         while let Some(w) = waiters.pop() {
//                             w.send(batch_id).unwrap();
//                         }
//                         batch_id += 1;
//                         curr_batch_size = 0;
//                     }
//                 }
//             }
//         }
//     });

//     let make_svc = make_service_fn(|_conn| {
//         // let cmd_tx = cmd_tx.clone();
//         async move {
//             Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
//                 // let mut cmd_tx = cmd_tx.clone();
//                 async move {
//                     // let (req_tx, resp_rx) = oneshot::channel::<u64>();
//                     // cmd_tx
//                     //     .send((Command::Increment, req_tx))
//                     //     .await
//                     //     .ok()
//                     //     .unwrap();
//                     // let res = resp_rx.await.unwrap();
//                     // println!("We are batch {:#?}", res);
//                     Ok::<_, Infallible>(Response::new(Body::from("Hello, World")))
//                 }
//             }))
//         }
//     });

//     let server = Server::bind(&addr).serve(make_svc);

//     started_event.send(true);
//     if let Err(e) = server.await {
//         eprintln!("server error: {}", e);
//     }
// }

async fn bench(num_requests: usize) {
    // Still inside `async fn main`...
    let client = Arc::new(Client::new());

    // Parse an `http::Uri`...

    // Await the response...
    let mut futures: Vec<tokio::task::JoinHandle<_>> = vec![];
    let start_of_the_world = Instant::now();
    let (tx, mut rx) = mpsc::channel::<Instant>(9999);

    // TODO: use channels to implement a sink for sent timestamp
    //       so we can track the empirical requests per seconds

    // let mut sent_timestamps: Vec<Instant> = vec![];
    // let mut before = Instant::now();
    for _ in 0..num_requests {
        let uri = "http://localhost:3000".parse().unwrap();
        // let uri = "http://localhost:8000".parse().unwrap();
        let shared_client = client.clone();
        let mut tx = tx.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let resp = shared_client.get(uri).await.unwrap();
            let end = Instant::now();
            let duration = end - start;
            let ms = duration.as_millis();
            tx.send(start).await;
            // println!("Response: {}, took {} ms", resp.status(), ms);
        });
        // handle.await;
        // sending 10 qps
        let before = Instant::now();
        // delay_for(Duration::from_nanos(100)).await;
        Timer::new(Duration::from_micros(4800)).await;
        let after = Instant::now();
        println!("Precison: {} us", (after - before).as_micros());
        // before = after;
        futures.push(handle);
    }

    drop(tx); // Drop the original tx because it will never be used.

    for fut in futures {
        fut.await;
    }
    println!("All request done!");
    let mut empirical_sent_ts: Vec<Instant> = vec![];
    loop {
        let val = rx.recv().await;
        println!("{:?}", val);
        match val {
            Some(v) => empirical_sent_ts.push(v),
            None => break,
        }
    }
    println!("Got requests {}", empirical_sent_ts.len());

    let mut counter = HashMap::new();
    for ts in empirical_sent_ts {
        let sec = ts.duration_since(start_of_the_world).as_secs();
        if let Some(val) = counter.get_mut(&sec) {
            *val += 1;
        } else {
            counter.insert(sec, 1);
        }
    }
    println!("{:?}", counter);
}

fn main() {
    let mut rt = Runtime::new().unwrap();

    let mut server = ::server::Server{}

    rt.block_on(async {
        let (req_tx, resp_rx) = oneshot::channel();
        let server_future = tokio::spawn(server.serve(req_tx));
        let signal = resp_rx.await.unwrap();
        println!("signal received, it is {}", signal);
        let bencher_future = tokio::spawn(bench(2000));
        bencher_future.await;
        drop(server_future);
        // server_future.await;
    });
}
