# Equivalent stack addresses through joins

2026-10-25. Extend experimental scalar-store lowering to retain stack provenance
when every incoming edge supplies the same address. Distinct SSA expressions can
agree when each is directly anchored in the same compiler stack slot and checked
constant offset. Identical SSA values/aliases also agree. Keep a representative
value; a single disagreement permanently makes that parameter unknown.

This is a compiler proof, not permission to skip native validation for arbitrary
pointers. Differing slots or offsets, unknown byrefs and unsupported expressions
remain guarded. Direct comparison does not follow unresolved join parameters, so
cycles cannot invent anchors. Subsequent bounded traversal retains its cycle limit
and checked offset arithmetic. The existing store-extent check still rejects an
out-of-slot access before modifying IR. Known-caller stack auditing uses the same
edge analysis; its complete-closure, escape and visibility constraints remain.

## Validation and effect

52 focused mutation-store/stack-audit tests pass; 31 integration tests requiring
external native fixtures remain explicitly ignored in that invocation. Extended
JIT fixtures execute both branch outcomes for identical addresses, differing
offsets, distinct slots and mixed stack/byref joins. They verify output, adapter
counts and injected failure behavior. Additional checks preserve conflict regardless
of edge traversal order, reject joined out-of-bounds stores without IR edits and
retain cycle/overflow rejection. In the equal-join fixture the stack write becomes
direct, reducing adapter calls from two to one while the output byref stays checked.

Recompile all five retained complete workload selections with the same speed,
forwarding and guard-removal options: tasks, collections, arrays, plaintext and
JSON. **Every emitted object is byte-for-byte identical** to its control. Therefore
this bounded extension supplies no measured workload performance or allocation gain;
repeating native capacity runs would execute identical machine code. Existing
workload execution evidence remains applicable to those objects. This is not broader
JIT HTTP, shared-worker, pause or concurrent-default qualification.

[equal-join-evidence](equal-join-evidence/) retains the release test output, build
driver, exact commands/options, old/new object hashes, read-only audits and compiler
source hashes. The runner is `neoclr_codegen-12f33f26d2ff4df0`; all five build commands
run the retained full-selection integration test explicitly. The change is confined
to neoCLR backend proof/lowering. Raven emission and Runtime Contract configuration,
CLI/native metadata representation, provider capabilities and managed semantics are
unchanged. It introduces no new bridge encoding or replacement obligation.

## Next work

The rejected native counter/outlining screens and unchanged workload objects show
that these local changes do not close the guarded-sync CPU gap. Investigate which
remaining dynamic stores cannot be proven local, and whether their origin can be
represented more precisely through value construction, loads and call boundaries.
Do not infer stack provenance from address magnitude or cache it across unqualified
lifetimes. Any further proof must preserve native ownership and active-epoch safety,
then show an actual generated-code and measured workload effect before a speed claim.
