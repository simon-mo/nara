use async_timer::oneshot::{Oneshot, Timer};
use hyper::Body;
use hyper::{Client, Request};

use crate::reporter::Event;
use std::sync::mpsc as ThreadedMpcs;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct Bencher {
    pub num_requests: usize,
    pub url: String,
    pub delay_for_us: u64,
    pub max_conn: usize,
}

impl Bencher {
    pub async fn bench(self, send_channel: ThreadedMpcs::Sender<Event>) {
        let mut conn = hyper::client::HttpConnector::new();
        conn.set_reuse_address(true);
        conn.set_nodelay(true);

        let client = Client::builder()
            .pool_idle_timeout(None)
            .pool_max_idle_per_host(self.max_conn)
            .build::<_, Body>(conn);

        let mut futures: Vec<tokio::task::JoinHandle<_>> = vec![];

        let sema = Arc::new(Semaphore::new(self.max_conn));

        let query_id = Arc::new(AtomicUsize::new(0));

        for _ in 0..self.num_requests {
            send_channel
                .send(Event::RequestStart)
                .expect("Error sending request start event");
            let uri = self.url.clone();
            let shared_client = client.clone();
            let shared_sema = sema.clone();
            let cloned_channel = send_channel.clone();
            let query_id = query_id.clone();
            let handle = tokio::spawn(async move {
                let bgn_acq = Instant::now();
                let permit = shared_sema.acquire().await;
                cloned_channel
                    .send(Event::WaitForConn(bgn_acq.elapsed()))
                    .expect("Error sending wait for conn profile event");
                let start = Instant::now();

                let req = Request::builder()
                    .method("GET")
                    .uri(uri)
                    .body(Body::empty())
                    .expect("request builder faled");
                let resp = shared_client.request(req).await;
                match resp {
                    Ok(r) => {
                        let _ = hyper::body::to_bytes(r.into_body()).await;
                        cloned_channel
                            .send(Event::RequestDone((query_id.fetch_add(1, Ordering::SeqCst), start, Instant::now())))
                            .expect("Error sending request done event");
                    }
                    Err(err) => {
                        cloned_channel
                            .send(Event::RequestErrored(err))
                            .expect("Error sending request errored event");
                    }
                }
                // Explicitly release the semaphore here.
                drop(permit);
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
