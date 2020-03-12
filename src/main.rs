use async_timer::oneshot::{Oneshot, Timer};
use hyper::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot};

use load_generator::bencher::Bencher;
use load_generator::server::Server;

// async fn bench(num_requests: usize) {
//     let client = Arc::new(Client::new());

//     let mut futures: Vec<tokio::task::JoinHandle<_>> = vec![];
//     let start_of_the_world = Instant::now();
//     let (tx, mut rx) = mpsc::channel::<Instant>(9999);

//     // TODO: use channels to implement a sink for sent timestamp
//     //       so we can track the empirical requests per seconds

//     // let mut sent_timestamps: Vec<Instant> = vec![];
//     // let mut before = Instant::now();
//     for _ in 0..num_requests {
//         let uri = "http://localhost:3000".parse().unwrap();
//         // let uri = "http://localhost:8000".parse().unwrap();
//         let shared_client = client.clone();
//         let mut tx = tx.clone();
//         let handle = tokio::spawn(async move {
//             let start = Instant::now();
//             let resp = shared_client.get(uri).await.unwrap();
//             let end = Instant::now();
//             let duration = end - start;
//             let ms = duration.as_millis();
//             tx.send(start).await;
//             // println!("Response: {}, took {} ms", resp.status(), ms);
//         });
//         // handle.await;
//         // sending 10 qps
//         let before = Instant::now();
//         // delay_for(Duration::from_nanos(100)).await;
//         Timer::new(Duration::from_micros(4300)).await;
//         let after = Instant::now();
//         println!("Precison: {} us", (after - before).as_micros());
//         // before = after;
//         futures.push(handle);
//     }

//     drop(tx); // Drop the original tx because it will never be used.

//     for fut in futures {
//         fut.await;
//     }
//     println!("All request done!");
//     let mut empirical_sent_ts: Vec<Instant> = vec![];
//     loop {
//         let val = rx.recv().await;
//         println!("{:?}", val);
//         match val {
//             Some(v) => empirical_sent_ts.push(v),
//             None => break,
//         }
//     }
//     println!("Got requests {}", empirical_sent_ts.len());

//     let mut counter = HashMap::new();
//     for ts in empirical_sent_ts {
//         let sec = ts.duration_since(start_of_the_world).as_secs();
//         if let Some(val) = counter.get_mut(&sec) {
//             *val += 1;
//         } else {
//             counter.insert(sec, 1);
//         }
//     }
//     println!("{:?}", counter);
// }

fn main() {
    let mut rt = Runtime::new().unwrap();

    let mut server = Server {
        local_port: 3000,
        batch_size: 1,
    };

    let mut bencher = Bencher {
        num_requests: 300,
        url: "http://127.0.0.1:3000".to_string(),
        delay_for_us: 1000,
    };

    rt.block_on(async {
        let (req_tx, resp_rx) = oneshot::channel();
        let server_future = tokio::spawn(server.serve(req_tx));
        let signal = resp_rx.await.unwrap();
        println!("signal received, it is {}", signal);
        let bencher_future = tokio::spawn(bencher.bench());
        bencher_future.await;
        drop(server_future);
        // server_future.await;
    });
}
