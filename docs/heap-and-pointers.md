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
.local Point* point
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

This implementation supports access to allocations created by its own `heap.alloc` or `localloc`.
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
Free and frame return restore payload budget, but identities include freed allocations. The byte
limit excludes side tables, initialization maps, padding for zero-sized physical
allocations, and other host overhead; it is not a total memory quota or sandbox.
Layout expansion also has depth/complexity limits. Resource failures are currently
Faults; a recoverable allocation API can expose Result later.

## Remaining capabilities

Direct guest access to externally supplied memory, argument/local address-taking,
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

## Copying and initializing memory

The assembler and interpreter support four familiar CIL operations. Stack operands
below are listed bottom to top; each instruction consumes them and pushes nothing.

| Instruction | Operands | Behavior |
| --- | --- | --- |
| `initobj T` | `T* destination` | Zero the supported native layout, without calling a constructor |
| `cpobj T` | `T* destination, T* source` | Read an initialized value and copy it into the destination |
| `initblk` | `pointer destination, Int32 fill, integer size` | Fill `size` bytes with the low eight bits of `fill` |
| `cpblk` | `pointer destination, pointer source, integer size` | Copy `size` bytes and their initialization state |

Typed operations require matching pointee types and natural type alignment. Their
layouts are the same as `sizeof`, `ldobj`, and `stobj`: native scalars, pointers,
Void, and records composed of supported fields. Zero initialization produces numeric
zero, false, null pointers, and recursively zeroed records. It initializes padding
as well. It does not allocate storage or imply ownership. String, Error, unions,
and Ref still lack native layouts and cannot use these operations. This is not a
promise that every future type has a valid all-zero representation.

`cpobj` copies fields by value; source padding need not be initialized, and destination
padding becomes uninitialized, as with `stobj`. Pointer fields copy their addresses
and diagnostic identities; pointees are not cloned. Both typed and block copies
snapshot the source before writing, including when the ranges overlap.

Block operations accept nonnegative Int32/IntPtr counts or UIntPtr counts, measured
in bytes. They accept any pointer target and use byte alignment. These are explicit
prototype choices: CLR documents unsigned 32-bit counts, natural machine alignment
unless prefixed with `unaligned.`, and unspecified overlapping `cpblk` behavior.
See Microsoft's [cpblk documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.cpblk)
and [initblk documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.initblk).
No volatile or unaligned prefixes are implemented yet.

All four operations require live tracked allocations with in-bounds ranges. Even
zero-length operations validate addresses (one-past-end is permitted; null, stale,
and foreign pointers Fault), but change no bytes or pointer identities. Block copies
can carry uninitialized bytes; subsequent typed reads still Fault on those bytes.
Fill operations mark written bytes initialized, but arbitrary fills can still produce
invalid values, such as a Boolean other than zero or one.

A whole pointer representation copied by `cpblk` retains its diagnostic identity.
Partial writes and fills discard overlapping identities; pointer bits alone do not
establish tracked ownership. Copying pointer bytes in separate fragments therefore
does not reconstruct tracking. Range checks happen before mutation. These diagnostics
are interpreter checks, not reference counting or garbage collection.

Run `cargo run -- run examples/memory.neoil` for typed initialization, independent
record copying, byte filling/copying, and explicit freeing; it prints 42 twice.


## Frame-local allocation

`localloc` consumes a byte count and produces `Byte*`. The evaluation stack must
contain only the count when it executes. Nonnegative Int32/IntPtr and UIntPtr counts
are accepted, matching the prototype's block-operation count convention. Use
`sizeof T`, `localloc`, and `ptr.cast T` to obtain storage for a native-layout value.
The address is aligned for all currently supported primitive layouts, including
Double, Int64, and native pointers. Zero bytes still produces a distinct tracked,
non-null allocation with no accessible payload.

The storage starts uninitialized. Use `initobj`, `initblk`, or explicit stores before
reading it. There is no method-level `localsinit` flag yet. Each allocation belongs
to the current function invocation and lasts until that invocation returns, including
allocations inside loops. Returning releases all its local buffers before resuming
the caller. Terminal Faults drop the entire execution and its buffers. `heap.free`
rejects frame-local storage, including through casts or address conversions.

Pointers can be passed to callees and copied into records or other storage. Such
copies do not extend lifetime. After the allocating frame returns, tracked accesses
through escaped pointers Fault; returning a record copied with `ldobj` instead
preserves its independent value (but any pointer fields keep their original lifetime).
Native code may use a local pointer during a synchronous call, under the existing
P/Invoke safety contract, but must not retain it past the owning frame's lifetime.
This is an explicit frame-lifetime instruction, not inferred ownership or GC.

The interpreter uses aligned host buffers for these local pools, not the host
machine's call stack. Their guest lifetime is tied to a neoCLR frame. Both local
and heap buffers share `pointer_bytes` and `pointer_allocations` limits. Frame return
restores the live byte budget; allocation identities are never recycled, so repeated
calls can still reach the per-execution identity limit. `Execution.memory` statistics
include both storage kinds while live; completed entry-frame local storage is gone.

The instruction follows CLR's local-pool lifetime and primitive alignment contract;
see [Microsoft's localloc reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.localloc).
The prototype adds tracked pointer diagnostics and explicit uninitialized-read Faults.
It does not yet implement `ldloca`, `ldarga`, a byref type, or method initialization flags.
Run `cargo run -- run examples/stack.neoil` for local record storage passed to a callee
and returned by value; it prints 42 and leaves no live allocations.
