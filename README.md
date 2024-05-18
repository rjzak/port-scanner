## ports-scanner

A simple application and library for scanning a host for open ports.

Host and/or ranges to be given in [CIDR](https://en.wikipedia.org/wiki/Classless_Inter-Domain_Routing) notation:

* Scan a host: `ports-scanner -h 127.0.0.1/32`
* Scan a range: `ports-scanner -h 192.168.0.1/24`

For some found ports, `ports-scanner` may display the name/use of that port.
