# Slice release review against .NET Span — 2026-10-21

The author asked which parts of the .NET 10 Span page should affect this release.
This is a source/documentation review and a recommended qualification scope, not
an implementation claim or a new complete-Span milestone.

## Existing foundation

`runtime/raven/src/System/{Slice,ReadOnlySlice,Memory,ReadOnlyMemory}.rvn` provides
borrowed ref-like slices, persistent owner-retaining memory, implicit conversions,
length/emptiness, checked index/range operations and explicit `ToArray` copying.
Readonly aliases are documented as shallow. Array range results share storage.

`tests/memory_views.rs` already contains interpreter regressions for aliasing,
allocation-free nested views, readonly/UTF-8 access, invalid bounds, default/null
empty views, heap/boxing/suspension rejection and borrowed-return lifetimes.
`runtime/raven/tests/memory-lifetimes` contains managed suspension/GC tests and
negative borrowed-capture fixtures. Their existence is not fresh all-mode evidence.
The website's current native qualification scope is byte views; non-byte native
specializations remain unqualified. Do not infer general native `Slice<T>` parity.

## Recommended release priorities

1. Requalify the supported borrowed-lifetime and bounds contracts with the final
   compiler/runtime pair. Include default/empty/end-position views, integer
   overflow, shallow readonly aliases, GC owner retention, prohibited heap/capture
   escapes and views live across suspension. Unsupported native element shapes
   must produce a clear diagnostic rather than invalid code.
2. Add the most useful missing buffer operations: `CopyTo`/`TryCopyTo`, `Clear`,
   and `Fill`. The current slice declarations/extensions do not expose them.
   Copying must handle overlapping windows in both directions without an
   unconditional temporary array. Too-small `TryCopyTo` destinations must remain
   untouched; reference-containing elements also need correct GC handling.
3. Resolve/document the element-access distinction. Span's indexer returns `ref T`;
   our Raven API currently exposes a `T` getter/setter. General value-type slices
   need explicit behavior for nested field mutation and passing elements by
   reference, with compiler/backend tests. Do not casually change this contract
   based only on a byte-window test.
4. Keep region identity separate from element equality. .NET Span `==` compares
   the same region, not content. Persistent neoCLR Memory already documents
   owner/offset/length identity. Any borrowed equality and future `SequenceEqual`
   must name/document their distinct semantics; borrowed views must not acquire
   boxing-based Object equality or hashing.

Allocation-free iteration and basic sequence/search helpers are useful follow-ups,
but ordinary indexed loops already provide the primitive operation. Do not make
all .NET extension overloads, stackalloc, unmanaged constructors, pinning or
marshaller integration prerequisites for this bounded release. Unsupported memory
sources must remain explicit limitations, not implied by generic type spelling.

neoCLR keeps its chosen Slice names and implicit typed borrowing, avoiding an
AsSpan/AsSlice compatibility layer. UTF-8 byte views and grapheme Char semantics
remain distinct: .NET's Span<char> text examples are not a storage model to copy.

## Primary sources reviewed

- [Span<T>](https://learn.microsoft.com/en-us/dotnet/api/system.span-1?view=net-10.0)
- [TryCopyTo: overlap and unchanged short destination](https://learn.microsoft.com/en-us/dotnet/api/system.span-1.trycopyto?view=net-10.0)
- [Equality: region rather than contents](https://learn.microsoft.com/en-us/dotnet/api/system.span-1.op_equality?view=net-10.0)
- [Indexer: reference return](https://learn.microsoft.com/en-us/dotnet/api/system.span-1.item?view=net-10.0)

Read these alongside the existing [memory adoption audit](../../memory-view-library-audit.md)
and [native view contract](../../native-memory-views.md). Convenience methods are
recommendations; this review does not mark their implementation complete.
