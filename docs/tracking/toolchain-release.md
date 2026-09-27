# Toolchain and release tracking

**Consolidated 2026-09-27.** Follow the [platform roadmap](../platform-roadmap.md).
This page owns shared integration/delivery status. The [HTTP tracker](../http-capabilities.md#finish-this-poc)
owns whether its packaged sample passes and whether the POC is complete; do not
maintain a second HTTP completion checklist here.

## Integration and correctness

| Item | Recorded status | Remaining action / evidence |
| --- | --- | --- |
| Generic method-group compiler defect | Fixed independently on Raven main `13b9105d8`, integrated as `56083626e`; neoCLR helpers in `afcc8c3d` | Retain [regression evidence](../experiments/http-json-client-prototype/README.md#resolution--2026-09-26); do not reimplement the fix. |
| Unresolved call accepted as empty body | Fixed on Raven neoCLR branch `d48bf14ba`; main already rejected the minimal case | [Integration evidence](../raven-target-compilation.md#preview-10-terminal-flow-diagnostics--2026-09-27): 55 focused tests pass. Twenty-two packaged MSBuild checks pass; [release evidence](../preview-10-validation.json) records revision scope. |
| Conditional expression-bodied getter emits zero | Open observation; explicit getter workaround in the upload feature | [Upload evidence](../experiments/http-stream-upload/README.md); general compiler reduction/fix remains separate. |
| HTTP reference/library projection | Upload APIs and Disposable conversion integrated in `45b74768` | [623 signature checks and matching artifacts](../experiments/http-stream-upload/validation.json); no compiler Runtime Contract or native API change claimed. |
| Standard library unions | Option/Result/TaskOutcome migrated; focused contract and task checks pass | [Contract evidence](../experiments/http-error-unions/standard-contract-validation.json); final extracted packages pass; see [release evidence](../preview-10-validation.json). |
| Broad signature probe union assertions | After correcting TaskOutcome's stale zero-interface assertion to IUnion, the probe stops at the removed Result.Ok.Value setter | Refresh old manual-carrier assumptions separately. Six focused comparer metadata checks and the comparer consumer pass; no broad-probe pass is claimed. |
| Constructor-assigned private storage | The selected compiler reports getter-only storage for uninitialized private var/val in the new comparer slices | Comparer adapters and HashMap use explicit fields, documented in the Map contract. Revisit ordinary private storage after an independently validated compiler fix. |
| Private Boolean constructor adaptation | A top-level CLI Boolean conversion helper cannot call a private constructor; the focused StringComparer consumer exposed this access fault | StringComparer uses an internal integer constructor argument and retains private access. Revisit the bridge’s adapter placement with a minimal independent regression; do not widen public access. |
| Async callback parameter capture and interpolation | Independently reproduced with synchronous Main plus async helper during entry integration | [Reduced sources and observed failures](../experiments/entry-results/compiler-gaps/README.md); validate general Raven fixes independently. Direct argument use after await and explicit formatting pass. |
| Whole-site API coverage | Entry-point website validation stops at documented `System.LocalTimeMapping.Unique.Deconstruct(System.ZonedDateTime@)` missing from generated reference | Source/reference snapshot passes and changed Raven/Tasks page links/rendering pass; reconcile this separate existing API coverage gap before claiming a full-site build. |
| Task.Run source integration | Mutable scalar writes return the wrong caller value (0 instead of 42); Run lookup needs an explicit alias; block lambdas can need typed delegate locals; direct unit await leaves a Void stack value | [Reduced cases and working consumer](../experiments/task-run/compiler-gaps/README.md); keep typed-stack rejection and validate general compiler fixes independently. |
| API reference | Source changes require matching bridge/reference and useful member docs | [Maintenance procedure](../../api-docs/README.md), including explicit renderer exclusions; do not substitute a feature page for member coverage. |

| Boxed Completed pattern binding | Open compiler observation; explicit closed-case cast validates IUnion.Value | [Union guide](../raven-union-api.md#task-outcomes); ordinary task outcome patterns pass. |
| Historical source API checklist | Still references obsolete generated/File fragments; 2026-09-27 regeneration reaches an unreviewed UInt64ToString caller and cannot finish the old audit | Refresh the separate source checklist and its caller discovery across current Raven/archived profiles. The casing/Int64 slice leaves this historical artifact unchanged; current on-site metadata inventory and snapshot pass. |

General Raven fixes belong on independently tested main-based feature branches;
neoCLR-specific policies stay isolated. Compiler-affecting integration requires the
applicable documentation/changelogs in both repositories. The
[nine Raven issue assessments](../history/planning-20260927/issue-fix-roadmap.md#raven-issues-through-the-neoclr-lens)
are a 2026-09-26 snapshot, not a new live inventory or authorization to fix all nine.

## Delivery and documentation

| Item | Status / owner boundary | Next action when selected |
| --- | --- | --- |
| HTTP evaluator package | POC acceptance complete on macOS arm64; HTTP tracker owns the evidence | [Package evidence](../experiments/http-poc-package/README.md) records the exact artifacts. Requalify the selected release candidate and shipped targets; do not treat a local POC pass as full release readiness. |
| Next runtime release | [Preview 10 published](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10) on 2026-09-27; all eight remote asset digests verified | [Candidate notes](../preview-10-release-notes.md). Release work is complete; no new feature milestone is selected. |
| CI efficiency | All six jobs passed; slowest 8.88 minutes, about 17.6 runner-minutes | [Hosted evidence](../experiments/ci-split/README.md). Canonical evidence precedes the union migration; validate that contract and rebuilt packages separately. |
| [#9 RavenDoc](https://github.com/marinasundstrom/neoCLR/issues/9) | Dated inventory spans correctness/navigation and richer rendering | First reproduce a current label/navigation or coverage gap; separate small repairs from union modeling/hierarchy features. Keep linked manual coverage until rendering supports a member. |
| Website | Local validation and manual publication are separate | [Maintenance](../design/feature-pages.md), [site procedure](../../website/README.md); successful build/push is not deployment. |
| Tracking consolidation | Completed organization in this change | One owner per theme; keep contracts and evidence linked, archive superseded priority sequences, and update current rows instead of appending competing plans. |

The async/Tasks Preview 9 release is historical: its
[completed plan](../async-preview-plan.md) and [published evidence](../preview-9-validation.json)
remain unchanged. Earlier readiness reports do not reopen that release or certify
the next one. Portable Sample Pack (M6) remains a candidate milestone; no target set
is selected by this consolidation.

## Comparison and maintenance rules

This reuses existing integration and release research; it introduces no new compiler,
packaging or API contract. Preserve source/package/release distinctions and exact
artifact provenance. Issue numbers above retain the 2026-09-26 inventory; no issue
status was refreshed or closed. Status changes belong here; implementation details
and commands remain in the linked procedures and evidence.
