# Managed arrays

Implemented arrays use the same value/reference distinction as records. `T[]` owns
its elements; copying it copies the elements. `T[]&` aliases an array location.
There is no mandatory Object or System.Array base class. The native pointer/length
`System.Array<T>` descriptor remains a separate [buffer API](arrays-and-pointers.md).

## Storage and CLR alignment

| Instruction | Stack effect | Contract |
| --- | --- | --- |
| `newarr T` | length → T[]& | Create a zero-based, fixed-length managed heap array with initialized elements |
| `array.create T` | length, initial T → T[] | Create an owned array value by copying an explicit initializer |
| `ldlen` | T[] or T[]& → UIntPtr | Native unsigned length, as in CLR IL |
| `ldelem T` | T[] or T[]&, index → T | Copy an element, with normal evaluation-stack scalar normalization |
| `stelem T` | T[]&, index, T → | Replace an element after bounds/type checks |
| `ldelema T` | T[]&, index → T& | Managed element reference retaining its owner's provenance |

Lengths and indices accept Int32, IntPtr or UIntPtr. Negative lengths, invalid
indices, type mismatches and resource exhaustion produce terminal Faults. This
preview caps lengths at Int32.MaxValue; empty arrays are valid.

`newarr` retains the CLR's heap-allocation role. `array.create` is the explicit
extension for owned values: storing its result in a local gives frame-owned
storage; `heap.new` copies an owned value into the managed heap. No native pointer
or `localloc` is required. The interpreter represents payloads with host vectors;
“stack array” means frame-owned lifetime, not guaranteed native-stack placement.

Default initialization follows [initobj](managed-initialization.md): numeric values
and supported records can be defaulted; String, non-null managed references and
other types without a supported default require an explicit initializer. Thus
`newarr String` is rejected rather than manufacturing null. Element access is
invariant; this slice adds neither covariance nor reference-type array behavior.

## Lifetime and copying

A reference to a frame-owned element cannot return from that frame. A reference
into a caller's array can be forwarded back. A heap element reference roots the
whole allocation, including heap references nested in its elements. Runtime checks
apply even when static verification is skipped. Neo reads/writes those managed
references automatically, exactly as for record fields.

Ordinary array copies are independent. Embedded managed references retain their
aliases and must be heap-backed, as in record fields. Direct `T&[]` element types
are not supported in this slice; records containing heap references are supported.

Array locations have fixed shape: replacing an initialized array must preserve its
length, recursively through nested arrays and record fields. Existing element
references then remain valid and observe the replacement. This preview rule also
applies when no element reference is currently live. Rebinding a heap array reference
to another allocation can change length; references to its former elements keep
that former allocation alive. Arrays of array values work as nested copies with
this shape rule; CLR-style jagged reference arrays and multidimensional arrays are
future work.

## Neo projection

```swift
var local: int[] = [1, 2, 3]
let copied = local
let element: int& = &local[1]
element = 42
let shared: int[]& = new int[3]
shared[1] = local[1]
let words = new array(2, "Neo")
let empty = array(0, 0)
```

Nonempty literals and `array(length, initialValue)` produce owned values.
`new T[length]` lowers to `newarr`; `new array(length, initialValue)` uses explicit
initialization followed by `heap.new`. Indexing reads a value; `&items[index]`
forms a managed reference. Mutation of an owned local requires `var`; a managed
reference binding can mutate its target even when declared with `let`.
`.Length` converts `ldlen` to checked Int32. `typeof(T[])` has an exact array identity.
Array arguments follow the same explicit `T[]` versus `T[]&` convention as records.

Run the demonstration:

```sh
cargo run --locked -- run examples/source/arrays.neo --gc-stats
cargo run --locked -- assemble examples/source/arrays.neo /tmp/arrays.neo.json
cargo run --locked -- run /tmp/arrays.neo.json
```

It prints `2`, `9`, `3`, `42`, `Neo`, `CLR`, returns 42, and reclaims both heap arrays.
The grammar is in [neo.ebnf](neo.ebnf); [runtime tests](../tests/managed_arrays.rs)
and [Neo tests](../tests/neo_arrays.rs) exercise copying, provenance and failures.

## Limits and next decisions

`Limits.array_elements` (65,536 by default) and `Limits.array_bytes` (16 MiB)
bound aggregate logical payload across frame slots, evaluation stacks and managed
heap objects, including owned copies and nested payloads. Initializer multiplication
is checked before allocation. The interpreter scans payloads at instruction
boundaries and collects unreachable heap arrays before rejecting aggregate pressure.
These are payload budgets, not exact process-memory limits; metadata, host allocator
overhead and transient instruction copies are not fully represented. Scanning and
copying are intentionally simple preview implementations.

Existing GC object statistics include heap arrays as allocations; they do not
report array payload bytes. Host invocation does not yet accept array input schemas.
The JSON format remains 5 with additive Array type and array opcode variants.

Keep future work driven by examples: reference-element initialization/nullability,
checked slice views, copying cost and possible moves, managed collections, and
pinning/native layout. Native interop requires an explicit layout and pinning
contract; a managed element reference is not a native pointer. Inheritance must
preserve tracing and element provenance without imposing an Object root.

## Next source refinement: fixed-length types

The intended spelling distinguishes shape, addressing mode and allocation:

```swift
// Proposed next slice; not accepted by the current parser yet.
let arr: int[3] = [1, 2, 3]
let view: int[]& = &arr
let arr2: int[]& = new int[3] { }
```

`int[3]` expresses an owned array with a statically known extent. `int[]&` expresses
managed reference access to an array with runtime length. The reference can point
to either storage region; `new` selects the managed heap. An immutable owned binding
would need an explicit policy for forming a writable view; today's Neo requires
`var` to take a managed address of owned storage.

Before implementing this spelling, define fixed-extent type identity and conversions
from a fixed-length owner to a runtime-length array view, enforce initializer counts,
and specify empty braces as default initialization versus explicit element lists.
The current implementation uses owned `int[]` with runtime length and supports
`new int[3]` without braces. Fixed-length annotations must become real checked
contracts, not unchecked decoration.
