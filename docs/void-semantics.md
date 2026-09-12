# Void: a unit type with no-result calls

Decision recorded 2026-09-12. Void has one logical value and no payload. It is a
valid generic argument in neoCLR. Ordinary void-returning calls retain the CLR
calling convention: they leave no result on the evaluation stack. A targeting
compiler supplies or discards the unit value when an expression context requires
it. Users normally do not construct Void themselves.

Use `Option<T>` for a useful value or absence, and `Result<Void, E>` for completion
or error. `Option<Void>` remains technically valid, but it is not the recommended
API example: its useful distinction would be the case, not the payload. `None`
already expresses absence. The propagation protocol's Void residual for Option
is an internal representation of that absence, not a recommendation to use Void
as Option's public output type.

## CLR comparison and layer boundaries

[System.Void in .NET](https://learn.microsoft.com/en-us/dotnet/api/system.void?view=net-10.0)
is primarily a reflection representation of a method with no return value.
[ECMA-335, sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf)
excludes Void as a generic argument. Sources consulted 2026-09-12. NeoCLR deliberately
extends the type model while retaining the ordinary no-result call/ret behavior.

Treating Void only as a marker in generic arguments would require special container
rules. Treating every method as returning a physical unit would change the ordinary
IL stack convention. The chosen split preserves generic value semantics and the
familiar call convention, at the cost of an explicit compiler/metadata distinction.
It is not binary compatibility with arbitrary .NET assemblies.

The prototype distinguishes runtime `Void` from `noresult`. `ldvoid` represents a
logical value in the interpreter; this is not a promise that a future native backend
must allocate bytes for it. The Raven bridge encodes value storage with a named
System.Void value-type token, and no-result returns with CLI VOID. Raw VOID markers
are rejected in value storage. Cecil's named-Void member-reference classification
requires normalization for resolution; encoded signatures are validated separately.

Raven uses an internal Unit representation in some expression contexts. The bridge
admits its empty value representation. An unused target Void propagation statement
must not leave a value on the evaluation stack. Neither this rule nor the target
contract changes Raven's default .NET lowering.

The current executable examples are `Option<int>` and `Result<Void, OverflowError>`.
Broader generic payloads, reflection exposure and native ABI details still need
validation as those features enter the target profile.
