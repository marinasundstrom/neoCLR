# Native web benchmark workbench — work in progress

This is an implementation workbench, not a completed HTTP server benchmark or a
release qualification. Any POC included in the next release remains work in progress.

## Checked-in workloads

- `Routing.rvn`: same-source interpreted/native component workload, derived from
  `docs/experiments/aot-console/route-lifetime.rvn`, fixed at 1,024 routing operations.
  It checks eight target forms (including UTF-8, invalid escapes, missing routes and
  integer overflow), retains a pattern and capture, and prints `1024` only on success.
- `../../docs/experiments/http-server/Server.rvn`: the existing Raven HTTP app is
  the working one-request interpreter/native POC. It serves one GET `/greeting` with the UTF-8 bytes
  `Café 🌍`, then closes. Its source and interpreter verifier remain checked in there.
- `dotnet/Program.cs`: ASP.NET Core comparison candidate for that successful exchange,
  targeting .NET 10. It serves repeatedly until stopped. Invalid requests, request
  limits, shutdown and framework-generated headers are not yet equivalent. This is
  not a completed performance comparator, and its throughput must not be compared
  with a one-request Raven process.

## First compare the same Raven program

Build the current runtime with `cargo build --release --bin neoclr` and the AOT tool
with `cargo build --manifest-path tools/aot-poc/Cargo.toml`. Supply a matching Raven
compiler and native library bundle, as for the AOT console experiments:

```sh
SDKROOT=$(xcrun --show-sdk-path) python3 benchmarks/native-web/compare.py \
  --compiler /path/to/rvnc.dll --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output target/native-web-comparison
```

The output directory must be new. The script compiles one Raven artifact, runs that
artifact through the interpreter and compiles its same `Main` entry to ARM64. Native
adapters use Clang `-O2` and a 64 KiB collector buffer. One warm-up pair precedes five
measured pairs with alternating order. Build time is excluded; process startup,
interpreter metadata loading/verification, workload execution and output are included.
This is an end-to-end component invocation metric, **not steady-state handler latency,
requests per second over HTTP, or a language-wide speed comparison**. Interpreter
object limits and native byte limits are different policies, not equal memory budgets.
Collection still runs at every native boundary; the interpreter uses pressure scheduling.

Each timed invocation must exit successfully and print the expected result. Separate
untimed invocations collect GC diagnostics. `results.json` preserves raw samples,
commands, source/tool hashes, native dependencies, image size and server admission.
The script does not run the server or quietly substitute C socket/HTTP code for Raven.

## Recorded component result

On macOS ARM64, the 2026-10-08 run measured median process wall times of 29.447 s
interpreted and 0.392 s native for the same 1,024-operation entry. All outputs matched.
The native host retained 1,648 bytes of heap extent in its 64 KiB buffer. These are
not HTTP timings; loading, verification and differing GC policies remain in scope.
[Raw samples and provenance](routing-validation.json) also record the server's current
selection rejection and a successful .NET greeting correctness check (JIT, no timings).

## Socket-handle prerequisite

`SocketHandle.rvn` now runs in both modes with full-width signed/unsigned generic
values. Native erased values also admit Int64 and UInt64 with distinct private tags;
IL tests cover exact high bits, copies, calls and mismatched unpack faults. The
interpreter already supports this behavior, so no interpreter change was needed.
`Listen.rvn` now executes Socket.Listen/GetLocalPort/Close in both modes, including
invalid addresses/ranges, duplicate binding, idempotent close, stale handles and port
reuse. `ListenFault.rvn` leaves a listener open before a user fault; the native host
releases it and matches interpreter fault output. The complete HTTP server still
advances through stored callbacks and their ArrayList storage to value-array admission and still needs
asynchronous socket/task support. Reserved value-array storage now passes its own consumer;
the full HTTP app reaches the current specialization budget.
[Wide-value evidence](handle-validation.json) records the earlier admission boundary;
[current listener evidence](listener-validation.json) records the completed lifecycle.
This is correctness/admission work, not a performance optimization; the routing
benchmark is unaffected and was not rerun for a speed claim.

## HTTP comparison contract and next slices

1. Listener creation/local port/close now work with `--bind-socket-listener` and an
   explicit host resource scope. Continue with function-valued dependencies, callback
   ownership and async accept/read/write; the existing server must compile and retain
   pending callbacks safely across collections.
2. Extend the Raven app to serve a controlled number of requests in one process. Run
   the **same** source/artifact in interpreted and native modes with the same listener,
   handler and response. Check bytes, UTF-8 Content-Length, connection closure,
   malformed input, disconnects and bounded memory before timing.
3. Match the successful exchange in ASP.NET Core JIT and Native AOT first. Consider Go
   `net/http` later. Pin exact versions, flags and source revisions. No .NET or Go
   performance results have been measured here.
4. First use HTTP/1.1 loopback, GET `/greeting`, no TLS, connection close, concurrency
   one, identical payload and request count, no per-request logs. Compare this limited
   configuration explicitly; do not imply it represents normal keep-alive production
   performance. Add keep-alive, routing/JSON and concurrent load as separate cases only
   when all implementations support the same behavior.
5. Measure process start to readiness separately from warm steady-state throughput,
   p50/p95/p99 latency, failures, peak RSS and executable/deployment size. Keep GC
   telemetry outside timed runs unless its overhead is itself measured. Define the
   load generator, warm-up, duration, connection reuse, offered load and timeouts;
   preserve all samples/failures. Use a separate load-generator host for saturation
   claims; local loopback numbers include client contention.

A C socket host calling a compiled Raven handler could be an earlier mixed-language
experiment, but it would measure a different stack and must have a separate label.
The intended server is the Raven app using the existing HTTP API.

## .NET comparison candidate

```sh
dotnet build benchmarks/native-web/dotnet/Greeting.csproj -c Release
dotnet benchmarks/native-web/dotnet/bin/Release/net10.0/Greeting.dll \
  --urls http://127.0.0.1:8080
curl --http1.1 --include http://127.0.0.1:8080/greeting
```

The checked-in correctness harness starts an ephemeral-port server, verifies HTTP/1.1
status, UTF-8 bytes, Content-Length, Content-Type and the Connection: close header,
then stops its own process. It does not benchmark or assert malformed-request parity:

```sh
python3 benchmarks/native-web/check_greeting.py --command \
  dotnet benchmarks/native-web/dotnet/bin/Release/net10.0/Greeting.dll \
  --urls http://127.0.0.1:0
```

ASP.NET Core supports Native AOT for a subset of its features; compatibility must be
checked on the published binary, not inferred from a JIT run. See Microsoft's
[ASP.NET Core Native AOT guide](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/native-aot?view=aspnetcore-10.0)
and [Native AOT deployment contract](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
(primary documentation checked 2026-10-08). Publish/validate the candidate with
`dotnet publish -c Release -r osx-arm64 -p:PublishAot=true` before using an AOT label;
this workbench does not yet claim that validation. neoCLR similarly links runtime
support into its image; an OS library dependency remains. Its bounded GC/API coverage
is much smaller. No performance advantage follows from that difference alone.


## Reproduce listener lifecycle validation

```sh
SDKROOT=$(xcrun --show-sdk-path) python3 benchmarks/native-web/verify_listener.py \
  --compiler /path/to/rvnc.dll --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output target/native-listener-rerun
```

This opens only test loopback listeners, closes them on success/fault and compares
Raven interpreter/native output. It checks opt-in admission, sanitized adapters and
standalone dependency lists. No requests are accepted and no throughput is measured.
The [private native hosting contract](../../docs/experiments/aot-console/socket-listener.md)
requires entering/leaving a socket scope for every invocation; GC is not descriptor cleanup.

## Stored callback prerequisite

`Callbacks.rvn` keeps a bound Counter method inside a holder while creating and invoking
1,000 static callbacks. `CallbackFault.rvn` raises a user fault from a lambda. Both run
from the same compiled metadata in the interpreter and native code, with exact fault
output parity. The native host checks frame cleanup and collects the quiescent heap;
a separate IL test stresses receiver-only reachability in 2 KiB. These are synchronous
callback correctness tests, not task execution or throughput measurements.

```sh
SDKROOT=$(xcrun --show-sdk-path) python3 benchmarks/native-web/verify_callbacks.py \
  --compiler /path/to/rvnc.dll --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output target/native-callback-validation
```

[Callback evidence](callback-validation.json) includes tool/source hashes, commands,
standalone dependencies and the next HTTP admission boundary (Function arrays).
[Implementation and provisional limits](../../docs/experiments/aot-console/callbacks.md)
explain dispatch, ownership and the .NET comparison. No public API/bridge change or
interpreter modification is required for this native implementation slice.

`CallbackList.rvn` extends this consumer through the real ArrayList growth/Copy/indexed
replacement path, including shared receiver mutations and 1,000 discarded bound methods.
Run the same command with `--case CallbackList` for the focused container check; without
`--case` the script runs all callback and value-storage samples. [Container evidence](callback-array-validation.json)
records matching interpreter/native results and the next HTTP rejection: arrays of
`Result<Void, HttpError>`. Historical callback evidence retains its earlier admission boundary.

## Reserved value-array prerequisite

`ResultList.rvn` exercises the real ArrayList<Result<int,string>> path, including growth,
Copy, Error/Ok replacement, let-pattern and if-let matching, and 1,000 transient strings.
Use `verify_callbacks.py --case ResultList` with the same required path arguments for its
focused interpreted/native correctness run. The script allows 100 million interpreter
instructions; it does not claim equal execution budgets or measure performance.
[Value-array evidence](record-array-validation.json) records matching outputs, native GC
and standalone dependencies. [The implementation notes](../../docs/experiments/aot-console/record-arrays.md)
describe initialized snapshots, conservative tracing and unsupported operations.

The full HTTP driver initially exceeded the 128-specialized-type budget (see the later selection evidence below). `TaskResultList.rvn`
is a narrower admission probe for Result<Void,HttpError>; the explicit-core System.Void import fix now lets it compile and run interpreted.
The subsequent 64-lane value/array/GC slice also passes native execution with nested
HTTP error/status payloads, let/if-let patterns, copies and 1,000 replacements.
Run `verify_callbacks.py --case TaskResultList`; [execution evidence](http-result-validation.json)
records sanitized parity and standalone dependencies. The earlier
[admission failure](http-result-admission.json) is retained as historical evidence.

`EnumValues.rvn` now passes in both modes with HttpStatusCode named/unnamed values,
BindingFlags masks and ArrayList growth/copy isolation. Run `verify_callbacks.py
--case EnumValues`; [evidence](enum-validation.json) includes sanitized execution,
64 KiB native GC and standalone dependencies. See the [enum implementation notes](../../docs/experiments/aot-console/enum-values.md).

### Full-server selection after nested results

[Admission evidence](server-selection-validation.json) records the existing HTTP
Server source clearing specialization with 208 shapes, 570 functions and 213 clones.
Shared experimental bounds are now 256 types, 1,024 functions (including generated
boxing helpers) and 256 clones. Tests cover accepted/rejected edges and preserve
plain callees at the clone limit. Inspection includes counts and bounded failures.
The next rejection is nominal arrays of reference-class elements. This is compiler
admission evidence, not HTTP execution or a benchmark; no server comparison is ready.

These caps bound development work and memory; they are not platform type limits or
trimming. As in the existing .NET Native AOT comparison, the compiler must materialize
required closed instantiations. Raising checked caps accommodates this consumer but
can increase compilation time/memory; it does not reduce reachability or prove an
optimization. The original input-metadata, frame-size and runtime heap limits remain.

`ReferenceList.rvn` now passes the next dependency: checked ArrayList<Counter> growth,
copy sharing and 1,000 replacements while only the copy retains the original objects.
Run `verify_callbacks.py --case ReferenceList`; [evidence](reference-array-validation.json)
records matching interpreter/native results, sanitized GC and standalone dependencies.
The full server now reaches the unbound ObjectReferenceEquals service. See the
[reference-array contract and limits](../../docs/experiments/aot-console/record-arrays.md#reserved-reference-arrays-2026-10-08).

The following [identity binding evidence](object-identity-validation.json) extends
ReferenceList with public Object.ReferenceEquals alias, null and text-owner checks.
The reserved service lowers under --reference-arena to native identity comparison;
ordinary methods are not replaced. The full server next reaches StreamError.ToString's
value-member contract. No Equals/GetHashCode dispatch or server execution is claimed.

`ValueDisplay.rvn` passes real StreamError.ToString descriptions through a verified
by-reference value-override projection. Run `verify_callbacks.py --case ValueDisplay`;
[evidence](value-display-validation.json) records parity, sanitizers and standalone
linkage. See [the bounded contract](../../docs/experiments/aot-console/value-display.md).
The full server next reaches primitive member-owner admission.

The extended ValueDisplay also passes borrowed Int32 ToString/Equals and nested
HTTP status/content diagnostics. [Primitive-wrapper evidence](primitive-display-validation.json)
records signed extremes, zero and status 599 in both modes. Arbitrary input borrows
remain unsupported; the private projection preserves verified receiver storage.
The full server now reaches SocketConnectResult, moving the immediate dependency
work to task dispatch, native host roots and asynchronous socket completion.

`QueuePump.rvn` and `QueuePumpFault.rvn` exercise post-entry queue work, nested posting
and faults from queued callbacks. Run `verify_callbacks.py --case QueuePump --case
QueuePumpFault` with the usual compiler/runtime/AOT/bundle/output arguments. The private
native host pumps the existing Drain body; the same artifacts run under interpreter
implicit pumping. [Evidence](queue-pump-validation.json) records exact output/fault
parity, sanitized cleanup and standalone libSystem-only linkage. This is behavior
validation, not a Scheduler implementation or HTTP throughput benchmark.

## First native HTTP execution

The same Raven Server artifact now passes four cases in the interpreter, sanitized
native code and a standalone native image: greeting, fragmented request, duplicate
Content-Length, and handler rejection. Response bytes and application output match.
The native host verifies published-frame cleanup, socket/task scope release and zero
remaining managed heap bytes on normal termination. The standalone binary links only
libSystem. [Recorded evidence](http-validation.json).

```sh
SDKROOT=$(xcrun --show-sdk-path) python3 benchmarks/native-web/verify_server.py \
  --compiler /path/to/rvnc.dll --runtime target/release/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/matching-native-bundle --output target/native-http-validation
```

The output directory must be new. The script compiles Raven once, AOT-compiles that
artifact, links the matched C adapters and runs loopback clients against each mode.
The private host uses a 1 MiB nonmoving managed heap, polled nonblocking sockets,
explicit TaskQueue draining and an opt-in native stack guard. Its 15-second host
completion watchdog and the script's process watchdog are test guards, not scheduler
fairness or a guest instruction budget. Request error paths close without HTTP error
responses, matching this library contract. Four cases do not qualify every cancellation,
shutdown or disconnect race. Runtime Scheduler/green-thread design remains separate.

Each process serves one request, so this evidence is **not HTTP throughput**. Next use
one long-lived process and repeated equivalent requests, with startup recorded
separately. Keep .NET comparisons provisional until workloads and load settings match.

## Repeated requests and first local HTTP measurements

`PersistentServer.rvn` serves exactly 32 sequential `/greeting` requests in one process.
Its callback chain starts the next ServeOne after completion, then closes the server.
This exercises `let ... else`, `if let`, task completion and reclamation across exchanges.
The same artifact passes interpreter, sanitized native and standalone execution.

Add `--persistent` to `verify_server.py` for correctness/cleanup only, or
`--persistent --rounds 3` for at least three measured pairs. The harness first qualifies
all three modes, then alternates interpreter/standalone execution order. Every response
is checked; the first eight requests warm each process and the remaining 24 are timed.
No HTTP keep-alive, concurrency or external-platform result is implied.

On Apple M1/macOS ARM64, 2026-10-08, three local pairs produced:

| Mode | Median requests/s | Median of per-run request medians | Warm process launch to port |
| --- | ---: | ---: | ---: |
| Release interpreter | 3.08 | 324.40 ms | 1,868.40 ms |
| ARM64 native standalone | 20.09 | 49.65 ms | 3.54 ms |

[Raw per-request samples, commands and hashes](persistent-http-validation.json) retain
all runs. Throughput is 24 divided by the sum of those 24 request times, then the median
across runs. Latency includes connection establishment through response EOF on the same
machine. Startup is reported separately after warmup, not as a cold-machine measurement.
Native C adapters use -O2; the current Cranelift CIL backend uses its default unoptimized
setting. The interpreter uses its Release binary. GC/host scheduling policies differ;
native uses a fixed 1 MiB nonmoving heap and a private one-millisecond idle poll.
These short runs are a development baseline, not capacity planning or reliable tail
latency estimates. No .NET ranking is available. Profile before selecting an optimization.

The next-release teaser is this native execution path with reproducible samples and WIP
limits. Full Raven compiler-target bootstrap through neoCLR metadata without the .NET
bridge remains a separate release requirement; the development Core.dll input does not
satisfy that gate merely because the resulting executable has no .NET dependency.


## Native GC lookup follow-up

A profile-driven sparse address index improves the same native HTTP object from 20.08
to 96.05 requests/s in three short alternating adapter comparisons. Median per-request
latency changes from 49.74 to 10.43 ms. [Evidence](gc-index-validation.json) includes
correctness, cleanup, sanitizer and stack-frame checks. This is a native-adapter
before/after comparison; the interpreter was not timed again and no .NET ranking follows.

For focused C-adapter changes, `verify_server.py --reuse-build <validated-directory>`
checks compiler/source/library hashes and reuses its assembly/object while rebuilding
adapters. In persistent mode, `--native-baseline <validated-directory> --rounds 3`
compares the old standalone image against the new one, requiring identical assembly
and object hashes. Without that option, comparison remains interpreter versus native.
