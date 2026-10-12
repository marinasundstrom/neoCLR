# Managed HTTPS native selection budget

Development checkpoint, 2026-10-12. This qualifies compiler admission bounds,
not a working managed HTTPS exchange.

The author asked whether trimming could help the SslStream/HTTP/JSON showcase,
then directed the safest route for the immediate compiler limit. Keep the
existing closed-world reachability, interface conformance, reflection retention
and GC rules. More aggressive trimming is a separate optimization requiring its
own correctness evidence.

The in-development consumer retains 1,392 functions, including 606 generic
clones, and discovers 489 type shapes. Final selection retains 488 type entries
and excludes 2,913 function entries and 361 type entries. These are compiler
inventory counts, including private specializations, not binary-size savings.
The old 1,024-function bound rejects this reachable program. The intermediate
expanded inventory contains 4,305 functions, exceeding its old 4,096-entry bound.

Raise the selected-function and clone bounds to 2,048, and the input/intermediate
function inventory to 8,192. Keep the existing 512 selected-type and 1,024
input-type bounds. Keep per-method, layout, recursion, heap and runtime limits
unchanged. Specialization and selection use the same inventory constants, and
diagnostics report the configured bounds rather than stale hard-coded values.

This changes experimental native compiler resource admission only. It does not
change managed API or CLI metadata semantics. Relative to .NET Native AOT, the
current implementation still uses a bounded experimental closed-world pipeline;
these numerical limits are implementation guardrails, not language guarantees.
See the existing [native AOT research](../../design-research.md).

Validation includes selected-function admission and object emission at the exact
2,048 boundary, rejection at 2,049, oversized input rejection, existing type and
nesting boundaries, and interface-conformance/generic-specialization regressions.
The actual showcase passes selection with the new bounds but still fails native
value-layout admission: `HttpError` reaches 68 flattened lanes against the
unchanged 64-lane limit. That separate representation issue remains open; this
checkpoint does not claim interpreter/JIT/AOT HTTPS execution or Windows
qualification. See [measured inventory](selection-budget-validation.json).
