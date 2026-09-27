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

## Corrected: generic-method captures and constructed task results

Raven main `586cc8d89` (integrated as `b32459beb` on neoclr) repairs generic
async capture metadata. Closure fields and state-machine references use type-owned
generic parameters. Raven `4c8d60176` (neoclr `4cfc75b4e`) additionally fixes imported
signatures containing constructed source types such as `Task<Holder<string>>`.
The latter has independent default/target-metadata .NET execution coverage; all
16 selected signature, array and capture checks pass. No Runtime Contract options change.

neoCLR now imports the bounded generic state-machine and closure definitions, keeps
constructed field/member signatures, and binds public callbacks to their constructed
receivers. [GenericCapture.rvn](../GenericCapture.rvn) is a positive consumer:
`verify.py --case generic-capture` requires `42`, then `after`.
[GenericSuspension.rvn](../GenericSuspension.rvn) forces two pending awaits and checks
value/text results, array/object identity and cancellation. See the
[generic async checkpoint](../README.md#generic-async-application-import) for scope and evidence.

Raven main `70cc9dfae` (neoclr `7f35ba31c`) now fixes the generic-containing-type
arity and implicit instance-field receiver defects, with 35 focused CLR checks.
[GenericOwner.rvn](../GenericOwner.rvn) also passes on neoCLR after admitting nested
state types under bounded generic owners. Generic methods on generic owners remain
an importer limit, independently of the corrected general compiler behavior.
A separate target-metadata async-attribute lookup failure discovered during reduction
is recorded in Raven's runtime-contract docs. Async-lambda-owned locals and iterator
capture planning also need their own bounded coverage.

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
