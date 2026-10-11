# HTTP client and server through JIT

2026-10-11, development-only macOS ARM64. The managed JIT now runs the existing
Raven `HttpServer` sample and an awaited `HttpClient` end to end. A client resolves
`localhost`, connects to the server, and receives the exact UTF-8 greeting `Café 🌍`.
The same source assemblies pass in the interpreter and AOT.

## Contract and implementation

The private `--jit-managed` profile now admits `--bind-socket-listener`,
`--bind-socket-accept`, `--bind-socket-transfer` and `--bind-socket-client`, alongside
`--bind-task-queue` and `--bind-timers`. Previously the JIT rejected these bindings.
No new HTTP API, compiler bridge representation or socket implementation is needed:
shared AOT lowering already generates the HTTP calls and completion adapters.

The [managed host](../jit-managed/host.c) enters one native I/O scope and task scope.
Its rooted entry poller delivers DNS/connect, accept/read/write and timer completions.
Generated code and its helper library stay owned throughout the host call. On return
or fault, the host releases/cancels operation roots, releases the task queue, checks
remaining roots and collects before freeing the heap; JIT code is reclaimed afterward.
A broken cleanup invariant aborts. No callback is permitted to retain this code after
host return. Native DNS workers follow the existing AOT operation ownership contract.

This reuses the existing [.NET execution comparison](../../native-execution-investigation.md)
and [HTTP timeout contract](../../design/http-client-lifetime.md): compiling managed code is
separate from providing asynchronous runtime services. The benefit is consistent
HTTP library behavior across execution modes. The cost is retaining the bounded AOT
host's polling and operation-table limits. A new JIT-specific scheduler would duplicate
lifetime rules without resolving production scheduling requirements.

The profile remains eager, closed-world compilation with a trusted helper library,
a 1 MiB heap and bounded operation table. Native bootstrap, Windows JIT, concurrent
host invocations, general embedding and callbacks escaping the invocation are not
qualified. Execution defaults are unchanged. Explicit client disposal follows await;
async `use` lowering remains a separate compiler gap.

## Qualification

[Recorded results](validation.json) contain 22 scenario results:

- In each of interpreter, AOT and JIT: exact server response, fragmented request,
  duplicate Content-Length rejection, handler error, paired client/server using DNS,
  request timeout with peer-observed socket closure, and delayed success with timeout
  disabled and client disposal.
- A JIT handler fault: native fault reporting, peer connection closure and successful
  root/scope teardown before generated code is released.

The four guest assemblies are compiled afresh using the qualified Raven bundle
(revision `163050da2a79444878169f1d011abff644d37b2b`). AOT statically links the same
host adapter that JIT loads dynamically. The report records build commands, input and
artifact hashes, wire response bytes and guest results; guest fault status is checked
separately from JIT process status. [Queued-entry regression](async-validation.json)
also checks bootstrap rejection before invocation. Prior timer and console evidence
remains linked from the [timer slice](../jit-timers/README.md).

On the local Apple M1, with Xcode 26.2 selected:

```sh
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export SDKROOT="$DEVELOPER_DIR/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk"
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/jit-http/verify.py \
  --bundle target/extension-indexer-final-bundle \
  --console-qualification target/jit-startup-validation/validation.json \
  --output target/jit-http-validation
```

Use a fresh output directory. The prior successful console qualification supplies the
native helper build recipe and context bindings; see the
[managed probe reproduction](../jit-managed/README.md#reproduction). The verifier
uses `target/release/neoclr`, the release AOT/JIT tool and the bundle's Raven compiler
and matching libraries. Local port selection is ephemeral and test processes are
cleaned up on failure. This is functional evidence, not sustained-service or HTTP
performance evidence. The [later cross-platform benchmark plan](../../native-execution-investigation.md#later-benchmarking-against-net-and-other-platforms)
tracks equivalent client and server comparisons with .NET, Go and JavaScript.
