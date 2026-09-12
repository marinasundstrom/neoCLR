# Managed arrays

Implemented arrays use the same value/reference distinction as records. `T[]` owns
its elements; copying it copies the elements. `T[]&` aliases an array location.
There is no mandatory Object or System.Array base class. The native pointer/length
`System.Array<T>` descriptor remains a separate [buffer API](arrays-and-pointers.md).

## Storage and CLR alignment

| Instruction | Stack effect | Contract |
| --- | --- | --- |
| `array.alloc T` | length → T[]& | Reserve managed heap slots; reads fault until individually initialized |
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
aliases and must be heap-backed, as in record fields. Direct `T&[]` element types and records containing heap references are supported.
Use an explicit initializer or `array.alloc` for reference elements; non-null managed
references still have no default value. `ldelem` copies the reference and `stelem`
replaces it. `ldelema` on reference elements is rejected because nested managed
references are not exposed.

Array locations have fixed shape: replacing an initialized array must preserve its
length, recursively through nested arrays and record fields. Existing element
references then remain valid and observe the replacement. This preview rule also
applies when no element reference is currently live. Neo declaration initialization
uses local.reset to renew a local between iterations; that operation requires no live
managed aliases and differs from assignment to an existing array. Rebinding a heap array reference
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

`array.alloc` is the bounded storage primitive used by [ArrayList](array-list.md).
Spare slots have a type but no value or GC edges. Reads through ldelem or an element
address fault until written. Allocation bounds and fixed-shape replacement rules
still apply. Whole-array replacement cannot turn an initialized element back into
an uninitialized slot, including nested array elements. This extension preserves newarr's initialized-element contract.

Keep future work driven by examples: reference nullability,
checked slice views, copying cost and possible moves, managed collections, and
pinning/native layout. Native interop requires an explicit layout and pinning
contract; a managed element reference is not a native pointer. Inheritance must
preserve tracing and element provenance without imposing an Object root.

## Neo local extents and heap initializers

```swift
let arr: int[3] = [1, 2, 3]
var writable: int[3] = arr
let view: int[]& = &writable
let arr2: int[]& = new int[3]
let words: string[]& = new string[2] { "Neo", "CLR" }
```

The local annotation checks the extent at initialization. Literal mismatches are
compiler errors; a dynamically produced array is checked at runtime before binding.
The annotation requires a nonnegative Int32 literal and an initializer. Ordinary
replacement and writes through aliases preserve the runtime's fixed array shape.
`int[3]` is currently a local extent constraint lowered to int[], not a distinct
metadata type. Fields, signatures, generic arguments and typeof still use T[].

`int[]&` describes reference access independently of allocation; `new` chooses
managed heap storage. Empty initializer braces request supported default values.
Nonempty braces require exactly the specified number of elements and support types
without defaults, such as strings and records. Length is evaluated once, then checked
before evaluating elements once each from left to right. Trailing commas and multiline
initializers are accepted. Existing `new int[3]`, literals and array(length, value)
forms remain supported. Writable references to owned locals still require var.

## Nominal object-reference elements (2026-09-12)

Arrays now admit ordinary interface-typed elements in addition to nominal class
references. `newarr Read` initializes each element to a typed null reference; `stelem Read` stores an explicit `castclass Read` view, and `ldelem Read` returns the same object
identity for dispatch. An uninitialized `array.alloc` slot remains different from null.
Element types stay invariant and storage still requires an exact static type.

A managed address returned by `ldelema Read` addresses the **slot containing a reference**.
`ldobj Read` copies that reference; `stobj Read` rebinds the slot. It does not overwrite
the old object's fields. Ordinary interface locals and fields support the same indirect
operations. The slot's lifetime remains independently checked: a heap array element
address retains the array and its referenced objects, while a current-frame slot cannot
escape just because the object stored in it is on the heap. Legacy borrowed interface
views are dispatch projections, not addresses of ordinary interface slots.

This fills a storage gap in the [nominal interface contract](raven-interface-contract.md).
Primary sources checked 2026-09-12: Microsoft's
[Newarr contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.newarr?view=net-10.0)
and [Ldelema contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldelema?view=net-10.0)
distinguish the array object reference from a managed pointer to an element. Reusing the
existing element/slot machinery preserves that distinction inside elements without
adding instructions or changing metadata. A special interface-buffer wrapper would add
an unnecessary allocation and another library contract.

There remains an explicit ABI difference: neoCLR's existing `newarr` returns `T[]&`,
whereas CLI newarr returns an ordinary array object reference. This slice does **not**
resolve that difference, introduce covariance or make the existing owned `T[]` nullable.
Nominal class constructors still cannot default a legacy array-reference field; explicit
field-based construction can retain an already allocated buffer. The Raven importer
must continue rejecting array signatures until their ordinary-reference representation
is implemented. This is a prerequisite for adapting collections, not the completed
collection migration. String defaults also remain unsupported.

`tests/object_reference_arrays.rs` exercises nulls, dispatch/identity, slot rebinding,
GC retention through an escaped element address, generic buffer fields, invalid stores,
invariant access, rejected frame-slot escape and borrowed-view misuse. Ten new tests
and 90 related tests pass. Existing Neo array and ArrayList
regressions validate that their value-copy and managed-reference behavior remains intact.
