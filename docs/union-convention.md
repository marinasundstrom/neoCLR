# Preview 1 ordinary union member convention

This is the selected member contract for Preview 1 library carriers and a future
compiler. System.Option and System.Result now implement it in platform IL. The
runtime executes ordinary types and methods; no convention recognizer or high-level
compiler is implemented yet. The marker does not certify an implementation's behavior.

## Recognizing the contract

A carrier has the ordinary System.Runtime.CompilerServices.UnionAttribute marker.
Tools resolve its constructor reference to the System definition; a similarly named
attribute in some other module is not the marker. No new union type kind is introduced.

For this first convention:

1. The carrier declares one or more public instance `.ctor(Variant) -> Void` overloads.
   Every public constructor has exactly one parameter. Each parameter is an ordinary
   record wrapper type and identifies a permitted variant. Private/internal helpers
   and static factories do not add alternatives to the recognized set.
2. Variant types must be distinct after closing the carrier's type arguments. Their
   unqualified definition names are unique within the carrier. This final name component
   supplies the member suffix: System.Some<T> uses `Some`, independent of T.
3. For each suffix X, the carrier declares a public instance `get_IsX() -> Boolean`
   and an instance read-only `IsX` property associated with it, plus a public instance
   `GetX() -> Variant` method. Both methods have zero declared parameters. Return
   signatures match the exact closed wrapper type, not its payload or a base type.
4. Payload wrappers expose their data through ordinary members. The supplied Some<T>,
   Ok<T> and Err<T> have public `.ctor(T) -> Void` and read-only `Value: T` properties.
   None has zero fields and a public `.ctor() -> Void`; it needs no payload accessor.

Member IDs and signatures resolve calls; source identifier spelling alone is not
identity. The suffix rule is a narrow authoring convention, not VM dispatch. Variants
with colliding short names need distinct names for Preview 1; a future richer metadata
association can relax that restriction. General generic methods, out parameters,
inheritance and reflective member lookup are not prerequisites.

## Behavioral obligations

The constructor installs exactly its supplied variant value. On a constructed carrier,
exactly one IsX query is true; none of the queries changes the selected variant. GetX
returns a value copy when X is active. Calling another accessor violates its precondition
and raises a terminal Fault. An unsuccessful predicate returns only false and never
exposes an uninitialized, default or null payload.

Construct once, keep the same value for the predicate and accessor, and branch before
extracting. This permits a match without a recursive dependency on Option-returning
queries or out-parameter initialization. A language compiler can verify exhaustiveness
over the recognized constructor set; arbitrary IL may omit branches and still be
well-formed IL. Runtime structural validation and typed verification do not prove the
behavioral obligations of every user-authored marked type.

The IL extraction pattern mirrors modern .NET usage: query the discriminator, branch
on it, then call the typed accessor.

```text
dup
call instance System.Result<Int32,Error>::get_IsOk()
brfalse WrongCase
call instance System.Result<Int32,Error>::GetOkCase()
call instance System.Result.Ok<Int32>::get_Value()
```

The `dup` preserves the carrier for the accessor. Calling an accessor for an inactive
case raises a terminal Fault.

Representation fields are private and public members are read-only except for
construction. Ordinary value-copy semantics apply. Pointer payloads preserve aliasing
and do not acquire ownership. Trusted host record construction and unsafe access remain
outside constructor provenance guarantees; visibility is not a memory-safety sandbox.

## Implemented System surface

Nested generic case types are a requested next capability. The current wrappers below
are top-level types. The selected [companion-type direction](nested-types.md) uses a
non-generic Result owner for generic cases, separate from Result<T,E>. Name/arity
identity and ordinary nesting must be implemented before that library migration.

| Carrier | Constructor parameter types | Predicates | Checked wrapper access |
| --- | --- | --- | --- |
| System.Option<T> | System.None, System.Some<T> | IsNone, IsSome | GetNone(), GetSome() |
| System.Result<T,E> | System.Ok<T>, System.Err<E> | IsOk, IsErr | GetOk(), GetErr() |

Some<Void> contains a real Void payload and differs from None. Ok<Void> is valid.
Ok<T> and Err<T> remain distinct for Result<T,T>. Error parameters are unconstrained;
they need not derive from System.Error. Nested carriers retain their closed type and
copy semantics. Each carrier stores one private System.Value and composes ordinary
constructors, fields and [explicit type-erasure operations](value-storage.md). It has
no tag table, union-specific opcodes or runtime service binding by carrier name.

Example lowering for the success arm of a Raven-like result match:

```text
ldloc result
call instance System.Result<Int32,Int32>::get_IsOk()
brfalse Failed
ldloc result
call instance System.Result<Int32,Int32>::GetOk()
call instance System.Ok<Int32>::get_Value()
; The stack now contains the Int32 payload. Consume it and branch to the join.
```

Property source syntax becomes ordinary getter calls. Wrapper construction likewise
becomes two ordinary operations, such as `newobj instance System.Ok<Int32>::.ctor(Int32)`
followed by `newobj instance System.Result<Int32,Int32>::.ctor(System.Ok<Int32>)`.
The [Raven-like program contracts](preview-1-programs.md) explain the source-level intent.

## Migration boundary

Use **fully qualified** System.Option<T> and System.Result<T,E> for the new library
types. At this stage, unqualified Option<T>/Result<T,E> still encode the old bootstrap
VM categories. They are not aliases and cannot be passed interchangeably. This is
temporary migration scaffolding, not the final language naming policy.

Int32.Parse now returns an ordinary carrier through a [migrated native boundary](int32-parse.md).
Console, arithmetic, slicing and file helpers still return bootstrap carriers.
[Host invocation now imports](erased-inputs.md) records containing
System.Value with bounded shape checks; this does not recognize or enforce the convention.
No automatic conversion or extra host intrinsic bridges the two representations.
Those boundaries must be migrated, then the bootstrap instructions and type categories
removed with an explicit serialized-format break before Preview 1 is complete.

```sh
cargo run --locked -- run examples/ordinary_unions.neoil
```

Expected lines: `success`, `7`, `failure`, `7`, `Some<Void> is present`, `=> Void`.

## Later composition APIs

The intended consumer model is Rust-like Option/Result composition expressed through
Raven-like source syntax. Map transforms a success payload, MapError transforms an
error payload, and AndThen chains an operation returning another Result. On Err,
Map/AndThen preserve the error without invoking the success callback. Option should
offer corresponding Map/AndThen behavior that propagates None. These remain ordinary
library methods; they require generic methods and callable values, not union opcodes.
Broad inheritance or virtual dispatch is not inherently required, but these APIs are
deferred until their platform prerequisites exist. The current .NET-style access model
remains sufficient; a redesigned access model is separate future work.
