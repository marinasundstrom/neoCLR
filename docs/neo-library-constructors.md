# Public library constructors in Neo

Generic case types can be constructed independently, with inferred or explicit type arguments:

```swift
let ok = System.Result.Ok(42) // System.Result.Ok<int>
let result = Result<int, string>(ok)
let Ok(value) = result else { return -1 }

let some = System.Option.Some<int>(value)
let option = Option<int>(some)
let none = System.Option.None()
let missing = Option<int>(none)
```

These are ordinary calls to declared public instance constructors on bundled library
types. There is no Result-specific lowering: the compiler selects from the exact
owner's constructor metadata, substitutes owner type arguments and emits an existing
`newobj instance Owner::.ctor(...)` instruction. It produces a value under NeoCLR's
existing constructor semantics, not an automatic managed heap allocation.

Constructor parameters provide argument context. `System.Result.Ok<Foo&>(reference)`
stores the reference; Foo-valued parameters request value copies. Readonly reference
arguments, Void and delegate payloads use their normal contracts, including method-group
conversion to an expected delegate. The carrier's overload taking the exact case type
is selected by ordinary constructor applicability. Wrong payloads, missing arguments,
owners with unresolved generic arguments, inaccessible constructors and multiple applicable candidates
are rejected. Constructors are not inherited and no default constructor is fabricated.

## Boundaries and comparison

Explicit construction remains available alongside [implicit case-to-carrier conversion](case-to-carrier-conversion.md):
`let result: Result<int,string> = ok` invokes the accepting constructor when the target
is a marked union. `System.Result.Ok(42)` infers its owner arguments. `import System.Result.*` also enables the short `Ok(42)` spelling. Source union case conversions retain their existing contract. Library heap-construction syntax is not added here. No arbitrary external
assembly loading is added: this frontend targets the bundled System metadata.

Microsoft's [C# constructor guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/constructors)
(consulted 2026-09-09) describes instance constructors and multiple signatures. Neo reuses
the familiar declared-constructor contract, while its existing value-default model
separates constructing a value from choosing managed heap storage. This slice changes
compiler access to existing metadata, not runtime initialization or allocation rules.

Trying applicable signatures preserves each candidate's parameter context. Only one
successful lowering is emitted, so runtime argument evaluation is once, left to right.
Unlike full C# overload resolution, this bounded frontend does not rank multiple
applicable conversions; ambiguity requires a more explicit argument. The benefit is
usable independent generic cases without type-name special cases. Costs are explicit
owner arguments when inference lacks evidence, conservative ambiguity and compiler work per candidate. No performance
improvement is claimed. Runtime lifetime checks still reject escaping frame references
inside case values, including when construction itself succeeds.

## Run and validate

```sh
cargo run --locked -- run examples/source/case-constructors.neo
cargo test --locked --test neo_library_constructors --test neo_result_factories --test neo_generics
```

The example prints 42 twice and returns 42. Tests cover standalone cases, overloaded
carrier construction, JSON artifact loading, reference identity, readonly/delegate/Void
payloads, argument evaluation count, invalid calls and frame escape rejection. Existing
static factories remain available and retain their behavior.

## Constructor argument inference

When a bundled generic type is called without `<...>`, Neo matches argument types
against public constructor parameter types. All owner parameters must be inferred.
For a bare parameter `T`, the argument's stored type supplies evidence, including
managed references and readonly references. Structural matching also handles arrays
and constructed types. No implicit reference is created from a value.

`System.Result.Ok(42)` therefore creates `Ok<int>` even without a carrier target.
`let result: Result<Void,string> = System.Result.Ok(42)` fails conversion; the target
does not rewrite the argument evidence. `Result(System.Result.Ok(42))` cannot infer
E and requires explicit carrier arguments. Method groups and lambdas still need a
known delegate target; use explicit owner arguments when they are the only evidence.
Distinct inferred owners are conservatively ambiguous. Applicable constructors for
the selected owner still undergo normal argument validation without overload ranking.
Nested call inference is cached; only the selected lowering emits argument evaluation.

This follows the case-first behavior inspected in Raven at commit
`92352e228f1a026d85de08b385e910001bbe2ebc`, particularly
[`TryInferConstructedTypeForConstructor`](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/src/Raven.CodeAnalysis/Binder/BlockBinder.MemberAccess.cs).
Raven also requires the constructor's own parameters to be resolved from argument
evidence. This is source inspection, not a claim that Raven's tests were run here.
Neo deliberately keeps a smaller exact structural inference algorithm.

Microsoft's [generic types and methods guide](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/generics)
(consulted 2026-09-09) illustrates argument inference for generic methods and explicit
type arguments for generic type construction. Neo extends argument inference to
constructor owners in the compiler. The benefit is less repeated type spelling for
independent cases; the cost is additional inference and ambiguity rules. The runtime
still receives closed constructor signatures, with no new opcode or metadata contract.
