# Slices and memory

Choose a buffer type by the access and lifetime it needs. `Slice<T>` and
`ReadOnlySlice<T>` borrow contiguous storage for immediate work. `Memory<T>` and
`ReadOnlyMemory<T>` retain a storage owner and a window, so they can be stored in
objects and kept across asynchronous operations.

These are development APIs. Use a matching compiler, runtime and class-library
bundle; the published preview link does not imply every development capability
is included in that release.

## Share storage explicitly

This excerpt comes from the executable native memory-view qualification:

```raven
{{MEMORY_WINDOW_SAMPLE}}
```

The array, memory window and slices all see the changed byte. Slicing an array
returns a `Slice<T>` without copying. Nested slices narrow the same storage.
`ReadOnly` prevents writes through that view; it does not freeze other aliases.
Use `ToArray()` when an independent, shallow copy is required.

| Type | Access | Lifetime |
| --- | --- | --- |
| `Slice<T>` | Read and write | Immediate borrowed access; cannot be stored on the heap or retained across suspension |
| `ReadOnlySlice<T>` | Read | The same borrowed lifetime rules |
| `Memory<T>` | Read and write through a borrowed slice | Retained owner, offset and length; can cross `await` |
| `ReadOnlyMemory<T>` | Read through a borrowed slice | The same persistent ownership, without writable access through this view |

Bounds are checked. Empty windows are valid, including at the end of storage.
Invalid indexing and slicing fault; they do not clamp the requested range.
`Index` and `Range` support from-end positions and half-open array windows.

## Put the right type at the boundary

The design follows the role of .NET's Span/ReadOnlySpan and Memory/ReadOnlyMemory,
using Slice names and direct range results in Raven. Conversions establish views;
callers do not need a parallel layer of `AsSlice` methods. APIs that require an
owned array still say so explicitly.

Borrowed views keep immediate parsers and codecs from creating temporary arrays.
Persistent views let asynchronous socket and TLS operations retain exactly the
selected bytes and their owner. Callers must keep borrowed input stable until an
operation finishes and avoid competing writes to pending receive destinations.
GC reachability provides lifetime, not exclusive access.

HTTP memory content retains `ReadOnlyMemory<byte>`; response construction retains
the supplied content producer instead of converting a sequence into a new body.
JSON byte parsing and UTF-8 decoding borrow slices, materializing owned results
where needed. Shared stream interfaces still have array-based contracts; their
coordinated migration remains open. These integrations reduce specific copies,
not every allocation in an HTTP or JSON operation.

The benefit is a consistent distinction between storage, access and ownership.
The cost is migration from older array-range copying and content helpers, plus
explicit attention to aliases and asynchronous lifetimes. This is not .NET source
compatibility or a claim of universally faster code.

## Text keeps its own semantics

A String owns UTF-8 bytes, while Char represents a grapheme rather than a storage
unit. String indexing and slicing remain text operations. `String.Utf8Bytes`
exposes a borrowed readonly byte view for byte-oriented work without a conversion
array; it does not change the string's grapheme indexing contract.

## Qualified scope

The interpreter supports all four view types. Current native JIT/AOT qualification
covers byte windows, checked indexing and mutation, nested ranges, readonly UTF-8
views, owner retention through collection, and persistent views across suspension.
Socket and TLS integration includes completion, cancellation and selected-window
checks. Other native element specializations remain unqualified.

[Type and member reference](../../docs/memory-views.html) ·
[Reference and value types](../types/) · [Arrays](../arrays/) · [HTTP](../web/) · [Garbage collection](../gc/) ·
[Implementation and evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/memory-view-library-audit.md)
