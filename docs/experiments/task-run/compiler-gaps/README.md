# Task.Run compiler integration status

The original submission failures recorded on 2026-09-27 are corrected by the
compiler changes below. Their reduced sources are now positive consumers beside
[Main.rvn](../Main.rvn), checked by `verify.py`. General fixes have independent CLR
validation and individual integration into Raven main and neoclr. Unit-representation
handling stays target-specific. Importer stack checks remain intact. Generic-method
closure metadata is a separate follow-up; these passing cases do not establish all
possible capture shapes.

## Corrected: ordinary async mutable-local sharing

The former 0-instead-of-42 failure is corrected by Raven main `dc7b87eff`, integrated
individually as `08815ceaf` on Raven's `neoclr` branch. The positive regressions now
live beside the consumer: [native submission](../MutableCapture.rvn) and
[inline callback](../MutableCaptureInline.rvn). Their required result is 42;
`verify.py --case mutable-capture --case mutable-capture-inline` checks the contract.
See the [capture checkpoint](../README.md#shared-mutable-local-capture-checkpoint)
for implementation scope and validation.

<a id="separate-general-raven-follow-up-generic-method-closure-metadata"></a>

## Generic captures: compiler repaired, importer gap remains

Raven main `586cc8d89` (integrated as `b32459beb` on neoclr) repairs generic
async capture metadata. Closure fields and state-machine closure references now
use type-owned generic parameters rather than source-method parameters. Both scalar
and array captures execute on ordinary .NET with int and string substitutions;
all 11 focused compiler checks pass. No Runtime Contract option changes.

[GenericCapture.rvn](GenericCapture.rvn) now gets past compiler emission and closed
method specialization. The [ordinary generic helper slice](../../generic-helpers/README.md)
removes the former numeric-only method restriction. Import now rejects the constructed
application async state-machine type (``NamespaceMembers/<>c__AsyncStateMachine0`1<System.Int32>``).
That is missing neoCLR generic application-type import, not a reason to fork Raven's
capture semantics. `verify.py --case generic-capture-import-gap` checks the specific
rejection after a raw assembly is emitted; it does **not** claim execution succeeds.
The intended result after type import support is `42`, then `after`.

The next bounded compatibility slice should start with a closed generic application
holder, preserve field/member signatures and type identity, then revisit the generated
state machine and display class. Keep unsupported shapes checked; admitting ordinary
generic helpers alone does not establish support for generated generic types.

Raven also retains `docs/compiler/development/async-generic-containing-type.rvn`:
an async method inside a generic class fails on ordinary .NET with a state-machine
generic-arity TypeLoadException **without any capture**. That independent Raven
issue, async-lambda-owned locals and iterator capture planning remain separate.

## Corrected: direct completion-only await

The target compiler now recognizes the configured unit representation when discarding
an imported generic result. `Task<System.Void>.GetResult()` produces an inhabited
value; it is not a CLI no-result call. The positive [UnitAwait.rvn](../UnitAwait.rvn)
regression replaces the former negative fixture. Main and MutableCapture now await
completion directly, without mapping to an integer. Importer stack checks remain
unchanged. See the [unit-await checkpoint](../README.md#direct-unit-await-checkpoint)
for the exact validation scope.

## Corrected: unqualified Task.Run lookup

Raven main `f1a3792b8`, integrated as `09f584523` on neoclr, fixes selection of a
simple imported type receiver when generic and nongeneric types share a name.
The generic-first metadata order reproduced the same missing-Run diagnostic using
an ordinary C# reference assembly; all 28 focused CLR checks pass after the fix.
[Unqualified.rvn](../Unqualified.rvn) is now a positive 42-result consumer, and the
shared-capture example no longer uses an explicit alias. The change preserves
local/parameter/alias precedence and explicit generic annotations. No target policy
or importer relaxation is introduced.

## Corrected: inline value-returning block callbacks

Raven main `6cc4fed66`, integrated as `a01fb6245` on neoclr, corrects an initial
completion-only delegate hint being imposed before overload selection. Parameter
hints remain available while unannotated synchronous callbacks infer their returns
when another candidate can return a value. All 64 focused ordinary CLR checks pass,
including explicit/inferred Task.Run results, completion-only work, parameter hints,
async callbacks, expression trees and required diagnostics for a unique Action target.
[BlockLambda.rvn](../BlockLambda.rvn) is now a positive 42-result consumer. Main
passes its capturing block directly, without a typed Func local or Task alias.
