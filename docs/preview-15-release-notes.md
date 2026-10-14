# Preview 15: a unified runtime

This preview brings the components developed as separate experiments into one
integrated neoCLR runtime. It is the central change in this release, alongside
improvements to HTTP processing, memory management and development tools.

## One runtime, modular components

Execution through the interpreter and JIT, garbage collection, and scheduling
with suspension and resumption now form a shared runtime foundation. Native AOT
is the publishing target, using that foundation in compiled applications.

The components keep their own responsibilities and boundaries. Integration lets
us develop, validate and optimize them together, and gives profiling and debugging
tools a common runtime to observe. JIT is the default for development; AOT builds
native applications. Interpreter execution remains supported.

## Runtime await

Raven's native target uses runtime-owned suspension and resumption for `await`,
including async `Main`. The same programming model works across interpreter, JIT
and AOT. Windows x64 uses fibers to preserve suspended execution.

## HTTP and memory management

HTTP parsing scans URI components and known header names as bytes, reducing
intermediate strings. Common header names share lazily initialized values.

The collector reduces repeated heap inspection and tracing work for strings and
byte buffers. Allocation helpers preserve live inputs when collection occurs.

## Libraries and tools

JSON values can produce compact or indented JSON through `ToJsonString` and
`JsonSerializerOptions.WriteIndented`. String and stream serializers share the
same formatting options.

HTTP JSON helpers preserve `HttpError | JsonError`, and object-mapping examples
retain the original error. The API reference includes links to declaration files
on GitHub, including PostgreSQL APIs.

## Updating

Use the matching SDK, extension, runtime and libraries, and rebuild applications.
Native hosts also require a rebuild for the updated runtime context. The native
compiler tool is now named `neoclr-codegen`.

Task APIs live in `System.Concurrency.Tasks`. Stream operations return tasks;
use `await` to consume them.

## Scope

neoCLR is experimental: APIs and metadata may change. The release targets macOS
ARM64 and Windows x64. HTTP supports HTTP/1.1; HTTP/2/3 and connection pooling
are not available. PostgreSQL is a local development provider with loopback/trust
authentication, not a general production database connection.

## Downloads

Choose the matched macOS ARM64 or Windows x64 bundle below. Each includes the
runtime, SDK, native libraries, VS Code extension and executable samples. The
standalone SDK and extension assets are provided for custom installations; keep
their matching tool notices. SHA256SUMS covers all attached artifacts.

The macOS bundle also includes the optional PostgreSQL provider. The Windows
bundle includes the four base libraries. See the [qualification record](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.15/docs/validation/release-20261014/README.md)
for checks and component revisions.
