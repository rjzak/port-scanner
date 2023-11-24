use std::fmt::Display;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{fmt, thread};

use clap::{Parser, ValueHint};
use ipnet::IpNet;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Host {
    pub host: String,
    pub port: u16,
}

impl Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

/// Simple port scanner
#[derive(Parser)]
#[command(author, about, version)]
struct Args {
    /// Maximum number of threads, default is the number of CPUs on the system
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// IP addresses to scan
    #[arg(short, long)]
    pub net: Option<IpNet>,

    /// Path to list of hosts
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub list: Option<PathBuf>,
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    if args.net.is_none() && args.list.is_none() {
        eprintln!("Must include list of hosts or IP range, run with `-h` for details.");
        return Ok(ExitCode::FAILURE);
    }

    let (sender, receiver) = crossbeam_channel::bounded(args.threads);
    let mut threads = Vec::with_capacity(args.threads);

    for _ in 0..args.threads {
        let recr = receiver.clone();
        threads.push(thread::spawn(move || loop {
            if recr.is_empty() {
                break;
            }
            let server: String = recr.recv().unwrap();
            if let Ok(found) = port_scanner::scan(&server) {
                if found {
                    println!("{server}");
                }
            }
        }));
    }

    if let Some(ips) = args.net {
        for ip in ips.hosts() {
            for port in 1..65535u16 {
                let server = format!("{ip}:{port}");
                sender.send(server)?;
            }
        }
    }

    if let Some(host_file) = args.list {
        let file = File::open(host_file)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.deserialize() {
            let host: Host = result?;
            sender.send(host.to_string())?;
        }
    }

    for thread in threads {
        thread.join().expect("failed to join() thread");
    }

    Ok(ExitCode::SUCCESS)
}
