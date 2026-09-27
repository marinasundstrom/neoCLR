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

## Mutable captured locals: semantic release blocker

`MutableCapture.rvn` should return 42 under Raven's documented shared-variable
capture semantics, but currently returns 0 with no fault. The callback assignment
is not reflected in the caller's captured scalar local. This is a failing contract,
not another successful Task.Run consumer. The runtime's shared object identity/mutation
probes and the shared-state object consumer pass; using an explicit shared object is
an interim workaround, not a replacement for the required lexical-variable contract.

`MutableCaptureInline.rvn` removes Task.Run and await, invoking an int-returning
callback inline from async Main. Its imported MoveNext copies local0 into the closure
field, invokes the callback, then passes unchanged local0 to SetResult. The callback
writes 42 into the closure field. This independently narrows the suspect boundary to
closure/async lowering or import, rather than native task result transfer. The inline
reduction's runtime outcome has not been checked; do not claim that it passed or failed.
The first fix should establish one shared storage location for the captured local,
with independent Raven CLI coverage before target integration.
