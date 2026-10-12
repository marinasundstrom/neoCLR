---
title: Native compilation
---
# Native compilation

neoCLR's experimental ahead-of-time (AOT) compiler produces standalone executables
on macOS ARM64 and Windows x64 for a supported subset of Raven applications,
including console programs and the HttpClient/HttpServer showcases. The interpreter
remains available; interpreter API support does not automatically imply native support.

Native Raven sources use logical module declarations such as `module System.Collections`.
Modules organize declarations; assemblies remain packaging and binding identities.
Legacy `namespace` spelling remains accepted by the development compiler.

Development entry points also accept `Main(arguments: string[])`, with arguments
excluding the executable name. Empty strings, spaces and valid Unicode are preserved;
Windows hosts use wide command-line arguments rather than the active ANSI code page.
This requires the matching development AOT tool and native host adapters.
[Entry contract and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-entry-arguments.md)

## From Raven to an executable

Raven source is compiled to neoCLR metadata and instructions, then lowered to native
object code and linked with the required library and runtime support. The resulting
program needs no separately installed neoCLR or .NET runtime. Tested macOS binaries
use libSystem; development programs that query Culture.Current also link the system
CoreFoundation framework. Windows console EXEs import KERNEL32; the HTTP EXEs also use WS2_32
for networking. Windows builds link the CRT statically.

AOT moves compilation work to build time and produces an architecture-specific binary.
Memory management, text, I/O and fault handling still require runtime implementations
linked into that binary. See [architecture](../../architecture/) for those layers.

The development project workflow builds an ordinary Raven project with a matching
compiler/library bundle, links its executable and records diagnostics and dependencies.
The development runtime tests also exercise primitive numeric and Boolean parsing
in native macOS ARM64 and interpreted modes, using the same culture-independent
parser. Development civil date/time parsing also passes local native/interpreted
tests for three numeric culture profiles, with a host-preference parity probe.
Windows validation is configured; its result remains pending. Path and IP-address tests also cover class equality/hash overrides and
display through Object. Development AOT now also supports nongeneric reference-class
virtual dispatch, inherited implementations and direct base calls. The HTTP handler
pipeline exercises this on macOS ARM64. Generic class virtual dispatch and intrinsic
boxed/String Object equality/hash adapters remain incomplete. Windows execution of these newly
migrated suites remains pending.

Choose the profile for the host and workload:

| Host | Console profile | HTTP profile | Build tools |
| --- | --- | --- | --- |
| macOS ARM64 | `console` | `http` | Matching .NET SDKs and Apple's macOS tools |
| Windows x64 | `windows-console` | `windows-http` | Matching .NET SDKs and MSVC x64 tools |

The macOS development kit packages the compiler, libraries, AOT tool and native
adapters, so application builds need no source checkout or Cargo. Windows currently
uses the source-checkout workflow. These native build workflows are separate from
Preview 13; a packaged Windows native build kit remains future work.

[macOS development build instructions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-poc-bundle.md#development-project-to-executable-workflow-2026-10-09) ·
[Windows development build instructions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-poc-bundle.md#windows-x64-console-source-checkout)

<a id="development-http-platform-parity"></a>

## A working HTTP proof of concept

The client resolves `localhost`, connects and awaits a UTF-8 greeting. The server
accepts a loopback GET `/greeting`, sends `Café 🌍` and closes its listener. Both
programs run as separate standalone executables. Application code uses `await`,
including in `Main`; a unit-returning async function completes when it reaches the end.
Current samples use the development Task completion helpers and require
[rebuilt libraries](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-poc-bundle.md#rebuild-development-libraries-for-current-http-samples).

The tested server awaits one exchange and reports its outcome:

```raven
{{HTTP_SERVER_AWAIT_SAMPLE}}
```

Its entry point awaits the helper that opens the listener and starts serving:

```raven
{{NATIVE_HTTP_ENTRY_SAMPLE}}
```

The native host drives task continuations and I/O completions until the exchange
finishes. Explicit `OnCompleted` callbacks remain useful for completion adapters
and tests, but are not required by native application entry points.

The macOS and Windows checks compare native behavior with the interpreter for
successful and fragmented exchanges, malformed HTTP, refused connections, timeouts,
cancellation and guest faults. They also run a Raven client against a separate
Raven server, with only the executable in each deployment directory.

[Web and HTTP](../web/) shows client calls, response handlers and complete samples.
[Native HTTP build and validation instructions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-http-parity.md)
provide the development setup and retained evidence. A repeated-request server
workbench also exists; sustained load and general concurrent operation still need
qualification.

<a id="coverage-baseline"></a>

## Supported workloads

| Area | Current scope |
| --- | --- |
| Application data | Selected class, value, union, generic and collection programs |
| Text and console | UTF-8 input/output and supported string operations on both hosts |
| Files | Bounded UTF-8 file reads/writes and lexical paths on macOS |
| HTTP | Tested client DNS/connect/read/write and server accept/request/response paths on both hosts |
| Async | Selected Tasks and cancellation; async Main can await host I/O in the HTTP profiles |
| Arithmetic | Integer operations and a bounded Double arithmetic/comparison subset |

This describes tested workloads, not universal support for every API in those areas.
Development array comprehensions support a single unfiltered range with explicit
constant Int32 endpoints. Empty/reversed ascending ranges produce empty arrays;
inclusive maximum endpoints terminate correctly. Dynamic bounds, filters and mixed
collection elements remain unsupported by this native lowering. These are compiler
coverage gaps; the intended language semantics remain shared with Raven/.NET.

The compiler reports unsupported contracts. Use matching development compiler,
library and runtime artifacts; this is not a general publishing workflow for all apps.

The routing workbench also parses a route, extracts an integer parameter and prints
`42` in both interpreted and native execution:

```raven
{{NATIVE_ROUTING_SAMPLE}}
```

<a id="current-caveats"></a>

## Current limitations

- **Platform:** native Windows ARM64 is planned but unqualified. Windows file/path
  services and packaged native build kits remain open.
- **HTTP:** these bounded showcases do not establish native support for the entire
  Web API surface, TLS, HTTP/2 or arbitrary concurrent servers. The native transport
  uses IPv4, bounded DNS workers and polled socket operations.
- **Code and APIs:** development includes opaque closed type-token identity and the
  runtime type-equality, name, generic-arity and type-shape queries for primitive and closed
  nominal/vector token producers, plus concrete source-class/primitive-box `GetType()` identity,
  as first steps toward native JSON. Retained vectors support checked creation,
  length, readback and semantic `GetType()` identity; other compound queries remain unsupported.
  Explicit development reflection roots now support checked parameterless construction
  and property-access adapters. Explicit module-function and method-attribute roots
  also qualify namespace-scoped `TestAttribute` discovery on macOS ARM64 JIT/AOT,
  without invoking test bodies; dynamic registration remains separate work.
  On macOS ARM64 and Windows x64, explicit attribute-data retention
  also supports fixed/enum arguments and member/parameter annotations without
  executing attribute constructors. Named primitive field/property snapshots also pass
  on macOS with the updated source library; their Windows gate is pending. This metadata-authored gate still requires
  compiler-produced source annotation integration;
  development JSON round trips cover nested objects, typed arrays, lists/sequences,
  string-keyed maps and explicitly declared JSON nodes on macOS and Windows x64.
  These bounded, explicitly rooted cases are not a published native JSON capability.
  Development init accessors also work through native library imports and object
  initializers; ordinary later assignments are rejected by Raven. Runtime/reflection
  calls remain ordinary calls, so this is not object freezing. Full record
  equality, hashing and formatting remain unsupported.
  Broader generic/virtual dispatch, native introspection/reflection,
  Single operations, floating conversions and floating Math services are incomplete.
- **Async and hosting:** the HTTP profiles have a bounded completion loop, not a
  public scheduler. General suspension, green threads, stack migration and reusable
  application hosting remain work in progress. A private development experiment
  now transfers rooted callbacks into retained sessions with explicit shutdown.
  A bounded HTTP consumer preserves a counter and listener across three exchanges;
  general async export completion and reusable hosting remain unfinished.
- **Memory:** the nonmoving collector uses conservative payload scanning and a bounded
  buffer. It can retain extra objects and fragment storage; collection policy is provisional.
- **Reliability:** the tested failures preserve Fault details, traces and cleanup;
  broader disconnect, cancellation and hosting combinations still need qualification.
- **Deployment:** OS dependencies remain. Trimming, a stable public native ABI and a
  fully independent production-core bootstrap are unfinished.

Production JIT optimization and native hot reload remain future work. See [project direction](../../proposals/#runtime)
and [garbage collection](../gc/) for related topics.

<a id="measurements-and-comparisons"></a>
<a id="development-follow-up-native-gc-lookup"></a>

## Performance measurements

The [benchmark report](../../benchmarks/) compares recorded interpreter and native
builds using routing and HTTP workloads. It includes methodology, measurements
and limitations. The cross-platform HTTP checks establish correctness, not a
performance ranking against .NET.

The development JSON corpus additionally exercises two nested documents through
JsonValue and typed objects, plus typed scalar/object vectors, with macOS
interpreter/native parity. The same corpus now passes Windows x64 project execution. Lists/sequences and maps
remain under development; no comparative benchmark results are published yet.

## JIT investigation

Development-only macOS ARM64 JIT probes share verified lowering with AOT and execute
generated code in memory. The managed probe runs the real 20-test Index/Range assembly,
including filtering and bounds faults, with interpreter/AOT parity. Queued async entry
completion, consecutive awaits and fault cleanup now also pass a focused
[three-mode qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-async/README.md).
Timer completion, cancellation and fault cleanup also pass the existing timer suite.
[HTTP client/server qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-http/README.md)
now covers DNS/connect, accept/read/write, fragmented requests, timeout and fault cleanup
with interpreter/AOT parity. Development JIT bundles now default `neoclr run app.dll`
to JIT on macOS ARM64 and Windows x64; `--execution interpreter` remains available.
The installed bundle supplies a matching helper, compiler and libraries. The Windows
x64 gate exercises installed default execution; Windows ARM64 remains open.
[CLI contract and readiness](https://github.com/marinasundstrom/neoCLR/blob/main/docs/jit-execution.md).
Those gates establish functional behavior. A separate development
[HTTP execution-mode workbench](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/jit-http/README.md)
checks 32-request client/server samples in interpreter, JIT and AOT, with startup
separated from request timing. Its initial macOS ARM64 timing run had background
CPU contention and is exploratory; it does not establish Windows performance or
comparisons with other frameworks. An initial
[JSON node workbench](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/json/README.md)
uses Raven workers and separate serialization/deserialization batches. The
development framework also discovers `[Benchmark]` module functions from metadata
and generates typed registrations for AOT; the JSON deserializer uses this path
alongside manual serialization registration to retain its prebuilt input. It measures external batch latency including validation and protocol
costs; typed-object/collection benchmarks and internal operation timing remain open.
[Contracts, validation and local benchmark](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-managed/README.md).

A subsequent shared metadata lookup optimization reduces startup work in both the JIT
and interpreter; [before/after evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-managed/startup-optimization.md)
keeps process timing separate from execution speed.

The scalar probe also retains compiled code across repeated calls, with a
[checksum-validated CPU comparison](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/jit-cpu/README.md)
that measures invocation separately from startup. This remains a synthetic workload,
not service throughput or general JIT/AOT equivalence.

The intended direction is JIT by default where supported, AOT for deployment, and a fully
supported interpreter for constrained environments. Current execution defaults are unchanged.
Cross-target hot reload and explicit AOT/interpreter hybrid execution remain future work.


A separate opt-in [runtime-async generator POC](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/async-generators.md#generic-owner-and-lexical-metadata-checkpoint-2026-10-11)
now passes generic reference-class generators, pending awaits, delegation, cancellation
cleanup and live suspended-object GC in interpreter, macOS ARM64 JIT and AOT.
It requires the documented native Raven compiler branch and matching experimental
libraries. Abandoned-generator collection, Windows suspension and performance gates
remain open; this is development evidence, not a change to published execution defaults.


Development native Raven support also covers generic async methods and methods on
generic owners, with synchronous `use` resources retained across awaits. Runtime
bodies and generated state machines dispose on ordinary completion or cancellation;
Fault remains terminal. [Qualification and limits](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/runtime-async/generic-cleanup/README.md)
include pending GC, captured values and interpreter/JIT/AOT execution. Windows uses
the generated-state-machine path; runtime suspension there remains future work.
