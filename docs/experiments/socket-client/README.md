# First Raven TCP client (development)

This sample resolves localhost through Dns.GetHostAddresses, then connects to a real loopback echo server, sends `Hi` and receives the reply across potentially
short reads, observes EOF, closes the socket twice, and checks that a subsequent
receive or send reports Closed. Other TaskQueue work runs while input is pending. Allocation
churn forces collection during pending lookup, connect, send and receive, while the native registry
owns callbacks, native send snapshots and the receive destination. The source array
is modified after each Send to verify that only the submitted snapshot reaches the peer.

The Python verifier starts a host server on an ephemeral port, substitutes that port
in the sample, compiles with the matching development reference/importer, and runs
with a 256-object heap. It checks exact output, multiple collections, zero retained
managed objects, peer-observed close, and compile-time rejection of the private
constructor/completion types. All network access is loopback; waits have watchdogs.

```sh
python3 docs/experiments/socket-client/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
cargo test --lib socket_io::tests
```

The published Preview 9 SDK does not contain these APIs. Connect accepts only numeric
IPv4 and a port 1–65535. Send and Receive can return fewer bytes than requested; both loops advance by the
reported count. Send snapshots the submitted range, allowing immediate source reuse. Positive
receive requests returning zero mean EOF; zero-sized requests do not test EOF. The pending receive
buffer must not be mutated or aliased by overlapping operations until completion.
Close and invocation teardown release native resources; there is no finalizer or
individual cancellation/deadline API yet. No thread is created per socket operation.

This is a neoCLR echo client against a host server. The full two-sided neoCLR echo
milestone still requires listener/accept;
endpoint value types, stream integration and IPv6 remain future slices. DNS now
returns a read-only Sequence of immutable IPv4Address values with a five-second lookup deadline.
The controlled sample uses the first address; multi-address fallback and an overall
connection deadline remain open. TcpClient
and UdpClient are candidates for later convenience layers, not current APIs.

The sample uses `?` for Result propagation and `if let` for payloads. Empty error
cases use the generated SocketError case-matching contract. SocketError no longer
requires Is*/Get* helpers. Other not-yet-migrated errors retain their current surface.


A second existing state-machine limitation surfaced during validation: keeping a
Result local across a later await hoists a union carrier without a readable default
into the generated heap state-machine constructor, which faults on uninitialized
storage. This sample reads its completed Closed result synchronously after Close,
so it does not require that additional suspension. The general hoisted-union case
remains open; the runtime's constructor initialization rule was not relaxed.

Initial receive-only validation on 2026-09-24: the compiled source and both negative visibility checks
pass. The greeting run allocated and reclaimed 914 managed objects, with zero live
objects and 22 collections. Native checks cover validation, shared budgets, refusal,
result consumption, buffer ranges, cancellation/close and EOF. This is local
macOS evidence, not a cross-platform release or performance result.

Validation on 2026-09-24: 35 focused backend, VM, scheduler and service tests pass.
The compiled send/receive sample uses `await Foo()?`, reclaims all 1,380 allocated
objects over 30 collections, and passes both negative visibility checks. The matching
API snapshot and combined 555-page website build pass. Evidence is local macOS.


DNS integration limits: a direct array nested in Task/Result exposed a compiler
MetadataLoadContext mismatch; Sequence is the current read-only contract. A nested
callback capturing the newly created exchange task also faulted with a null task
capture; StartExchange now registers that callback in its own function. Neither
change claims to fix those general compiler/importer paths. See the socket design
record for the deferred independent reproduction work.

Validation on 2026-09-24: eight resolver tests, nine runtime-service tests and eight
scheduler tests pass (25 total). The compiled hostname/echo sample reclaims all
1,428 allocations across 31 collections, with zero live objects; all three negative
visibility checks pass. Ten website tests and the combined 572-page API/site build
pass. Networking was inspected in the local browser. This is local macOS evidence,
not cross-platform networking certification or a runtime release.


The client now passes a `Sequence<IPAddress>` to Socket.Connect. For this controlled
loopback POC it prepends 127.0.0.2, where the verifier has no listener, before the
addresses returned for localhost. Connect snapshots the inputs; the sample overwrites
the first entries afterwards and still reaches 127.0.0.1. All attempts share five
seconds, with at most one second on an address while alternatives remain. DNS is
separate. This demonstrates fallback and snapshot ownership, not configurable
connection policy, IPv6 racing or a combined lookup/connection deadline.
