---
title: Native compilation
---
# Native compilation

neoCLR's experimental ahead-of-time (AOT) compiler produces standalone executables
on macOS ARM64 and Windows x64 for a supported subset of Raven applications,
including console programs and the HttpClient/HttpServer showcases. The interpreter
remains available; interpreter API support does not automatically imply native support.

## From Raven to an executable

Raven source is compiled to neoCLR metadata and instructions, then lowered to native
object code and linked with the required library and runtime support. The resulting
program needs no separately installed neoCLR or .NET runtime. Tested macOS binaries
use libSystem. Windows console EXEs import KERNEL32; the HTTP EXEs also use WS2_32
for networking. Windows builds link the CRT statically.

AOT moves compilation work to build time and produces an architecture-specific binary.
Memory management, text, I/O and fault handling still require runtime implementations
linked into that binary. See [architecture](../../architecture/) for those layers.

The development project workflow builds an ordinary Raven project with a matching
compiler/library bundle, links its executable and records diagnostics and dependencies.
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
- **Code and APIs:** broader generic/virtual dispatch, native introspection/reflection,
  Single operations, floating conversions and floating Math services are incomplete.
- **Async and hosting:** the HTTP profiles have a bounded completion loop, not a
  public scheduler. General suspension, green threads, stack migration and reusable
  application hosting remain work in progress.
- **Memory:** the nonmoving collector uses conservative payload scanning and a bounded
  buffer. It can retain extra objects and fragment storage; collection policy is provisional.
- **Reliability:** the tested failures preserve Fault details, traces and cleanup;
  broader disconnect, cancellation and hosting combinations still need qualification.
- **Deployment:** OS dependencies remain. Trimming, a stable public native ABI and a
  fully independent production-core bootstrap are unfinished.

JIT and native hot reload remain future work. See [project direction](../../proposals/#runtime)
and [garbage collection](../gc/) for related topics.

<a id="measurements-and-comparisons"></a>
<a id="development-follow-up-native-gc-lookup"></a>

## Performance measurements

The [benchmark report](../../benchmarks/) compares recorded interpreter and native
builds using routing and HTTP workloads. It includes methodology, measurements
and limitations. The cross-platform HTTP checks establish correctness, not a
performance ranking against .NET.
