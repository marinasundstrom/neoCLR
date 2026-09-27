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

## Separate general Raven follow-up: generic-method closure metadata

A generic async method that captures a `T` local still fails during CLI metadata
normalization. This reproduces on unchanged Raven main and with the capture-storage
fix, using ordinary .NET references. The retained Raven repro is
`docs/compiler/development/async-generic-capture.rvn`, recorded in its
`docs/compiler/neoclr-fix-integration.md` at `dc7b87eff`. It needs an independent
method/type-parameter ownership fix. This is not a new Runtime Contract setting or
a reason to relax importer validation. Its neoCLR-specific impact has not yet been
established. Async-lambda-owned locals and iterator capture planning also need their
own bounded coverage; the source-method fix does not establish every capture shape.

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
