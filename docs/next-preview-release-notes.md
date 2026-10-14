# Next preview

neoCLR brings together its interpreter, JIT, native compiler, scheduler and memory
management. JIT is the default for development; AOT builds native applications.

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
