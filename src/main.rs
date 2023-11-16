use clap::Parser;
use ipnet::IpNet;
use std::net::{Shutdown, TcpStream};

/// Simple port scanner
#[derive(Parser)]
struct Args {
    /// IP addresses to scan
    pub net: IpNet,
}
fn main() {
    let args = Args::parse();

    for ip in args.net.hosts() {
        for port in 1..65535u16 {
            let server = format!("{ip}:{port}");
            let stream = match TcpStream::connect(&server) {
                Ok(s) => s,
                Err(_) => {
                    continue;
                }
            };
            println!("{server}");
            stream.shutdown(Shutdown::Both).unwrap_or_default();
        }
    }
}
