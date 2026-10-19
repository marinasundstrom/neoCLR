# Class-library memory-view adoption audit

Author-directed follow-up, 2026-10-18. This is an adoption plan following the
[interpreter foundation](native-memory-views.md), not a claim that Raven/native
libraries have already migrated. Array range APIs now return views; broader buffer migrations remain planned.

## Scope and method

Scan all 233 authored Raven files under `runtime/raven/src`, across Runtime,
Data, Networking and Web. The initial explicit-array scan found 44 files;
including inferred allocations (`CheckedStorage`) and materialization (`ToArray`)
finds 47 files / 266 matching code lines (baseline before the new view files). The accompanying
[inventory](validation/memory-views-20261018/array-inventory.tsv) preserves paths,
line numbers and signatures/uses. Generated neoIL is not an independent API;
changes must originate in the owning Raven source and native project.
Also scan all 119 bootstrap/service neoIL files under runtime/System and
runtime/neoCLR: The inventory excludes the discarded temporary MemoryViews bootstrap draft.
The [bootstrap inventory](validation/memory-views-20261018/bootstrap-array-inventory.tsv)
covers hand-authored adapters absent from Raven source. The inventories are
 discovery aids, not proof that every array can be replaced.

Author clarification, 2026-10-19: do not reproduce compatibility-only .NET layers.
Choose the correct primary parameter/result types; avoid public AsSlice plumbing
and duplicate array overloads solely for compatibility. Explicitly owned or
array-specific contracts remain arrays.

Review calls by retention, mutation, contiguity and ownership. A returned owned
array and an array retained in a field have different contracts from a borrowed
input. Inferred internal buffers need the same review as public parameters.
Runtime service/FFI declarations must change with their consumers; converting an
array to a view immediately before a runtime call that requires an array does not
remove a copy or establish native support.

## Decisions by library area

| Source files (under System/) | Recommended direction | Reason / compatibility boundary |
| --- | --- | --- |
| `Array.rvn` | Use view results for range syntax, implicit array-to-view conversion and explicit copying; migrate syntax with compiler validation | Fixed contiguous ownership can be borrowed; implemented: Slice and range results share storage; ToArray copies |
| `Text/Decoder.rvn`, `Text/Encodings.rvn` | Decode(ReadOnlySlice<byte>, final) | Synchronous input borrow; carry bytes must still be copied into owned decoder state |
| `Text/Encoder.rvn`, `Text/Encodings.rvn` | Drain(Slice<byte>) | Writable synchronous destination; retain byte-count/progress and short-output semantics |
| `Security/Cryptography/{SHA256,HMACSHA256,RandomNumberGenerator}.rvn` | ReadOnlySlice for inputs/key/tag; Slice for hash/random destinations | Inputs not retained; preserve overlapping input/output, unchanged short destination and trailing bytes |
| `IO/Compression/GZipEncoder.rvn` | ReadOnlySlice input, owned byte[] output | Synchronous encoder consumes input; result and coding tables own storage |
| `IO/Compression/GZipDecoder.rvn` | Retained source stays array or ReadOnlyMemory; local decode helpers may use slices | Decoder keeps input in a field; tree tables and produced output remain owned |
| `IO/{InputStream,OutputStream}.rvn` | Memory<byte> read destination; ReadOnlyMemory<byte> write source | Methods return Task and can retain input until settlement; slices cannot cross suspension |
| `IO/{MemoryStream,FileInputStream,FileOutputStream}.rvn`, `Console/Streams.rvn` | Implement Memory-based async contracts; Slice/ReadOnlySlice for synchronous cores where applicable | Even immediately completing implementations must satisfy the asynchronous interface contract |
| `IO/{StreamReader,StreamWriter}.rvn` | Keep scratch array ownership; pass Memory to I/O and slices to codec calls | Reusable buffers must outlive awaits; decoder state must not borrow scratch after reuse |
| `IO/Compression/GZipStream.rvn` | Memory-based stream contract; keep decoded/scratch arrays | Stored output and async operation ownership are distinct from local windows |
| `Networking/Sockets/Socket.rvn` | Memory receive / ReadOnlyMemory send | Cancellation, short reads/writes, deadlines and completion-root release must preserve backing owner |
| `Networking/Security/SslStream.rvn` | Memory for retained plaintext/I/O; slice only inside synchronous packet helpers | Pump crosses await; empty owner-token arrays are resource identity, not byte payloads |
| `Networking/Security/SslOptions.rvn` | ReadOnlySlice PEM inputs; slices for ReadCount/WriteCount; keep session/config owner tokens and allocated output | PEM factories explicitly do not retain input; resource handles must retain identity |
| `Web/Http/HttpConnection.rvn` | Memory-based read/write/receive contracts | Adapter inherits underlying I/O lifetime and cancellation rules |
| `Web/Http/HttpServer.rvn` | ReadOnlySlice for synchronous decoder Push/PushCore; Memory for active transfers; keep rented arrays and encoded response ownership | Parser must copy retained headers/body before receive buffer reuse; returning a rented array is ownership transfer |
| `Web/Http/{HttpBodyInput,HttpClient}.rvn` | Memory for retained windows and upload/read operations | Body state and callbacks outlive each synchronous call; keep reusable owned buffers |
| `Networking/IPAddress.rvn` | Slice for synchronous ReadIPv4 destination; keep owned octets | Address objects must not alias caller-mutable input accidentally |
| `String.rvn` | ReadOnlySlice<string> Join overload is a candidate; retain String/grapheme semantics | Element text remains immutable; UTF-8 view is separate and byte-relative |
| `Text/{StringBuilder,Utf8}.rvn` | Keep owned builder/materialized storage; expose borrowed input only at synchronous boundaries | A mutable builder can grow; views need invalidation/ownership rules before exposing backing storage |
| `Collections/{ArrayList,HashMap,HashSet}.rvn` | Keep backing arrays; consider explicit bounded views only after growth/invalidation contract | Replacement on resize and mutation invalidate assumptions; arbitrary collection is not necessarily contiguous |
| `Collections/{SequenceAccess,List}.rvn` | Keep copying range contract for general Sequence/List | Interface does not guarantee contiguous storage; do not promise zero-copy views |
| `Linq/Operators.rvn` | Keep materialized results and captured buffers; consider slice-specialized eager algorithms separately | Deferred execution/closures cannot capture slices; ToArray must continue to allocate owned output |
| `Runtime/CompilerServices/RuntimeHelpers.rvn` | GetSlice returns a borrowed view; retain internal CopyRange for collection copies | Compiler must intentionally select aliasing vs copying; silently changing helper semantics breaks callers |
| `Runtime/Reflection/{ArrayReflection,ReflectionExtensions,FunctionReflection}.rvn` | Keep params-array adapters; investigate readonly borrowed arguments only with dispatch/lifetime support | Invocation can enter arbitrary code and suspend; current byref-like values are not reflection-boxable |
| `Introspection/{Descriptors,ParameterInfo}.rvn` | Keep owned metadata arrays/snapshots | Snapshots and descriptor fields outlive queries; caller mutation must not alter metadata |
| `Environment/Functions.rvn` | Keep owned command-line result | Returned snapshot owns independent storage; no useful borrowed lifetime contract |
| `Data/Json/{CollectionCalls,CollectionAdapters,ObjectMapper}.rvn` | Keep materialization/owned reflection arrays; evaluate synchronous slices only with reflection support | Newly constructed collections own values; invocation bridge is not slice-aware |
| `Data/Sql/SqlParameters.rvn` | ReadOnlySlice<Object?> candidate for synchronous statement creation | Statement must own/copy any retained parameters; preserve placeholders and validation |
| `Data/Sql/SqlMapper.rvn` | Keep params adapters and use owned/Memory parameters if retained across async query | Task lifetime forbids a borrowed parameter escaping into its state |

## Bootstrap and runtime boundary review

- `System/Collections/Iterator.neoil` retains array storage in iterator fields:
  keep an owner/reference or Memory, never Slice. `System/Function.neoil` ForEach
  takes a callback; a slice overload needs a callback non-suspension contract or
  must fault on suspension. Do not silently narrow the existing array API.
- `System/{Environment,Reflection,Type,TypeInfo}.neoil` and runtime assembly/type/
  enum/reflection services return owned snapshots or invoke arbitrary code. Keep
  output arrays and existing argument adapters; apply the reflection gate above.
- `System/LocalDateTime.neoil`, runtime `UnixTimeTicks` and `TimeZones` use array
  results as transport for fixed date/time parts or variable local-time mappings.
  A caller-owned slice is not a valid replacement for newly returned data. A
  future typed value/tuple result is separate API cleanup, not slice migration.
- Runtime `Dns` results are owned address snapshots retained across async work.
  Runtime `StringScalars`/`StringGraphemes` materialize semantic text elements:
  keep results; `StringFromChars` and `StringJoinParts` can accept synchronous
  readonly views after their runtime contracts support them. `Utf8Decode` is a
  readonly-byte-view candidate; keep Utf8Encode's owned-result semantics and use
  the separate shared String.Utf8Bytes only where an immutable borrowed result fits.
- Runtime `ConsoleStreams`, `FileStreams`, `Sockets`, `Tls` and `Cryptography`
  declarations must migrate with the source contracts and native implementation.
  Async completion must trace the actual Memory owner, not a temporary slice.
- Bootstrap ArrayList wrappers retain the same growth/iterator invalidation gate
  as Raven collections. Internal view factories intentionally retain their array
  inputs: they are the boundary from owning storage to borrowing, not candidates
  for recursive replacement with slices.

## Order and acceptance

1. Raven/compiler APIs and interpreter metadata/GC support now cover all four
   types. The bounded native persistent-byte path now covers creation, length,
   explicit integer and Range windows, readonly conversion and traced field/call storage. Finish borrowed
   views and remaining JIT/AOT operations before migrating shared buffer services.
   Interpreter-only support is insufficient for native HTTP and crypto providers.
2. Migrate codec and cryptography synchronous contracts first, without redundant
   compatibility overloads. Test subwindows, empty input, overlapping buffers, readonly
   aliasing, destination sentinels, bounds and no mutation on documented failures.
3. Migrate InputStream/OutputStream and implementations together to Memory. Test
   retained windows under forced GC, cancellation, partial progress, disposal and
   faults. Preserve complete-write vs partial-read semantics.
4. Follow with sockets/TLS/HTTP. Test buffer reuse only after completion, pooled
   owner release, and parser copying of bytes retained beyond Push. Use existing
   wire/cancellation suites; benchmark a real workload before claiming savings.
5. Revisit general collections, reflection and compiler range syntax separately.
   Do not migrate unrelated array outputs simply to remove array spelling.

Each behavior change is a separate validated commit, with matching API reference
and compiler documentation where contracts change. The audit authorizes planning;
it does not make every candidate an unconditional signature replacement.

## .NET comparison and deliberate differences

.NET's [Memory/Span usage guidance](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/memory-t-usage-guidelines)
separates synchronous borrowing from retained asynchronous use. neoCLR follows
that ownership distinction, using Slice names instead of Span. Slices do not
transfer ownership and Memory does not make concurrent mutation safe.

[SHA256.HashData](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.sha256.hashdata)
provides readonly input and writable destination span APIs; this supports the
crypto candidates above. [MemoryStream.ReadAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.memorystream.readasync)
uses writable Memory for async input. neoCLR retains its separate InputStream /
OutputStream and Task<Result<...>> contracts; adopting views does not silently
adopt .NET exception/ValueTask or combined Stream semantics. String remains UTF-8
with grapheme indexing; a byte view is not a text slice. Sources checked 2026-10-18.
