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

Static calls work through generic bounds. Generic instance calls use an explicit
borrowed receiver: `callself borrow T = instance Clonable::Clone()`. The stack
supplies a managed `T&` slot. For values, the runtime forwards the original slot
to a byref implementation; for classes, it reads the object reference from the
slot. It checks null, lifetime, assignment, readonly compatibility and conformance.
By-value value implementations are rejected for this mode instead of silently
copying the receiver. Open generic instance calls without `borrow` remain invalid.
The serialized `borrowed` flag defaults to false for older callself metadata. Ordinary class methods may use Raven `Self` as their
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
There is no performance claim. Borrowed Self dispatch has no boxing fallback.

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


## Generic cloning follow-up (2026-09-30)

The initial [Raven cloning probe](experiments/native-self/README.md) used a separate
nongeneric application interface with `Clone() -> Self`. Its generic Copy function
returns T for both a struct and a class. At that stage System.Clonable<T> was unchanged; the subsequent migration below replaces it.
The bounded importer specializes closed static helpers with one exact application
cloning bound, preserving native borrowed dispatch. It requires a direct concrete
implementation and a single public abstract nongeneric Clone method; arbitrary
instance Self contracts and inherited application conformances remain outside it.
No covariant subclass interpretation of Self is introduced.

.NET's shipped [constrained prefix](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.constrained?view=net-10.0)
also takes a managed receiver pointer and chooses value/reference treatment.
neoCLR adopts that useful receiver adaptation, with native Self signature
substitution and no boxing fallback. The benefit is shared generic code without
copying a mutable value receiver; the cost is an explicit borrowed operand and
rejection of implementations that cannot honor it. Source reviewed 2026-09-30.

The follow-up passed 12 Self tests and 31 generic-bound/interface regressions,
including original-slot mutation, virtual reference dispatch/reachability, two-object class clone allocation, null,
missing bounds, erased receivers and invalid receiver modes. The Raven compiler's
three focused Self tests include generic instance binding/emission; the imported
class/struct consumer passed execution and three compiler rejection cases.


## System.Clonable migration (2026-09-30)

The author selected Clonable as the next actual library migration. The Raven
library now declares nongeneric `System.Clonable` with `func Clone() -> Self`.
The reference marker, importer, API reference and generated native contract match.
The existing struct/class consumer imports System.Clonable instead of declaring
its own interface. Change Clonable<T> implementations and constraints to Clonable;
Clone must return the implementing type. Rebuild matching artifacts. The archived
Neo bootstrap's legacy generic contract and published artifacts are unchanged.

.NET's shipped [ICloneable.Clone](https://learn.microsoft.com/dotnet/api/system.icloneable.clone)
returns Object and permits either deep or shallow copying (reviewed 2026-09-30).
Native Self removes result casts and redundant implementing-type arguments, but
does not resolve that copying-policy ambiguity. Each implementation must document
sharing and ownership. The cost of this narrower contract is losing Clone-to-another-
type relationships; conversion/factory APIs should express those separately.
Inheritance semantics remain a separate decision; the importer admits direct
implementations. See the [API guide](../api-docs/cloning.md).

The obsolete `Clonable<T>` bound currently reaches the checked importer and is
rejected there. Earlier Raven generic-arity diagnostics are a deferred general
compiler candidate, to validate independently before any integration outside the
neoCLR branch. Missing bounds, wrong Self results and erased calls remain compiler
rejections.

Validation: all 13 native Self tests pass, including the generated System.Clonable
contract with value/reference clones. The matching Raven consumer passes native
verification/execution and all four rejection checks; [evidence](experiments/native-self/validation.json)
records artifact hashes. The signature probe, regenerated library snapshot and
refreshed API snapshot checks pass. No website build or fresh editor probe was run.

## Class inheritance and Self (2026-09-30)

Self is anchored to the class declaring the interface conformance. If Base declares
Clonable, Derived inherits the ordinary Base-returning Clone member, but it does
not thereby satisfy `T: Clonable` with T = Derived. `Copy<Base>(derived)` retains
Base as its result type. Virtual overrides preserve the exact Base signature;
returning a derived instance does not change that static promise.

Derived can redeclare Clonable and supply a matching Derived-returning implementation.
Native metadata supports an explicit interface mapping to a method returning Derived
for this purpose. This leaves the inherited public Clone and its
virtual slot intact. Ordinary method hiding and covariant virtual return contracts
remain outside this slice. Interface inheritance can carry the conformance; the
implementing class must still declare it at its own level to promise its own Self.
The runtime enforces the distinction for concrete generic bounds and direct
callself operations. The experimental Raven compiler/importer supports this bounded
contract, including explicit `func Clonable.Clone() -> Self` implementations.
Broader neoCLR target integration awaits Raven’s multi-target refactor.

Comparison: [C# interface mapping and reimplementation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces#1967-interface-re-implementation)
(reviewed 2026-09-30) preserve inherited mappings until reimplementation; virtual
methods can change execution without changing the interface's declared signature.
The .NET encoding `IClonable<Base>` likewise does not imply `IClonable<Derived>`.
neoCLR retains that distinction without requiring a source type parameter. A
late-bound Self that automatically changes on every subclass would require stronger
override/result guarantees; accepting an inherited Base clone as Derived would be
unsound. Requiring all cloneable classes to be sealed would be simpler but would
exclude ordinary inheritance. Anchoring conformance costs an explicit redeclaration
and implementation where a derived-result promise is wanted. No speed improvement
is claimed, and ordinary CLR execution of native Self remains unsupported.

Inheritance validation: 16 native Self tests and 48 related native tests pass
(class/interface inheritance, explicit mappings, arrays and generic constraints).
Raven passes 12 focused Self tests and 17 nearby declaration/constraint regressions.
The [actual Clonable consumer](experiments/native-self/README.md) passes native
verification/execution and six rejection checks; its artifact hashes are recorded.
The bridge signature probe and API/runtime snapshot checks pass. No website build
or new LSP validation was required for this slice.

The artifact/reachability follow-up passes all 17 Self tests. Reloaded modules
retain Base-returning virtual clone targets; explicit derived mappings retain
the Derived result and do not replace base-view cloning. A modified serialized
call that supplies Derived to an inherited-only Self bound is rejected by the
load/execution validation path without invoking the compiler or explicitly calling
`verify()`. This strengthens regression evidence; no additional runtime behavior
or Raven compiler integration was needed.
