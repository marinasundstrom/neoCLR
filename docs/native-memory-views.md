# Native slices and persistent memory views

Author-directed implementation, 2026-10-18. Slices are the selected direction for
contiguous managed-memory access; raw pointers remain available for explicit
low-level work. This does not change the platform's separate UTF-8 text model.

## Status

The interpreter recognizes explicit opaque memory-view metadata and checked runtime
services. The four shapes are exercised by [neoIL test fixtures](../tests/fixtures/memory_views.neoil),
not hand-authored class-library implementations. Production APIs belong in Raven
`.rvn` source; source/compiler integration supplies ref-like types, implicit conversions and
array range views. Native JIT/AOT lowering is not implemented and selected
view types reject instead of being treated as empty records.

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
This is executable C support, not yet bound to Raven InternalCalls or admitted
by JIT/AOT selection. Other element types and String UTF-8 views remain pending.

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
