# Native heap and pointers: implemented subset

neoCLR types describe values and their behavior. They do not choose allocation or
lifetime through a class/struct distinction. Values copy by default; a pointer is
itself a value whose copy aliases the same storage. Neither copying nor dropping a
pointer acquires or releases ownership. Counted references and optional GC remain
separate future abstractions. Familiar CLR capabilities are the baseline, with
intentional departures and incomplete prototype behavior distinguished below.

## Allocation, initialization, and access

`heap.alloc T` consumes a nonnegative Int32, IntPtr, or UIntPtr element count and returns `Ptr<T>` (`T*`). Storage
comes from the host native allocator and is aligned for T. It is logically
uninitialized: reads require initialization through stores. Allocation does not
construct a value. `newobj T` constructs a record value; `stobj T` copies it into
storage. `ldobj T` copies it out. Changing storage later does not change that copy.
For records containing pointers, the copied pointer fields still alias their targets.

```text
.local point: Point*
ldc.i4 1
heap.alloc Point
stloc point
ldloc point
ldc.i4 10
ldc.i4 32
newobj Point
stobj Point
ldloc point
ldflda 0
ldind.i4
pop
ldloc point
heap.free
pop
```

`ldflda` uses a field index to select its address. `ptr.add` consumes an Int32 or IntPtr
**byte** offset, not an element index. Multiply by `sizeof T` for element stepping.
`ptr.cast T` preserves the address and changes its target type. Casts do not perform
access or confer validity. Typed loads/stores require a matching pointer target;
use a cast to reinterpret storage. `ldind.i4`/`stind.i4` specialize Int32 access.
Stores consume their operands without pushing Void, like their CIL counterparts.
`heap.free` does produce the inhabited Void value, consistent with this prototype's
explicit allocation service operations.

`heap.free` requires an allocation base pointer, possibly cast to another target
type. Interior-pointer free and double free Fault. Freeing null is a no-op. Heap
pointers may escape a call frame and remain live until explicit free. Execution
teardown releases remaining allocations as host resource cleanup, not guest GC or
automatic guest object destruction. It invokes no guest finalizers.

## Native representation and layout

Pointer values contain real native addresses. In allocated storage a pointer takes
`sizeof(usize)` bytes with native pointer alignment and native byte order. The Rust
interpreter also carries diagnostic allocation identity and offset information
outside those bytes. These are not part of the guest pointer's native layout.
`ceq` compares addresses for matching pointer types; it does not compare pointee data.
`ptr.null T` creates address zero, representing an actual null pointer. Ordinary
absence in the library remains Option.

Supported storage layouts:

| Type | Size | Alignment |
| --- | --- | --- |
| SByte / Byte | 1 | 1 |
| Int16 / UInt16 / Char | 2 | 2 |
| Int32 / UInt32 | 4 | 4 |
| Int64 / UInt64 | 8 | Host u64 alignment |
| Single | 4 | Host f32 alignment |
| Double | 8 | Host f64 alignment |
| Boolean | 1 | 1 |
| Void | 0 | 1 |
| IntPtr / UIntPtr | Native pointer width | Native pointer alignment |
| Ptr<T> | Native pointer width | Native pointer alignment |
| Record | Sequential fields, padding between fields and at end | Largest field alignment, or 1 |

Scalars use native byte order. Boolean reads accept only 0 or 1. Record reads require
initialized fields, but not padding. Pointer fields support recursive structures;
recursive by-value layouts are rejected. String, Error, Ref, Option, and Result do
not yet have native storage layouts. Their pointers can be represented and cast,
but the unsupported pointee layouts cannot be allocated or dereferenced. Layout is
computed on the execution host, not serialized as a fixed architecture's offsets.
This sequential subset is not a complete foreign ABI or StructLayout implementation.

Zero-count and zero-size allocations still return distinct live non-null addresses
and must be freed. They have zero logical payload bytes. Void loads need no payload
initialization but still require a valid pointer. One-past addresses are allowed;
access must fit within the allocation, including for zero-sized values.

## Checked interpreter scope

This implementation supports access to allocations created by its own `heap.alloc`.
It checks null, live allocation identity, address consistency, alignment, bounds,
initialization, and load/store types. Invalid operations terminate with a Fault and
instruction location. Pointer offsets are restricted to the allocation through its
one-past address. These are current interpreter restrictions, not a promise that
arbitrary native pointers are memory-safe or that the final VM requires ownership
tracking. They are also not full CLR unsafe pointer semantics.

Storing a pointer writes its actual address bytes and retains tracking in a side
table. Loading it preserves that tracking. Overlapping nonpointer stores discard
the tracking, even if they happen to write identical bytes. Reading those bytes as
a pointer is allowed; dereferencing/freeing an untracked non-null pointer currently
Faults. This limitation must be addressed alongside external native memory access.
Freed allocation identities are never reused, so tracked dangling pointers Fault
even if the native allocator later reuses an address. Explicit conversion through
an integer discards that identity: `ptr.fromint T` resolves the current live allocation
at the address, if any. An address may therefore identify reused storage; no historical
lifetime is inferred from integer bits. See [native integers](native-integers.md).

Defaults allow 16 MiB of live payload and 4096 allocation identities per execution.
Free returns payload budget, but identities include freed allocations. The byte
limit excludes side tables, initialization maps, padding for zero-sized physical
allocations, and other host overhead; it is not a total memory quota or sandbox.
Layout expansion also has depth/complexity limits. Resource failures are currently
Faults; a recoverable allocation API can expose Result later.

## Remaining capabilities

Direct guest access to externally supplied memory, stack allocation/address-taking,
block operations, unaligned access, explicit layout/packing, and foreign ownership
contracts remain unimplemented. The current native pointer subset is groundwork
for those capabilities, not a claim of .NET binary or unsafe-code compatibility.
Allocator selection, reference counting, GC, pinning, and automatic destruction
remain deferred; no universal management policy is selected by this slice.

The older `heap.new/load/store` instructions and execution-owned Ref arena remain
separate scaffolding, without reference counting or per-object free.
See the [pointer sample](../examples/pointers.neoil), [opcode reference](neoil.md),
and [memory layers](memory-model.md).

A first P/Invoke subset now supports scalar and pointer calls; see [native interop](native-interop.md).
