# Designing types and APIs for NeoCLR and Neo

Treat a type as the sum of its parts and the guarantees of its operations. Its outer
value/reference mode is only one part of that contract. Examine the fields, their
copy and aliasing rules, and what a recipient may read, mutate, retain, replace or
dispose. Then choose storage and measure copying costs. These are working preview
guidelines informed by the [application experiment](experiments/reference-experience/README.md).

An API taking T can still change shared state through T's reference-valued fields.
An API taking T& receives access to the same instance, with implications for aliasing
and lifetime, but not automatic ownership transfer or permission to retain a local
borrow indefinitely. Passing a reference value does not by itself let a callee rebind
the caller's reference slot; output/rebinding contracts are separate. Readonly restricts
an access path, not the entire reachable graph. A good API specifies both its guarantees
and what callers must not assume.

## Platform capability and language usability

NeoCLR's managed references can provide a base or interface view of an existing object
in either frame or managed heap storage. The view preserves identity and dispatches
against the concrete object; this operation does not require CLR-style boxing. That
is a platform capability, supported by its reference, lifetime and GC contracts. See
[the constrained-view comparison](generic-constraints.md#constrained-reference-conversions)
for the implemented behavior and .NET probe.

This capability is useful independently of how a language spells it. It is not proof
that Neo is easier to use than C#, or that explicit references are always worth their
annotation cost. Syntax, inference, API defaults, diagnostics and tooling largely
shape that experience. Languages can project the same runtime guarantees differently.
Continue evaluating realistic programs and separating compiler/library friction from
runtime restrictions; the [order-workflow experiment](experiments/reference-experience/README.md)
is initial evidence, not a general usability verdict. Avoid inferring performance
advantages from the absence of boxing without measuring the relevant workload.

## Values for independent data

Small, stable data such as coordinates, amounts, settings snapshots and receipts are
good candidates for value use. Assignment and passing by value copy their fields;
changing one copy should not unexpectedly change another's logical data.

```swift
record Receipt(Product: string, Quantity: int, Total: int)
func ReadTotal(receipt: Receipt) -> int { return receipt.Total }
```

Immutability makes sharing less observable, but does not guarantee small size or cheap
copying. A large immutable structure may still benefit from reference-based access.
Do not introduce an arbitrary size threshold without measuring the target workload.
Copying a value is not necessarily recursive deep copying: reference-valued fields
still point to their existing targets. Review field contracts before calling a type
a snapshot. Current Neo records do not automatically make their fields immutable.

## References for identity and shared changes

Use T& when an operation should observe or update the same instance as its caller:

```swift
record Product(Stock: int)
func Restock(product: Product&, amount: int) -> () {
    product.Stock = product.Stock + amount
}
```

A caller can keep a short-lived product locally and pass `&product` down a call chain,
or create `new Product(5)` on the managed heap and pass that existing reference.
Neo accesses the target automatically; dereferencing is not part of ordinary managed
reference use. Requiring a pointer for this would be the wrong abstraction: pointers
belong to the separate native-interop story.

References are useful even for small objects when identity or shared mutation matters.
Conversely, heap allocation does not force every operation to share: a caller can
explicitly request a value copy. In Neo, class/record declaration syntax does not
establish a CLR-style nominal reference-type category.

Use readonly T& for borrowed observation when shared access is appropriate but mutation
through that reference is not. This is shallow: other aliases can mutate the target,
and reference-valued fields can retain their own access capabilities. Readonly does
not imply a frozen object graph, thread safety or exclusive access.

## Storage and lifetime are a separate choice

| Need | Starting choice | What to check |
| --- | --- | --- |
| Independent small snapshot | T | Are any fields shared references? |
| Update caller-owned state during a call | T& with a local borrow or existing reference | Does the callee retain it? |
| Observe an existing instance | readonly T& where useful | Is shallow readonly sufficient? |
| Store an instance in a collection or returned callback | Managed heap instance, referenced explicitly | Current aggregate/capture rules require heap-backed stored references. |
| Share a mutable collection | ArrayList<T> wrapper or its reference | Wrapper copies share complete list state. |
| Independent sequence | list.Copy() | Reference-valued elements still share their targets. |
| Represent one of several data alternatives | A union of case types | The carrier copies the chosen case according to its fields. |
| Own a native resource | A separately specified ownership/disposal API | GC alone does not define when a handle may be closed. |

A local value's scope can be shorter than its managed backing resources' lifetime.
A value facade can contain references to GC-managed state, as ArrayList now does.
Dropping one facade does not invalidate copies that still reference that state.
Under the intended future deterministic value-destructor model, its destructor would
run at scope exit; that is distinct from GC reclamation of shared backing memory.
Guest destructor support is not implemented by these slices. Purely managed storage
needs no manual Free, and GC does not automatically call Dispose/Close.

A resource-owning value with a native handle needs an explicit copying/ownership
policy before deterministic cleanup is safe. Do not close a shared handle merely
because one facade leaves scope. Also, shared managed structures can retain more
memory than intended even when GC is functioning correctly.

## Make API contracts communicate intent

Prefer a value parameter for independent input data when its complete field contract
actually makes it independent. Use a writable reference when mutation of the caller's
instance is part of the operation, and readonly access for borrowed observation where
its shallow guarantee is sufficient. Passing a value is not a promise of no side
effects, and passing a writable reference is not a promise that mutation will occur.
Do not add references everywhere solely to avoid hypothetical copies.

Document retained references: a callback or collection accepting T& currently requires
heap-backed storage if it captures/stores that reference. This requirement is not fully
expressed by T& alone. Compiler diagnostics or richer retention contracts remain a
separate investigation. Do not promise that any local borrow can be retained.

When a value wraps shared state, document the logical assignment contract and provide
an explicit independent-copy operation if useful. The [ArrayList contract](array-list.md)
and [order workflow](../examples/source/order-workflow.neo) are concrete examples.
Use union case constructors to describe accepted alternatives rather than inheritance
solely for tagging. [Neo unions](neo-unions.md) show separate variant types.

## Familiarity with .NET and remaining evidence

Microsoft's [C# value-type guide](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-types)
and [reference-type guide](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/reference-types)
(consulted 2026-09-08) distinguish field/value copies from shared class instances;
reference fields inside value copies continue sharing. NeoCLR retains that useful
field-copy distinction while selecting value versus managed-reference use separately
from nominal type identity. It adds storage/lifetime decisions that ordinary C# class
use often hides. That flexibility is a tradeoff, not a demonstrated productivity win.

For each API, try a realistic caller and ask whether it predicts sharing, whether a
routine refactor changes behavior, and whether an allocation choice leaks into the
caller unexpectedly. Benchmark allocation/copy cost before claiming an improvement.
A future high-level language may infer more access/storage choices; the runtime's
ability to express them does not require every language to expose identical syntax.

## Decision: explicit reference creation in Neo

Separate the stored value from the passing mode. A Foo& binding stores a managed
reference; passing its reference value shares the target without exposing the caller's
binding for replacement. Passing the reference-holding slot itself by reference is a
second level of indirection, currently unsupported. Today `out Foo&` initializes
the referenced Foo storage; it does not replace a Foo& binding. See
[output references](neo-outputs.md). An existing
Foo& argument already passes without another `&`. The alternative considered below concerns
obtaining a reference from a Foo value, not changing how reference values are passed.

Explicit managed-reference types remain a runtime contract. We considered removing some call-site
`&` expressions through contextual borrowing: an unambiguous
expected Foo& could borrow an addressable Foo automatically. Inferred `let copy = value`
would continue to copy; no implicit heap promotion or lifetime extension is proposed.
Explicit `&value` would remain available. Ordinary pointer operations are separate.

Compare C#'s [parameter modifiers](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/method-parameters)
(consulted 2026-09-08): ref/out normally expose the passing mode at the call site,
whereas in can omit its modifier. Neo's managed references also express object views,
so this is an analogy rather than a direct mapping. Contextual borrowing could reduce
noise, but hide mutation/sharing and make overload or API changes alter copying.
Before adopting it, test value/reference overload ambiguity, readonly targets,
fields/indexers versus temporaries, generic inference and retained references in the
order workflow. Runtime escape validation must remain unchanged. No syntax change
is implemented by recording this proposal.

For the current preview, reference creation stays explicit: `Use(&value)` obtains a
reference from a value, while `Use(reference)` forwards an existing Foo&. Implicit
argument borrowing is not adopted. This
keeps changes from value parameters to reference parameters visible at call sites.

Run `cargo run --locked -- run examples/source/reference-passing.neo` to see alias
passing, automatic member access, explicit local retargeting and output writes.
It prints 2, 102, 2, 11, 40 and returns 42. [Reference assignment](reference-assignment.md) now copies reference-valued right-hand
sides into mutable reference slots. Value RHS assignment still writes the target;
output-reference initialization remains a separate contract.
