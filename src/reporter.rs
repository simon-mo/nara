use crate::util::Counter;
use hdrhistogram::Histogram;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn count_qps(mut instants: Vec<Instant>) -> BTreeMap<u64, u64> {
    instants.sort();
    let start = instants[0];
    let mut counter = Counter::new();
    for ts in instants {
        counter.record(ts.duration_since(start).as_secs());
    }
    return counter.map;
}

type Message = (Instant, Instant);

pub enum Event {
    RequestStart,
    RequestDone(Message),
    RequestErrored(hyper::Error),
    TrialDone,
    WaitForConn(Duration),
}

pub struct Reporter {
    pub receiver: mpsc::Receiver<Event>,
}

impl Reporter {
    pub fn start(&mut self, num_requests: u64) {
        let sty = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");

        let sent_progress = ProgressBar::new(num_requests);
        sent_progress.set_style(sty.clone());
        sent_progress.set_message("Sending Requests");

        let mut empirical_sent_ts: Vec<Instant> = vec![];
        let mut empirical_recv_ts: Vec<Instant> = vec![];
        let mut error_counter = Counter::new();
        let mut wait_time_hist = Histogram::<u64>::new(3).unwrap();

        while let Ok(message) = self.receiver.recv() {
            match message {
                Event::RequestStart => sent_progress.inc(1),
                Event::RequestDone((begin_instant, end_instant)) => {
                    empirical_sent_ts.push(begin_instant);
                    empirical_recv_ts.push(end_instant);
                }
                Event::RequestErrored(error) => {
                    error_counter.record(format!("{}", error));
                }
                Event::TrialDone => sent_progress.finish(),
                Event::WaitForConn(duration) => {
                    wait_time_hist.record(duration.as_micros() as u64).unwrap();
                }
            }
        }

        println!("Successful {}/{}", empirical_recv_ts.len(), num_requests);

        if empirical_recv_ts.len() != num_requests as usize {
            println!("Errors");
            println!("{:?}", error_counter.map);
        }

        println!("Wait Time Percentiles");
        for perc in &[0.1, 0.5, 0.9, 0.95, 0.99] {
            println!(
                "{} Percentile: {}",
                perc,
                wait_time_hist.value_at_quantile(*perc)
            )
        }

        println!("Sent QPS {:?}", count_qps(empirical_sent_ts));
        println!("Recv QPS {:?}", count_qps(empirical_recv_ts));
    }
}
