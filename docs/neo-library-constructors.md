# Public library constructors in Neo

Generic case types can be constructed independently with explicit type arguments:

```swift
let ok = System.Result.Ok<int>(42)
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
unclosed generic owners, inaccessible constructors and multiple applicable candidates
are rejected. Constructors are not inherited and no default constructor is fabricated.

## Boundaries and comparison

Explicit construction remains available alongside [implicit case-to-carrier conversion](case-to-carrier-conversion.md):
`let result: Result<int,string> = ok` invokes the accepting constructor when the target
is a marked union. `System.Result.Ok(42)` and imported `Ok(42)` still need future
inference/import work. Source union case conversions retain their existing contract. Library heap-construction syntax is not added here. No arbitrary external
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
owner arguments, conservative ambiguity and compiler work per candidate. No performance
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
