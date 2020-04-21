use structopt::StructOpt;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;


use nara_core::bencher::Bencher;
use nara_core::reporter::Reporter;
use nara_core::server::Server;


#[derive(Debug, StructOpt)]
#[structopt(name = "nara", about = "Profiler for HTTP and Python Services.")]
struct Opt {
    #[structopt(short = "n", default_value = "1")]
    num_benchers: usize,

    #[structopt(long)]
    url: String,

    #[structopt(long, default_value = "1000")]
    num_requests: usize,

    #[structopt(long, default_value = "1000")]
    delay_us: u64,

    #[structopt(long, default_value = "5")]
    max_conn: usize,
}

fn main() {
    env_logger::init();
    let mut opt = Opt::from_args();
    println!("Configuration: {:?}", opt);

    let mut start_sever = false;
    if opt.url.eq_ignore_ascii_case("local") {
        println!("Using bundled server");
        start_sever = true;
        opt.url = "http://127.0.0.1:3000".to_string();
    } else {
        assert!(
            opt.url.starts_with("http://"),
            "url doesn't start with http:// !"
        );
    }

    let mut rt = Runtime::new().unwrap();

    let mut benchers: Vec<Bencher> = vec![];
    for _ in 0..opt.num_benchers {
        let url = opt.url.clone();
        benchers.push(Bencher {
            num_requests: opt.num_requests,
            url,
            delay_for_us: opt.delay_us,
            max_conn: opt.max_conn,
        });
    }

    rt.block_on(async {
        let mut server_future: Option<JoinHandle<()>> = None;
        if start_sever {
            let server = Server {
                local_port: 3000,
                batch_size: 1,
            };
            let (req_tx, resp_rx) = oneshot::channel();
            server_future = Some(tokio::spawn(async move {
                server.serve(req_tx).await;
            }));
            let _ = resp_rx.await.unwrap();
        }

        let (send_chan, recv_chan) = std::sync::mpsc::channel();

        let reporter_thread = std::thread::spawn(move || {
            let mut reporter = Reporter {
                receiver: recv_chan,
            };
            reporter.start(opt.num_requests as u64 * opt.num_benchers as u64);
        });

        let mut bencher_handles: Vec<JoinHandle<()>> = vec![];
        for bencher in benchers {
            let cloned_chan = send_chan.clone();
            bencher_handles.push(tokio::spawn(
                async move { bencher.bench(cloned_chan).await },
            ));
        }
        drop(send_chan);
        for handle in bencher_handles {
            handle.await.expect("Errored benching");
        }

        if start_sever {
            drop(server_future.unwrap());
        }

        reporter_thread
            .join()
            .expect("Error joining reporter thread");
    });
}
