# Direct managed heap references

Format 5 makes heap.new consume an ordinary value T and produce T& directly. Frame
and heap access share ByRef, ldobj, stobj and ldflda. There is no Ref<T> signature,
heap.load/store pair, or boxing/unboxing bridge. newobj still constructs an ordinary
value; initobj still defaults addressed storage without invoking a constructor.

```text
ldc.i4 42
newobj Counter
heap.new
ldflda Counter::Age
ldobj Int32
```

The [heap reference sample](../examples/heap_references.neoil) returns a Counter& from
a factory, forwards its field address, and mutates the field after the factory exits.
An interior reference roots the entire heap allocation, including referenced objects
in its other fields. Interface views likewise preserve the concrete heap root.

## Lifetime and storage

Frame provenance and heap provenance are explicit runtime facts. Returning an address
into the current frame still faults, regardless of aliases or helper calls. Returning
a heap reference is valid; final collection roots the returned value's reachable graph.
There is no implicit promotion when &local escapes and no manually invalidated handle.

Reference-valued record fields and erased payloads may contain heap-backed references.
The runtime rejects frame-backed references in those positions, even without optional
verification. This conservative subset prevents a returned record, heap object or
erased payload from hiding a scoped reference. Nested managed addresses (T&&) remain
unsupported. ByRef generic arguments and reference array elements are supported;
stored references must be heap-backed. See [managed arrays](managed-arrays.md) and
[ArrayList](array-list.md). Use ldobj on an owner and ldfld to read its reference-valued
field; taking that field's address would create T&&.

Reference copies preserve identity; value copies copy inline fields and preserve
embedded reference identities. A new heap allocation gets a fresh identity, even
when its value equals another allocation. Replacing a root value of the same type
preserves existing field paths. An explicit local-to-heap copy is simply ldloc then
heap.new; aliases to the original local continue to address that local.

The interpreter stores heap roots in stable cells owned by the collector. Heap
references hold weak host links, so guest cycles do not become uncollectable host
reference-count cycles. Tracing determines reachability; live guest references keep
their roots in the collector. A reclaimed or dropped execution's cell cannot be
revived through a stale reference. No guest destructor/finalizer runs during collection.

## Hosting, monitoring and migration

Execution.value may contain a heap-backed T&. ManagedHeap.read_reference inspects its
value or field only in the owning execution; get(allocation_id) copies a whole root
for inspection. Numeric identities alone never authorize cross-execution access.
Passing managed references into another invocation remains unsupported. Host snapshots
and opaque reference copies are not independent root registrations.

Collection statistics remain available. A bounded history now reports collection
reason, incoming root-edge count, before/after counts and reclaimed objects; CLI
`run --gc-events` prints it on completion. See [GC monitoring](garbage-collection.md#monitoring-the-collector).

Format 4 artifacts must be reassembled, including System. The loader rejects them
before interpreting the changed heap.new stack effect. Replace Ref<T> signatures
with T&, heap.load with ldobj T, and heap.store with stobj T. Remove a following pop
when it consumed heap.store's old Void result: stobj has no result. The runtime-service
name is now ManagedHeap and heap.new also requires SlotReferences.

Heap constructor destinations, block-scope lifetimes, [pinning](pinning.md) and persistent host
roots remain future work. The [optional Object hierarchy](object-hierarchy.md) does
not determine allocation or addressing mode.
