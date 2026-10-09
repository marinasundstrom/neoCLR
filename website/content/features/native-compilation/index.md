---
title: Native compilation
---
# Native compilation

neoCLR's experimental ahead-of-time (AOT) compiler produces standalone ARM64 executables
for a supported subset of Raven applications. The interpreter remains available;
interpreter API support does not automatically imply native support.

## From Raven to an executable

Raven source is compiled to neoCLR metadata and instructions, then lowered to native
object code and linked with the required library and runtime support. Tested macOS
executables use the OS's libSystem and need no separately installed neoCLR or .NET
runtime to run. The compiler toolchain has its own requirements.

AOT moves compilation work to build time and produces an architecture-specific binary.
Memory management, text, I/O and fault handling still require runtime implementations
linked into that binary. See [architecture](../../architecture/) for those layers.

Development builds now have a bounded project-to-executable console workflow on
macOS ARM64. It builds an ordinary Raven project with a selected native bundle,
links a standalone executable and records build diagnostics and dependencies.
A separate development kit now packages the compiler, libraries, AOT tool and
native adapters, so application builds need no source checkout or Cargo. Building
still requires the compiler's .NET SDKs and Apple's macOS tools; the resulting
executable needs only the OS libraries. The kit is not included in Preview 13. See the
[development build instructions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-poc-bundle.md#development-project-to-executable-workflow-2026-10-09)
for the supported console profile and prerequisites.

The development kit also offers an explicit `--profile http` project build for the
existing one-request HTTP sample. It links the bounded socket/task host and native
GC support; the console profile remains the default. This makes the POC easier to
build, without claiming a production scheduler, runtime suspension or green threads.

## A working HTTP proof of concept

This library example parses a route, extracts an integer parameter and prints `42`
in both interpreted and native execution:

```raven
{{NATIVE_ROUTING_SAMPLE}}
```

The HTTP server example accepts a loopback GET `/greeting` and returns the UTF-8 body
`Café 🌍`. A repeated-request version serves sequential connections in one process.
These examples exercise compiled library code, sockets, queues and managed memory.
Sustained load and general concurrent server operation still need qualification.

[Web and HTTP](../web/) explains the APIs. The
[native examples and build instructions](https://github.com/marinasundstrom/neoCLR/tree/main/benchmarks/native-web)
provide the development setup for these executables.

<a id="coverage-baseline"></a>

## Supported workloads

| Area | Current scope |
| --- | --- |
| Application data | Selected class, value, union, generic and collection programs |
| Text and console | UTF-8 input/output and supported string operations |
| Files | Bounded UTF-8 file reads/writes and lexical paths on macOS |
| HTTP | Routing and the tested accept/read/write/task-completion path |
| Async | Selected Tasks and cancellation cases; queue-only async entry draining |
| Arithmetic | Integer operations and a bounded Double arithmetic/comparison subset |

This describes tested workloads, not universal support for every API in those areas.
The compiler reports unsupported contracts. Use matching development compiler,
library and runtime artifacts; this is not a general publishing workflow for all apps.

<a id="current-caveats"></a>

## Current limitations

- **Platform:** console/HTTP project-kit qualification is on macOS ARM64. Native Windows path
  behavior and broader platform support remain open.
  A separate development Windows x64 scalar/literal-console profile now passes
  MSVC linking and native/interpreter comparisons, including standalone Hello World
  from both retained metadata and freshly compiled Raven source on Windows, through
  PE/#Neo and NEOX. Fresh-source coverage also passes calls, branches, empty/repeated
  loops and byte-exact UTF-8/NUL output. Scalar comparison results retain typed
  Boolean stack checks; this remains the primitive/console bootstrap. Windows managed services and
  native project kits remain unsupported.
- **Code and APIs:** broader generic/virtual dispatch, native introspection/reflection,
  Single operations, floating conversions and floating Math services are incomplete.
- **Async:** async entry points waiting for host I/O are unsupported; selected
  queue-driven cases do not establish general async parity.
- **Memory:** the nonmoving collector uses conservative payload scanning and a bounded
  buffer. It can retain extra objects and fragment storage; collection policy is provisional.
- **Reliability:** supported paths preserve Fault details and stack traces, but not every
  disconnect, cancellation or cleanup path is qualified.
- **Deployment:** OS dependencies remain. Trimming, a stable public native ABI and a
  fully independent production-core bootstrap are unfinished.

JIT and native hot reload remain future work. See [project direction](../../proposals/#runtime)
and [garbage collection](../gc/) for related topics.

<a id="measurements-and-comparisons"></a>

<a id="development-follow-up-native-gc-lookup"></a>

## Performance measurements

The [benchmark report](../../benchmarks/) compares recorded interpreter and native
builds using routing and HTTP workloads. It includes the methodology, measurements
and limitations. No performance ranking against .NET is established.
