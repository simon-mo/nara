use async_timer::oneshot::{Oneshot, Timer};
use hyper::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

pub struct Bencher {
    pub num_requests: usize,
    pub url: String,
    pub delay_for_us: u64,
}

impl Bencher {
    pub async fn bench(self) {
        let client = Arc::new(Client::new());
        let start_of_the_world = Instant::now();
        let (tx, mut rx) = mpsc::channel::<Instant>(self.num_requests);

        let mut futures: Vec<tokio::task::JoinHandle<_>> = vec![];
        for _ in 0..self.num_requests {
            let uri = self.url.parse().unwrap();
            let shared_client = client.clone();
            let mut tx = tx.clone();

            let handle = tokio::spawn(async move {
                let start = Instant::now();
                let resp = shared_client.get(uri).await.unwrap();
                let end = Instant::now();
                let duration = end - start;
                let ms = duration.as_millis();
                tx.send(start).await;
            });

            let before = Instant::now();
            Timer::new(Duration::from_micros(4300)).await;
            println!("Precison: {} us", (before.elapsed()).as_micros());

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
}
