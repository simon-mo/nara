use async_timer::oneshot::{Oneshot, Timer};
use hyper::Body;
use hyper::Client;

use crate::reporter::Event;
use std::sync::mpsc as ThreadedMpcs;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone)]
pub struct Bencher {
    pub num_requests: usize,
    pub url: String,
    pub delay_for_us: u64,
}

impl Bencher {
    pub async fn bench(self, send_channel: ThreadedMpcs::Sender<Event>) {
        let mut conn = hyper::client::HttpConnector::new();
        conn.set_reuse_address(true).set_nodelay(true);

        let client = Arc::new(
            // Client::new()
            Client::builder()
                .pool_max_idle_per_host(0)
                .build::<_, Body>(conn),
        );

        let mut futures: Vec<tokio::task::JoinHandle<_>> = vec![];

        for _ in 0..self.num_requests {
            send_channel
                .send(Event::RequestStart)
                .expect("Error sending request start event");
            let uri = self.url.parse().unwrap();
            let shared_client = client.clone();

            let cloned_channel = send_channel.clone();
            let handle = tokio::spawn(async move {
                let start = Instant::now();
                let resp = shared_client.get(uri).await;
                match resp {
                    Ok(_) => {
                        cloned_channel
                            .send(Event::RequestDone((start, Instant::now())))
                            .expect("Error sending request done event");
                    }
                    Err(err) => {
                        cloned_channel
                            .send(Event::RequestErrored(err))
                            .expect("Error sending request errored event");
                    }
                }
            });

            Timer::new(Duration::from_micros(self.delay_for_us)).await;

            futures.push(handle);
        }

        for fut in futures {
            fut.await.expect("Single request panicked.");
        }
        send_channel
            .send(Event::TrialDone)
            .expect("Error sending trial done event");
    }
}
