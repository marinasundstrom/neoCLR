# Task.Run compiler integration gaps

Observed 2026-09-27 with Raven `2f62361ef` on `neoclr`, freshly built compiler/bridge,
and the matching development Task.Run reference. These are current limitations, not
selected API semantics or evidence of a runtime failure. No Raven source was changed
in the submission slice. General fixes need isolated Raven branches, independent CLI
contract tests and extraction to main before target integration.

- `Unqualified.rvn`: wildcard-imported `Task.Run` reports RAV0117 (no Run member).
  Fully qualified `System.Tasks.Task.Run` or `alias Task = System.Tasks.Task` works;
  the alias also permits Task<T> result annotations in the compiled consumer.
- `BlockLambda.rvn`: block callbacks can report RAV1503 (int to void), including an
  explicit Run<int> call. An independently target-typed `Func<int>` local works.
- `UnitAwait.rvn`: direct completion-only await leaves a Void value at a no-result
  return and fails importer stack validation. The larger consumer also exposes
  a branch merge between empty and Void-valued stacks. Assigning the result to a local instead encounters
  the unsupported CLI VOID storage marker. Do not weaken typed-stack validation to
  admit it. The working consumer maps Task<unit> to an integer before awaiting;
  the completion-only Task.Run overload itself executes and completes correctly.

Reproduce by copying one source to Main.rvn beside the existing
[entry contract project](../../entry-results/Contracts.rvnproj) and a matching
NeoCLR.CoreProbe.dll, then invoking `Probe.dll --project <project> <new-output>`.
The focused public consumer script also checks the reduced failures. Unit-await
lowering/metadata should be resolved before advertising friction-free completion-only
await. Keep negative observations distinct from passing implementation evidence.

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
