# First Raven TCP client (development)

This sample connects to a real loopback server, receives `Hi` across potentially
short reads, observes EOF, closes the socket twice, and checks that a subsequent
receive reports Closed. Other TaskQueue work runs while input is pending. Allocation
churn forces collection during pending connect and receive, while the native registry
owns callbacks and the receive destination.

The Python verifier starts a host server on an ephemeral port, substitutes that port
in the sample, compiles with the matching development reference/importer, and runs
with a 256-object heap. It checks exact output, multiple collections, zero retained
managed objects, peer-observed close, and compile-time rejection of the private
constructor/completion type. All network access is loopback; waits have watchdogs.

```sh
python3 docs/experiments/socket-client/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
cargo test --lib socket_io::tests
```

The published Preview 9 SDK does not contain these APIs. Connect accepts only numeric
IPv4 and a port 1–65535. A receive can return fewer bytes than requested. Positive
requests returning zero mean EOF; zero-sized requests do not test EOF. The pending
buffer must not be mutated or aliased by overlapping operations until completion.
Close and invocation teardown release native resources; there is no finalizer or
individual cancellation/deadline API yet. No thread is created per socket operation.

This is a greeting client, not the full two-sided echo milestone. Send, listener/accept,
endpoint value types, stream integration, DNS and IPv6 remain future slices. TcpClient
and UdpClient are candidates for later convenience layers, not current APIs.

The sample uses `?` for Result propagation and `if let` for payloads. Empty error
cases are tested with their IsClosed predicate: the current application importer
does not yet admit the direct `error is SocketError.Closed` value-type test. This
is an explicit frontend limitation, not different runtime union semantics.


A second existing state-machine limitation surfaced during validation: keeping a
Result local across a later await hoists a union carrier without a readable default
into the generated heap state-machine constructor, which faults on uninitialized
storage. This sample reads its completed Closed result synchronously after Close,
so it does not require that additional suspension. The general hoisted-union case
remains open; the runtime's constructor initialization rule was not relaxed.

Validation on 2026-09-24: the compiled source and both negative visibility checks
pass. The greeting run allocated and reclaimed 914 managed objects, with zero live
objects and 22 collections. Native checks cover validation, shared budgets, refusal,
result consumption, buffer ranges, cancellation/close and EOF. This is local
macOS evidence, not a cross-platform release or performance result.
