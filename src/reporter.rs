use crate::util::Counter;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

fn count_qps(mut instants: Vec<Instant>) -> HashMap<u64, u64> {
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
            }
        }

        println!("Successful {}/{}", empirical_recv_ts.len(), num_requests);

        if empirical_recv_ts.len() != num_requests as usize {
            println!("Errors");
            println!("{:?}", error_counter.map);
        }

        println!("Sent QPS {:?}", count_qps(empirical_sent_ts));
        println!("Recv QPS {:?}", count_qps(empirical_recv_ts));
    }
}
