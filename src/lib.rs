use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};

use ipnet::IpNet;
use lazy_static::lazy_static;

pub const PORT_MAX: u16 = 65535u16; // (2^16)-1

lazy_static! {
    /// Source: https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers
    pub static ref PORTS: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(20, "ftp data");
        m.insert(21, "ftp control");
        m.insert(22, "secure shell (ssh)");
        m.insert(23, "Telnet");
        m.insert(25, "Simple Mail Transfer Protocol (SMTP)");
        m.insert(43, "whois");
        m.insert(53, "Domain Name System (DNS)");
        m.insert(
            67,
            "Bootstrap Protocol (BOOTP) or Dynamic Host Configuration Protocol (DHCP)",
        );
        m.insert(
            68,
            "Bootstrap Protocol (BOOTP) or Dynamic Host Configuration Protocol (DHCP)",
        );
        m.insert(69, "Trivial File Transfer Protocol (TFTP)");
        m.insert(70, "Gopher");
        m.insert(79, "Finger");
        m.insert(80, "http");
        m.insert(88, "Kerberos authentication system");
        m.insert(109, "Post Office Protocol (POP) version 2");
        m.insert(110, "Post Office Protocol (POP) version 3");
        m.insert(113, "Ident/Authentication protocol");
        m.insert(115, "Simple File Transfer Protocol (SMTP)");
        m.insert(119, "Network News Transfer Protocol (NNTP)");
        m.insert(123, "Network Time Protocol (NTP)");
        m.insert(137, "NetBIOS Name Service");
        m.insert(138, "NetBIOS Datagram Service");
        m.insert(139, "NetBIOS Session Service");
        m.insert(143, "Internet Message Access Protocol (IMAP)");
        m.insert(161, "Simple Network Management Protocol (SNMP)");
        m.insert(162, "Simple Network Management Protocol (SNMP) Trap Service (SNMPTRAP)");
        m.insert(177, "X Display Manager Control Protocol (XDMCP)");
        m.insert(179, "Border Gateway Protocol (BGP)");
        m.insert(194, "Internet Relay Chat (IRC)");
        m.insert(389, "Lightweight Directory Access Protocol (LDAP)");
        m.insert(443, "https");
        m.insert(445, "Microsoft-DS Directory Services/SAMBA (SMB)");
        m.insert(456, "Simple File Transfer Protocol (SMTP) over SSL");
        m.insert(510, "FirstClass Unified Communications");
        m.insert(514, "Remote Shell or syslog");
        m.insert(546, "Dynamic Host Configuration Protocol (DHCP) IPv6 (DHCPv6) client");
        m.insert(547, "Dynamic Host Configuration Protocol (DHCP) IPv6 (DHCPv6) server");
        m.insert(548, "Apple Filing Protocol (AFP) over TCP");
        m.insert(631, "Internet Printing Protocol (IPP), Common Unix Printing System (CUPS) admin console");
        m.insert(636, "LDAP over SSL (LDAPS)");
        m.insert(666, "DOOM");
        m.insert(853, "DNS over TLS");
        m.insert(873, "rsync");
        m.insert(989, "FTP data over TLS");
        m.insert(990, "FTP control over TLS");
        m.insert(992, "Telnet over TLS");
        m.insert(993, "Internet Message Access Protocol (IMAP) over TLS (IMAPS)");
        m.insert(995, "Post Office Protocol (POP) version 3 over TLS");
        m
    };
}

pub fn scan(host: &str) -> bool {
    let stream = match TcpStream::connect(host) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };
    stream.shutdown(Shutdown::Both).unwrap_or_default();
    true
}

pub fn scan_range(range: IpNet) -> Vec<String> {
    let mut found = vec![];

    for ip in range.hosts() {
        for port in 1..PORT_MAX {
            let host = format!("{ip}:{port}");
            if scan(&host) {
                found.push(host);
            }
        }
    }

    found
}

#[cfg(test)]
mod tests {
    use super::scan_range;

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::str::FromStr;
    use std::thread;

    use ipnet::IpNet;
    use rand::Rng;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test() {
        let mut rng = rand::thread_rng();

        let port = rng.gen_range(1025..1040) as u16;
        let expected = format!("127.0.0.1:{port}");

        let server = tokio::spawn(async move {
            let listener = TcpListener::bind(expected).unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        thread::spawn(move || {
                            loop {
                                let mut read = [0; 1028];
                                match stream.read(&mut read) {
                                    Ok(n) => {
                                        if n == 0 {
                                            // connection was closed
                                            break;
                                        }
                                        stream.write_all(&read[0..n]).unwrap();
                                    }
                                    Err(err) => {
                                        eprintln!("{err}");
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(_) => {
                        println!("Error");
                        break;
                    }
                }
            }
        });
        assert!(!server.is_finished());
        let expected = format!("127.0.0.1:{port}");
        eprintln!("Scanning localhost.");

        let found = scan_range(IpNet::from_str("127.0.0.1/32").unwrap());
        let found = found
            .iter()
            .filter(|&h| *h == expected)
            .clone()
            .collect::<Vec<&String>>();

        server.abort();

        assert_eq!(found.len(), 1);
    }
}
