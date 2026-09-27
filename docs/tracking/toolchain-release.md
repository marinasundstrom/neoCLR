# Toolchain and release tracking

**Consolidated 2026-09-27.** Follow the [platform roadmap](../platform-roadmap.md).
This page owns shared integration/delivery status. The [HTTP tracker](../http-capabilities.md#finish-this-poc)
owns whether its packaged sample passes and whether the POC is complete; do not
maintain a second HTTP completion checklist here.

## Pre-release assessment — 2026-09-27

**Author-selected stabilization pass.** Assess main `a79d0b34` before selecting a
new release candidate. This pauses automatic expansion of the next generic-type
slice while feature gaps and regressions are prioritized. Preview 10 remains a
completed release; no new version, tag or publication is selected. Findings and
fresh check results are recorded in [the assessment evidence](../experiments/release-readiness/2026-09-27.json).

### Features needing an explicit finish line

| Priority | Capability | Recommended disposition before release |
| --- | --- | --- |
| First | Task.Run and ordinary async consumers | Require shared identity, mutation across suspension, unit/typed results, unwrapping, cancellation, shutdown and GC to pass together with the selected compiler. Recheck old async capture/interpolation reports: silent loss of output in a supported path blocks release. |
| First | Web API payload contract | Option/nullability is the most useful missing JSON slice for real models; enum mapping follows. Uuid plus parsing/routing is a separate selected feature. Complete these if the release promises the selected richer Web API increment, otherwise explicitly release the current bounded model/array surface and retain these requests. They are missing features, not demonstrated regressions. |
| First | Runtime route mapper distribution | The working mapper is reusable sample source, not an installed SDK API. Ship/test its project and connecting client in the extracted package if it is a release demonstration. Do not require a new hosting framework or generator integration. |
| First — author required | Generic application types / generic async callers | Functioning async/await is explicitly release-critical. Generic methods on nongeneric owners now pass constructed state/closure import, two suspensions, captures, identity and cancellation. Generic instance async receivers now pass as well; Raven owner-arity/receiver fixes are independently integrated. The silent interpolation defect remains open. See [focused evidence](../experiments/task-run/generic-async-validation.json). |
| Second | Interfaces and reflection | Retain the established ordinary class/default/static/explicit-method surface. Generic/value-type implementations, explicit accessors, private instance helpers and wider constraints need separate admission work; they are not automatically release blockers. Run negative access/signature checks with reflection, JSON and mapper consumers. |
| Defer | Additional text codecs/builder, HTTP/2/3, streaming expansion, SQLite | Existing proposals and useful directions do not reopen the completed POC or become release prerequisites. Keep UTF-8/strict ASCII, fixed HTTP/JSON budgets and current calendar/zone ranges explicit. |

### General bugs and gate repairs

1. **Fix confirmed canonical gate failures first.** `cargo fmt --check` fails;
   strict all-target Clippy reports eight diagnostics (six library diagnostics and
   two test diagnostics). These prevent the existing CI gate from certifying a candidate.
   See evidence for exact locations; formatting/lint failures are not runtime-test failures.
2. **Restore the broad signature probe.** It still crashes at the removed
   `Result.Ok.Value` setter in `SignatureProbe.cs:219`. Replace manual-carrier
   assumptions with current union contracts and rerun the probe to completion;
   passing selected metadata checks does not establish the unexecuted remainder.
3. **Fix the reproduced async interpolation failure before release.** The retained
   program compiles and exits 0 with empty output instead of `Value 1`. Inspection
   of `App.raw.dll` shows that the literal/WriteLine call is already absent before
   neoCLR import. Reduce it against ordinary .NET to determine whether the compiler
   repair is general or target-specific. Recheck the older expression-bodied getter
   separately; do not infer a current defect from its historical observation. General compiler defects belong
   in independently tested Raven main fixes, then individual neoclr integration.
4. **Keep known owner boundaries.** Raven's generic-containing-type arity and
   implicit receiver defects are fixed independently (`70cc9dfae`, neoclr
   `7f35ba31c`), with 35 ordinary .NET checks. neoCLR's generic instance async
   consumer suspends and updates both int/string receivers correctly. Generic
   methods on generic owners remain outside the bounded importer. Explicit
   `self.value` assignment to generic `private var` storage has a separate Raven
   binding diagnostic, retained as a compiler reproduction.
5. **Requalify delivery after fixes.** Build matching runtime/library/reference,
   Raven SDK, bridge and editor assets from clean pinned revisions. Test archived,
   extracted demos, failure/stale-output rejection and API coverage. Existing package
   evidence is for older source and cannot certify this increment.

### Validation sequence

The first pass runs all native library unit tests, five risk-focused integration
suites, relevant metadata catalogs, the host-validator tests, formatting, Clippy,
API snapshot checks and two old async reproductions. Exact outcomes are in the
linked evidence; this is not the complete integration suite or a release approval.
Current results: **213 native library tests, 36 selected integration tests, 422
metadata checks and two host-validator tests pass**. API snapshot validation passes.
Formatting, strict Clippy and the broad signature probe fail as listed above. The
async interpolation failure is reproduced. The old parameter capture now compiles
and runs; a strengthened Main returning its Task exits 1 for one input argument,
confirming the captured length reaches the result. This does not certify every capture shape.
After the gate repairs and selected feature closure, run the canonical full source
archive suite **once** on the candidate, then focused macOS/Windows host/ABI checks
and extracted-package consumers. Retain the efficient split CI plan; do not repeat
the full suite on every platform. No website build or performance benchmark is part
of this audit; add performance checks only for a demonstrated regression or release criterion.

## Integration and correctness

| Item | Recorded status | Remaining action / evidence |
| --- | --- | --- |
| Generic method-group compiler defect | Fixed independently on Raven main `13b9105d8`, integrated as `56083626e`; neoCLR helpers in `afcc8c3d` | Retain [regression evidence](../experiments/http-json-client-prototype/README.md#resolution--2026-09-26); do not reimplement the fix. |
| Unresolved call accepted as empty body | Fixed on Raven neoCLR branch `d48bf14ba`; main already rejected the minimal case | [Integration evidence](../raven-target-compilation.md#preview-10-terminal-flow-diagnostics--2026-09-27): 55 focused tests pass. Twenty-two packaged MSBuild checks pass; [release evidence](../preview-10-validation.json) records revision scope. |
| Conditional expression-bodied getter emits zero | Open observation; explicit getter workaround in the upload feature | [Upload evidence](../experiments/http-stream-upload/README.md); general compiler reduction/fix remains separate. |
| HTTP reference/library projection | Upload APIs and Disposable conversion integrated in `45b74768` | [623 signature checks and matching artifacts](../experiments/http-stream-upload/validation.json); no compiler Runtime Contract or native API change claimed. |
| Routing union prerequisites | Case attributes fixed independently on Raven main and integrated; bounded Int32-only carrier projection validated. Nested constant patterns/static Object.Equals remain open. | [Generated mapper and limits](../experiments/route-union-mapper/README.md). SDK integration, other scalar overlays and runtime attribute discovery remain future work. |
| Standard library unions | Option/Result/TaskOutcome migrated; focused contract and task checks pass | [Contract evidence](../experiments/http-error-unions/standard-contract-validation.json); final extracted packages pass; see [release evidence](../preview-10-validation.json). |
| Broad signature probe union assertions | After correcting TaskOutcome's stale zero-interface assertion to IUnion, the probe stops at the removed Result.Ok.Value setter | Refresh old manual-carrier assumptions separately. Six focused comparer metadata checks and the comparer consumer pass; no broad-probe pass is claimed. |
| Constructor-assigned private storage | The selected compiler reports getter-only storage for uninitialized private var/val in the new comparer slices | Comparer adapters and HashMap use explicit fields, documented in the Map contract. Revisit ordinary private storage after an independently validated compiler fix. |
| Private Boolean constructor adaptation | A top-level CLI Boolean conversion helper cannot call a private constructor; the focused StringComparer consumer exposed this access fault | StringComparer uses an internal integer constructor argument and retains private access. Revisit the bridge’s adapter placement with a minimal independent regression; do not widen public access. |
| Async callback parameter capture and interpolation | Current-main interpolation still exits 0 with missing output; raw compiler image already lacks the call. The old parameter-capture import failure no longer reproduces; see current assessment evidence | [Reduced sources and observed failures](../experiments/entry-results/compiler-gaps/README.md); validate general Raven fixes independently. Direct argument use after await and explicit formatting pass. |
| Whole-site API coverage | Entry-point website validation stops at documented `System.LocalTimeMapping.Unique.Deconstruct(System.ZonedDateTime@)` missing from generated reference | Source/reference snapshot passes and changed Raven/Tasks page links/rendering pass; reconcile this separate existing API coverage gap before claiming a full-site build. |
| Task.Run source integration | Ordinary async mutable-local sharing and generic-method capture metadata are corrected in Raven. neoCLR now imports bounded ordinary generic static helpers and their constructed state-machine/closure types, with forced-suspension and cancellation evidence; generic instance async receivers also pass after independently integrated Raven owner-arity/receiver fixes. Run lookup and block-lambda inference now work without the recorded source workarounds. Direct unit await is corrected by recognizing the configured inhabited unit result | [Reduced cases and working consumer](../experiments/task-run/compiler-gaps/README.md); keep typed-stack rejection and validate general compiler fixes independently. |
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
| Next runtime release | Preview 10 remains published; a new pre-release assessment is active on main, with no version selected | Complete the assessment above, fix blockers, freeze scope and qualify a matching candidate. Preserve [Preview 10 evidence](../preview-10-validation.json). |
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

## Local async toolchain — 2026-09-27

The author requested a matching SDK/extension installation and HTTP launch after
the async work. [The installed snapshot](../local-sdk-snapshot.md) records neoCLR
03947b07 and Raven b7bc6838d, three packaged generic async consumers, LSP completion,
independent HTTP requests and the Raven pair. The server remains available in a
Terminal window; the client returned accepted:true. This is local development
evidence, not a published release or a waiver of the open gates above.
