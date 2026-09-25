# Runtime library API design

This document defines the API review policy for bundled `System`, including its
existing Neo projection and the active Raven target. The target is .NET semantics unless a specific divergence has a demonstrated
benefit and documented costs. Type categories determine ordinary value/reference
behavior, including reference-type arrays. Runtime-library adaptation for Raven is
ongoing; Neo is outside this experiment and remains unchanged. Descriptions below of explicit `T&` API
contracts are an implementation inventory, not a requirement to preserve that model.
Managed byrefs remain distinct slot-access capabilities; raw pointers belong to native
interop and explicit native storage.

Expected recoverable failures use `Result<T, E>`. Terminal runtime faults are reported
to the host with diagnostic information, not modeled as a guest Exception class
hierarchy. Error types in Result are ordinary domain data and need no Exception base.
See [error policy](errors.md#platform-policy-clarified-2026-09-12).

This is a preview design. The inventory below distinguishes implemented contracts
from follow-up work; it does not imply that every library method has been migrated.

## Presenting the proof of concept

API feedback is an intended outcome of the POC. Familiar platform principles and
useful code compatibility guide the design, while neoCLR can define its own APIs.
Result-based error handling deliberately changes calling contracts and requires
adaptation of exception-based code. Evaluators should help test whether these
contracts are useful and coherent; similarity to .NET does not promise identical APIs.

The POC intentionally makes the library's state of development visible: familiar
.NET APIs, deliberate differences and undecided or missing pieces coexist. These
rough edges give evaluators useful evidence about the experiment and its direction.
Document provisional choices and current behavior clearly, with executable examples
and a coverage matrix. A familiar API name is not a claim of complete .NET behavior,
and implemented preview behavior remains open to revision.

## Modern library direction (2026-09-13)

The author wants modern APIs, including base types and namespace-level functions,
rather than inheriting historical library shapes solely for familiarity. Compare
current .NET designs as well as older APIs. Choose instance members, static helpers
or namespace functions by the contract they express, discoverability and target-language
projection; namespace syntax alone is not a reason to change CLI metadata.

Date/time is a future review example: separate date/time data from acquiring the
current time, and investigate a replaceable clock usable in deterministic tests.
This is a library-design direction, not a change to the implemented preview contract.
See [clock alternatives and validation](local-clock.md#future-clock-contract-review-2026-09-13).

## References are values

A `T&` local or parameter contains a reference value. Passing or returning it copies
that reference value, preserving the target's identity; it does not copy the T at
the target. The reference binding and the referenced storage are distinct locations.
Rebinding a callee's reference parameter does not rebind the caller's reference
variable. Writing through the reference changes the shared target.

Neo automatically dereferences managed references in value-access contexts, including
field access, arithmetic and writes through a reference. When a context expects `T&`,
the compiler forwards the reference value. When it expects T, it loads the referenced
T according to the ordinary copy rules. An explicit address expression forms or
selects a reference; it does not require users to manually dereference it afterward.
See [Neo managed access](neo.md) for the supported assignment/rebinding syntax.

Thus “reference parameter” is shorthand for a parameter whose value is a managed
reference. It does not mean that the caller's variable binding is itself passed by
reference. That would require a separate reference-slot contract. Likewise, an
instance byref receiver receives a reference value as `this`.

## Choosing a signature

| Shape | Meaning | Use when |
| --- | --- | --- |
| `T` parameter | Pass a value using T's ordinary copy rules | The operation consumes an input value, stores a copy, or computes a result independently of the caller's slot |
| `T&` parameter | Pass a reference value addressing initialized storage | The operation must mutate that storage, preserve its identity, or explicitly operate through a reference |
| Value receiver | `this` is a value copy | Snapshot-style behavior is intentional and copying is acceptable |
| `.method instance byref` | `this` is a managed reference to the original receiver | Mutation or receiver identity matters; callers supply an address or an existing reference |
| `T` return | Return a value | Produce a result or an independent value using ordinary copy rules |
| `T&` return | Return a reference value addressing existing storage | Expose a stable location with a lifetime that permits the caller to use it |
| Output reference | Assignment obligation on a referenced destination | The operation initializes caller storage; use existing output metadata rather than inventing a pointer convention |
| `T*` parameter/return | Native address and explicit interop obligations | Native APIs or explicit native buffers require a pointer |

Passing T by value does not promise a deep copy of everything reachable from T.
Managed-reference fields retain their targets; pointer fields retain their addresses.
Document such sharing, especially in resource-owning descriptors. Ordinary copying
never implicitly calls `Clonable<T>.Clone`.

Choose reference contracts for semantics first. A large value may justify an explicit
reference input to avoid copying, but today's writable `T&` is not a readonly borrow.
Do not claim readonly or exclusive access that the runtime does not enforce. Small
numeric inputs and immutable text operations need not become reference parameters
merely because references now exist.

For every public method, record:

- What is copied, mutated, retained, or returned by reference.
- Whether stored inputs are independent values or aliases, and how long they remain reachable.
- Whether a returned interior address remains meaningful after mutation or resizing.
- Expected failure outcomes, terminal faults, and initialization obligations.
- Resource ownership and explicit completion/disposal obligations, if any.

## Lifetime and allocation

A reference argument does not transfer ownership or authorize an invalid escape.
The runtime validates frame provenance when references are returned or stored.
A method may return an address into caller-owned storage that outlives its frame,
or a managed heap reference. It must fault when a return exposes storage owned by
its own ending frame. Taking an address does not silently promote a local to the heap.

A reachable heap reference, including a supported interior reference or interface
view, keeps its managed owner reachable. Heap reclamation follows GC reachability;
it is not deterministic disposal. Frame-owned values follow their frame lifetime.
`Disposable` and `Closable<E>` express explicit resource operations, not heap freeing.
No library contract should require manually invalidating a managed reference.

Returning `T&` must identify the owner and invalidation behavior. A future resizable
managed collection should not promise that an element reference tracks the logical
index after replacing its backing array. Keeping an old array alive can preserve
memory safety while the reference no longer addresses the collection's current item.
Settle that semantic choice before adding an element-reference API.

Pointers do not root GC objects. Native exports of managed storage require an explicit
pin/root lifetime contract before they are supported; the current native collection
buffers are not evidence that arbitrary managed objects can be exported as pointers.
See [managed heap](managed-heap-strategy.md) and [native interop](native-interop.md).

## Collections: first migration

`System.Collections.List<T>` and every instance method of `ArrayList<T>` now use
managed-reference receivers. This includes Count, Capacity, Item get/set, Add, the
private bounds-check helper. Interface dispatch receives the original concrete
slot; it does not copy the descriptor to supply `this`.

```text
.interface System.Collections.List<T>
    .method instance byref Add(T value) -> Void
    .end
    .method instance byref get_Item(Int32 index) -> T
    .end
.end
```

`Add(T)` and `set_Item(Int32,T)` still store value copies. `get_Item(Int32)` still
returns a value copy. A mutating receiver does not imply a reference element argument.
When T is itself a managed-reference type, copying T copies the reference, as
described below.

This is a breaking receiver-contract change: direct IL callers must load a managed
address (`ldloca`, `ldarga`, or an existing managed reference) before invoking these
methods. Rebuild artifacts against the matching System library. Interface
implementations must declare the matching byref receiver. Native `InterfaceRef<I>`
views cannot call this managed-receiver contract.

ArrayList holds a reference to shared managed state containing Data: T[]& and Count.
Ordinary copies share the complete collection across growth. Copy() explicitly creates
an independent sequence with shallow element copies. See [the collection contract](array-list.md)
for storage costs, readonly access, checked capacity, GC and migration from Preview 3.

## Generic reference elements

The generic argument determines the stored value and the substituted API signature:

| Constructed collection | Add signature | Item result | Copy behavior |
| --- | --- | --- | --- |
| `ArrayList<Foo>` | `Add(Foo)` | `Foo` | Copy the Foo value according to its field copy rules |
| `ArrayList<Foo&>` | `Add(Foo&)` | `Foo&` | Copy the managed-reference value; preserve the referenced Foo's identity |

Both rows are implemented using managed array storage. No raw-pointer conversion
is involved. Newly allocated spare capacity is uninitialized and contributes no GC
roots. Copies of descriptors may retain shared initialized array slots beyond their
own Count, as described in the collection contract.

There is no need to change generic `Add(T)` to `Add(T&)` to support a list of
references: T is already `Foo&`. Likewise `get_Item` returns T, so it returns a
reference value for this instantiation. Reading a member or writing through the
returned reference in Neo follows it automatically. Users do not write a dereference
operator to access the Foo. Copying/rebinding that returned reference value does not
replace the collection's stored reference; an indexed set replaces the stored value.

Mutation of the referenced Foo is visible through every reference to it. Removing
or replacing an element removes that particular stored reference; it does not
explicitly destroy the Foo while other roots keep it reachable. A managed backing
store must trace reference elements and enforce provenance on writes, including
rejecting references to shorter-lived frame storage when stored in heap storage.
The same rules apply whether Foo was originally a frame-owned value or a managed
heap allocation; the actual owner lifetime determines which stores are valid.

Tests cover reference copying, shared mutation, replacement versus referent
mutation, GC retention/reclamation, invalid frame escapes and automatic Neo access.

## Current library inventory

| API family | Current contract and decision |
| --- | --- |
| `Collections.List<T>` / `ArrayList<T>` | Managed-reference receivers and managed T[]& backing storage; value/reference elements follow T; no Free |
| Native-buffer `Array<T>` | Explicit pointer-containing descriptor with value receivers and caller-managed Free; review together with native-buffer naming and ownership, separately from managed arrays |
| `Disposable`, `Closable<E>`, `Clonable<T>` | Already use byref receivers; retain these contracts. Clone explicitly returns T; Close returns `Result<Void,E>` |
| `EquatableTo<T>` | Readonly managed receiver and T input, aligned with ComparableTo. Preserve value equality, not reference identity; see [migration](equality.md). A separate reference-input comparison strategy remains future work |
| `Option<T>` / `Result<T,E>` and case accessors | Constructors and extraction use values; keep independent extraction semantics. Review predicate receiver copying separately. Existing TryGet output contracts remain authoritative |
| Numeric operations, Math, parsing | Scalar/value inputs and typed value results remain appropriate; parsing failure is an ordinary Result |
| String, Console, File text APIs | Current text/value inputs and results remain appropriate; host primitives and their wrappers must be changed together if reference inputs are later justified |
| Type / RuntimeTypeHandle | Descriptors and handles describe metadata identity without a boxing requirement. Use typeof(T) for declared types; TypeOf<T>.Of was removed in development on 2026-09-19. |
| `System.Value` and internal runtime helpers | Bootstrap representation boundaries, not the model for new user-facing generic APIs; preserve explicit packing/unpacking and typed binding validation |

No receiver migration should silently change equality into address comparison,
make every type derive from Object, or add an erased-value allocation to satisfy an
interface parameter. The [object hierarchy direction](object-hierarchy.md) remains
separate from addressing mode.

## Neo projection

Neo reads library receiver metadata for instance calls and property getters.
An existing managed-reference expression supplies the receiver directly. An
addressable mutable value can supply its address automatically, as with source
record methods. Immutable values and non-addressable temporary values cannot be
used as writable reference receivers in this subset. Readonly receiver support is
a future language/runtime contract, not an implicit exception for getters.

For example, this Neo function uses the bundled library contract:

```swift
func Append(values: System.Collections.ArrayList<int>&) -> int {
    values.Add(42)
    return values.Count
}
```

The same form works with `System.Collections.List<int>&`; methods and properties
use virtual dispatch through that interface. Neo does not require `*values`.
Generic static factory calls are available in Neo. The complete
[collection example](../examples/source/collections.neo) creates its own owner and
passes references through the bundled List interface.

For a unique library instance method, substituted parameter types supply context,
including reference arguments. Overloaded library methods still select exact signatures;
explicit address expressions select reference inputs, while ordinary value contexts
read through references. Broader contextual overload selection remains follow-up work. Implicit conversions
from references to bundled types into their declared implemented interfaces are
supported alongside source-declared interface conversions. Do not present these
compiler limitations as runtime lifetime or type-system restrictions.

## Upcoming decisions

[Reflection introspection](reflection.md) now applies these rules: Type queries return
owned descriptor values and arrays, preserve T& signatures and reference receiver mode,
and do not retain guest object references. The following items remain queued.

1. Extend the implemented [Neo output contracts](neo-outputs.md) with stronger
   static callee/alias analysis and a future conditional declaration syntax. Ordinary
   and output references already preserve one destination and its provenance; an
   output is not a way to replace a caller's reference binding.
2. Specify readonly observation contracts before broadly converting predicates,
   equality, and large immutable receiver APIs to references.
3. Build on the managed growable collection with explicit Clone and any future
   element-slot views; preserve the documented copy and lifetime contracts.
4. Extend Neo library argument binding as reference-taking APIs are introduced,
   with tests for evaluation order, value/reference overload selection, and no
   implicit addressing of arbitrary value arguments.
5. Design native pinning and exported-handle ownership alongside an actual interop
   scenario; keep raw pointers separate from managed references.

A migration is complete when System metadata and IL, its callers, the verifier/runtime
checks, Neo lowering where supported, examples, and documentation agree. Use executable
examples that exercise both frame and managed heap owners, and verify that value-return
APIs still copy while reference operations affect the intended original storage.

The planned [mutability contracts](mutability.md) place readonly references and
receiver permissions in the runtime, with Neo diagnostics above them. Use these
contracts when implemented; current writable T& APIs do not already promise readonly
access. Immutable bindings and deep object immutability are separate concerns.

The [readonly input-parameter slice](readonly-parameters.md) now enforces restricted
managed access at runtime, with Neo declarations and ParameterInfo.IsReadOnly.
Readonly instance receivers now use the same enforcement and expose MethodInfo.IsReadOnly.
ArrayList Count/Capacity/Item getters and List Count/Item contracts support readonly
observation. [Readonly storage and return signatures](readonly-storage.md) are implemented;
full alias/lifetime analysis remains outside the verifier.

Characteristic placement should follow its meaning: nominal definitions describe
members and relationships; use-site type signatures carry reference access and the
agreed future nullability characteristic; source compilers control binding reassignment.
Slots hold runtime state validated against their signatures. Null will be a special
state for explicitly nullable values or references, not zero/default payloads or
uninitialized storage. Prefer Option for domain optionality. See [nullability](nullability.md).

## Common ordering and iteration

See [common interfaces](common-interfaces.md) for ComparableTo<T>, Iterable<T>,
Iterator<T> and the List<T> inheritance change. CompareTo and EquatableTo.Equals take T by value and use readonly managed receivers;
see the [equality migration](equality.md). GetIterator and Current use readonly receivers; MoveNext and
Dispose require writable managed receivers. Current returns T without erasing an
explicit T&. ArrayList iterators retain their initial backing buffer and extent,
with documented mutation visibility rather than .NET List version checking.

The historical [ArrayList predicate searches](predicate-search.md) used Option<T>
for Find, an index/-1 for FindIndex, and Boolean for Exists. The Raven profile now
uses [direct filtering](arraylist-filtering.md), including Option<Int32> index
results and FindLast/FindLastIndex/FindAll/TrueForAll. [LINQ](raven-query-api.md)
is a separate library surface. These predicates do not require default comparers.


## Preview alignment follow-up — 2026-09-19

After removing TypeOf<T>.Of in favor of typeof(T), the author proposed evaluating
Object.GetType as part of the smallest preview API. This is a candidate for the
next alignment stage, not an implemented member or a finalized requirement. Compare
.NET Object.GetType's runtime-object identity with typeof(T)'s declared identity,
and reuse the existing [type inspection and reference discovery research](type-inspection.md)
when choosing the minimum implementation. Validation must distinguish a base view
of a derived object from its declared variable type and cover unavailable/null
references. Adding an Object member would require an explicit runtime-reference
contract and consumer metadata change; it is outside this API-preserving port.


### Development compatibility principle

The author's 2026-09-19 clarification sets .NET as the ergonomic comparison baseline:
retain the minimum familiar API and developer experience where it serves neoCLR.
Compatibility may break during development, including names, signatures and behavior,
when that supports the smallest coherent preview surface. Record the specific .NET
comparison, benefit, costs and caller migration for each intentional divergence;
existing compatibility alone does not settle the design. This refines the earlier
API-preserving migration scope for the next alignment phase and authorizes the
explicit TypeOf<T>.Of removal above. It does not imply a commitment to the full CLR
object model or to implementing all .NET members.
