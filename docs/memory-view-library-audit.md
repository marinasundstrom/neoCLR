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
array-specific contracts remain arrays. The author also expects this consistent
usage to reach HTTP Server and Client APIs. Follow the stream/socket dependencies
below: active transfers retain Memory owners until completion, synchronous parsing
borrows slices, and parser-owned or returned materialized bodies keep ownership.
This does not imply automatic pooling, concurrent mutation safety or zero-copy
transport.

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
| `Text/Decoder.rvn`, `Text/Encodings.rvn` | Implemented: Decode(ReadOnlySlice<byte>, final) | Synchronous input borrow; carry bytes must still be copied into owned decoder state |
| `Text/Encoder.rvn`, `Text/Encodings.rvn` | Implemented: Drain(Slice<byte>) | Writable synchronous destination; retain byte-count/progress and short-output semantics |
| `Security/Cryptography/{SHA256,HMACSHA256,RandomNumberGenerator}.rvn` | Migrated: ReadOnlySlice inputs/key/tag; Slice hash/random destinations | Inputs not retained; preserve overlapping input/output, unchanged short destination and trailing bytes |
| `IO/Compression/GZipEncoder.rvn` | Migrated: ReadOnlySlice input and CRC windows; owned byte[] output | Synchronous encoder consumes input; result and coding tables own storage |
| `IO/Compression/GZipDecoder.rvn` | Migrated: ReadOnlyMemory source field and synchronous readonly slice access | Decoder keeps input in a field; tree tables and produced output remain owned |
| `IO/{InputStream,OutputStream}.rvn` | Memory<byte> read destination; ReadOnlyMemory<byte> write source | Methods return Task and can retain input until settlement; slices cannot cross suspension |
| `IO/{MemoryStream,FileInputStream,FileOutputStream}.rvn`, `Console/Streams.rvn` | Implement Memory-based async contracts; Slice/ReadOnlySlice for synchronous cores where applicable | Even immediately completing implementations must satisfy the asynchronous interface contract |
| `IO/{StreamReader,StreamWriter}.rvn` | Keep scratch array ownership; pass Memory to I/O and slices to codec calls | Reusable buffers must outlive awaits; decoder state must not borrow scratch after reuse |
| `IO/Compression/GZipStream.rvn` | Memory-based stream contract; keep decoded/scratch arrays | Stored output and async operation ownership are distinct from local windows |
| `Networking/Sockets/Socket.rvn` | Migrated: Memory receive / ReadOnlyMemory send | Cancellation, short reads/writes, deadlines and completion-root release must preserve backing owner |
| `Networking/Security/SslStream.rvn` | Migrated internal plaintext windows and synchronous copy/control slices; public I/O contracts pending | Pump crosses await; empty owner-token arrays are resource identity, not byte payloads |
| `Networking/Security/SslOptions.rvn` | ReadOnlySlice PEM inputs; ReadCount/WriteCount slices implemented; keep session/config owner tokens and allocated output | PEM factories explicitly do not retain input; resource handles must retain identity |
| `Web/Http/HttpConnection.rvn` | Memory-based read/write/receive contracts | Adapter inherits underlying I/O lifetime and cancellation rules |
| `Web/Http/HttpServer.rvn` | Migrated: ReadOnlySlice decoder Push/PushCore; active transfer signatures pending; keep rented arrays and encoded response ownership | Parser must copy retained headers/body before receive buffer reuse; returning a rented array is ownership transfer |
| `Web/Http/{HttpBodyInput,HttpClient}.rvn` | Migrated: Memory body-read destination; upload/read signatures pending | Body state and callbacks outlive each synchronous call; keep reusable owned buffers |
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
- Runtime `ConsoleStreams`, `FileStreams`, `Sockets` and `Tls`
  declarations must migrate with the source contracts and native implementation.
  Cryptography now uses checked slices through the interpreter/native boundary.
  Async completion must trace the actual Memory owner, not a temporary slice.
- Bootstrap ArrayList wrappers retain the same growth/iterator invalidation gate
  as Raven collections. Internal view factories intentionally retain their array
  inputs: they are the boundary from owning storage to borrowing, not candidates
  for recursive replacement with slices.

## Order and acceptance

1. Raven/compiler APIs and interpreter metadata/GC support now cover all four
   types. The bounded native persistent-byte path now covers creation, length,
   explicit integer and Range windows, readonly conversion and traced field/call storage.
   Array-backed borrowed byte views now have native indexing, updates, windows,
   readonly conversion and persistent borrowing. Qualify each migrated consumer
   and its service contracts; native asynchronous retention remains a separate gate.
   Interpreter-only support is insufficient for native HTTP and crypto providers.
2. Incremental codec signatures and their stream-adapter calls now use slices.
   Cryptography synchronous contracts now use readonly input/key/tag slices and
   writable hash/random destination slices in interpreter/JIT/AOT, without redundant
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

### Cryptography integration (2026-10-19)

SHA256.HashData, HMACSHA256.HashData/VerifyData and RandomNumberGenerator.Fill
now use borrowed byte slices end to end, including exact runtime service binding.
Native providers access checked array-backed windows without copying; interpreter
inputs retain their existing provider-copy implementation. Alias-safe hash/MAC
publication, short-destination results, untouched tails and random-failure clearing
apply to the selected window. Whole arrays still convert implicitly. There are no
array overloads or public AsSlice adapters. Native string-backed UTF-8 views and
non-byte views remain separate backend gaps. Async streams/socket Memory retention
is the next integration step; this change does not qualify HTTP transfer lifetimes.

[Validation](validation/memory-views-20261019/crypto-integration-validation.json):
three public Raven tests in all modes, twelve private-service cases in all modes,
seven interpreter tests, exact native binding checks, sanitized window/provider
failure checks, installed default-JIT execution and matching API snapshots.

### Managed async lifetime prerequisite (2026-10-19)

The [managed suspension qualification](native-memory-views.md#persistent-byte-memory-across-managed-suspension-2026-10-19)
now passes four Memory/ReadOnlyMemory cases in interpreter/JIT/AOT (12 executions)
and preserves both borrowed-capture diagnostics. Raven admits persistent captures
and local-copy instance receivers for native async entry; explicit progress promises
separate the two suspension phases. This closes the managed owner-retention gate
for byte windows, not native I/O retention or completion-root release. Step 3 above
remains the next API migration; no production stream/socket signature changes here.

### Gzip implementation windows (2026-10-19)

The encoder and CRC helper consume ReadOnlySlice<byte>. GZipStream retains a
ReadOnlyMemory<byte> window across output awaits, then borrows each bounded block
for synchronous encoding and CRC accumulation. The decoder stores ReadOnlyMemory
and borrows locally for byte access; header CRC uses a checked window. Compression
output, decoded output and code tables remain owned storage. This follows the
same .NET borrowed-versus-retained distinction already documented above.

This is an internal migration, with no new public overloads or AsSlice helpers.
The shared InputStream/OutputStream range signatures still await migration together
with file/console/socket services. The change neither removes the decoder's input
buffering nor claims zero-copy transport or improved throughput.

[Validation](validation/memory-views-20261019/gzip-integration-validation.json):
seven gzip groups pass in AOT and JIT, and two focused interpreter cases pass.
The independent output interoperability rerun is blocked by native emission of
its unchanged async sample; the full interpreter suite retains its earlier timeout.

### Interpreter file transfer boundary (2026-10-19)

Production `RuntimeCoreCalls.rvn` now declares `FileReadInto(int, Memory<byte>)`
and `FileWriteChunk(int, ReadOnlyMemory<byte>)`. FileInputStream/FileOutputStream
validate their existing public array/range contracts before constructing windows;
no new public overload is added. This temporary boundary will disappear when the
shared InputStream/OutputStream contracts migrate together. Four-argument internal
bindings remain only for legacy bootstrap/test consumers; their declarations are
not the current Raven class-library API.

The interpreter releases heap access while blocking file I/O runs. It roots the
persistent read descriptor until completion and writes back only the returned
prefix, rejecting an oversized provider result before modifying bytes. Writes
snapshot only the selected initialized window before releasing heap access; the
snapshot has no managed roots and uses the existing array-payload budget. This
preserves the established detached-call concurrency model, rather than retaining
raw pointers or borrowed slices across it. Copying on writes remains an explicit
cost; no throughput or zero-copy claim is made.

The .NET comparison remains persistent Memory for potentially retained operations
and temporary borrowing under controlled heap access. Native JIT/AOT file-service
execution, asynchronous OS completion and public stream/socket signatures are not
qualified by this interpreter work. See the
[consumer](../runtime/raven/tests/file-memory/README.md) and
[evidence](validation/memory-views-20261019/file-integration-validation.json).

Validation passes five host-adapter tests, twelve legacy file-service tests and
the production Raven file round trip with exact output bytes. The read-root test
collects from another heap participant while the caller's roots are published;
the snapshot test allows uninitialized bytes outside the selected source window.
Reference regeneration leaves public assembly contents unchanged and refreshes
source fingerprints. No JIT/AOT or asynchronous OS file-I/O result is inferred.

## Console output memory boundary (2026-10-19)

Production Raven console output now passes `ReadOnlyMemory<byte>` to its private
runtime service. Public OutputStream signatures remain unchanged for the atomic
shared stream/socket migration. The facade preserves cancellation, closed, range
and transfer-limit precedence before creating the selected window.

The interpreter uses the same bounded selected-byte snapshot as file output before
releasing graph access. Native JIT/AOT lowering recognizes retained readonly
byte-memory metadata and calls the checked byte-window kernel before the existing
synchronous console provider. The provider cannot retain the pointer, allocate
managed objects or re-enter the guest during that access. Default empty descriptors
are valid at the kernel boundary; end-position empty windows are exercised through
the production library. This does not establish asynchronous OS completion.

As in the existing .NET comparison, potentially retained stream work uses persistent
Memory while immediate native access borrows under runtime control. No public
AsSlice adapter or array compatibility overload is added. Interpreter snapshots
remain a copying cost; native access makes no performance claim. Legacy four-argument
internal bindings remain bootstrap/test-only. See the
[consumer](../runtime/raven/tests/console-memory/README.md) and
[evidence](validation/memory-views-20261019/console-integration-validation.json).

## Socket memory contracts (2026-10-19)

The production public Socket Receive/ReceiveUntil methods now take Memory<byte>;
Send/SendUntil take ReadOnlyMemory<byte>. Their array/offset/count overloads are
removed, including token-taking forms. Whole arrays convert implicitly; callers
construct bounded memory windows before invoking the operation. Invalid window
construction is a bounds fault, not a SocketError that pre-cancellation can hide.
HTTP's socket adapter, PostgreSQL transport and current native consumers use the
new signatures. Historical CLI/neoIL fixtures retain their private array services.

The interpreter retains the receive descriptor and traces its owner until settlement,
then patches only the received prefix. The native provider retains the checked array
owner plus offset/count in its operation table, with an explicit host root for every
non-null receive owner, including end-position empty windows. Default empty memory
does not invent a backing allocation. Both backends snapshot selected initialized
send bytes after admission; source aliases may change after submission without
changing that operation. Existing operation budgets, per-direction Busy state,
deadlines, EOF, cancellation acknowledgement and cleanup remain intact.

This follows .NET's persistent Memory/ReadOnlyMemory distinction for socket operations,
while deliberately keeping neoCLR's existing send snapshots and Task/Result contract.
Removing compatibility-only array forms makes buffer contracts consistent at a source
compatibility cost. Copying remains part of send semantics; no speedup or zero-copy
claim is made. No public Native API or asynchronous raw-pointer lifetime is introduced.

The [three-mode consumer](../runtime/raven/tests/socket-memory/README.md) exercises
owner-only retention, selected windows, snapshots, GC, empty operations, cancellation
and EOF. [Evidence](validation/memory-views-20261019/socket-integration-validation.json)
also records interpreter contracts, exact native binding rejection and the native
provider regression. Shared InputStream/OutputStream and higher HTTP body-transfer
signatures still require coordinated migration; this step does not claim those
public APIs have changed.

## HTTP parser and body windows (2026-10-19)

Request decoder Push now takes ReadOnlySlice<byte>; the receive callback supplies
only the returned prefix. The parser copies retained headers/body before buffer reuse
and keeps no borrowed descriptor. HttpBodyInput retains a single Memory<byte> output
window across receives instead of separate array/offset/count fields. Drain borrows
Slice<byte> synchronously; completion, failure and cancellation clear retained memory.
Owned transport/read-ahead arrays remain owned storage.

This applies the existing .NET Span/Memory lifetime comparison to HTTP internals:
borrow for immediate parsing, retain an owner for asynchronous output. It adds no
public compatibility adapters or extra data copies and makes no performance claim.
Public InputStream/OutputStream and higher HTTP upload contracts still await their
coordinated migration; existing range-error and cancellation precedence is preserved.

The [public consumer](../runtime/raven/tests/http-memory/README.md) covers fixed and
chunked request bodies, streaming client responses, selected output windows, GC,
cancellation and EOF in interpreter/JIT/AOT. See the
[evidence](validation/memory-views-20261019/http-integration-validation.json).
Public signatures are unchanged; the source fingerprint is refreshed while the
native public API rendering remains applicable.

## TLS plaintext windows (2026-10-19)

SslStream retains selected Memory<byte> read destinations and ReadOnlyMemory<byte>
write sources across pump awaits. Plaintext staging copies borrow readonly/writable
slices synchronously. ReadCount/WriteCount now accept only the selected control-field
slice instead of the full control array plus an offset. No slice crosses suspension.

This follows the existing .NET Span/Memory lifetime comparison. TLS staging arrays,
encrypted buffers and native owner tokens remain owned arrays. This change neither
removes the existing TLS copy nor claims a performance improvement. Public stream
signatures, PEM factories and the native TlsPump array ABI remain separate migrations.
Existing range errors and cancellation ordering are preserved.

The [managed TLS consumer](experiments/shared-tls/stream-project/MemoryWindows.rvn)
exercises nonzero source/destination windows, a 16386-byte write spanning plaintext
blocks, collection during pending operations, empty windows and graceful EOF.
The HTTPS consumer also checks JSON exchange and rejection/timeout/cancellation.
[Evidence](validation/memory-views-20261019/tls-integration-validation.json) records
execution modes and matching library identities. Public signatures are unchanged;
only API source fingerprints require refresh.
