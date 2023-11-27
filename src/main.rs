use std::fmt::Display;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{fmt, thread};

use ports_scanner::{PORTS, PORT_MAX};

use clap::{Parser, ValueHint};
use ipnet::IpNet;
use serde::Deserialize;

/// It's expected this will be read from a CSV, and the `csv` crate expects headers.
/// Example:
/// ```
/// host,port
/// 127.0.0.1,22
/// 127.0,0,1,80
/// ```
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

    let (sender, receiver) = crossbeam_channel::bounded::<(String, u16)>(args.threads.max(1));
    let mut threads = Vec::with_capacity(args.threads);

    for _ in 0..args.threads.max(1) {
        let recr = receiver.clone();
        threads.push(thread::spawn(move || loop {
            if recr.is_empty() {
                break;
            }
            let (server, port) = recr.recv().unwrap();
            if ports_scanner::scan(&server) {
                if let Some(service) = PORTS.get(&port) {
                    println!("{server} – {service}");
                } else {
                    println!("{server}");
                }
            }
        }));
    }

    if let Some(ips) = args.net {
        for ip in ips.hosts() {
            for port in 1..PORT_MAX {
                let server = format!("{ip}:{port}");
                sender.send((server, port))?;
            }
        }
    }

    if let Some(host_file) = args.list {
        let file = File::open(host_file)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.deserialize() {
            let host: Host = result?;
            sender.send((host.to_string(), host.port))?;
        }
    }

    for thread in threads {
        thread.join().expect("failed to join() thread");
    }

    Ok(ExitCode::SUCCESS)
}
