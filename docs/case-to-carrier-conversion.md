# Case-to-carrier conversion

```swift
let ok = System.Result.Ok<int>(42)
let result: Result<int, string> = ok

func Success() -> Result<int, string> {
    return System.Result.Ok<int>(42)
}
```

The case is an independent value. An expected carrier type enables conversion through
its declared constructor, with no special treatment of Result or Option names.
Source unions already generate and use such constructors. This slice extends the
same exact-case rule to marked carriers in the bundled System metadata.

The destination must be a publicly constructible record marked with UnionAttribute.
Exactly one public instance constructor taking the actual case type by value must
match after substituting the carrier's type arguments. Output/readonly input contracts
are not value-case constructors. No match is a type error; multiple matches are
ambiguous. A one-argument constructor on an ordinary unmarked type does not opt in.
The compiler emits the existing constructor instruction after evaluating the case once.

Expected types from annotations, returns, parameters, fields, arrays and collection
setters can supply the carrier. Inferred `let ok = ...` remains the case type. Explicit
`Result<int,string>(ok)` construction remains valid. Conversions do not infer missing
carrier arguments, chain through intermediate case wrappers, select an arbitrary
carrier, widen generic case arguments, or allocate storage for a carrier reference.
For example, Ok<int> cannot convert to Result<Void,string>, Some<int>, or Result<int,string>&.

Payloads retain ordinary copy rules. A case containing Foo& copies that reference;
readonly access is preserved and frame-backed payloads cannot escape via wrapping.
A context expecting a value still applies Neo's existing automatic read through a
managed reference before value conversion. This does not add implicit address-taking,
heap promotion, a new union opcode, or a new cleanup policy.

## Raven implementation inspected

Inspected the clean Raven checkout at commit
`92352e228f1a026d85de08b385e910001bbe2ebc` on 2026-09-09. These findings are from
source/spec/test inspection; Raven's tests were not run as part of this comparison.

- The [union specification](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/docs/lang/spec/unions.md)
  separates case construction from contextual conversion. Public single-parameter
  constructors define the carrier's variant set; accessors inspect it.
- [Conversion classification](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/src/Raven.CodeAnalysis/Compilation.Conversions.cs)
  identifies a carrier constructor and records it in a union conversion. It also has
  case-family/generic compatibility rules and can consider ordinary implicit parameter
  conversions with user-defined conversions excluded. It is broader than exact matching.
- [Conversion lowering](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/src/Raven.CodeAnalysis/BoundTree/Lowering/Lowerer.Conversions.cs)
  creates the carrier with the case expression as its argument. PE union metadata
  [discovers member types from public constructors](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/src/Raven.CodeAnalysis/Symbols/PE/PEUnionSymbols.cs),
  with accessor fallback when no constructor types are found.
- Raven's [generic union metadata test](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/test/Raven.CodeAnalysis.Tests/CodeGen/Runtime/UnionGenericsTests.cs)
  checks separate Ok<T> and Error<E> case types under a nongeneric companion.
  Its [semantic tests](https://github.com/marinasundstrom/raven/blob/92352e228f1a026d85de08b385e910001bbe2ebc/test/Raven.CodeAnalysis.Tests/Semantics/Unions/UnionSemanticTests.cs)
  cover independently constructed cases, contextual conversion, and rejection when
  target typing conflicts with argument-inferred case types. Ok<int> must not become
  Ok<Unit> merely to satisfy a destination.

## .NET comparison and bounded choice

The [existing union research](neo-unions.md#comparison-and-validation) compares the
.NET/C# constructor-based union convention. Raven provides a concrete compiler
implementation of that direction on .NET. Neo adopts constructor-backed conversion
without adopting Raven's complete language, CLR object-storage representation, nullable
carrier behavior, leading-dot case syntax or generic inference in this slice.

Exact-case matching is deliberately narrower than Raven's conversion search. It
matches Neo's existing source-union contract and avoids implicit widening or extra
reference/value conversions inside case selection. Its cost is requiring a precisely
typed case; broader applicability would need overload ranking and cross-frontend
storage/access tests. Carrier constructors stay authoritative: no separate membership
table or case-family association is required by NeoCLR. The marker alone does not
prove arbitrary handwritten carrier IL obeys its semantic contract.

## Run and validate

```sh
cargo run --locked -- run examples/source/case-constructors.neo
cargo test --locked --test neo_case_conversion --test neo_library_constructors --test neo_generics --test neo_unions --test neo_conditional_patterns
```

The sample prints 42 twice and returns 42. Tests cover annotations, returns, parameters,
fields, arrays and collection setters; Option/Result and nested carriers; generic
reference payloads; once-only evaluation; unmarked/wrong-case/wrong-type rejection;
readonly payload retention under collection pressure, invariant generic case arguments,
and runtime frame escape rejection. [Generic constructor inference](neo-library-constructors.md#constructor-argument-inference)
is now available independently, together with [bundled case imports](neo-case-imports.md).
