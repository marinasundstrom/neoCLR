# IPAddress closed hierarchy checkpoint

2026-09-25. An isolated target experiment, **not a public API or a completed address
slice**. The author selected a closed IPAddress class hierarchy after first selecting
a union. IPv4Address and IPv6Address are distinct, immutable reference types with
address value equality. Parsing, formatting, numeric IPv6 scope, public construction
and socket/DNS integration remain the next work.

## Comparison and choice

The .NET 10 baseline is one [IPAddress class](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress?view=net-10.0)
with address-family-dependent behavior. The selected neoCLR hierarchy makes the two
families explicit types behind one root. It retains familiar reference identity but
uses address value equality. A value union would instead copy its representation;
its layout/importer limits are not the reason for choosing the hierarchy. A closed
interface with struct implementations would introduce boxing at the shared boundary
and is not selected. No performance advantage is claimed for any representation.

The prototype uses privately copied byte arrays to exercise ownership and GC. This
is not a commitment to array-backed public addresses: compact numeric fields remain
an implementation option. Array input mutations must never change address identity.
The probe distinguishes the families by their validated byte lengths; the eventual
IPv6 equality contract must also decide scope semantics. No parsing constructor or
public fault-based validation contract is selected here.

Primary parsing references reviewed 2026-09-25:
[.NET TryParse](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress.tryparse?view=net-10.0),
[RFC 4291 section 2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2), and
[RFC 5952](https://www.rfc-editor.org/rfc/rfc5952.html).
A local .NET 10 comparison accepted abbreviated IPv4 `127.1`, hexadecimal
`0xffffffff`, octal `1.1.1.010`, numeric IPv6 scope and bracketed IPv6. Whether to
accept those legacy IPv4 and URI-specific forms is separate from representation.
Strict decimal four-part IPv4 is a candidate, not implemented policy. IPv6 value
support must not silently imply IPv6 socket support.

## Compiler and runtime boundary

Raven's `sealed class` declares an abstract closed root; ordinary leaf classes are
sealed. The existing compiler emits the closed-hierarchy marker and the bridge
emits `.closedhierarchy`. The probe required one bridge correction: an instance
constructor may `call` its direct base's protected constructor. This does not admit
private constructors, unrelated protected calls, `newobj` access or general protected
members. No Raven emission or Runtime Contract configuration changes are required.
The public-core projection must preserve the permitted family and reject forged
external branches; that integration is still pending.

## Validation

Use a matching rebuilt bridge, reference and system bundle:

```sh
python3 docs/experiments/ip-address-hierarchy/verify.py \
  --toolchain-root /path/to/bundle --runner target/release/examples/measure_async
```

The initial run passed root/Object equality, different identities, family distinction,
hash agreement, null/unrelated objects, input copying and allocation churn with live
base-typed references: 914 allocations, 17 collections, zero final live objects.
The focused library importer suite passes, including a protected-base positive
case and private-base/private-cross-type rejection cases. Raven rejects an external
address branch in a different source file with RAV0334. Website build is skipped
under the author's per-slice validation direction. Public `/docs/` entries belong
with the future public API, not these unexported experiment types.
