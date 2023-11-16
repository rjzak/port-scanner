use clap::Parser;
use ipnet::IpNet;
use std::net::{Shutdown, TcpStream};
use std::thread;

/// Simple port scanner
#[derive(Parser)]
struct Args {
    /// Maximum number of threads, default is the number of CPUs on the system
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// IP addresses to scan
    pub net: IpNet,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (sender, receiver) = crossbeam_channel::bounded(args.threads);
    let mut threads = Vec::with_capacity(args.threads);

    for _ in 0..args.threads {
        let recr = receiver.clone();
        threads.push(thread::spawn(move || loop {
            if recr.is_empty() {
                break;
            }
            let server = recr.recv().unwrap();
            let stream = match TcpStream::connect(&server) {
                Ok(s) => s,
                Err(_) => {
                    continue;
                }
            };
            println!("{server}");
            stream.shutdown(Shutdown::Both).unwrap_or_default();
        }));
    }

    for ip in args.net.hosts() {
        for port in 1..65535u16 {
            let server = format!("{ip}:{port}");
            sender.send(server)?;
        }
    }

    for thread in threads {
        thread.join().expect("failed to join() thread");
    }

    Ok(())
}
