# Native compilation

Ahead-of-time compilation turns a supported Raven application into a native
executable. AOT is the intended deployment path; the development CLI also supports
JIT execution and an interpreter for testing and constrained environments.

<a id="jit-investigation"></a>

## Development: default method inlining

JIT and AOT enable the backend’s speed optimizations and independently inline small,
safe integer/Boolean leaf methods. Instruction selection follows the target architecture.
`MethodImplOptions.NoInlining` opts out; `AggressiveInlining` permits a larger
candidate budget but cannot bypass safety limits. There is no `Inlining` flag.
Calls, branches, allocation, virtual dispatch and faulting operations currently
remain ordinary calls. Small SDK helpers benefit without annotations. These are
unreleased development changes; see the [contract and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/inlining-20261028/README.md).

## Choose an execution mode

| Mode | Use it for | Compilation happens |
| --- | --- | --- |
| JIT | Development and running bytecode with the matching native backend | During execution |
| Interpreter | Portable execution and checking interpreter behavior | No native code compilation |
| AOT | Publishing a target-specific executable | Before deployment |

A matching development JIT bundle selects JIT by default. Select interpreter mode
explicitly when needed. API support in one mode does not automatically establish
support in every other mode or target.

## Development: lazy JIT

Post-Preview-18 source builds have an opt-in lazy backend (`NEOCLR_JIT_LAZY=1`).
Method bodies are lowered and compiled to machine code on first call through
stable trampolines; metadata verification remains eager. This can reduce preparation work but adds cold-call
latency and retains an indirect call after warmup. Eager compilation and synchronous
GC remain the defaults. This is not a published Preview 18 capability.

[Design and matched startup/HTTP measurements](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/lazy-jit-20261026/README.md)
cover synchronous and concurrent GC separately, including limitations.

The development follow-up uses a bounded arena with system fallback for lazy JIT
on macOS ARM64. It improves measured warm HTTP throughput while retaining first-use
compilation waits. A compiler worker does not yet let other managed tasks proceed
during that wait. [Profiling and workload tradeoffs](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/lazy-jit-optimization-20261026/README.md)
compare both collectors and explain why startup, tail latency, CPU and sustainable
load must be considered separately.


Shared native selection and generic expansion reuse temporary immutable metadata indexes, preserving
verification and lazy method counts. The [paired startup follow-up](/benchmarks/http/)
records improvements and regressions separately for eager/lazy JIT and both collectors.


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
Cargo. Its development HTTP profile includes a prebuilt native TLS provider needed
by the current networking code; explicit HTTPS-server configuration remains
source-only. Windows currently uses a source-checkout workflow. Windows ARM64 is not
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

Development native project builds now read `<project>.retention.json` beside the
`.rvnproj` file. Each entry declares an exact type name and the capabilities needed:

```json
{
  "schemaVersion": 1,
  "types": [
    {
      "name": "Examples.DependencyInjection.Greeter",
      "retain": ["constructors"]
    }
  ]
}
```

This is the checked-in policy used by the native DI sample. Supported modes are
`constructors` (checked activation), `properties` (construction plus property
getters/setters for object mapping), and `methods` (inspection only). They can be
combined. `--retention-file config/native.json` selects a different file relative
to the project directory. Invalid declarations and conflicts with legacy
reflection-policy arguments fail the build. Generic definitions retain only the
supported closed shapes known to compilation. This does not add automatic JIT
file discovery.

Development `DynamicallyAccessedMembers` analysis can now infer constructor/property
retention from known type tokens passed to annotated `TypeInfo` parameters or returned from annotated factories, fields and properties. JSON
deserialization and DI registration APIs carry corresponding requirements.
Unknown flows produce diagnostics; generic-parameter, return, field and receiver
annotations remain unsupported. Explicit files are still needed for unsupported
flows. [Attribute API reference](/docs/api/System/Diagnostics/CodeAnalysis/DynamicallyAccessedMembersAttribute/).
Inferred constructors and property accessors are also analyzed for
further annotated calls, including cycles; types discovered through arbitrary
storage or reflection descriptors still need explicit retention. Executable bodies
admitted by project retention files also seed annotation analysis; metadata-only
policies do not gain invocation rights. Nongeneric class/interface parameter
annotation mismatches are reported as `DAM2005`. Nongeneric reference-class overrides
are checked against their nearest base declaration with `DAM2006`; generic mappings,
generic derived classes and value-type overrides remain outside these checks.
Closed generic base classes of nongeneric derived classes are included in DAM2006;
this does not add generic-parameter annotations.

Use matching compiler and library builds for native JSON, attributes and reflection.
Qualified examples cover specific supported shapes; they do not imply unrestricted
.NET reflection or support for arbitrary library code.

## Qualified HTTP workloads

The shared HTTPS/JSON consumer and failure checks cover macOS ARM64 and Windows
x64 in interpreter, JIT and AOT modes, including certificate rejection, recovery,
deadlines and cancellation. Windows also has TLS 1.2/1.3 interoperability checks.

Coverage depends on the exact runtime, compiler and async lowering. See the
[qualification record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/release-20261013/README.md)
and [runtime-await qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/runtime-product-integration.md#sdk-default-runtime-await-integration--2026-10-13)
for tested builds. [Downloads](/try/) identify published availability.

## Measured output and installation size

Development measurements from **21 October 2026**, on **macOS ARM64**. These are
real outputs from two tested programs, not promised sizes for every application.

| Application | neoCLR application assembly | Complete AOT executable |
| --- | ---: | ---: |
| Unicode console greeting | 85.04 KiB | 94.96 KiB |
| JSON object-mapping sample | 380.28 KiB | 9.37 MiB |

Both executables ran successfully and depend only on macOS **libSystem**. The AOT
column includes linked runtime services and selected application/library code.
The assembly column excludes the shared class libraries and execution engine it
needs to run; it is **not a complete deployment size**. These builds use the
release backend and the default `clang -O2` link configuration, with no extra
stripping step. The JSON sample retains its object-mapping metadata.

### Build tools are a separate installation cost

| Development package | Installed file bytes | Compressed archive |
| --- | ---: | ---: |
| Raven SDK: compiler and language server | 89.98 MiB | 31.69 MiB |
| Complete neoCLR JIT toolchain bundle, including that SDK | 151.93 MiB | 53.48 MiB |

The complete bundle also includes the runtime/backend, class libraries, editor
extension, samples and documentation. Its class-library directory accounts for
19.14 MiB, including documentation. These are overlapping quantities, not amounts
to add together. Sizes use logical file lengths (1 KiB = 1,024 bytes;
1 MiB = 1,048,576 bytes), excluding filesystem block rounding and generated caches.
External .NET SDKs and Apple build tools are prerequisites and **are not included**
in these installation figures. Neither SDK is needed to run the AOT executables.

The JSON example is substantially larger than the greeting: this baseline makes
that cost visible rather than implying that every native application is tiny.
Further size optimization needs its own measurements. This study does not qualify
HTTP workloads or establish Windows output sizes.

[Exact byte counts, hashes and build inputs](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-size/macos-arm64-20261021.json) ·
[Measurement method and reproduction](https://github.com/marinasundstrom/neoCLR/blob/main/benchmarks/native-size/README.md)

## Current limits

Development after Preview 16: the native backend and matching host now admit up to
4,096 selected methods. The rebuilt macOS ARM64 HTTP JSON sample uses 2,055 methods
and passes server requests and a client/server exchange in both JIT and standalone
AOT. This is source-build validation; refreshed installed packages and Windows
qualification remain pending. The bound is an experimental backend limit, not a
Raven language rule.

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


## Native interoperability direction

Native compilation is the primary deployment direction for Raven on neoCLR;
JIT and interpretation remain useful execution options. Development NativeImport
support covers the interpreter and an opt-in macOS ARM64 scalar AOT proof for free
Int32/native-sized-integer/pointer C functions, now qualified from Raven source with explicit metadata
dependencies and C allocation/release. The development scalar JIT probe now reuses
the same import path behind explicit opt-in, including Raven source metadata with
explicit verification dependencies. Full managed-program AOT imports,
managed JIT imports and managed-memory
pinning remain gaps. [Current scope](/features/native-interop/#development-native-execution).

The development interpreter also provides explicit `NativeMemory.Copy` overloads
for slices and tracked native allocations, including Memory windows and UTF-8 byte
views. These avoid an intermediate managed transfer array; they still copy bytes.
[Copy contract and limitations](/docs/native-interop.html#native-byte-copies)

The next foundation connects native calls with pointers, stack and heap allocation,
and explicitly scoped access to managed buffers. A true pin must retain existing
storage at a stable native-compatible address; an explicit copy is a separate operation.

Longer term, referenceable metadata for native implementations, an extended native
hosting API and dynamic dispatch should let different languages use runtime services
and each other's components. Those languages need not share one memory-management
strategy. These are future directions, not available component APIs. Trusted native
imports also remain distinct from sandboxed VM execution.

[Interop direction and completion gates](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/native-import-export-contract.md#language-neutral-interoperability-direction--2026-10-22)

A source-only development experiment runs 1–4 isolated HTTP application workers
against one listening socket. Heaps, statics and async state remain per worker;
shared-object GC qualification is a separate gate before shared-heap parallelism.
This provides server primitives, with no new application framework or change to
the default host. [Experimental contract and evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/http-worker-integration/README.md).

The private worker experiment tests concurrent native reads and separately
synchronized native mutations of shared objects. GC waits for active phases to
yield, retaining explicitly published roots. An opt-in compiler profile now also runs synchronous generated callbacks on one
shared heap, with participant root publication and coordinated GC. Optimized/UBSan
tests cover four active participants, disjoint graph mutation and isolated faults.
Allocation/helpers remain serialized; shared async HTTP and throughput qualification
remain open. [Development evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-worker-pool/README.md#concurrent-compiled-shared-heap--2026-10-23).
