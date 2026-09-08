# Ordinary and output references in Neo

An ordinary `value: Foo&` parameter requires an initialized Foo. An
`out value: Foo&` parameter receives the same kind of managed reference but must
assign its destination during this invocation before returning normally. It may
receive uninitialized caller storage. The modifier is an initialization contract,
not another address layer or an ownership transfer.

```swift
func Initialize(out value: Counter&) -> () {
    value = Counter(40)
}

var counter: Counter
Initialize(out counter)
```

Neo supports uninitialized `var name: Type` declarations with an explicit type.
`let` still requires an initializer. The compiler uses the existing verifier to
reject local reads and ordinary reference calls before definite initialization.
An unconditional output call establishes initialization on its normal-return path.
Declarations in loops renew local storage through local.reset.

The `out` call-site marker is required and must match a declared output parameter.
It forms an address of writable storage, or forwards the target of an existing
managed reference. For example, `SetAge(out age)` with `age: int&` writes the integer
that age references. It does not retarget age or replace the caller's reference
binding. An uninitialized Foo& binding cannot be filled this way: that would require
an output reference to a reference slot, which the current nested-reference rules
exclude. Source and bundled interface calls retain these contracts.

An output may be forwarded with `Initialize(out destination)`. Interior fields must
belong to an initialized containing value. Assigning a field does not establish that
an entire uninitialized record has been assigned. Ordinary frame/heap provenance,
aggregate storage restrictions and current block-local address restrictions apply.
Outputs cannot hide a current-frame reference escape.

## Runtime enforcement

The runtime enforces output obligations even if the destination was initialized
before the call. Reads through an unassigned output reference fault; returning without
assigning an unconditional output also faults. Forwarded writes count for the original
destination. Unrelated field writes do not fulfill an obligation. Aliases remain
permitted under the existing managed-reference model.

Neo does not yet statically prove every callee output-write path or alias. Such
violations may compile and then fault. The caller's definite-initialization assumption
is valid because a violating callee cannot return normally. Faults remain terminal;
this is not rollback or exception-based recovery.

## Conditional library outputs

Existing library `out(true)` contracts are callable with the same `out` marker:

```swift
let element = typeof(Counter&).GetElementType()
var some: System.Option.Some<System.Type>
if element.TryGet(out some) {
    WriteLine(some.Value.Name)
}
```

The verifier establishes initialization on the direct successful Boolean branch.
Ignoring the result, or reading a previously uninitialized destination after merging
success/failure branches, does not establish initialization. A false result carries
no assignment guarantee. Arbitrary Boolean dataflow remains conservative.

Source declarations in this slice expose unconditional `out` only. Conditional
source declaration syntax remains deferred; library conditional contracts are preserved
from their metadata. This is distinct from making all Boolean output functions
conditional automatically.

Run `cargo run -- run examples/source/outputs.neo`. It prints 42 and Counter and
returns 42, demonstrating forwarding, writing through an existing field reference,
and conditional type metadata extraction. See [runtime slots](reference-slots.md),
[API contracts](api-design.md), and [tests](../tests/neo_outputs.rs).

## Reference passing example

Run `cargo run --locked -- run examples/source/reference-passing.neo`.
`Forward(alias)` passes an existing reference without an address operator and writes
through to the original Counter. `selected = &other` changes only the local selection.
`Initialize(out reference)` replaces the Counter at the original target; existing
aliases observe that replacement. It does not change which storage reference addresses.
See [the design guidance](type-design.md#decision-explicit-reference-creation-in-neo)
for the explicit-reference-creation decision and assignment differences.
