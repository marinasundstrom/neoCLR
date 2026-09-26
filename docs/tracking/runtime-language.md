# Runtime and language tracking

**Consolidated 2026-09-27.** [Platform priorities](../platform-roadmap.md) govern
selection. This is the status owner for runtime/type-system work, not an active
foundation sweep. HTTP POC closure remains selected; expansion below is deferred
unless a concrete blocker or later author direction selects it.

## Recorded checkpoints and open scope

| Area | Recorded evidence | Remaining scope / next evidence when selected |
| --- | --- | --- |
| Object, values and references | [Object review](../object-model-review.md), [records](../experiments/records/README.md), [array contract](../managed-arrays.md) | Keep explicit value-copy/reference identity, alias and GC tests. Broader Object consistency and generic/inherited record components are not declared complete. |
| Text storage and identity | [Shared storage](../string-storage-design.md), [execution-owned interning](../experiments/string-interning/README.md) | Automatic literal interning/shared session pools remain unselected. Public text API expansion belongs to the library tracker. |
| Async execution and workers | [Task contracts](../task-contracts.md), [state-machine assessment](../async-state-machine-assessment.md), [cancellation](../cancellation.md) | Runtime-owned suspension and broader scheduling policies remain separate experiments. Preserve working generated state machines; do not infer a backend rewrite from the HTTP POC. |
| Array payload accounting | [Slot-summary evidence](../experiments/http-json/slot-budget-summary.md) | Cost reduction landed; earlier timeout is not diagnosed. No further optimization is selected. HTTP owns application acceptance. |
| Resource/host ownership | [Worker bounds](../isolated-workers.md), [socket ownership evidence](../experiments/socket-completion/GC-OWNERSHIP.md) | General host-memory accounting and lifetime changes need a concrete consumer; do not equate logical payload budgets with process memory. |

## Deferred issue inventory

Issue numbers preserve the 2026-09-26 inventory; open/closed GitHub status has not
been refreshed. See the [original analysis](../history/planning-20260927/issue-fix-roadmap.md#larger-changes-separate-experiments-and-decision-gates)
for alternatives and costs. These are candidate questions, not implementation promises.

| Issue | Next bounded question / finish condition |
| --- | --- |
| [#2 return-directed overloads](https://github.com/marinasundstrom/neoCLR/issues/2) | Compare inference with explicit type arguments; specify ambiguity and diagnostics independently of already-fixed method-group bugs. |
| [#12 importer transition](https://github.com/marinasundstrom/neoCLR/issues/12) | Separate a measured loader/import experiment from a new format or Raven backend. Require equivalent identity, validation, diagnostics and execution; no optimization experiment is currently selected. |
| [#15 metadata unions/intersections](https://github.com/marinasundstrom/neoCLR/issues/15) | Compare nominal interfaces/constraints with a concrete capability-composition case before adding metadata forms. |
| [#17 GC helper](https://github.com/marinasundstrom/neoCLR/issues/17) | Select a diagnostics consumer and invocation/heap contract; a collection request is not deterministic resource cleanup. |
| [#19 Raven bootstrapping](https://github.com/marinasundstrom/neoCLR/issues/19) | Inventory dependencies and run one compiler component before claiming self-hosting; no HTTP release dependency. |
| [#20 tuple structs](https://github.com/marinasundstrom/neoCLR/issues/20) | Clarify syntax versus nominal/structural representation, then test layout, identity and reflection on one pair-valued API. |
| [#21 Self](https://github.com/marinasundstrom/neoCLR/issues/21), [#22 generic math](https://github.com/marinasundstrom/neoCLR/issues/22) | Exercise one algorithm and static interface-member dispatch; Self syntax is not automatically required. |
| [#23 function types/objects](https://github.com/marinasundstrom/neoCLR/issues/23) | Compare nominal delegates, syntax lowering and a runtime callable representation on capture/escape/invocation/GC cases. Replacement remains proposed. |

## Boundaries and evidence

[Library/data](library-data.md) owns public library gaps; [toolchain/release](toolchain-release.md)
owns current Raven diagnostic defects and packaging. Native networking acceptance
belongs to [HTTP/networking](../http-capabilities.md). Keep source/Rust checks with
the changed contract; retain .NET/CLR comparisons in the linked designs and
[design research](../design-research.md). No tests or completion claims were added
by consolidating this record. Historical planning detail remains in the
[previous platform backlog](../history/planning-20260927/platform-backlog.md).
