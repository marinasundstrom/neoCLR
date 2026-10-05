# Source DNS/socket execution — 2026-10-05

Six unchanged production networking sources compile with two internal native adapter
files into Sockets.dll. The unchanged network-cancellation/Main.rvn is then compiled
with only that artifact, the cumulative 115-input Numbers library, explicit primitive
bootstrap and retained runtime seed. No library sources or consumer stubs participate
in the application build.

The consumer passes runtime verification and executes with status 0 and exact stdout:

```text
Network token cancellation checks passed
```

Its assertions cover pre-cancellation before invalid argument admission; pending DNS,
accept and transfer cancellation; unchanged receive buffers; typed IP addresses;
listener/connection reuse; registration removal after completion; disposal without
cancelling an accepted operation; and zero-sized receives. Only localhost DNS and an
ephemeral loopback listener are used. The existing native provider owns sockets and
callbacks; the test does not depend on a public network server.

## Changes and comparison

The explicit adapters declare the 21 existing DNS/socket services. DnsAddresses shares
the StorageNames conversion from an erased native string vector to independently owned
managed Array<String> storage, retaining array/heap budgets. Legacy ordinary neoil
vector helpers are unchanged. Native callbacks retain the existing function signature
representation, not a nominal delegate identity. No metadata version or new runtime
network semantics are introduced.

Raven a59df5635 admits converted value receivers through existing one-time temporary
storage and immutable by-value parameter captures through native closure fields.
Mutable bindings and byref parameters remain unsupported in this native closure path.
The ordinary .NET emitter already handles these constructs. Main has no portable
adapter, so there is no independent main backport for these changes. No importer
objects enter emission and no Runtime Contract switch changes.

All 59 focused shared-body compiler tests pass. The managed snapshot and exact native
binding tests pass; the executable consumer exercises the newly connected provider
path. This matches the existing .NET-style asynchronous completion/cancellation
contract while using native function values and managed UTF-8 string snapshots.
No performance improvement or broader cross-platform guarantee is claimed.

## Reproduction

Use `verify_source_network.py --help` and supply the compiler, cumulative library,
bootstrap ownership manifest, primitive core, seed, runtime and a new output directory.
The harness records compilation, verification, execution, source/artifact hashes and
repository revisions. Each subprocess has a 180-second watchdog; the guest instruction
budget is 100 million. The script tests exact stdout/status in addition to in-guest
assertions. The [recorded evidence](source-network-2026-10-05.json) includes working
binary hashes; its runtime revision is the pre-slice base.

The guest public API is unchanged, so no generated guest reference refresh is required.
The already-stale guest API snapshot remains documented separately. No website build
was run. The native adapter README and both repositories' integration docs are updated.

## Next blocker

The full networking/web group clears the initial service and apparent Error-identity
binding failures but fails emission with BoundPropagateExpression in HTTP sources.
This is the next high-unlock compiler/lowering investigation. HTTP execution, complete
System source ownership and the dual-target class-library gate remain open.
