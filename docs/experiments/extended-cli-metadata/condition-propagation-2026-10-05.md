# HTTP condition propagation — 2026-10-05

The full networking/web source group exposed a nested propagation node in
HttpClient.ParseHead: `Text(0, 9, 0)? != "HTTP/1.1 "` inside a short-circuit condition.
Generated child traversal bypassed the lowerer's VisitExpression override. Dispatch
through VisitPropagateExpression fixes that gap. If-condition rewriting spills eager
operands and preserves logical AND/OR laziness with a conditional Boolean temporary,
so an error returns at a statement boundary instead of crossing live stack operands.

This is a general Raven lowering fix; ordinary .NET semantics and backend remain.
No metadata schema, opcode or Runtime Contract change is required. Runtime propagation
still uses the explicit Numbers/System.Propagatable contract in this bootstrap gate.

Eight .NET cases and the native `bootstrap/condition-propagation-consumer.rvn` check
skipped operands, successful extraction, errors, one-time effects and no work after
an error, for AND/OR with propagation on either side of a comparison. The native
consumer compiles using only emitted Numbers metadata, verifies, exits 0 and prints:

```text
Native condition propagation passed
```

[Commands, artifact/source hashes and full-group diagnostics](condition-propagation-2026-10-05.json)
record the explicit primitive bootstrap, retained seed and dependency selection.
Compiler base revision refers to the pre-fix working line; binary hashes identify
exact tested builds. The library sources are absent from the native consumer command.

The unchanged HTTP source group now reaches BoundDelegateCreationExpression emission.
That callback-capability gap is the next task; HTTP execution and full-System
bootstrapping are not claimed complete. The reduction initially used constant union
payload patterns, which the native adapter rejects; the committed condition test uses
payload extraction and comparison to isolate propagation. Constant-pattern support
remains a separate capability gap.

The shared fix is committed on Raven integration as `d707396de` and independently
integrated into local Raven main as `7b971d6c6`. Main passes 26 propagation/runtime-
contract tests on .NET 11; its temporary fix branch was removed. The integration
branch passes 21 propagation and 65 shared-body/runtime-contract tests. No remote
push was performed.
