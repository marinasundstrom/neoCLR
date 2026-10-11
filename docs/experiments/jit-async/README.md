# Managed async entry points through JIT

2026-10-11, development-only macOS ARM64 profile. The managed JIT now admits the
existing task-queue bindings and native async entry wrapper. Fresh Raven programs
execute queued continuations and complete `Task<int>` or `Task<()>` entry points with
interpreter/AOT/JIT parity. This advances functioning application execution beyond the
scalar benchmark; it does not yet enable timer or socket completion sources.

## Ownership and behavior

The JIT still owns executable code and its helper library for the entire host call.
The host now enters a thread-affine task scope before invoking the generated entry.
The existing AOT entry wrapper drains the default queue, preserving the startup frame
and its GC roots while continuations execute. No change to Raven async lowering,
native metadata or the generated callback ABI is needed.

On success or guest fault, diagnostics are rendered while their backing storage is
alive. The host then requires the native frame stack to be empty, leaves the task scope
(releasing its default-queue root), checks that no owned roots remain, collects, and
frees the invocation heap. Only afterward can Rust release generated code and the
helper library. An ownership invariant failure aborts instead of unloading live code.
The host uses the same 1 MiB invocation heap as the earlier managed probe.

This reuses the established neoCLR Task/queue and AOT entry contracts. Relative to the
[.NET execution comparison](../../native-execution-investigation.md), JIT compilation is
an execution backend for the existing async lowering, not a new language async model.
Cancellation and unresolved-task faults retain neoCLR's current semantics; they are
not silently reinterpreted as CLR exceptions. The runtime pumps queued work inside one
host invocation. It does not add background workers, runtime suspension or green threads.

`--jit-managed` now accepts `--bind-task-queue`. Timers, sockets, native bootstrap,
Windows JIT and general embedding remain unsupported. Both timer and socket-binding
requests are checked to fail before guest invocation. A task with no completion source
still faults with the existing pending-task diagnostic. Native loops and callbacks have
no instruction budget; this private trusted host is not an isolation boundary.

## Qualification

[Retained evidence](validation.json) uses the same fresh compiled assembly in each mode:

| Case | Expected result |
| --- | --- |
| Async integer entry with arguments and queued completion | Exit 23 |
| Unit entry with two successive awaits and captured values | Ordered callback output, then 42; exit 0 |
| Await a cancelled task | Cancellation diagnostic; exit 1 |
| Await a task with no completion source | Pending-task diagnostic; exit 1 |
| Queued callback faults while entry awaits | Callback fault diagnostic; exit 1 |

All five cases pass in interpreter, AOT and JIT (15 executions), including native root
and scope cleanup on fault paths. The AOT consumer links the same private host adapter
statically, while JIT loads its helper library and retains generated code in memory.
This qualification does not replace the separate existing native HTTP host checks.

The [console regression](console-validation.json) additionally passes all 20 Index/Range
framework tests, no-match filtering and a bounds fault across all three modes, missing
helper rejection, and byte-identical AOT object output. Existing scalar benchmark evidence
is unaffected; no new performance claim is made for async execution.

## Reproduction

Use the compiler and runtime libraries from the qualified Index/Range bundle, including
Raven compiler `163050da2a79444878169f1d011abff644d37b2b`. Both `--object-library` and
`--async-library` must select `System.Runtime` for this existing native compiler contract.
The initial harness omitted the Object selection and was rejected during Raven emission;
adding the existing contract fixed the harness without a compiler change.

```sh
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/jit-async/verify.py \
  --bundle target/extension-indexer-final-bundle \
  --console-qualification target/jit-startup-validation/validation.json \
  --output target/jit-async-validation
python3 docs/experiments/jit-managed/verify.py \
  --build target/jit-iteration-release-baseline/index-range-build \
  --output target/jit-async-console-regression
```

Choose fresh output paths and set the Xcode/macOS SDK environment described in the
[managed probe](../jit-managed/README.md#reproduction). The async verifier reuses the
qualified console helper link recipe, adding task-scope support. It compiles every Raven
case afresh, records commands and hashes, and validates results/faults explicitly. Large compiler
inventory output is retained as a prefix, byte count and SHA-256; execution output is
retained in full.
The console verifier now includes the task-scope helper even for synchronous guests.
There is no public API addition, API snapshot change or website build for this private profile.

Next: supply a polled timer completion source while retaining code and roots through
completion/cancellation, then qualify HTTP client/server socket completions. Those host
services are still needed before the JIT can run the showcase. Execution defaults remain
unchanged.

Follow-up: [timer polling and cancellation](../jit-timers/README.md) are now qualified.
The earlier timer rejection above records this page’s initial slice; socket integration remains next.
