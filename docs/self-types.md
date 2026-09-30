# Native Self contracts (development)

`Self` names the implementing type in an interface contract. It is a native
signature form, distinct from both indexed generic parameters and ordinary
nominal types. Capitalization is intentional: Raven's `self` is the instance
value, and is not a type alias.

The development numeric contract is now:

```raven
public interface Number : ComparableTo<Self> {
    static val Zero: Self { get; }
    static val One: Self { get; }
    static func +(left: Self, right: Self) -> Self
    static func -(left: Self, right: Self) -> Self
    static func *(left: Self, right: Self) -> Self
    static func /(left: Self, right: Self) -> Self
}
```

A consumer writes `where T: Number`. The interface has zero generic parameters;
there is no hidden `TSelf` argument. This replaces the development `Number<T>`
contract. Rebuild reference, runtime-library and application artifacts together.
Published Preview 11 artifacts are unchanged.

## Runtime contract

The serialized signature is `SelfType`; neoIL spells it `Self`. It may occur in
bodyless interface parameter/result signatures, including arrays, references and
constructed types, and in inherited interface arguments such as
`ComparableTo<Self>`. The runtime substitutes the selected implementing type
recursively before requiring an exact implementation signature.

Typed dispatch supplies the implementing type separately from interface identity:

```text
callself Int32 = System.Number::op_Addition(Self,Self)
callself T = System.Number::get_Zero()
```

The second form requires the declaring generic function's `T` to have the Number
bound. Generic instantiation supplies its concrete type at execution. Both source
and serialized metadata are checked; compiler acceptance cannot bypass conformance,
argument types, access or generic-bound checks. Calls participate in metadata
reference binding, reachability and runtime-service discovery.

The first slice supports static calls through generic bounds and instance calls
with a concrete receiver type. Open generic instance calls need a receiver-mode
contract and are rejected. Ordinary class methods may use Raven `Self` as their
declaring constructed type; inherited methods do not acquire covariant signatures
for each subclass. Redeclaring a conformance must supply matching signatures.

Unresolved Self is not storage: fields, locals, free-function signatures,
allocation and standalone runtime type identity cannot use it. Interface defaults
with Self signatures are not included. Calls through erased interface values are
rejected when the member signature contains Self; the caller must retain the
implementing type. Ordinary interface members remain callable. No existential
packaging or cross-implementation operand conversion is implied.

## Raven and metadata transport

Raven enables this only with `RuntimeSelfTypeContract` (MSBuild properties
`RavenSelfAssemblyName` and `RavenSelfType`). The neoCLR profile selects
`NeoCLR.CoreProbe` and `System.Runtime.CompilerServices.Self`. That fieldless marker
is a transport TypeRef in the emitted assembly, not a managed runtime class or a
hidden generic parameter. The importer maps it to native Self signatures. Such
assemblies target neoCLR; native Self interface implementations are not claimed to
be executable CLR assemblies.

Semantic lookup projects Self to the constrained type for argument checking,
operators, properties and result typing. Emission retains the original interface
contract. The numeric bridge still specializes the existing bounded application
helpers over ten primitive number types, but emits native `callself` for Number
operations instead of treating an implementation choice as a compiler-only fact.

## Comparison and tradeoffs

.NET 7's shipped generic-math APIs and C# 11's static abstract interface members
express this relationship through explicit `TSelf` parameters. The CLI metadata
contains those ordinary generic arguments. See Microsoft's
[generic math overview](https://learn.microsoft.com/en-us/dotnet/standard/generics/math)
and [static interface tutorial](https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/interface-implementation/static-virtual-interface-members)
(retrieved 2026-09-30).

Rust traits have an implicit implementing `Self` and restrict uses through erased
trait objects; see the shipped [trait rules](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
(retrieved 2026-09-30). neoCLR follows the explicit native relationship and a
conservative erased-call restriction, without claiming Rust's complete trait,
associated-type, object-safety or ownership semantics.

The benefit is an enforceable implementing-type relationship without recursively
repeating a generic argument at every conformance. The cost is a new metadata
signature, substitution/dispatch rules, compiler projection and an incompatible
Number arity change. An explicit `TSelf` encoding would retain ordinary CLR
execution compatibility; it is deliberately not the selected architecture.
There is no performance claim and no new boxing policy.

## Evidence

- [Native contract tests](../tests/self_types.rs): Number, actual generated Int32
  bodies, generic constraints, clone results, nested signatures, inherited generic
  interfaces and invalid metadata/source contracts.
- [Numeric consumer](experiments/numeric-contracts/Main.rvn): the real Number API,
  its identities/operators, ordering and concrete primitive parsing.
- Raven feature tests and target integration details are recorded in the
  [integration log](experiments/raven-target/README.md).


Validation on 2026-09-30: 42 focused native tests passed (Self, generic bounds,
interfaces, interface helpers and static numeric conformance). Fourteen focused
Raven tests passed. The actual numeric consumer verified and executed arithmetic,
Zero/One, ordering and parsing across all ten primitive number types, with native
`callself` retained; Boolean arguments and extra constraints were rejected. See
[recorded numeric evidence](experiments/numeric-contracts/native-self-validation.json).
The existing generic-helper consumer also passed identity and rejection checks.
No website build or publication was performed.
