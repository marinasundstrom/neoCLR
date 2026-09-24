# IP address values — 2026-09-25 development slice

The author selected a closed class hierarchy after initially requesting a value
union. `System.Networking.IPAddress` is the abstract closed root; `IPv4Address` and
`IPv6Address` are immutable sealed implementations. `Parse` constructs the family;
there are no public byte-array constructors in this first iteration.

## Contract and comparison

The baseline is [.NET 10 IPAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress?view=net-10.0),
a single class with address-family-dependent behavior. neoCLR keeps familiar reference
identity, Object equality, hashing and formatting, while exposing the families as
separate types. Typed equality takes a non-null IPAddress; Object equality accepts
null and returns false. Equal byte values in the same family have equal hashes.
An IPv4-mapped IPv6 address remains IPv6 and is unequal to its IPv4 counterpart.

A value union would copy its representation; a closed interface implemented by
structs would box at the shared boundary. Neither is selected. Reference instances
and privately copied backing arrays allocate; numeric fields could reduce that
storage later, but this change makes no performance claim. Public members cannot
mutate an address or obtain its backing array. Compiler and importer closure checks
prevent a consumer from adding mutable family implementations.

`Parse(string)` returns `Result<IPAddress, IPAddressError>` and performs no I/O.
The normal Raven error union contains InvalidFormat and UnsupportedScope. Compared
with [.NET TryParse](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress.tryparse?view=net-10.0),
this version deliberately rejects abbreviated/hex/octal IPv4 and leading zeros.
It accepts exactly four decimal octets. This reduces ambiguous interpretation at
network boundaries at the cost of rejecting some .NET inputs. URI brackets are
not part of raw address parsing. IPv6 scope identifiers are explicitly unsupported;
a percent sign returns UnsupportedScope. Scope representation/equality remain future
work, not an implicit decision to discard scopes.

IPv6 accepts full or compressed groups and a final dotted IPv4 suffix, following
[RFC 4291 section 2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2).
Formatting follows [RFC 5952](https://www.rfc-editor.org/rfc/rfc5952.html): lowercase,
longest zero run, earliest on ties, no single-zero compression, and mixed dotted
format for IPv4-mapped addresses. Parsing is bounded to 45 UTF-8 bytes. These primary
references were reviewed on 2026-09-25; the executable .NET 10 comparison records
accepted forms and intentional differences, without claiming .NET ABI compatibility.

## DNS and transport

Development `Dns.GetHostAddresses` now returns
`Task<Result<Sequence<IPAddress>, DnsError>>`. The native resolver still supplies
bounded IPv4 numeric strings; the managed completion validates them and creates
address values. An invalid native result becomes LookupFailed. This preserves the
existing native resolver, deadlines, ownership and completion machinery.

Socket gains Connect(IPAddress, port), Connect(Sequence<IPAddress>, port) and
Listen(IPAddress, port, backlog). String overloads remain. The typed sequence has
the same 1–16 address bound and is converted to the existing owned string snapshot
before returning. An IPv6 member rejects the entire sequence before I/O with
SocketError.UnsupportedAddressFamily; it is not silently skipped. IPv6 parsing does
not add IPv6 transport or DNS answers. HttpClient's internal DNS stage consumes
the new address values while retaining the same shared deadline.

This is a development API migration: rebuild reference, library and callers together,
replace DNS Sequence<string> annotations with Sequence<IPAddress>, and use ToString
when text is actually required. Cancellation, HTTPS, endpoint values and HostEntry
are separate slices.

## Integration and evidence

The core reference projects the permitted family; library admission validates the
abstract root, sealed leaves, inheritance, closed-family marker and member signatures.
Guest imports reject external derivation from any core address type. The bridge uses
the protected direct-base constructor allowance from the previous checkpoint.
No Raven emission policy or Runtime Contract configuration changed.

The [address verifier](experiments/ip-address-hierarchy/README.md) exercises parsing,
canonical round trips, family distinctions, typed/Object equality, hashing, GC,
closed-family rejection and transport limits. Signature checks include forged
arguments, a missing family marker and external metadata inheritance. The
[socket client](experiments/socket-client/README.md) exercises actual lookup, typed
fallback, snapshot ownership and loopback transfer. The HTTP client gets a focused
independent-server check because its DNS contract changed. API snapshots are refreshed;
website source is reviewed, with the website build skipped by author direction.

Two observations remain separate compiler/runtime investigation candidates. Returning
Result<byte[], string> from a private helper caused a MetadataLoadContext mismatch
in the current target compiler; the parser instead fills caller-owned temporary
storage. Calling ToString directly on a byte selected an inherited Object path that
faulted in this target; octets are widened to int for numeric formatting. Neither is
claimed fixed or established as a general Raven bug. Reduce them independently before
changing Raven main; do not broaden this address slice into a compiler migration.
