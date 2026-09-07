# Explicit borrowed interface references

## Managed slot views

The preferred call-scoped view is now `I&`. Start with `ldloca value` or an existing
Concrete& parameter, then `interface.borrow I`. The resulting view refers to the
same initialized concrete slot and retains no owner. It can be passed and forwarded
through active calls, but cannot escape into locals, fields, erasure, returns or native
calls. It supports ordinary values such as String-containing records without requiring
native layout. It is used for dispatch, not for loading an abstract interface value.

`callvirt` respects the declared receiver mode: ordinary instance implementations
receive a copy, while `.method instance byref` implementations receive Concrete& and
can explicitly write the original slot with stobj. Conformance requires matching
receiver modes and output contracts. The native-pointer view cannot supply a managed
byref receiver. See [slot reference contracts](reference-slots.md).

| Spelling | Formation | Lifetime/access |
| --- | --- | --- |
| I& | Concrete& followed by interface.borrow I | Managed, initialized, call-scoped concrete slot; value or byref dispatch |
| InterfaceRef<I> | Concrete* followed by interface.borrow I | Existing explicit native pointer rules; value receiver dispatch only |

[The List sample](../examples/interfaces.neoil) now passes List<Int32>& directly
from an ArrayList local, without native receiver-descriptor allocation. ArrayList's
existing value-receiver methods retain their shared backing-state behavior; a safe
view alone does not change a method's receiver mode. [The Counter sample](../examples/reference_receivers.neoil)
demonstrates byref interface dispatch that changes an inline field.

[System.Equatable<T>](equality.md) supplies typed Equals(T) dispatch for primitives,
type descriptors and user-defined records through the same managed views.
[System.Clonable<T>](cloning.md) supplies explicit Clone() dispatch with a byref
receiver, independently of ordinary value copying.

The following sections describe the retained native-pointer view specifically.

## Native-pointer interface views

An interface is a contract, not a base storage type. An ordinary value does not
implicitly convert to an interface value, become boxed, or acquire an owner.
`InterfaceRef<I>` is an explicit borrowed view of an implementing value in typed
pointer storage. It carries the receiver pointer and the information needed to
select the implementation. Copying the view copies that capability.

A future language could express the distinction with Raven-like pseudo syntax:

```text
let view: &List<Int32> = (&values) as &List<Int32>
```

This is illustrative, not implemented language syntax. The address operation must
supply live storage for the concrete value; taking the interface view must not
silently allocate a box. A language may enforce borrowing rules or hide some syntax,
but ownership and allocation remain separate platform decisions.

## Declarations and dispatch

```text
.interface Read
    .method instance Get() -> Int32
    .end
.end

.type Cell
    .implements Read
    .field Value Int32
    .method instance Get() -> Int32
        ldarg this
        ldfld Cell::Value
        ret
    .end
.end

.function ReadCell(Cell* cell) -> Int32
    ldarg cell
    interface.borrow Read
    callvirt instance Read::Get()
    ret
.end
```

`.interface` emits an ordinary type definition with interface representation.
`.implements` records the declared contract on the implementing type. Public IL
instance methods implement slots by name and exact parameter and return types.
Overloads use full signatures. Generic interfaces and generic implementing types
substitute their own type arguments before matching. Existing property/indexer
metadata associates abstract accessor declarations with the public contract.

`interface.borrow I` consumes `Concrete*` and produces `InterfaceRef<I>`. It checks
that Concrete declares and satisfies I and has a native layout. It does not allocate,
copy the concrete value, retain its lifetime, or access the pointed bytes.
The concrete type comes from the typed pointer, not an object header. An explicit
unsafe pointer reinterpretation retains the usual memory and layout obligations.

`callvirt instance I::Member(...)` consumes the matching `InterfaceRef<I>` receiver
followed by the declared arguments and returns one value, including Void. It checks
receiver access, reads the concrete value and invokes the selected implementation.
A direct `call` cannot invoke an abstract interface declaration. This `callvirt`
subset does not introduce class virtual methods or .NET null-check-only callvirt.

The interpreter represents the view using a typed pointer and interface metadata.
A native backend could use a data pointer and dispatch table, but this preview fixes
neither its machine layout nor a native ABI. Bare I has no value storage layout;
InterfaceRef<I> also has no native-memory layout yet. Views can be passed in guest
arguments, locals and ordinary interpreter value records, but cannot currently be
stored with stobj or imported through the host invocation API.

## Lifetimes and receiver semantics

The concrete receiver may use frame-local or explicitly allocated heap storage.
The caller keeps it initialized and alive. A view neither frees nor owns that storage.
Null or stale pointers may be carried by a view; dispatch faults when receiver memory
is inaccessible. Returning a view of a local allocation does not extend the frame's
lifetime. Tracked pointer diagnostics are not static lifetime verification, nor a
sandbox guarantee for arbitrary native addresses.

Instance dispatch preserves the current receiver-copy semantics. Changing inline
receiver fields does not write them back into pointed storage. Mutations through
pointer fields affect their shared targets. This makes the current ArrayList state
model usable through a borrowed List contract. By-reference `this` is available through the managed slot view described above;
the native-pointer view does not acquire that capability implicitly.

System.Collections.ArrayList<T> now implements System.Collections.List<T> with Count,
Item get/set and Add. Allocate and Free remain concrete ownership operations, outside
the borrowed list contract. See [the executable sample](../examples/interfaces.neoil):
it addresses an ArrayList local, borrows a managed List view, adds values,
passes the view to Sum and releases through the concrete owner. It prints 42 and 2.

```sh
cargo run --locked -- verify examples/interfaces.neoil
cargo run --locked -- run examples/interfaces.neoil
```

## Preview boundaries

There is no interface inheritance, variance, default implementation, static interface
member, method-generic slot, explicit slot mapping, automatic cast, owning interface
reference, or native interface ABI. Implementations must be public IL instance methods;
native work can be reached through an ordinary IL wrapper. Contracts cannot declare
fields, layout or method bodies. Interface references preserve exact closed interface
identity, even when two contracts happen to have identical methods.

Reachability conservatively expands each interface call to all matching concrete
implementations in the loaded module set. Several edges may therefore share one IL
instruction index. If an implementation has generic parameters that cannot be inferred
from its interface, analysis faults rather than claim a complete closed graph.
This limitation does not prevent interpreter dispatch to a known concrete receiver.
Interface formation and dispatch conservatively report InterfaceDispatch and
SlotReferences; the latter covers the managed operand path.

See the [managed slot-reference contracts](reference-slots.md) for implemented
reference parameters, output assignment and explicit reference receivers, and the
[Raven-like pseudocode guide](references-in-pseudocode.md) for their language projection.
