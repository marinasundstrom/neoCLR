# Void: a unit type with no-result calls

Decision recorded 2026-09-12. Void has one logical value and no payload. It is a
valid generic argument in neoCLR. Ordinary void-returning calls retain the CLR
calling convention: they leave no result on the evaluation stack. A targeting
compiler supplies or discards the unit value when an expression context requires
it. Users normally do not construct Void themselves.

Use `Optional<T>` for a useful value or absence, and `Result<Void, E>` for completion
or error. `Optional<Void>` remains technically valid, but it is not the recommended
API example: its useful distinction would be the case, not the payload. `None`
already expresses absence. The propagation protocol's Void residual for Optional
is an internal representation of that absence, not a recommendation to use Void
as Optional's public output type.

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

The author clarified on 2026-10-22 that `void` and `System.Void`
must denote the same type and ordinary no-result return contract. The author further
clarifies that bare `Void` should be rejected: IL should use a type keyword or a
fully qualified name. Generic returns
originally declared as a type parameter retain their value ABI when closed over
Void. There is no proposed case-sensitive semantic distinction.

Migration status: `void` and `System.Void` now select the same no-result contract
in direct method returns and function-shape returns. Both remain inhabited types
in storage and generic argument positions; substituting Void into an originally
generic result preserves that result's value ABI. Producers prefer `void` and
`fn<void>`. The obsolete `noresult` spelling is rejected.

Bare `Void` remains accepted by legacy bootstrap fixtures until their bodies,
callers and producers are migrated together. Its existing value-return behavior
is migration debt, not an additional supported spelling in the intended contract.
Rejecting bare `Void` is unfinished work.

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

The current executable examples are `Optional<int>` and `Result<Void, OverflowError>`.
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

## Source spelling clarification — 2026-09-19

The author considered introducing lowercase `void` in Raven, then selected the
existing `unit` keyword for now. On neoCLR, `unit` and the value `()` use the
configured System.Void representation. This is not a second platform type and
does not change no-result call behavior. See [Raven conventions](raven-conventions.md).

### Fault-call migration and frontend consumers — 2026-10-22

The bootstrap `neoCLR.Runtime.Fault(String)` service and its `System.Panic` wrapper
now use the canonical no-result contract. The CLI RuntimeFailure intrinsic emits
the call without an obsolete `pop`; the migrated generated fragment and seed
copies match that change. This is a text/ABI migration of existing behavior, not a
fresh compilation of every bootstrap slice. The fragment digest is updated while
its previous compiler/source provenance is retained.

The prototype `.neo` frontend now respects imported no-result metadata for calls
and constrained member calls. It materializes logical Void for stored expression
results, removes immediate unit/discard pairs for expression statements, and
resolves namespace functions such as `System.Panic` without inventing a nominal
System owner. Generic value returns retain their existing result. This frontend
is separate from Raven: Raven source semantics and Runtime Contract settings are
unchanged. Canonical callback text projection and the remaining legacy producer
migration are still open; signature checks have not been relaxed.

The AOT fault binding admits the migrated service while retaining the explicit
legacy fixture contract until its migration. Native diagnostics retain the same
message, function identity and instruction trace. Validation: 70 focused frontend,
constrained-call, Function, no-result and fault interpreter tests; seven native
fault-details tests; CLI bridge build (warnings remain).

### Worker notification return migration — 2026-10-22

The bootstrap NotifyWorker service now returns `void`; the CLI intrinsic and its
worker/socket-completion callers no longer pop a nonexistent result. Its callback
parameter remains `fn<Void>` because the bootstrap TaskQueue.Post contract still
uses the legacy value-returning callback ABI. This increment deliberately migrates
the registration call's result separately from the completion callback contract.
The receiver remains traced through notification and queue dispatch. No native C
callback support or pinning is implied. RuntimeContract configuration and Raven
compiler semantics do not change.

GCCollect and GCKeepAlive now also use canonical no-result declarations. Their
bootstrap wrappers and CLI intrinsic mappings remove the old pops; GC roots,
collection behavior and KeepAlive's lifetime contract do not change. The migrated
fragment digest is updated without claiming a fresh complete bootstrap build.
Ten GC (including the real bootstrap facade), sixteen structural-tuple, twelve
service and seventeen worker tests pass across the main and focused corrected
runs. The bridge builds with eleven existing warnings.
