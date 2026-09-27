# Runtime and language tracking

**Consolidated 2026-09-27.** [Platform priorities](../platform-roadmap.md) govern
selection. This is the status owner for runtime/type-system work, not an active
foundation sweep. Useful API coverage is now selected in the
[library/data tracker](library-data.md#active-direction--useful-api-gaps); runtime
expansion below remains deferred unless that work exposes a concrete dependency
or later author direction selects it.

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

## Interfaces as a platform capability

**Author direction, 2026-09-27.** Support static interface members, default
implementations and member accessibility (public/private and the applicable other
levels). Number is a concrete first consumer; numeric-only importer admission is
an interim boundary, not the final interface model. Do not infer that accepted
Raven syntax establishes equivalent neoCLR runtime behavior.

Compared with .NET's static abstract/virtual and default interface members, neoCLR
should offer familiar contracts while preserving its explicit receiver/lifetime
rules. The [existing native default implementation contract](../default-interface-implementations.md)
already specifies most-specific selection, reabstraction and ambiguity; the
[explicit mapping contract](../explicit-interfaces.md) separates dispatch from
ordinary private access. These are working native foundations, not evidence of
complete Raven-facing support. Public/private helper declarations inside an
interface are distinct from a private explicit implementation of a public member.

Next evidence, following the selected [Number slice](../design/numeric-contracts.md):

1. Compile and execute one ordinary static interface helper and one default body
   calling a private helper through the Raven import path; verify rejected external
   access as well as successful internal access.
2. Exercise derived defaults, conflicting diamonds and reabstraction through that
   same path, reusing existing native evidence where unchanged.
3. Add static virtual defaults and constrained selection, plus the applicable
   protected/internal combinations. Define each level's assembly/inheritance scope
   and reject unsupported combinations explicitly.

Broader interface support is selected direction, not a completion claim or a
reason to build the entire platform matrix. Avoid claiming an improvement over
.NET without an observable benefit and its receiver, metadata and dispatch costs.
