---
uid: N:System.Networking
---
## Addresses and name resolution

IPAddress, IPv4Address and IPv6Address model addresses, while Dns performs supported
host-backed name resolution. NetworkDeadline represents the deadline used by
network operations. DnsError and address errors distinguish expected failures.

Address representation and resolution are separate from opening a connection.
See [networking](/features/networking/) and System.Networking.Sockets for the
current IPv4 TCP transport scope; an IPv6 value does not imply IPv6 socket support.
