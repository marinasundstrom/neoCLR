# Native slices and persistent memory views

Author-directed implementation, 2026-10-18. Slices are the selected direction for
contiguous managed-memory access; raw pointers remain available for explicit
low-level work. This does not change the platform's separate UTF-8 text model.

## Status

The interpreter recognizes explicit opaque memory-view metadata and checked runtime
services. The four shapes are exercised by [neoIL test fixtures](../tests/fixtures/memory_views.neoil),
not hand-authored class-library implementations. Production APIs belong in Raven
`.rvn` source; source/compiler integration supplies ref-like types, implicit conversions and
array range views. Native JIT/AOT supports a bounded array-backed byte-view path:
all four kinds support array conversion, default values, Length/IsEmpty and
integer/Range windows. Borrowed byte views support element reads, writable element
updates, readonly conversion, borrowing from persistent byte views and the Raven
ToArray loop. Persistent byte views retain typed argument/return/field storage.
Other element types and String.Utf8Bytes remain gated. See the native borrowed
byte-view qualification below; this does not qualify asynchronous view retention.


`.memoryview slice|read_only_slice|memory|read_only_memory` serializes as the
optional `TypeDef.memory_view` enum. Admission requires the corresponding canonical
System name, a System declaration or canonical native origin, one type parameter
and a fieldless value record without base/interfaces/layout/constraints. Name alone
grants no capability. Ordinary construction and boxing reject; checked factories
and `initobj` produce an opaque payload. Exact InternalCall signatures cover array
factories, Length, Get/Set, nested windows, readonly conversion, persistent-to-borrowed
conversion and immutable String UTF-8 bytes.

Slices reject heap storage, erasure, arrays, fields, ordinary generic arguments and
Function signatures regardless of owner. Returns recursively check frame references.
At suspension boundaries every resident borrowed value in the active stack, locals
and arguments rejects suspension. This conservative rule may reject a dead local;
compiler liveness relaxation needs separate validation. Persistent views require
heap ownership, trace owners through fields and survive collection and suspension.

Readonly references restrict access rather than define a separate overload identity.
Getter results are values, not element byrefs. Null arrays and `initobj` create empty
views; empty windows over arrays retain their owner. Bounds use the ordinary
IndexOutOfRange fault category. No native ABI or performance claim is made.

## Runtime concepts and future host access

Author discussion, 2026-10-19: slices and memory are managed runtime concepts
projected through the library types. The runtime owns element identity, owner
tracing, bounds, access capability and lifetime enforcement; the Raven types make
those operations available to programs. They are not independent library wrappers
whose semantics an embedding host would need to reconstruct.

A future Native API can expose the same concepts through checked runtime
operations. The proposed host model is a rooted persistent view handle plus a
scoped borrow for element access. Rooting an owner and stabilizing its address are
separate obligations: a future moving collector would need pinning or another
explicit access protocol. Managed-reference writes also require type,
initialization and GC-barrier checks. Externally owned storage would need a
separate ownership/release contract.

This is design direction, not a published Native API or ABI commitment. The Rust
MemoryView payload has no public access methods, and the private C byte-window
kernel neither roots owners nor grants host borrow lifetimes. Neither descriptor
layout is promised as the future embedding representation. This preserves the
.NET-inspired ownership distinction while allowing an explicit neoCLR hosting
contract; its cost is designing and validating another lifetime boundary.

## Contract and comparison with .NET

Primary sources consulted 2026-10-18:

- [Span<T>](https://learn.microsoft.com/en-us/dotnet/api/system.span-1?view=net-10.0)
  supplies a byref-like view of contiguous storage. Copying a view shares its
  elements, and slicing adjusts a window rather than copying its contents.
- [Memory<T>](https://learn.microsoft.com/en-us/dotnet/api/system.memory-1?view=net-10.0)
  supplies a persistent counterpart suitable for asynchronous use.
- [C# ref struct rules](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/ref-struct)
  describe heap, boxing, capture and suspension restrictions. Modern C# permits
  some local ref-struct use in async methods; the relevant restriction is crossing
  a suspension boundary, not simply appearing anywhere in an async method.

The author selects `Slice<T>`/`ReadOnlySlice<T>` instead of the .NET `Span` names,
and `Memory<T>`/`ReadOnlyMemory<T>` for persistent views. The naming divergence
aligns the API with Raven slicing syntax; it costs direct name compatibility with
.NET libraries. Safety and aliasing semantics remain the target, not a claim of
matching native performance before measurement.

The intended native slice shape is a managed interior reference plus length.
The interpreter currently models that logical address using an existing
`SlotReference` to the array and a start position. A slice additionally records
its length. This is **not** a native layout or a promise of a two-word ABI.
Persistent views require a heap owner and retain offset/length. View construction
does not read or copy any element and does not allocate a guest object. Internal
Rust handle/path cloning can allocate host metadata; no allocation-free host or
native performance claim follows from the zero-copy payload contract.

Range checks use subtraction before addition, rejecting overflowing windows.
The empty window at the end is valid but has no addressable element. Empty views
still publish their owner when collected. Array element types remain exact;
readonly is shallow access restriction, not immutable storage. Writes through a
separate writable alias remain visible through readonly views.

The author explicitly selected `data[1..4] -> Slice<T>` (2026-10-19).
Array range syntax and Array.Slice return views; callers request copying through
ToArray(). This intentionally breaks the previous copy contract and removes
Array's Sliceable<T[]> implementation. Owned-array APIs remain arrays where
ownership is their purpose. Implicit array-to-view and writable-to-readonly
conversions avoid repeated AsSlice call-site plumbing. Raven's ordinary .NET
target retains its existing GetSubArray copy contract.

String indexing/slicing remains grapheme-based. The internal UTF-8 view shares
the immutable String owner; its offsets count bytes and may split a UTF-8 scalar.
It has no writable operation. Consumers must decode before interpreting such a
window as text. This is deliberate divergence from .NET's UTF-16 string views.

## Implementation sequence

1. Complete Slice integration: metadata identity and byref-like classification;
   readonly/writable API signatures; managed address lowering; rejection of
   storage/boxing/capture/suspension; caller-frame reference returns; GC tracing;
   then source API and array range migration. No new neoIL instruction has been
   added by the foundation. Reuse reference/array operations where possible.
2. Expose Memory and ReadOnlyMemory using the same window checks with traced
   heap ownership. Qualify object-field storage and a real awaited operation.
   Add String's immutable UTF-8 view without converting text to an array.
3. Migrate buffer consumers in bounded steps. Measure an actual stream/network/
   parser workload before adding intrinsics or removing bounds checks. Every
   backend must preserve exact element type, readonly and lifetime rules.

Compiler changes must be documented in both repositories, behind the neoCLR
target contract. Raven's ordinary .NET behavior remains unchanged. No compiler
or CLI bridge change is included in the first foundation commit.

## Focused validation

Run `cargo test --lib memory_views::tests`. The tests exercise shared mutation,
exact element types, readonly element addresses, nested receiver-relative bounds,
empty windows, extreme bounds, uninitialized elements, frame-return checks,
heap-only persistent ownership, same-ID foreign heaps, publication/reclamation of
containing objects and their reachable elements, and zero-copy UTF-8 owner lifetime.

Local result, 2026-10-18: **9 passed, 0 failed**. The test executable initially
waited in macOS program startup; after startup all nine tests completed normally.

The second slice adds tests using normal `Slot::trace_heap`, heap-field traversal,
erasure and array/field storage guards, plus the recursive return check now used by
the VM. Tests include foreign-heap host publication and readonly/type identity.
These use internal view construction, not a Raven producer, and do not establish
async retention or native execution. Existing heap-reference executable contracts
guard the changed VM return path.

Second-slice validation: **16 memory-view tests and 9 heap-reference executable
tests pass**; `cargo check -p neoclr-codegen` also passes. The checks establish
interpreter value integration, not native view lowering.

The Rust-only opaque payload is documented in the
[host value reference](../api-docs/objects.md#rust-host-memory-view-payloads).
No guest API was added, so no RavenDoc type selection or reference assembly/snapshot
update is required. The website's direction page still accurately describes the
unfinished public APIs; no website build is needed for this internal runtime change.

Third-slice validation, 2026-10-19: `cargo test --test memory_views` passes all
nine executable contracts for shared windows, readonly/UTF-8 access, persistent
GC roots and suspension, bounds/default/null behavior, borrowed escapes and
caller-versus-callee frame lifetimes. These declarations live only in test fixtures.

Raven source integration (2026-10-19): compiler `d3519e49b` and the matching
native metadata provider compile Slice/ReadOnlySlice ref structs, implicit array
and readonly conversions, Array range views and String.Utf8Bytes. The checked-in
[Raven consumer](validation/memory-views-20261019/README.md) returns 0; three
negative programs reject. Sliceable was removed from Raven and the documentation
bridge. This was followed by the persistent source integration below; native
JIT/AOT execution remains pending. The complete native class-library bundle and API renderer build.

Persistent Raven integration (2026-10-19): Memory/ReadOnlyMemory are canonical
ordinary structs with shared range windows, ToArray copying and implicit array,
readonly and borrowed conversions. The [Memory consumer](validation/memory-views-20261019/memory-consumer.rvn)
retains a subwindow in object fields, returns its creator frame, crosses an awaited
Task.Run and collects twice before exercising aliases. It returns 0. Invalid
borrowed-to-persistent and readonly-to-writable conversions reject with RAV1503.
The [class-library audit](memory-view-library-audit.md) identifies synchronous
codec/crypto candidates and Memory-based async I/O candidates. Shared service
migration awaits native ABI/codegen so existing JIT/AOT workloads remain supported.


Native compiler preparation (2026-10-19): closed specialization now retains each
view's exact lowered element type in private, nonserialized IR. Nominal renaming
remaps that dependency. The memory_view kind continues to identify writable,
readonly, borrowed and persistent shapes. This follows the CLR requirement that
constructed generic storage retains its element identity; it is not a new guest
metadata encoding or a physical layout choice. Direct value profiling now rejects
opaque shapes as well as ordinary selection, preventing accidental empty-record
layout if a caller bypasses selection. Native operations, ABI lanes, GC tracing
and executable JIT/AOT consumers remain pending.

Focused native-preparation validation: three new codegen tests pass (closed
specialization identity, rename/serialization boundaries, direct layout rejection).
The five projection tests pass, including one of those new tests and four existing
array/String/Object/interface regressions. All nine interpreter memory-view
contracts also pass. Commands: `cargo test -p neoclr-codegen --lib memory_view`,
`cargo test -p neoclr-codegen --lib variance_projection_tests`, and
`cargo test --test memory_views`. No new guest API or API snapshot is introduced;
the Rust compiler bridge is covered in the host memory-view reference.

Final configuration checks pass with `cargo check -p neoclr-codegen` and
`cargo check -p neoclr --no-default-features`. The retained element field is
compiled only with native-metadata, with no added storage in interpreter-only builds.


## Native byte-window kernel (2026-10-19)

`runtime/native/memory/memory-views.h` and the existing packaged text-arena
implementation now provide checked native operations for byte-array windows.
The initial kernel was C-only; the bounded persistent-byte binding below now
uses its creation and slicing operations from generated Raven code. Other
element types and String UTF-8 views remain pending.

The provisional private carrier is 24 bytes on the supported 64-bit targets:
an allocation-base owner pointer, two uint32 element bounds, and a uint64 kind.
Only the owner is a root; bounds/kind are numeric. Compared with .NET's typed
Span/Memory contracts, this kernel keeps an explicit dynamic kind to check both
readonly restrictions and borrowed-to-persistent promotion. That costs an extra
word and validation on access; it is not the final guest ABI or a performance
claim. Future typed lowering may remove redundant state after validation.

Factories accept mutable byte-array storage, not immutable value snapshots. The
future binder must enforce the exact typed source contract and supply the kind;
C callers are trusted and cannot manufacture guest lifetime authority. Every
operation validates its window before reading/writing storage. Reserved arrays
retain initialization checks; a successful write publishes only that byte.
Conversions may restrict access or borrow persistent memory, never grant write
access or promote a borrow. Failure leaves outputs and backing bytes unchanged.
Null arrays create empty windows; nonnull empty windows retain their owner.

Operations allocate nothing, collect nothing and register no roots. Callers must
publish owners across collecting boundaries and preserve the context lifetime.
GC builds check the exact live allocation base and context before dereferencing.
Arena builds check bounds within used storage but rely on the trusted allocation
base because that mode has no allocation registry. No native borrowed suspension
or frame-escape rule is implied by these helpers; those remain codegen work.

Validation: `python3 -m unittest discover -s scripts -p test_native_memory_views.py`
passes the support-package header check and both arena/GC C variants with
warnings-as-errors, UBSan and bounds checking on macOS ARM64. Cases include
subwindow aliasing, all 16 kind conversions, readonly writes, partial initialization,
null/empty/extreme bounds, failure atomicity, malformed extents, zero allocations,
empty-view rooting, persistent storage inside a managed object, foreign contexts
and stale owners after collection. The native GC validation runner includes the
same fixture for both modes; Windows/Linux execution is not qualified locally.
ASan attempts timed out in sanitizer initialization before main on macOS 27;
a sampled stack confirms this, so no ASan pass is claimed.

Existing static-storage consumers also pass in arena and GC modes through the
compatibility include wrappers, with UBSan/bounds and warnings-as-errors. The
packaging check confirms the new header is included in native support bundles.

The byte kernel also supplies `neoclr_byte_view_copy_v1(context, source,
destination)`. It copies the source into the destination prefix with the
overlap semantics of [.NET ReadOnlySpan.CopyTo](https://learn.microsoft.com/en-us/dotnet/api/system.readonlyspan-1.copyto?view=net-10.0):
the result reflects the original source even when the windows overlap. Native
`memmove` avoids a temporary allocation. Both descriptors and the destination
suffix remain unchanged. Readonly sources are accepted; readonly destinations
are rejected, including empty copies. A short destination returns the private
kernel's bounds fault (8), not .NET's ArgumentException; mapping to a future guest
copy API remains open and this helper does not establish that API's error contract.

neoCLR reserved storage additionally requires validating every source
initialization bit before moving any bytes. Otherwise an overlapping write could
initialize a later unread source slot, or leave a partial copy on failure. Only
the destination prefix becomes initialized after success. This adds a linear
validation pass for reserved sources; ordinary initialized arrays need no such
pass. No throughput improvement is claimed. Empty copies avoid null-owner
pointer arithmetic but still validate storage and capability. Ownership/rooting
rules remain the same as the other helpers; no allocation or collection occurs.

Focused validation uses the same Python command above, with warnings-as-errors
and UBSan/bounds in arena and GC modes. An independent snapshot oracle checks
50,176 combinations per mode: every window in six-byte storage, all four source
and destination kinds, shared/distinct owners and initialized/reserved arrays.
Additional cases cover partially initialized overlapping sources, precise
destination initialization, null/empty inputs, invalid windows, foreign contexts,
stale owners and unchanged allocation counts. Both tests pass. Public Raven
members, the API snapshot and native guest admission are unchanged; the manual
host reference documents this sixth private helper.


## Bounded native persistent-byte storage (2026-10-19)

The shared backend recognizes only closed Memory<byte>/ReadOnlyMemory<byte>
with retained specialization proof and their exact fieldless opaque shape. It
uses three logical lanes: owner (I64), offset (I32) and length (I32), occupying
24 bytes in the existing eight-byte-per-lane private storage format. Only owner
is a stack root. Nesting in records/tuples retains that root's actual offset;
heap fields participate in the existing nonmoving collector's object tracing.
Readonly capability remains in the nominal type, not a guest-writable kind word.
This follows the owner/offset/length model of .NET Memory while using neoCLR's
private aggregate ABI. The extra slot padding is an implementation cost, not
an externally stable layout or a performance improvement claim.

With explicit System compilation and reference-arena support, exact InternalCalls
bind array creation, Length, both integer Slice overloads and writable-to-readonly conversion. Creation accepts mutable
ArrayRef<Byte>, never an immutable byte value snapshot. Default values have a
null owner and zero bounds. Length reads the validated length lane; Slice packs
the three lanes into the checked C descriptor with the statically known kind,
then unpacks the successful result. For Slice(start), an unsigned bounds check
rejects negative or past-end starts before subtracting start from length; the
checked C kernel still validates owner storage and the resulting window. The C
kind word never enters guest storage.
The whole-array helper validates storage before deriving its length. Helpers do
not allocate or collect; ordinary call, field and return lowering preserves the
owner through published GC frames. This is temporary native IR/codegen, not a
new CLI encoding or a Raven compiler change.

Borrowed Slice/ReadOnlySlice, other element types, view arrays/unions, boxing, arbitrary
record construction, persistent-to-borrowed conversion and
ToArray services remain unsupported in native execution.
The full interpreter API is unchanged. Range APIs compose the admitted integer
primitive through the existing Raven Range.GetOffsetAndLength implementation. Async
view retention and shared buffer-service migration remain later validation work.

Reproduction: build `neoclr-codegen`, then run
`python3 docs/validation/memory-views-20261019/native-run.py --compiler <rvnc.dll> --output <directory>`
with the matching source-owned System.Runtime library. The Raven consumer covers
both persistent types, default empty values, array conversion, return/argument
copies, nested windows, object fields and an empty window retaining its own array.
It performs checked slicing after explicit GC, when each creating function has
returned. The harness also checks root-frame cleanup and full final reclamation.

The final VM verification pass uses a temporary nominal shell for each admitted
persistent-byte shape. It still checks type/member identity, definite
initialization and address lifetimes. The native profile checks the real opaque
layout and rejects record construction/boxing beforehand; the original load set
is verified before specialization. Shells are never emitted or executed. A
regression test checks that uninitialized locals still reject and serialized
metadata without the private specialization proof cannot request a shell.

Validation passes on macOS ARM64: both Raven consumers compile and run on the
interpreter, JIT and AOT; AOT is checked with UBSan/bounds. Success returns 0;
the invalid window returns native status 8 with the output untouched. Both paths
balance root frames and fully reclaim the heap. The codegen suite passes 98 tests
with one explicit performance test ignored; the two Python kernel tests pass in
arena/GC modes. [Retained evidence](validation/memory-views-20261019/native-validation.json)
includes commands, hashes, bindings and execution output. Windows execution and
native asynchronous retention are not qualified here. The public signatures are
unchanged, so the API snapshot is unchanged; manual API limits and website status
are updated without a routine website build.

The one-argument native follow-up is retained in
[short-window validation](validation/memory-views-20261019/short-slice-validation.json).
It extends the success consumer with default-empty, start=end and nested short
windows after GC, and isolates past-end and both signed Int32 extremes for each
persistent byte type. The .NET-compatible start-to-end contract is unchanged.

The readonly native follow-up binds only Memory<byte> to ReadOnlyMemory<byte>.
It copies the three validated representation lanes directly, preserving owner,
offset and length without allocating or calling a C helper. Readonly capability
remains nominal; reverse conversion, borrowed inputs/results, byref parameters,
other elements and unproven serialized shapes reject. This implements the same
capability restriction as .NET's implicit Memory<T>-to-ReadOnlyMemory<T> operator;
there is no new API or conversion from borrowed to persistent storage.

[Readonly conversion evidence](validation/memory-views-20261019/readonly-validation.json)
covers default, nonempty and owner-retaining empty conversions, implicit argument
and return conversion, fields and slicing after explicit GC on interpreter/JIT/AOT.
Use `native-run.py --consumer-only` with the other documented arguments to repeat
this focused consumer; prior bounds evidence is reused. Borrowed native support
and shared buffer-service migrations remain pending.

The [range qualification](validation/memory-views-20261019/range-validation.json)
executes existing Raven Memory<byte>/ReadOnlyMemory<byte> range indexers and
Slice(Range) without a new intrinsic. Open/from-end ranges, nested windows,
Range.All, end-empty/default-empty views and range-created fields survive explicit
GC in the interpreter, JIT and AOT. AOT runs with UBSan/bounds. The syntax remains
receiver-relative and half-open, following .NET Range semantics; unlike .NET's
array-range copy, neoCLR array ranges separately return borrowed Slice views.

Reversed, excessive end and excessive from-end bounds are isolated for both
persistent types. At that qualification checkpoint, Range.GetOffsetAndLength used System.Fail, producing
UserFault (native code 4), whereas integer Slice faults with IndexOutOfRange
(native code 8). Those historical checks recorded a consistency gap, resolved by the follow-up below.
No public API or compiler/backend change was needed for this qualification.
The current native-run.py with `--consumer-only --range-bounds` tests the corrected
classification below; the old results remain tied to the prior qualification commit. Native borrowed views and
buffer-service migration remain pending.

### Consistent terminal range bounds

Range.GetOffsetAndLength now calls the private, argument-free ViewBoundsFault
InternalCall on both invalid length and invalid interval paths. The exact
System.Void/no-result contract terminates with IndexOutOfRange in the interpreter
and native status 8 in JIT/AOT. Native binding of this nonallocating fault service
does not require reference-arena storage; actual byte-view operations still do.
Outside the arena profile, --bind-user-fault supplies the diagnostic context
contract. Without either context-enabling flag the InternalCall remains unbound
and compilation rejects, rather than emitting its verification placeholder.
TryGetOffsetAndLength retains Result/InvalidRangeError and System.Fail retains
UserFault. All callers of GetOffsetAndLength receive the corrected category,
including borrowed views, arrays and other collection/text range consumers;
text boundaries still use the existing text semantics.

This corrects the earlier UserFault classification and is a development migration
for hosts inspecting fault codes. It adds no public member or neoIL instruction.
The source validator remains Raven; the runtime helper supplies only the terminal
bounds classification and native diagnostic/root-frame unwinding. Native borrowed
view execution remains unsupported and is tested through the interpreter here.

.NET's [Range implementation](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Range.cs)
validates resolved boundaries and throws ArgumentOutOfRangeException. neoCLR keeps
its existing terminal-fault model and maps this failure to the same IndexOutOfRange
category used by integer slicing. This improves consistency within neoCLR at the
cost of the above fault-code migration; it does not reproduce CLR exceptions.
Negative supplied lengths remain explicitly rejected, as before.

[Consistent-bounds evidence](validation/memory-views-20261019/consistent-bounds-validation.json)
replaces the old expected UserFault results, covers all four view kinds in the
interpreter and both persistent byte kinds in native execution, and checks a
standalone negative-length Range call without reference-arena admission in AOT
(managed JIT uses its required GC/context profile). Public
signatures remain unchanged; summaries and the matching API snapshots are refreshed.

## Native array-backed byte slices — 2026-10-19

The native admission profile now accepts all four exact byte-view kinds. Borrowed
views use the existing owner/offset/length lanes and owner-only roots. This is a
private array-backed representation of a logical interior reference; it does not
admit stack-backed or arbitrary-pointer slices. Source verification runs before
specialization and rejects borrowed heap/compound/erased storage. Direct native
profiling also rejects borrowed fields before verification shells are created.

Exact runtime bindings cover whole-array and bounded array creation, length,
integer/Range slicing, readonly restriction, Memory-to-Slice borrowing, element
reads and writable element updates. Borrowing and readonly conversion copy the
three lanes without allocation; the nominal contract continues to distinguish
all four capabilities. Get/set use the checked C kernel and propagate bounds or
storage faults through native diagnostics. The no-result setter does not write a
caller result. The existing Raven ToArray loop works through these operations.

Qualification uses the [Raven consumer](validation/memory-views-20261019/native-memory.rvn)
and [driver](validation/memory-views-20261019/native-run.py): shared mutation,
readonly aliases, integer/from-end/nested windows, independent ToArray copies,
implicit array and persistent borrowing, default/empty views and collection with
a returned slice as sole owner. Invalid ranges and element access use terminal
IndexOutOfRange; native hosts check untouched failure results, balanced roots and
complete reclamation. AOT runs use UBSan/bounds instrumentation. Focused Rust tests
cover capability conversion, exact service signatures, field rejection and layout.
See [retained results](validation/memory-views-20261019/borrowed-byte-validation.json).

No native String.Utf8Bytes, non-byte elements, stack-backed borrows or asynchronous
retention qualification is claimed. Codec consumers are the next adoption step;
stream/socket/HTTP Memory-based contracts still need their own completion-lifetime
validation. No speedup or stable host ABI is inferred from these checks.

## First shared class-library adoption: incremental codecs — 2026-10-19

The Raven Decoder contract now accepts ReadOnlySlice<byte> plus final, and Encoder
Drain accepts Slice<byte>. Built-in UTF-8/ASCII sessions, StreamReader/StreamWriter
and maintained codec tests use the new contracts. Source arrays convert implicitly;
subwindows use normal range syntax. There are no compatibility overloads or public
AsSlice adapters. Encoding's one-shot Sequence-based methods are unchanged.

The decoder continues to copy incomplete scalar bytes into its private carry array,
and the encoder retains immutable accepted text rather than its output view. Quotas,
zero-capacity progress, finalization and output suffix/surrounding sentinels retain
their contracts. Bounds now belong to view construction; the unused
EncodingError.InvalidRange case is removed. Callers and custom codec implementations
must migrate signatures, and error-case metadata must be rebuilt together.

This first migration reaches immediate codec calls within stream adapters. It does
not change the asynchronous InputStream/OutputStream buffer contract yet. HTTP
Server/Client transfer integration still depends on that migration and completion
ownership validation. The [codec API reference](../api-docs/streams.md#incremental-encoding-development)
and native-generated signatures describe the development API; old CLI scaffold
members are removed rather than retained as misleading overloads.

[Codec integration evidence](validation/memory-views-20261019/codec-integration-validation.json)
records 35 tests in each of interpreter/JIT/AOT (105 passing executions), including
quota preflight, sentinel preservation, every UTF-8 split, carry ownership, pending
reads/writes and cancellation. The final run uses a freshly built release interpreter;
an earlier debug-interpreter quota run hit its 180-second limit and is retained as
an unsuccessful attempt. Both API snapshots are refreshed and their focused checks
pass. No benchmark or full website build was performed.

### Synchronous provider access (2026-10-19)

The private `neoclr_byte_view_access_v1` helper validates allocator ownership,
window bounds, writable capability and (for reads) every initialization bit in
the selected window. It yields temporary data/initialization pointers to trusted
synchronous adapters only. It does not allocate, collect, root, pin or extend a
borrow. Adapters cannot retain pointers, call guest code or suspend, and publish
only the written initialization prefix after provider completion. Empty windows
return null pointers and zero count; failed validation leaves output unchanged.
Cryptography uses this to avoid whole-array copies at the native provider boundary.
This helper is not the future public Native API or a stable host ABI.

### Persistent byte memory across managed suspension (2026-10-19)

The native async entry closure now accepts immutable Memory/ReadOnlyMemory
parameters through an explicit Raven target capability. Its descriptor is copied;
the existing native layout traces the shared backing owner. Captured receiver
access uses a defensive local descriptor without exposing a closure-field address.
Borrowed Slice/ReadOnlySlice captures remain rejected.

The [pending-transfer fixture](../runtime/raven/tests/memory-lifetimes/README.md)
qualifies four cases in interpreter, JIT and AOT on macOS ARM64: sole-owner buffers
after creator return, two pending awaits and explicit collections, readonly reads,
writable subwindows and sentinels, cancellation/recoverable rejection, and empty
windows. The tests use async Main and explicit progress promises; a synchronous
TaskQueue.Drain is not a native scheduler completion barrier.
[Evidence](validation/memory-views-20261019/async-lifetime-validation.json) includes
15 compiler regressions and two native borrowed-capture rejections. Five libraries
were rebuilt with the matching compiler; production public signatures are unchanged.

This follows .NET's persistent Memory/borrowed Span lifetime distinction with
neoCLR's implicit typed borrowing. It qualifies managed suspension only. Native
I/O retention/pinning, completion-root release, terminal runtime fault cleanup,
Task<Memory> results, non-byte native views and Windows remain unqualified here.
InputStream/OutputStream and socket contracts still require migration and separate
completion ownership tests. No public Native API or stable host ABI is added.

### Interpreter detached file access (2026-10-19)

The [file boundary](memory-view-library-audit.md#interpreter-file-transfer-boundary-2026-10-19)
uses persistent byte-memory descriptors while the host runs without heap access.
Reads root their destination until bounded write-back; writes snapshot only the
selected initialized window. Readonly source capability does not freeze aliases;
callers must still honor the stream's unchanged-input-until-completion rule.
No raw managed pointer crosses this boundary. This is private interpreter service
integration, not a public Native API or native file-I/O implementation.

### Console output integration (2026-10-19)

The [console boundary](memory-view-library-audit.md#console-output-memory-boundary-2026-10-19)
uses ReadOnlyMemory<byte> in the production Raven service. Interpreter output
snapshots the selected initialized bytes under heap access. Native lowering borrows
a checked window only for the synchronous console provider call; it neither pins
nor retains a raw pointer across suspension. Public stream signature migration and
native asynchronous I/O ownership remain separate work.

### Socket lifetime qualification (2026-10-19)

[Socket transfer APIs](memory-view-library-audit.md#socket-memory-contracts-2026-10-19)
now use persistent byte memory in interpreter/JIT/AOT. Native receive completion roots
the backing owner and retains checked bounds; sends retain an owned selected-byte
snapshot. The managed descriptor is not exposed as a raw pointer across suspension.
Default empty values, owner-only receive retention, GC, cancellation and EOF are
qualified. General Native API access, other element kinds and shared stream/HTTP
body signatures remain separate work.
