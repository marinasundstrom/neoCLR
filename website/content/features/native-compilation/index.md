# Native compilation

Ahead-of-time compilation turns a supported Raven application into a native
executable. AOT is the intended deployment path; the development CLI also supports
JIT execution and an interpreter for testing and constrained environments.

<a id="jit-investigation"></a>

## Choose an execution mode

| Mode | Use it for | Compilation happens |
| --- | --- | --- |
| JIT | Development and running bytecode with the matching native backend | During execution |
| Interpreter | Portable execution and checking interpreter behavior | No native code compilation |
| AOT | Publishing a target-specific executable | Before deployment |

A matching development JIT bundle selects JIT by default. Select interpreter mode
explicitly when needed. API support in one mode does not automatically establish
support in every other mode or target.

## Build a native application

Raven produces neoCLR metadata and instructions. The AOT tool lowers them to object
code and links the required runtime services and libraries. The executable does not
need a separately installed neoCLR or .NET runtime, though it still uses host system
libraries. Build tooling requires the matching SDKs and native toolchain.

| Target | Console profile | HTTP profile | Native tools |
| --- | --- | --- | --- |
| macOS ARM64 | `console` | `http` | Apple's macOS tools |
| Windows x64 | `windows-console` | `windows-http` | MSVC x64 tools |

The macOS development kit supports project builds without a source checkout or
Cargo. Windows currently uses a source-checkout workflow. Windows ARM64 is not
qualified. See [development setup](/try/#development) and the
[project-to-executable instructions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-poc-bundle.md).

## Keep application code asynchronous

The HTTP server uses an ordinary async entry point:

```raven
{{NATIVE_HTTP_ENTRY_SAMPLE}}
```

The host drives completions while the program awaits I/O. Application entry points
do not need explicit completion callbacks. Arguments can be accepted through
`Main(arguments: string[])`; they exclude the executable name and preserve Unicode.

[HTTP client and server guide](/features/web/) ·
[Complete server case](/cases/http-server/)

## Retain metadata used dynamically

Directly referenced code is visible to the compiler. Reflection and object mapping
also need the types and members they discover at runtime. Development JIT accepts
`--reflection-roots roots.json` to retain them; packaged sample runners prepare the
appropriate roots. The interpreter does not accept that option.

Use matching compiler and library builds for native JSON, attributes and reflection.
Qualified examples cover specific supported shapes; they do not imply unrestricted
.NET reflection or support for arbitrary library code.

## Qualified HTTP workloads

Preview 14 passes the shared HTTPS/JSON consumer and HTTPS failure
checks on macOS ARM64 and Windows x64 in interpreter, JIT and AOT modes. The checks
cover certificate rejection and recovery, deadlines and cancellation. Windows also
passes independent TLS 1.2/1.3 interoperability cases.

Those published Windows JIT checks use compiler-generated async state machines.
Rebuilt development SDKs now select runtime-owned suspension with matching Windows
x64 and macOS ARM64 hosts; see the separate [development qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/runtime-product-integration.md#sdk-default-runtime-await-integration--2026-10-13).
The published Preview 14 evidence does not qualify that newer lowering. See
[release availability](/try/#development) and the
[qualification record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/release-20261013/README.md)
for the matching builds and tested scope.

## Current limits

Native support remains experimental. Generic virtual dispatch and some runtime
services have incomplete target coverage. The HTTP showcases are bounded HTTP/1.1
applications, not a promise of arbitrary concurrent server support. HTTPS needs
explicit credentials and trust, and qualification depends on the host and build kit.
See the [TLS contracts](/docs/ssl-streams.html).

AOT does not remove allocation, garbage collection, scheduling or I/O costs. Workload,
startup and deployment size should be measured separately.

[General execution benchmarks](/benchmarks/execution/) ·
[HTTP benchmark matrix](/benchmarks/http/) ·
[Runtime architecture](/architecture/)

<a id="development-http-platform-parity"></a>

<a id="coverage-baseline"></a>

<a id="current-caveats"></a>

<a id="measurements-and-comparisons"></a>

<a id="development-follow-up-native-gc-lookup"></a>
