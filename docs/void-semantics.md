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

In neoIL return position, lowercase `void` selects the ordinary CLI no-result
calling convention. `System.Void` explicitly names the inhabited unit type, including
in generic arguments. The older `noresult` spelling remains an internal/legacy alias;
existing uppercase `Void` return declarations retain their unit-return behavior.
This case-sensitive distinction avoids changing legacy library stack contracts while
restoring familiar spelling for ordinary methods. `void` in a generic argument still
names the unit type; the no-result convention applies only in return position.

The prototype retains a return-mode flag internally. `ldvoid` represents a
logical value in the interpreter; this is not a promise that a future native backend
must allocate bytes for it. The Raven bridge encodes value storage with a named
System.Void value-type token, and no-result returns with CLI VOID. Raw VOID markers
are rejected in value storage. Cecil's named-Void member-reference classification
requires normalization for resolution; encoded signatures are validated separately.

## Raven Runtime Contract — 2026-09-14

The author clarified that neoCLR has **Void, not a second Unit platform type**.
Raven's target profile selects `RuntimeUnitContract("NeoCLR.CoreProbe", "System.Void")`.
The shared project properties in `build/NeoCLR.Raven.props` select
`RavenUnitAssemblyName` and `RavenUnitType`, alongside the iteration and propagation
contracts. Raven's normal .NET default continues to use System.Unit.

The language expression `()` represents the selected unit value. A no-result call in
statement position still leaves no value; a call used as an argument or stored value
materializes Void afterwards. `Ok(())` can supply the payload of `Result<Void, E>`.
Raven's Reflection.Emit stage may use a temporary Unit structure internally, but target
emission removes it and emits references to the core's System.Void. That structure is
not a neoCLR type or public API. Unit-specific implementation members are not projected
onto Void. This is not a complete migration of every unit-related compiler feature.

A source-built experimental Raven compiler is required; older SDKs do not interpret
the unit contract properties. Verify both final metadata and execution:

```sh
python3 docs/experiments/raven-target/verify_unit_contract.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

The probe rejects emitted Unit definitions/references, checks core-owned Void
references and ordinary no-result signatures, then executes literal/call value
arguments and both success/error paths of Result<Void, E> propagation. The generic
Runtime Contract mechanism is validated separately on .NET using System.ValueTuple;
that does not establish Void-generic execution on the .NET CLR.

The current executable examples are `Option<int>` and `Result<Void, OverflowError>`.
Broader generic payloads, reflection exposure and native ABI details still need
validation as those features enter the target profile.

## Generic call results — 2026-09-14

A generic method returning `T` returns a value when instantiated with Void. This is
different from a method declared with a CLI no-result return signature: its call
already supplies the result. Raven preserves that value in expression contexts and
pops it when discarded, without synthesizing another unit value. The generic library
probe in [Raven authoring](raven-system-library.md#generic-implementation-gate--2026-09-14)
checks both contexts with the selected Void contract. This reuses existing metadata,
importer handling and runtime generic functions.
