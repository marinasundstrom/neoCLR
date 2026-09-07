# ArrayList<T>: explicit growable storage

System.Collections.ArrayList<T> is a small platform-written growable list for
native-layout values. It uses ordinary generic types, pointer fields, member calls
and existing heap instructions. There is no Collections.Generic namespace, reference
type flag, special collection opcode, interface implementation or built-in GC.
A future System.Collections.List<T> interface can describe its list contract without
an I prefix. The interface and its final member set are not implemented here.

| Member | Contract |
| --- | --- |
| Allocate(Int32 capacity) -> ArrayList<T> | Allocate an empty list with explicit initial capacity; zero is valid |
| Count: Int32 | Number of initialized elements |
| Capacity: Int32 | Number of element slots in the current buffer |
| Add(T value) -> Void | Append a value copy; grow the buffer when full |
| Item[Int32]: T | Checked get/set for indices from zero through Count - 1 |
| Free() -> Void | Release the buffer and shared state exactly once |

Indexer metadata associates get_Item/set_Item. There is no implicit constructor,
automatic destruction, iterator protocol, Remove, Clear, sorting or comparer API.
This is a generic type despite the CLR's historical non-generic ArrayList spelling;
we intentionally use System.Collections directly for generic and non-generic types.

## Representation and copying

The list value contains one private pointer to an internal ArrayListStorage<T> record.
That record contains Data: T*, Count: Int32 and Capacity: Int32. Allocate explicitly
creates the data buffer and this state block. Spare capacity is uninitialized; it is
not filled with fabricated default T values. Count starts at zero.

Copying the list descriptor copies its state pointer. Both copies see subsequent Add,
indexer writes, count changes and capacity growth. They are aliases, not independent
collections and not reference-counted owners. Allocate/Free make allocation and lifetime
explicit even though consumers call ordinary methods instead of writing heap IL.
The caller must keep the list live for every borrower and free it only once. Free
invalidates all aliases; the interpreter reports subsequent tracked access as a Fault.

Growth from zero capacity chooses four slots. Otherwise capacity doubles with checked
Int32 arithmetic. Add allocates a new buffer, copies only Count initialized elements
with cpobj T, releases the old buffer, updates shared Data/Capacity, then stores the
new element and increments Count. The stable state block keeps copied descriptors
current after a buffer replacement. Growth policy is a preview implementation detail.

Element loads, stores and relocation follow ordinary native value-copy semantics.
Pointer fields remain aliases to their targets. Free releases the list's two allocations;
it does not follow element pointers or invoke destructors. Any buffer-interior pointers
obtained through lower-level access become stale when growth replaces that buffer.
No thread-safety, mutation-during-enumeration or concurrent access contract is provided.
As with other pointer-containing records, the current hosting API does not import a
list descriptor into a fresh invocation. Construct and use it within the guest execution;
copying it to the host does not create a cross-execution ownership transfer.

## Bounds, errors and supported elements

Indexer access is checked against Count, not Capacity. Negative capacity, invalid
indices, capacity/offset overflow, allocation limits and expired storage produce
terminal Faults under the existing array/memory contract. Add is not a recoverable
allocation Result API. On a terminal Fault, execution teardown reclaims tracked
allocations; there are no guest cleanup handlers. Native host failure containment
is subject to the existing runtime limitations.

Supported elements include native numeric primitives, Void, typed pointers and
records composed of native-layout fields. Byte storage conversion is preserved during
Add and indexed writes. Void elements have zero payload size but still count as list
elements and use the same allocation-lifetime rules. String, Error, RuntimeTypeHandle,
System.Value and current System.Option/Result carriers have no native payload layout,
so lists of those values are not yet supported, even with capacity zero. Pointer
signatures to such types can be stored, without adding support for their pointee layout.

These limits match the current Array<T>/native storage foundation. Supporting more
payloads requires the [ordinary storage migration](value-storage.md#retirement-decision),
not a hidden erased-value fallback inside ArrayList.

## Run the sample

```sh
cargo run --locked -- verify examples/array_list.neoil
cargo run --locked -- run examples/array_list.neoil
cargo test --locked --test array_list
```

[ArrayListDemo](../examples/array_list.neoil) allocates with capacity one, copies the
descriptor, and appends squares through the copy. It reads the count and items through
the original descriptor and releases once. Output is `ArrayList count:`, `5`, `0`, `1`,
`4`, `9`, `16`, and `=> Void`, each on a separate line. The walkthrough also exercises
the source and assembled-artifact paths.

Raven-like explanatory pseudocode (not a compiler input):

```text
let values = ArrayList<Int32>.Allocate(1)
let alias = values
for (var i = 0; i < 5; i = i + 1) {
    alias.Add(checked(i * i))
}
Console.WriteLine(values.Count)
for (var i = 0; i < values.Count; i = i + 1) {
    Console.WriteLine(values[i])
}
values.Free()
```

The library implementation is [ordinary neoIL](../runtime/System/Collections/ArrayList.neoil).
This demonstrates shared access through an explicit pointer field without making a
class declaration inherently reference-allocated or adding a mandatory ownership model.
