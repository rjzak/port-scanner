use std::net::{Shutdown, TcpStream};

use anyhow::Result;
use ipnet::IpNet;

pub fn scan(host: &str) -> Result<bool> {
    let stream = match TcpStream::connect(host) {
        Ok(s) => s,
        Err(_) => {
            return Ok(false);
        }
    };
    stream.shutdown(Shutdown::Both).unwrap_or_default();
    Ok(true)
}

pub fn scan_range(range: IpNet) -> Result<Vec<String>> {
    let mut found = vec![];

    for ip in range.hosts() {
        for port in 1..65535u16 {
            let host = format!("{ip}:{port}");
            if scan(&host)? {
                found.push(host);
            }
        }
    }

    Ok(found)
}
