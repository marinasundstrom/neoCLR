# Preliminary managed-reference identity

Neo provides `ReferenceEquals(left, right)` as a preliminary intrinsic. Its operands
must already be managed references, or explicit address expressions. It compares the
referenced location, without automatically reading values, boxing, or requiring Object.
The IL operation is `ref.eq`, with stack effect T&, U& → Boolean. The referent types
may differ. This intrinsic is not yet an ordinary reflected library method.

```swift
var a = Counter(42)
var b = a
let view: Counter& = &a
ReferenceEquals(&a, view) // true
ReferenceEquals(&a, &b)   // false
```

The identity is the owning storage cell plus its logical field/element path:

- Two aliases of the same frame or heap location compare equal.
- Concrete and interface views of the same location compare equal, regardless of
  interface view type. No virtual Equals method is invoked.
- Two separately stored equal values compare unequal.
- Repeated references to the same array element or field compare equal. Different
  indices/fields compare unequal, even within one allocation. The allocation root
  and its field are distinct locations, even if native offsets could coincide.
- Replacing a value in an existing location preserves that location's identity.
  Rebinding a reference changes what that binding addresses; other aliases keep
  addressing their original location. GC collection does not change live identities.

Both operands must designate initialized, live targets. Invalid/uninitialized targets
fault rather than compare false. There is no null managed-reference value in this
subset. Raw pointers are rejected; their existing explicit address comparisons remain
separate. Comparing identity neither extends frame lifetime nor registers persistent
host roots. It does not expose a native address, allocation number or identity hash.

Ordinary Neo `==` retains its existing value-access behavior. Equatable<T>.Equals and
value hashing contracts are unchanged. User-declared functions or records named
ReferenceEquals retain normal resolution; the intrinsic is a fallback when no such
source declaration exists. No new operator syntax is added.

Run `cargo run -- run examples/source/reference-identity.neo` for concrete/interface,
value-copy, heap and interior-field comparisons. The operation requires the
SlotReferences runtime service. See [tests](../tests/reference_identity.rs),
[managed references](managed-reference-semantics.md), and the future
[object/equality model](object-hierarchy.md). The name and library placement may evolve
before a stable platform API is selected.
