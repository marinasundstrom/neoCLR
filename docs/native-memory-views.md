# Native slices and persistent memory views

Author-directed implementation, 2026-10-18. Slices are the selected direction for
contiguous managed-memory access; raw pointers remain available for explicit
low-level work. This does not change the platform's separate UTF-8 text model.

## Status

The first implementation is an **internal interpreter foundation** in
[`src/memory_views.rs`](../src/memory_views.rs). It is not a public standard-library
API and is not yet connected to guest values, Raven emission, JIT or AOT. Existing
array range syntax still makes a shallow copy. Do not use the proposed examples
as working Raven examples yet.

| Layer | Implemented here | Remaining integration |
| --- | --- | --- |
| Slice capabilities | Checked array windows; shared writes; readonly propagation through element references; nested and empty windows | Public generic Slice/ReadOnlySlice types, byref-like metadata and runtime storage restrictions |
| Lifetimes | Owner validation and a checked frame-return helper | Invoke that helper from guest returns; reject boxing, heap/static/array fields, captures and suspension of slices |
| GC | Checked publication of actual containing allocation, including empty/interior views | Trace views automatically from guest frames and persistent Memory values in every backend |
| Persistent views | Heap-only owner validation, offset/count, slicing and borrowed access | Public Memory/ReadOnlyMemory values, object fields and real async consumer |
| UTF-8 | Shared immutable byte view with byte-relative windows | Public String byte-view API and ReadOnlySlice<byte> lowering |
| Syntax | Existing Index and Range can resolve boundaries | Raven array range result changes, view indexers and explicit conversion APIs |
| Native execution | No native ABI selected by this change | Metadata recognition, GC maps, lowering and interpreter/JIT/AOT differential tests |

These internal capabilities deliberately cannot be put into `Value` or exposed as
ordinary guest structs yet. Existing stored-reference checks allow heap-backed
byrefs in ordinary values; that alone would incorrectly admit heap-backed Slice
instances into boxes or fields. Guest exposure must wait for type-level checks
which prohibit these operations regardless of the particular slice's owner.

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

The proposed `data[1..4]` changes the existing array range contract from copy to
alias. The public migration must provide an explicit copying operation, update
consumers and test aliasing before changing syntax. `Index`/`Range` already exist;
their direction and half-open endpoint semantics should be reused. The existing
fallible `Slice(range)` API also needs a coordinated signature migration because
ordinary Result containers must not carry byref-like slices without a separately
validated generic contract. This is a gap to solve, not permission to box them.

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

GC tests explicitly publish roots. They do not establish automatic guest tracing
or async retention. Frame-return tests call the helper explicitly and do not
establish guest escape checking. No public API was added, so no RavenDoc selection,
reference assembly or snapshot update is required at this stage. The website's
direction page records the work as unfinished; no website build is needed for
this internal runtime change.
