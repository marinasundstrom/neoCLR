# Async expression and logical assignment regression

Raven `bab04d508` preserves pattern assignment through logical-value branches and
uses ordinary ordered-call lowering for overloaded binary operators. The prior
compiler rejected Public.rvn with a pattern local loaded before assignment; the
retained diagnostic dump shows the merged Boolean edges. RuntimeExpressions.rvn
checks positive/negative patterns and both success/error Result propagation after
await. All three engines pass, as do 79 shared linear-body CLR tests.

The test bundle changes only Raven.CodeAnalysis.dll from the reason-phrase cohort;
libraries retain their earlier compiler producer. Validation records the actual
compiler/adapter hashes. This is a development qualification bundle, not a release.
Runtime Contract settings and native metadata formats are unchanged. Numeric
nested-operand early exits remain restricted, pending separate CLR emitter work.

The larger Public.rvn now emits native metadata successfully. AOT then exposes a
separate backend specialization limit on `ArrayRef(ArrayRef(Int32))`; its retained
build failure is not counted as a successful native execution. This closes the
reported lowering failure, not every limitation of the broader JSON fixture.

The full fixture also fails during interpreter execution in Console.WriteLine
called by a reflectively constructed PostalAddress inside an async JSON read.
The recorded stack is evidence of an unresolved runtime failure, not a diagnosis
of its root cause. The reduced expression regression remains green.

Raven main uses ExpressionGenerator and has no portable LinearMethodBody. Main is now independently qualified by `f9e2cb5d2`: six baseline tests and
fourteen focused tests (including five new cases) pass on .NET 11, without an
emitter change. Tests cover positive/negative pattern variables, short-circuit
evaluation and success/error propagation inside string equality after both
completed and pending awaits. The enclosing call runs once on success and is
skipped on error. See main-baseline.log and main-tests.log. No main failure was
reproduced; no production fix or full branch merge was carried between branches.
Older .NET targets were not execution-qualified. The existing neoCLR three-engine
regression and 79 planner tests remain the native-line evidence.
