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

## A working step toward the server

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

[Web and HTTP](../web/) explains the existing application APIs and server case.
The server source compiles to neoCLR metadata, but its native dependency selection
now clears experimental specialization (208 type shapes and 570 functions) and now reaches asynchronous socket-result binding. StreamError and nested HTTP diagnostic descriptions, including integer formatting, run in both modes. Object.ReferenceEquals now preserves alias, null and text-owner identity. Checked reference-array collections now pass growth, shared copies, replacement and GC tests. Reserved value arrays now run a tested Result collection consumer; Int32 HTTP status/flags enums now pass casts and collection-copy tests in both modes. Nested HTTP-result collections now pass pattern matching, copies and GC replacement tests; asynchronous execution remains incomplete. Stored callbacks and their ArrayList containers now execute natively with receiver retention, growth/copy/replacement and matching fault traces. The ordinary Raven socket listener now runs in both modes,
including local-port lookup, typed errors, close and host cleanup on faults. Async
accept/read/write, task integration and sustained
server execution are not yet qualified in AOT.

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

Once the native server works, compare interpreted and native neoCLR first, then ASP.NET
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
  This is not a general-purpose native publishing command for every neoCLR app.
- **Library coverage:** Hello World, unions, console/text operations and selected routing
  paths work. Listener creation and cleanup also work. Broader generics, async/task and
  socket accept/read/write paths still need admission and native service support. Interpreter API availability does not imply AOT availability.
- **Memory:** the opt-in collector is nonmoving and conservatively scans object payloads.
  It can retain extra objects, fragment its bounded buffer and collect too frequently.
  Private host root handles now retain allocations across guest frame returns, with
  explicit release and entry-reset checks. Private host dispatch can invoke retained
  zero-argument Void callbacks between guest calls. A native accept kernel passes
  loopback completion/cancellation tests, including compiled CIL callbacks. Transfer
  kernels also pass snapshot/copy-back, cancellation and EOF tests. A compiled CIL echo
  chains accept/receive/send across collections, including deadline-service calls.
  A Raven TaskQueue/Promise consumer preserves default and explicit queue behavior.
  Full-server recursive-call admission and automatic queue pumping remain open, along with
  precise maps, pressure scheduling and general hosting handles remain open.
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
HTTP now passes compiler admission with unit entry adaptation, but still needs automatic
queue pumping and an executed request. Runtime async
suspension and green threads remain separate design work.
