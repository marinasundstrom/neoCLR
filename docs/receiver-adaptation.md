# Generic receiver adaptation

A constrained Neo method can now use the same body for a value or an existing
managed reference, without boxing:

```swift
func Adjust<T>(value: T) -> int where T: Counter {
    var local = value
    local.Add(1)
    return local.Read()
}
```

For `T = Cell`, the argument/local copies follow Cell's field contracts and the call
borrows the local value. For `T = Cell&`, the local contains the same reference and
the call changes its existing target. A readonly reference stays readonly. Neither
case introduces a box, promotion, nested managed reference or extra receiver copy.
This is a platform capability; it is not by itself a usability or performance result.

## Runtime instruction

`ldreceiver arg|local index-or-name [readonly]` pushes one managed receiver:

| Declared slot type after substitution | Result |
| --- | --- |
| `Foo` (or another value) | A reference to that slot; `readonly` narrows this borrow |
| `Foo&` | The stored reference, retaining its existing permissions and identity |
| `readonly Foo&` | The stored readonly reference, even without the instruction's `readonly` flag |

The flag concerns the **value borrow only**. It never grants write access and never
changes a stored reference's permissions. This lets a language distinguish an immutable
binding from the mutability of the object addressed by its reference value. It does
not make bindings runtime-immutable: raw IL can request a writable value borrow, just
as it can use ldloca/ldarga. Native pointers are not automatically dereferenced; a
pointer-valued slot remains a value slot, not a managed view of native memory.

The operation checks slot indices, initialization and managed-target validity. It
cannot adapt a constructor's receiver binding or read through unassigned output
storage. It is classified under SlotReferences. Existing castclass/interface.borrow,
call/callvirt, frame-escape, readonly and GC checks handle the resulting capability.
It does not select a method or change virtual/explicit/default implementation lookup.

Serialized format 5 gains an additive instruction variant:

```json
{"ldreceiver":{"argument":true,"index":0,"readonly_value":true}}
```

Older readers reject the unknown opcode. Existing artifacts and ldarga/ldloca
semantics are unchanged; they still cannot expose a reference to a managed-reference
slot. Rebuild tools before consuming artifacts using ldreceiver.

## Neo projection and current limits

Neo uses ldreceiver for constrained calls on a bare T parameter or direct method-level
local when its addressing mode is open. An immutable binding emits `readonly`;
a mutable `var` local emits the writable value-borrow form. Existing T& calls retain
their previous lowering. A `notreference` proof still allows ordinary slot-address
lowering, including addressable fields and array elements.

For example, `func Add<T>(value: T) where T: Counter` can mutate a writable reference
target. With a value argument, its immutable parameter produces a readonly receiver
and a writable method call faults. Use an explicit mutable local, as in Adjust, when
mutation should work with either mode. Readonly methods work with both. A language
could diagnose more concrete invalid instantiations ahead of time; Neo currently
relies on runtime capability checks for this open case. It does not silently copy an
immutable receiver to make a mutating call succeed.

The verifier checks initialization and uses declared bounds to prove nominal views.
For an open parameter it models an adapted receiver abstractly; its final addressing
mode and access permissions remain subject to concrete runtime checks. Verification
of an open generic body is not a guarantee that every substitution will execute
without a capability fault. Known closed readonly receivers remain checked statically.

Adaptive receiver expressions currently require direct parameters/locals. Captured
bindings, block-local bindings, field/array receivers with open addressing mode and
value temporaries remain separate work. Bind an expression to an eligible local when
its copy/reference semantics are intended, or retain a notreference/T& API. This does
not add implicit conversions from bare T to base/interface reference parameters or
support constrained properties/fields/method groups.

## CLR comparison and decision

Microsoft's [OpCodes.Constrained contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.constrained?view=net-10.0)
(consulted 2026-09-09) describes a pointer to the generic receiver: the call either
loads a stored reference or uses the value address, with a boxing fallback in certain
Object/ValueType/Enum cases. This is the familiar receiver-selection requirement.
NeoCLR already supports checked managed views of values, so it needs no such boxing
fallback for the implemented base/interface calls.

Adopting the CLR prefix unchanged would require representing a pointer to a reference
slot. Introducing general nested managed references would also affect storage, output
parameters and lifetimes. Specializing every generic body in Neo would leave other
frontends without this platform facility. The provisional choice is instead one
receiver-load instruction with an explicit slot operand. Ordinary dispatch remains
separate, and the nested-reference restriction remains intact.

The benefit is a common runtime operation preserving value/reference and access
contracts. The costs are a new preview opcode, a runtime mode selection, partial
open-generic verification and a deliberately bounded source projection. Eventual JIT
or AOT compilation may specialize the operation when T is known; no speed or allocation
benchmark is claimed. A more general storage operand or CLR-like prefix can be
reconsidered with field/array/capture scenarios. The instruction is not a runtime
policy for immutable bindings.

The [pinned .NET 10 probe](experiments/generic-bounds-dotnet/Program.cs) runs a generic
adjustment with both a struct and a class, confirming independent value mutation and
shared reference mutation. The comparison uses a mutable local in both languages;
it does not claim their parameter-binding rules are identical.

## Run and validate

```sh
cargo run --locked -- run examples/source/generic-receivers.neo
cargo test --locked --test generic_receivers --test generic_bounds --test neo_constrained_values --test readonly_references
cd docs/experiments/generic-bounds-dotnet
dotnet run
```

The Neo example prints and returns 42. Source/IL/artifact tests cover both receiver
modes, readonly capabilities, virtual and explicit/default interface dispatch,
initialization, indices, output/construction guards, caller/heap lifetime retention
and invalid current-frame escapes. Inferred readonly reference arguments retain their
closed type in emitted call signatures.
