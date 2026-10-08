---
title: Native compilation
---
# Native compilation

**Work in progress.** neoCLR can compile a bounded set of Raven applications into
standalone ARM64 executables. The goal is a web server app we can run in interpreted
and native modes, then compare with equivalent apps on other platforms. A POC included
in the next release will still carry this work-in-progress status.

## From Raven to an executable

The current path is Raven source → neoCLR metadata and CIL → native ARM64 object code
→ an executable with the required library and runtime support linked in. This consumes
neoCLR CIL, not arbitrary .NET assemblies. Tested macOS executables depend on the OS's
libSystem, with no separately installed neoCLR or .NET shared runtime.

The interpreter remains supported. Ahead-of-time compilation trades build work and
an architecture-specific executable for direct native execution. It does not remove
runtime services: allocation, garbage collection, text operations and fault handling
still need implementations. Hot reload is a separate design problem, not a feature
reserved permanently for interpreted execution. JIT and native hot reload are future work.

## A working HTTP proof of concept

The real HTTP routing library already runs in both modes. This example parses a route,
extracts an integer parameter and prints `42`:

```raven
{{NATIVE_ROUTING_SAMPLE}}
```

A larger checked-in workload runs 1,024 routing operations, including missing routes,
invalid escapes, UTF-8 text and integer overflow. A pattern and an earlier capture
remain reachable across later operations. Native GC completes this workload in a
64 KiB buffer. This exercises application library code and reclamation; it does not
listen on a socket or measure HTTP throughput.

The checked-in Raven server now runs in interpreted and native modes. It listens on
loopback, serves one HTTP/1.1 GET `/greeting` with the UTF-8 body `Café 🌍`, and closes.
The same compiled Raven artifact passes normal and fragmented requests, duplicate
Content-Length rejection, and handler rejection in both modes. Application output
and response bytes match. The native host checks frame/root cleanup and reclaims its
1 MiB managed buffer after each successful run.

[Reproduction and evidence](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/native-web)
include sanitized and standalone executions. Required GC, queue, socket and fault
support is linked into the image. A repeated-request sample also serves 32 sequential connections in one process.
Long-running load, concurrency and broad failure-path qualification remain next steps.
[Web and HTTP](../web/) describes the application APIs.

## Measurements and comparisons

Our first comparison runs the **same Raven artifact and entry point** through the
interpreter and native backend. The checked-in harness validates output before accepting
a timing, alternates execution order, warms up, and saves raw samples and tool hashes.
It uses a Release interpreter and optimized C adapters. The current metric includes
process startup, interpreter metadata loading/verification, routing and output; it is
not steady-state handler latency. The modes currently use different GC scheduling and
memory-limit policies.

Local macOS ARM64 results (2026-10-08), five measured pairs after one warm-up pair:

| Same routing workload, 1,024 operations | Median process wall time |
| --- | ---: |
| neoCLR interpreter, Release build | 29.447 s |
| neoCLR ARM64 native image, native GC | 0.392 s |

Both runs print `1024` after validating all routing outcomes. These numbers include
startup/loading and use different collector policies; they must not be read as an HTTP
throughput ratio. [Raw samples and build/input provenance](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-web/routing-validation.json)
allow the result to be checked and repeated as the implementation changes.

A separate collector experiment improved an adverse reverse-linked graph from 317 ms
to 110 ms for 100 collections. The real routing workload was effectively unchanged
(391 ms versus 386 ms). That illustrates why a microbenchmark gain needs an application
check; it is not a server or cross-platform performance claim.

[Samples, reproduction commands and comparison contract](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/native-web)
are checked into the repository. The ASP.NET Core greeting sample is an initial comparison
candidate. No HTTP performance ranking against .NET or another platform is available yet.

A first local HTTP comparison now uses one persistent process for 32 sequential
connection-close requests, excluding the first eight requests from request timing.
Three alternating pairs on Apple M1/macOS ARM64 (2026-10-08) produced:

| Same Raven HTTP artifact | Median requests/s | Median request latency |
| --- | ---: | ---: |
| Release interpreter | 3.08 | 324.40 ms |
| ARM64 native standalone | 20.09 | 49.65 ms |

Each response is validated. Latency is the median of per-run medians and includes
connection establishment and response reading by a client on the same machine.
The short runs use different GC/host scheduling policies; native adapters use -O2,
while native CIL currently uses Cranelift's default unoptimized setting. These are
POC baselines, not capacity or tail-latency claims. Startup is measured separately.
[Raw samples, tool hashes and full method](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-web/persistent-http-validation.json)
make the comparison reviewable.

Extend the short repeated-request comparison to longer matched loads, then ASP.NET
Core JIT and Native AOT with matching successful requests and responses. Start with
HTTP/1.1, identical UTF-8 payloads, connection-close behavior and concurrency one.
Measure startup separately from steady-state throughput, latency percentiles, errors,
peak memory and deployment size. Keep-alive, concurrency and JSON will be separate cases
as support grows. Record platform versions, build flags, hardware and load-generator
settings with every published result.

.NET already provides [ASP.NET Core Native AOT](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/native-aot?view=aspnetcore-10.0)
with its own compatibility constraints. neoCLR's smaller POC scope does not establish
an advantage over it; useful comparisons require equivalent behavior and measured costs.

## Current caveats

- **Target and toolchain:** ARM64 is the primary target; executable evidence currently
  comes from macOS ARM64. Use the matching development Raven/compiler/library bundle.
  The development compiler target still uses a temporary CLI core bootstrap; full
  native-metadata compiler-target bootstrap without the .NET bridge is a release
  requirement. A standalone output image does not by itself establish that build-path
  qualification. This is not a general-purpose publishing command for every neoCLR app.
- **Library coverage:** Hello World, unions, console/text operations and selected routing
  paths work, along with the tested HTTP accept/read/write and task-completion path.
  Broader generics, library coverage and sustained server behavior still need qualification.
  Interpreter API availability does not imply AOT availability.
- **Memory:** the opt-in collector is nonmoving and conservatively scans object payloads.
  It can retain extra objects, fragment its bounded buffer and collect too frequently.
  Private host root handles now retain allocations across guest frame returns, with
  explicit release and entry-reset checks. Private host dispatch can invoke retained
  zero-argument Void callbacks between guest calls. A native accept kernel passes
  loopback completion/cancellation tests, including compiled CIL callbacks. Transfer
  kernels also pass snapshot/copy-back, cancellation and EOF tests. A compiled CIL echo
  chains accept/receive/send across collections, including deadline-service calls.
  A Raven TaskQueue/Promise consumer preserves default and explicit queue behavior.
  Private host queue draining now drives the HTTP POC. Precise maps, pressure-based
  collection scheduling and general hosting handles remain open.
- **Faults:** tested paths preserve fault codes, messages and managed stack traces.
  This does not yet qualify every server disconnect, cancellation or cleanup path.
- **Deployment:** required support is linked into tested images, but OS dependencies
  remain. Trimming, stable public native ABI/metadata interfaces and broad platform
  qualification are future work.
- **Results:** component timings are local development evidence. A runnable or released
  POC is not a production-readiness or performance guarantee.

See [garbage collection](../gc/) for lifetime behavior and
[direction and proposals](../../proposals/) for the broader execution roadmap.

The development backend now has an opt-in macOS ARM64 stack guard for recursive calls,
with bounded fault traces and GC-frame cleanup. This is a private POC adapter; native
HTTP requests now pass in both modes, including post-entry queue draining. Runtime-owned
scheduling and green threads remain future work. Their services should span interpreter,
AOT and eventual JIT; current TaskQueue and pthread adapters remain provisional.


### Development follow-up: native GC lookup

A later three-pair comparison of the same native object measured **20.08 → 96.05
requests/s** after improving GC address lookup (49.74 → 10.43 ms median request latency).
Correctness and native cleanup checks pass. This short local adapter comparison does
not rerun the interpreter or establish a .NET comparison. Collection still runs at
every native boundary; production GC policy remains future work.
See the checked-in `benchmarks/native-web/gc-index-validation.json` for raw evidence.


### Coverage baseline

A development survey of 104 Raven samples emitted 68 artifacts; 12 ran successfully
with matching interpreted/native output and 56 reached an AOT rejection. Primitive
arrays were the first blocker in 39 cases. This is an inventory of older API samples,
not the full native test suite or a requirement that every sample work in every mode.
The focused callback, Result-list, queue/fault and HTTP consumers also pass. The release
aims for a stable, documented subset while the compiler-target bootstrap still needs
to remove its temporary CLI Core dependency.

Future reflection may use IL-free metadata beside the executable, sharing descriptions
with native interop. Invocation would additionally require retained native code and
calling conventions. This is a proposal, not current support.


The first coverage follow-up adds bounded integer, Boolean and Void arrays, including
checked reserved storage and atomic GC payloads. The ordinary Raven array sample now
runs in both modes. Ordinary arrays of class references now also have null-initialized
slots, with identity and GC checks. Verified class-reference array views now support
collection interfaces; the checked-in order-collections sample runs Filter/Map/ToList
with matching interpreted/native output. Exact element identity and mutable-array
invariance remain enforced. Other array-view categories and value-record default
arrays remain unsupported. The ordinary lexical Path.Combine/GetFileName sample also
passes both modes with explicit native path bindings on macOS ARM64; filesystem I/O
and native Windows path behavior are not included. Nine other selected samples advance to further metadata, primitive
wrapper, callback and service gaps; broad Tasks/await parity is not yet claimed.


A further fix enables ordinary Boolean/Int64/UInt64 wrappers and 64-bit ordering.
The Boolean and generic-collection samples now also match interpreted execution.
General virtual dispatch, Tasks entry lifecycle and reflection remain separate gaps.


Wide division/remainder now preserve interpreter results and faults, with 72 boundary
comparisons. The calendar sample also passes; complete time-zone/service coverage is
not implied.

A further development slice admits callbacks bound through closed class interfaces.
Three async samples now match interpreter/native output, including cancellation and
task results. Queue-only async entry draining now keeps startup roots live while
callbacks run; async entry waiting for host I/O remains unsupported. The runtime protocol uses
`AsyncStateMachine` and `TaskAwaiter`, with matching Raven target mappings. Old
artifacts need recompilation. These coverage checks are not new benchmark results.

Development Math constants `Pi`, `E` and `Tau` now survive separate compilation
into native metadata and pass interpreted execution. AOT currently rejects the
sample because Double instructions are not yet supported. The
[tested sample](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/raven-target/samples/library-math-constants.rvn)
uses qualified and wildcard imports. Assembly-level constant metadata currently supports
finite Double values; rebuild compiler/runtime bundles together. This remains work in progress.
