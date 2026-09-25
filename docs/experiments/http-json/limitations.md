# Frozen compiler observations

Observed while integrating the public JSON HTTP sample on 2026-09-25, with compiler
SHA-256 `57b6c6e33727de470fe529ee4b5c2b8fb338aeb60d1d6405d9e515276dccd96a`.
These are regression candidates, not intended language or API rules. No compiler
fix is claimed. Independently reproduce/minimize against Raven main before deciding
whether a fix is general or target-specific.

- Some top-level async functions returning `Task<Result<unit, AppError>>` produced
  RAV2704 despite the target Task reference. Moving source order changed one outcome;
  moving async methods to application classes allowed emission. Parameterless Main
  also reproduced the diagnostic, so the initial suspicion of Main(string[]) alone
  was not supported. Root cause is unresolved.
- `readings.Add(JsonNumber.Parse("21")?)?` failed import with a nonempty return stack
  containing the JsonArray receiver. Separating Parse and Add into named steps works.
- Combining two kind-pattern captures with `&&` failed importer definite-assignment
  validation in Acknowledge. Separate nested type checks work.
- A discarded propagated `await context.Complete()?` failed a branch stack merge
  with an extra Void. The sample explicitly handles that completion Result.
- **Fixed in the development runtime/bridge:** the state-machine constructor
  faulted because a hoisted Result field has no readable System.Value default.
  This reproduced with both HttpError and AppError, not just custom error unions.
  [Explicit deferred field storage](../../value-storage.md#deferred-class-fields--development)
  allows initialization during MoveNext while preserving faults on early reads.
  The sample now retains structured AppError results through async methods.

The other compiler observations above remain open, with separate expression steps
used here. The constructor failure is covered by a focused async/GC regression
and this HTTP integration; that does not claim those other failures are fixed.
