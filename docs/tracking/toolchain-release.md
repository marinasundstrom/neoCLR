# Toolchain and release tracking

**Consolidated 2026-09-27.** Follow the [platform roadmap](../platform-roadmap.md).
This page owns shared integration/delivery status. The [HTTP tracker](../http-capabilities.md#finish-this-poc)
owns whether its packaged sample passes and whether the POC is complete; do not
maintain a second HTTP completion checklist here.

## Current candidate qualification — 2026-10-14

The 132-lane JSON composed-error carrier is now admitted by the bounded 256-lane
native value layout. Backend and native helper must be rebuilt together. The
[validation record](../validation/release-20261014/inline-validation.json) links
passing macOS three-mode JSON, 132 HTTP wire checks and extracted default-JIT
package acceptance, including the HTTP client/server pair. Runtime-product and
GC qualification pass on hosted macOS and Windows; Windows native HTTP also passes.

The native project builder can derive reflection roots from the freshly compiled
artifact using `--reflection-type`. This avoids retaining the wrong artifact's
reflection identity when the project is rebuilt during AOT qualification.

SDK/VSIX `0.1.15-neoclr.20261014.1` uses Raven native compiler
`fac4ac736024800151d6b285e68407ded864f655`. The
[expanded Windows package gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38539514281)
is still required for three-mode JSON, runtime-await, HTTP and editor acceptance.
Earlier package attempts exposed a missing ZIP PATH; the workflow now exports
the installed executable directory explicitly. Canonical source checks and other
hosted qualification repairs remain under review. Preview 14 remains published;
no new installation or release has occurred.

Additional source parity fixtures need review: their interpreter controls now select
that execution mode explicitly, and the local collections/imported-init gate passes
(`target/release-inline-collections-20261014/report.json`). The introspection/JSON controls now also pass locally after regenerating
reflection roots from the compiled artifact; the stale roots had selected a
generated static owner. Retained HTTP still stops at the
`RuntimeAsyncStartManaged` contract when built through its explicit bootstrap root.
These are separate from the passing packaged JSON and HTTP consumers.

The [editor qualification record](../validation/release-20261014/editor-validation.json)
records all 26 macOS editor checks, native and .NET inlay regressions, matching
SDK/VSIX language-server payloads and fresh extracted default-JIT acceptance.
The inlay failure was fixed on Raven main and carried separately to its native
branch. The updated `.2` Windows candidate includes that regression; the original
`.1` package has passed runtime/JSON/HTTP qualification and is completing editor
acceptance. Canonical source/host checks and Windows JIT HTTP/HTTPS now pass.
Installation/publication still awaits the updated Windows candidate and final
matching-asset review. The retained-bootstrap probe remains an internal gap.

The earlier [failed candidate](../validation/release-20261014/candidate.json) is
retained for provenance. It is superseded by the inline-value fix, not rewritten.

## Release preflight — 2026-10-13

[Local qualification evidence](../experiments/release-20261013/README.md) records the
notice audit repair, current JIT bundle notice refresh, three corrected stale test
expectations and their passing focused checks. The full run completed with 1,956
passing tests and those three failures; it remains recorded as failed. A fresh
source archive passes all 25 smoke programs, seed/notices, native FFI and embedding.
The hosted source/format/Clippy gate, macOS/Windows host checks and minimum-Rust
matrix now pass on a9adc97cb. The later mixed-callback backend repair has focused
Windows object-generation and macOS execution coverage; the callback-fixed Windows
package and all 26 editor checks also pass. Windows interpreter/JIT SSL passes, but
AOT exposed a separate old host heap cap. Its repair now passes the guarded heap
and full Windows SSL/HTTP/framework execution gate. The author installation now has the qualified refreshed
runtime/backend and notices, with the same SDK/VSIX and launch paths. Installed
samples pass, and all 26 real VS Code checks cover the unchanged compiler/extension.
[Draft release notes](../next-preview-release-notes.md) await final artifact review.
No release has been published.

## Installed default-JIT preview — 2026-10-12

The [local installation](../local-preview-20261012.md) closes the HTTP async object-size
blocker below by separating heap-object capacity from inline-value layout. The final
extracted bundle passes default-JIT collections, Tasks, JSON and HTTP; installed
client/server runners pass twice consecutively. SDK/VSIX 0.1.13-neoclr.20261012.2 is
installed side by side for macOS ARM64 author testing. This is not publication or
fresh Windows qualification. The earlier failed attempts below remain evidence.

## Next local preview acceptance — 2026-10-12

The forward async-return declaration issue is fixed on Raven main 0ccb9a265 and
carried independently to the native line as 6b5e052ce. The updated local SDK/VSIX
0.1.13-neoclr.20261012.2 builds all packaged projects, and its language server
returns Task and Promise completions. Extracted default-JIT collections, Tasks and
JSON pass. [Native type-budget qualification](../experiments/native-type-budget/README.md)
raises the bounded inventory consistently with the collector, but HTTP now exposes
a generated async state machine above the unchanged object-size bound. Installation
and release remain gated on complete default-JIT extracted HTTP acceptance; neither
happened. The matching extracted interpreter control passes, including HTTP requests
and the client/server pair.

The PostgreSQL website case passes positional and named-object mapping locally.
General Object display combined with Boolean boxing still reaches the native
`Object display with boxing or arrays requires a later receiver/metadata profile`
restriction; the example uses explicit typed formatting. Keep this backend gap
visible rather than claiming general boxed formatting support.

## Preview 13 qualification — 2026-10-09

[Preview 13](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.13) is published
for Windows x64 and macOS ARM64. Public bundle links return HTTP 200; all five
uploaded asset digests match the qualified local artifacts.

The author selected a bounded Windows x64/macOS ARM64 POC release after the native
String milestone. Both extracted toolchain bundles pass collections, Tasks, JSON
and live HTTP consumers; each installed VS Code extension passes 26 acceptance
checks, including ordinary .NET behavior. Canonical source/archive qualification,
focused host checks and minimum-Rust compilation pass. See the
[Preview 13 release record](../preview-13-validation.json).

This later release scope supersedes the full-bootstrap release prerequisite in the
earlier readiness assessment below. The primitive CLI core and native seed remain
explicit bundle inputs; full production native-only core bootstrap stays open.
ARM64 AOT is a separate source experiment, not a Windows x64 backend or a bundled
native-build task. See [release notes](../preview-13-release-notes.md).

## Native-only compiler-target readiness — 2026-10-09

The module foundation is followed by native-only project core selection on Raven
`67cd4da476f9bd8115567f3730c504ff62a63d1a`, integration branch
`codex/source-object-metadata-resolution`. The [project check](../experiments/native-core-bootstrap/project-execution-validation.json)
passes module binding, build/run and interpreter/ARM64 native result 42.
Native-only project-provider checks reject nine invalid configurations; legacy
bridge-backed project controls still pass. The shared provider watches the native
core, but this is not installed VS Code qualification.

A follow-up corrects native core ValueType classification and verifies copied-struct
execution. Unchanged production Option/Result now identifies missing String.Concat
as a clean compiler diagnostic; no union library is published.
[Frontier evidence](../experiments/native-core-bootstrap/value-foundation-validation.json)
keeps the real source failure visible. The shared diagnostic repair is a deferred
main-line candidate pending independent emitter-boundary validation. Real native UTF-8
fixture wrappers now pass project/interpreter/ARM64 execution after preserving System
module ownership during AOT primitive-method projection. The four-source build now
reaches imported Attribute inheritance; full formatting/core remain unqualified.
[Text evidence](../experiments/native-core-bootstrap/text-services-validation.json).
Including production Attribute alongside UnionAttribute resolves that subset's build:
all five unchanged sources now compile into a native library, with pattern/protocol
consumer parity in project run, interpreter and ARM64 execution. The imported-base
restriction remains a rejection control; it is not required by this source-built subset.
[Union evidence](../experiments/native-core-bootstrap/union-source-validation.json).
Generic String-to-Object boxing now passes display/identity checks alongside unrelated
unboxed union construction in all three execution paths. Native projection retains
reference identity and excludes unboxed values from Object dispatch candidates.
[Boxing evidence](../experiments/native-core-bootstrap/string-boxing-validation.json).
String-payload union display now passes Some/None/Ok/Error in all three paths after
bounded nonmatching Char-test lowering. Char box producers still reject without output;
escaped-string/primitive display and full production Object/bootstrap remain open.
[Display evidence](../experiments/native-core-bootstrap/union-display-validation.json).
Int32 union descriptions now also pass zero/negative/min/max and integer Result
payloads with the explicit native formatter; omitted binding rejects before output.
[Integer evidence](../experiments/native-core-bootstrap/integer-display-validation.json).
Other primitive boxes and escaping remain outside this qualification. The explicit
[escaping audit](../experiments/native-core-bootstrap/escaping-validation.json)
reproduces raw quoting in both modes, not correct escaping. Its initial constructor
receiver rejection is fixed in Raven 1190d03d3; real String replacement remains next.

Release remains gated on complete production source-runtime bootstrap without the
CLI semantic bridge, matching SDK/editor/native artifacts and extracted-package
acceptance. Next qualify core completeness with production sources; do not expand
optional AOT features or label the fixture as a production System.Runtime.
The HTTP teaser remains supported by its existing scoped evidence, not rerun here.

## Pre-release assessment — 2026-09-27

**Author-selected stabilization pass.** Assess main `a79d0b34` before selecting a
new release candidate. This pauses automatic expansion of the next generic-type
slice while feature gaps and regressions are prioritized. Preview 10 remains a
completed release. The author has now directed release continuation, with Windows
also considered; prepare Preview 11 from the current bounded surface, not the
unimplemented optional Web API expansion. Preview 11 qualification is recorded in [the release evidence](../preview-11-validation.json). Findings and
fresh check results are recorded in [the assessment evidence](../experiments/release-readiness/2026-09-27.json).

### Features needing an explicit finish line

| Priority | Capability | Recommended disposition before release |
| --- | --- | --- |
| First | Task.Run and ordinary async consumers | Require shared identity, mutation across suspension, unit/typed results, unwrapping, cancellation, shutdown and GC to pass together with the selected compiler. Recheck old async capture/interpolation reports: silent loss of output in a supported path blocks release. |
| First | Web API payload contract | Option/nullability is the most useful missing JSON slice for real models; enum mapping follows. Uuid plus parsing/routing is a separate selected feature. Complete these if the release promises the selected richer Web API increment, otherwise explicitly release the current bounded model/array surface and retain these requests. They are missing features, not demonstrated regressions. |
| First | Runtime route mapper distribution | The working mapper is reusable sample source, not an installed SDK API. Ship/test its project and connecting client in the extracted package if it is a release demonstration. Do not require a new hosting framework or generator integration. |
| First — author required | Generic application types / generic async callers | Functioning async/await is explicitly release-critical. Generic methods on nongeneric owners now pass constructed state/closure import, two suspensions, captures, identity and cancellation. Generic instance async receivers now pass as well; Raven owner-arity/receiver fixes are independently integrated. The interpolation runtime member and general compiler diagnostic repairs pass focused checks. See [focused evidence](../experiments/task-run/generic-async-validation.json). |
| Second | Interfaces and reflection | Retain the established ordinary class/default/static/explicit-method surface. Generic/value-type implementations, explicit accessors, private instance helpers and wider constraints need separate admission work; they are not automatically release blockers. Run negative access/signature checks with reflection, JSON and mapper consumers. |
| Defer | Additional text codecs/builder, HTTP/2/3, streaming expansion, SQLite | Existing proposals and useful directions do not reopen the completed POC or become release prerequisites. Keep UTF-8/strict ASCII, fixed HTTP/JSON budgets and current calendar/zone ranges explicit. |

### General bugs and gate repairs

1. **Canonical gate repair implemented.** Formatting and strict all-target Clippy
   now pass. Internal cleanup retains completion-value allocation behavior; ten
   focused slot/alias/thread tests pass. The hosted full Linux suite passes at 8e5ae745; the later validator-only correction
   passes extracted-source smoke checks. See the release evidence for the exact split.
2. **Broad signature probe restored.** Standard union cases have read-only payloads;
   remove the obsolete setter assertion and validate the new object Concat overload.
   All 639 checks pass with the generic-unit delegate contract update.
3. **Interpolation repaired.** The missing String.Concat(Object?, Object?) runtime
   overload now returns the expected `Value 1`; nulls, boxed primitives and virtual
   ToString consumers pass. Raven now diagnoses a missing usable overload instead
   of silently emitting no output (main 42834d791 / neoclr 0d5aa83b9; eight focused
   checks). A reduced conditional expression-bodied getter now returns 3/7 correctly;
   the historical default-zero observation does not reproduce in that case.
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
Those initial failures are retained in the dated evidence; the [release repairs](../experiments/release-readiness/preview11-repairs.json)
now pass formatting, strict Clippy, 639 metadata checks and interpolation consumers. The old parameter capture now compiles
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
| Conditional expression-bodied getter emits zero | Historical observation; current reduced true/false getter returns 3/7. The upload implementation retains explicit returns | [Upload evidence](../experiments/http-stream-upload/README.md); general compiler reduction/fix remains separate. |
| HTTP reference/library projection | Upload APIs and Disposable conversion integrated in `45b74768` | [623 signature checks and matching artifacts](../experiments/http-stream-upload/validation.json); no compiler Runtime Contract or native API change claimed. |
| Routing union prerequisites | Case attributes fixed independently on Raven main and integrated; bounded Int32-only carrier projection validated. Nested constant patterns/static Object.Equals remain open. | [Generated mapper and limits](../experiments/route-union-mapper/README.md). SDK integration, other scalar overlays and runtime attribute discovery remain future work. |
| Standard library unions | Option/Result/TaskOutcome migrated; focused contract and task checks pass | [Contract evidence](../experiments/http-error-unions/standard-contract-validation.json); final extracted packages pass; see [release evidence](../preview-10-validation.json). |
| Broad signature probe union assertions | Read-only payload assertions now match standard unions; 639 checks pass | 639 checks pass with the corrected final bridge. |
| Constructor-assigned private storage | The selected compiler reports getter-only storage for uninitialized private var/val in the new comparer slices | Comparer adapters and HashMap use explicit fields, documented in the Map contract. Revisit ordinary private storage after an independently validated compiler fix. |
| Private Boolean constructor adaptation | A top-level CLI Boolean conversion helper cannot call a private constructor; the focused StringComparer consumer exposed this access fault | StringComparer uses an internal integer constructor argument and retains private access. Revisit the bridge’s adapter placement with a minimal independent regression; do not widen public access. |
| Async callback parameter capture and interpolation | Interpolation now prints Value 1 after runtime object Concat repair; Raven diagnoses missing overloads. The old parameter-capture import failure no longer reproduces | [Reduced sources and observed failures](../experiments/entry-results/compiler-gaps/README.md); validate general Raven fixes independently. Direct argument use after await and explicit formatting pass. |
| Whole-site API coverage | The earlier missing-member coverage failure no longer blocks hosted website validation | Hosted website validation passes at 3b08108b ([run](https://github.com/marinasundstrom/neoCLR/actions/runs/36347624886)). Release content edits have 18 focused renderer/reference checks; no local full-site build or publication is claimed. |
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
| Next runtime release | Author-directed Preview 11 preparation: current bounded surface, macOS arm64 toolchain bundle and candidate Windows x64 native runtime | Complete exact-candidate CI and extracted-package qualification; preserve [Preview 10 evidence](../preview-10-validation.json). Windows SDK/bridge is not yet qualified. |
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

## Local development toolchain — 2026-09-27

The author requested a matching SDK/extension installation and HTTP launch after
the async work. [The installed snapshot](../local-sdk-snapshot.md) now uses SDK/VSIX
0.1.12-neoclr.20260927.cpu1 and neoCLR 46b40f80, including recovery and metadata
indexing. It records exact per-component Raven revisions, LSP completion, independent
HTTP requests and the Raven pair. The fresh server was left running in Terminal on
port 63796; the visible client returned accepted:true with zero live objects. Earlier
async bundle/evidence remain available. This is local development evidence, not a
published release or a waiver of the open gates above.

## HTTP CPU investigation — 2026-09-27

[The follow-up](../experiments/http-json/lookup-comparison-20260927.md) records completed
sample recovery, execution-scoped metadata candidate indexing and general Raven
workspace-discovery repair (main 264fcc7d9 / neoclr 2a3fbb346). GET/POST median CPU
fell approximately 24% in the local comparison; steady idle remains cheap. Oversized
requests close their connection and the next valid request succeeds. Header bounds,
deadlines, access/type checks and instruction-boundary quotas are unchanged.

Next performance work should address repeated quota aggregation with a proven
invalidation/ownership contract; do not remove locks or reduce checks speculatively.
Startup remains a separate loading/verification cost. The older language server's
JSON-RPC input-loop CPU and large memory footprint are not fully explained by the
workspace fix. These findings do not add HTTP features or waive existing release gates.
