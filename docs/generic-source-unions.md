# Generic source union carriers

Neo can compose independent generic records into a generic union:

```swift
record Success<T>(Value: T)
record Failure<E>(Error: E)
union Outcome<T,E>(Success<T> | Failure<E>)

func Accept<T>(value: T) -> Outcome<T,string> {
    return Success<T>(value)
}

let result: Outcome<int,string> = Success<int>(42)
let explicit = Outcome<int,string>(Success<int>(42))
let Success(accepted) = result else { return -1 }
```

Success<T> is independent of E, and Failure<E> is independent of T. The expected
carrier supplies both arguments when converting an exact case. Explicit case and
carrier constructor calls require type arguments; no unconstrained carrier argument
is guessed from a case. Generic function inference remains available, so Accept(42)
returns Outcome<int,string>.

The carrier emits one ordinary generic type definition, one constructor per case,
and typed case tests/extractors. Substitution closes these signatures for conversion,
match, if let and let … else. Source patterns bind the whole case, including its
fields, as for nongeneric source unions. Exhaustiveness still requires all named cases
or a wildcard. Distinct constructions of the same case definition in one carrier are
rejected because their simple pattern names would be ambiguous.

Empty nongeneric cases and nested generic source carriers are supported. Each case
must be a declared concrete source record/class or another source union, with matching
generic arity. A carrier cannot directly contain itself. Primitive, interface,
reference and bare type-parameter alternatives remain outside this projection.
References can instead be fields or type arguments of a case record.

Value copying and reference storage follow the existing contracts. A Success<Cell&>
keeps its managed reference when copied into or out of a carrier; a readonly Cell&
stays readonly. Storing a frame reference in an aggregate does not extend its lifetime.
No null/default case is introduced: default(Outcome<int,string>) is rejected. The
existing System.Value carrier storage retains its erased-storage and copying costs;
this compiler change adds no runtime opcode or new representation.

Inline generic case declarations, source constructor inference, constraints on the
carrier declaration, and richer patterns remain subsequent slices. Independent case
records may use the existing generic constraints; runtime metadata validates their
constructed types. See [generic constraints](generic-constraints.md).

## CLR comparison

The [C# union reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/union)
(consulted 2026-09-09) includes a generic Option<T> composed from independently declared
None and Some<T> cases, and constructor-based case conversions. This is the evolving
.NET 11-era comparison, not a requirement of older CLR versions. Neo reuses ordinary
generic metadata and its existing carrier convention, with exact conversions and
whole-case source bindings. Its reference permissions and explicit-default policy
remain different; there is no claim of matching C# nullability or conversion ranking.

Composing existing generic records first makes each case's parameter dependencies
explicit. Giving every inline case all carrier parameters would unnecessarily couple
Success<T> to E. Inferring minimal parameters for generated inline cases is another
possible language projection, but needs naming, import and constraint rules. This
slice leaves that decision open. See [union declarations](neo-unions.md) for storage
alternatives and the costs of the current implementation.

## Run

```sh
cargo run --locked -- run examples/source/generic-unions.neo
cargo test --locked --test neo_generic_unions --test neo_unions --test neo_generic_records
```

The example prints and returns 42. Tests cover source-to-JSON loading, generic forwarding,
explicit and contextual construction, match/conditional bindings, nested carriers,
reference identity, readonly fields and rejected case/default/arity contracts.
