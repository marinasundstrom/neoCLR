# neoCLR platform roadmap

**Updated 2026-10-01.** This is the authoritative default for work priorities,
milestone sequencing and scope. Explicit author directions take precedence.

## Current work

**Author-directed metadata exploration (2026-09-30).** Begin extended CLI metadata
on `codex/extended-cli-metadata`, based on main, with later Raven integration. The
author confirms the Cecil-style metadata API remains an independent project, consumed
by Raven's neoCLR target through compiler-owned loader/emitter adapters; integration
does not move the library into the compiler. [Project ownership](design/extended-cli-metadata.md#independent-metadata-project-raven-target-consumer-2026-09-30).
**Latest sequencing clarification:** establish a working metadata format and its APIs,
then integrate the refactored compiler, then add improvements such as structural types.
The direct native producer/load test below is the bounded acceptance baseline;
ordinary compiler-required metadata coverage is next.

**Architecture follow-up (2026-10-01):** after reviewing Reflection dependencies, the
author directed a staged codegen refactor. The first slice now feeds both bounded
backends compiler-lowered bodies, including implicit value returns. The first callable reference table now shares symbol identity resolution with typed
backend handles; broader type/field references and declaration traversal remain next,
followed by paired compiler/metadata body capabilities;
metadata loading remains a future slice. See [integration scope and validation](raven-cli-bridge.md#compiler-lowered-native-bodies--2026-10-01).

**Shared compiler pipeline:** Raven's explicit native backend now participates in
`Compilation.Emit`; rvnc and API wrappers share compiler setup and target validation.
The author directs reuse of common .NET/neoCLR lowering with backend abstractions for
builder differences. The shared linear-body model now feeds .NET/native method-builder adapters and
executes the same Int32 and Unit Hello/helper compilations on both runtimes, including
assembly functions, explicit/implicit returns and an empty Unit entry. Supported callable
signatures and declaration-builder contracts are now shared too; concrete adapters preserve
CLI type-method and native assembly-function ownership. Type/signature builders, generics and
control flow remain subsequent boundaries. The author asks to avoid large workarounds
and prioritize the Hello/helper end-to-end case. Shared metadata loading is deferred;
the optional System driver has exposed a host/projection type collision and explicitly
rejects that call. Its direct API case still passes; driver import is not yet reliable. [Integration boundary](raven-cli-bridge.md#shared-emission-pipeline--2026-09-30).

**Latest acceptance:** Raven's opt-in emitter now writes binary PE/#Neo assemblies;
neoCLR decodes their native metadata directly. Hello World, an entry-point call to
Greet, both source-file orders and the two-library chain all verify/run successfully.
Schema-1 JSON containers remain readable. The separate Cecil-style API still supplies
CLI reference projections to Raven's existing .NET semantic provider. The author
identifies class-library compilation and Raven symbol loading as the next consumer,
with existing JSON-to-assembly translation as a bootstrap. The complete current System
JSON now translates to a standalone binary native envelope, verifies all 641 IL functions
and runs both Hello cases and a generic dependency chain. This has no CLI reference
projection. Next: compile the runtime class library from Raven source and compare
against these translated artifacts; broad symbol import, a native symbol provider
and production registration remain unimplemented.
[Binary profile](design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30)
and [measured phases](experiments/extended-cli-metadata/binary-loading.md).

**Author-directed consumer exploration:** Exercise translation of the existing Raven
CLI bridge's test/sample output before expanding direct compiler emission. A proper
neoil assembler producing native assemblies is a future producer. The Raven collection
library profile exceeds the current binary transport bounds; retain that explicit gap
while isolating application translations with matching JSON System. Floating-point
operand tests additionally expose the need for full UInt64 bit-pattern encoding.
**Follow-up implemented:** standalone schema 3 adds that encoding and bounded larger
library capacity; FloatingMath, OptionPositional and ValueCopy now run against binary
Raven System. The neoil binary producer and direct native compiler/symbol import remain
next work. [Profile and validation](design/extended-cli-metadata.md#library-execution-profile-3--2026-09-30).
[Experiment and reproducible checks](experiments/extended-cli-metadata/raven-sample-translation.md).

**Direct assembler checkpoint:** `assemble --format neox` now emits schema-3 native
assemblies directly from neoil, with verification before output and JSON kept as the
default. The full Raven collection System and three sample applications pass independent
.NET metadata comparisons and native execution. Direct Raven class-library source emission
and native compiler symbol import remain next. [Producer evidence](experiments/extended-cli-metadata/direct-assembly.md).

**Author-requested performance evidence:** A release comparison finds the full Raven
System assembly 63% smaller and about 20% lower decode time than current pretty JSON,
but no meaningful complete-startup/run improvement (about 3.45 seconds). Linking and
admission dominate; direct-typed JSON is a faster diagnostic alternative to both current
decoders. The author explicitly accepts smaller files as an improvement and defers
runtime optimization to the future. Retain this baseline and continue metadata/compiler
integration; the benchmark does not reprioritize runtime optimization.
[Method and limits](experiments/extended-cli-metadata/json-vs-assembly-benchmark.md).

**Raven continuation:** The author selects continued Raven work after the size benchmark.
The native compiler adapter now emits Unit-returning helpers and static library methods,
reimports their CLI no-result projections and runs a separate application against the
binary library. This removes the artificial Int32 return from side-effecting helpers;
entry points remain Int32. Namespaced static classes now preserve identity through
compiler reimport and native calls, including same-name types in different namespaces
and both source orders. Wider signatures, native symbols and actual runtime class-library
source compilation remain next. [Compiler contract](raven-cli-bridge.md#unit-returning-native-helpers-and-library-methods--2026-09-30).

**Native compiler command:** An opt-in `rvnc neoclr` command now compiles source files
and native writer-produced PE references directly to PE/#Neo, including assembly-owned
functions. The author confirms symbol loading from translated System/System.Runtime is
also required. Prioritize that loader acceptance alongside native emission; host primitive
binding is only a bootstrap, not evidence of translated System symbol loading. Broader
native declaration reading/projection or a native symbol provider is still required.
[Command and loader boundary](raven-cli-bridge.md#opt-in-native-compiler-command--2026-09-30).

**First translated System symbol slice:** An explicit static Int32 callable view now
adapts selected native declarations to Raven's existing semantic importer. Math.Min binds
to that view and executes against the translated System through both API and rvnc paths.
This is not full core import: primitive binding remains hosted, and generic/instance/field/
property contracts remain next. The author supports adapting the existing importer first,
then a separate provider as needed, and removing Reflection.Emit from .NET in the future.
A common instruction/operand model and later ILProcessor-like editing are recorded future
metadata API directions. [Scope and architecture](raven-cli-bridge.md#translated-system-callable-import--2026-09-30).

**Native entry contract:** Raven now emits parameterless Unit Main as a no-result
native entry, including global/static ownership and helper calls, and neoCLR exits zero.
The metadata reader/writer reuse format 5's existing no-result encoding. This removes
an artificial compiler restriction while keeping metadata support and target integration
as the main objective. Full System import and richer callable/body coverage remain next.
[Entry contract and evidence](raven-cli-bridge.md#unit-entry-points--2026-09-30).

**Compiler-facing opcode surface:** The independent metadata API now exposes bounded
Emit overloads for typed opcode/operand construction, and Raven's native emitter uses
them. Convenience helpers share the same path. This advances target code generation;
it does not claim full opcode coverage or ILProcessor-style editing. Broader metadata,
System symbol import and native source coverage remain the objective.
[Contract](raven-cli-bridge.md#opcode-based-metadata-emission--2026-09-30).

**Earlier metadata checkpoints (historical progression):** The PE reader now recovers
owned global/type callable declarations and the writer's static Int32 signature subset;
21 C# contract groups cover ownership, unsupported signatures, decoding limits and
explicit MemberRef resolution for the emitted nominal static method subset. General
signatures, broader member contracts and body import remain readiness gaps. The author
now selects **Refactor Raven for platform targets** (activity
`01a0f154-2448-7df3-8536-c837097b46c2`) as the integration consumer: attempt a bounded
compiler import/emission/runtime case once this subset suffices, then drive remaining
metadata support from that case rather than waiting for exhaustive coverage. Stage 1
now passes: Raven binds a small source program and an API-produced PE dependency;
an opt-in adapter consumes public operations and emits native bytes through the separate
library, which neoCLR verifies and executes to 42 ([evidence](experiments/extended-cli-metadata/raven-compiler-validation.json)).
The frontend still uses the .NET metadata provider; native provider/production emitter
composition are next. The case exposed shared operation and parameter-import fixes,
validated by 87 focused Raven tests and integrated into local Raven main. See the
[inspected compiler boundaries and next case](design/extended-cli-metadata.md#refactored-compiler-as-the-next-end-to-end-consumer-2026-09-30). Structural types remain a future
design requirement (arrays, tuples, Function types, unions/intersections and synthesized
members), not a prerequisite for that first compiler integration. The
[design and staged acceptance plan](design/extended-cli-metadata.md) is exploratory;
an isolated codec/inspector now validates experimental framing and structural
signatures, host-catalog nominal resolution and synthesized-member references with
30 focused tests. Differently numbered fixture references resolve to equal structural
type/member keys. A bounded PE32 #Neo probe preserves conventional metadata for
.NET/Cecil inspection; Cecil rewriting strips the extension. An experimental marker/digest and explicit expected-input profile now reject
stripped/changed metadata in 11 recognition cases. The first .NET reader/writer library now covers envelope framing, with four shared
fixtures and 49 cross-reader rejection cases. Its structural signature codec now passes
14 cross-reader vectors and 103 rejection cases. .NET reference tables and explicit-catalog
structural identity pass 95 shared vectors (69 rejections). .NET synthesized-member
contracts now pass 49 shared vectors (30 rejections). A typed .NET reference-profile
Read/Create/Write facade passes 36 shared vectors (29 rejections). Bounded .NET PE32
artifact recognition/extraction passes 50 shared cases (44 rejections). Following the
author’s Cecil suggestion, the [provisional API direction](design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30)
places assembly/module/reference/definition objects above these codecs. An initial
read-only model now reads real assembly/module/TypeDef declarations, with generic/nested
Unicode and snapshot-scoped reference consumer checks. AssemblyRef identity and explicit
host resolution now pass 11 standalone C# contract tests. The author selects the
Cecil-like model as the primary compiler abstraction for metadata/PE manipulation,
with adaptations as needed. Physical TypeRef resolution and a controlled static-Int32
PE writer now support an end-to-end producer baseline. API-produced application and
library PEs execute in neoCLR through the existing CLI bridge and matching native
library, returning 42. Broader IL/signature coverage, general rewriting, Raven codegen
integration and direct native #Neo loading remain pending. The author also requires
functions outside types: the model now exposes assembly-owned functions and emits
native format-5 assemblies directly, with a cross-assembly function test returning 42
in neoCLR without the CLI bridge ([evidence](experiments/extended-cli-metadata/native-validation.json)). Production NEOX loading and structural runtime support remain unimplemented. This scopes the requested exploration
without promoting all proposals or merging the structural runtime experiment.
The author additionally requires eventual reader/writer support on both .NET and
neoCLR. The author further clarifies that the .NET API should support later Raven compiler
integration, with a potential Raven port beneath the pending Metadata Introspection API;
[readiness criteria and open choices](design/extended-cli-metadata.md#net-api-direction-and-potential-raven-port-2026-09-30)
keep this distinct from completed integration. The [cross-platform library plan](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr)
separates shared format/conformance contracts, .NET tooling, native support and an
actual neoCLR guest library. Concrete consumers are Raven’s symbol loader/code
generation and neoCLR assembly loading into Introspection/assembly emission; these
full library consumers remain planned; the .NET framing foundation is implemented.

**Immediate focus — Raven neoCLR target support (author-selected 2026-09-30).**
After the bounded main backport below, put the structural Function experiment on
hold and work in Raven to improve neoCLR target support. The Function runtime
branch remains isolated; its proposals are open, not completed on main. Specific
acceptance now includes integrating target-gated Self in Raven and neoCLR while
keeping structural types on feature branches in both repositories. This author
direction supersedes the earlier restriction on Raven main integration; a native
metadata layer and replacement backend remain future work. General compiler fixes
still require independent validation.
The earlier Web API direction below remains recorded for later resumption.

**Author-directed library backport (2026-09-30).** Bring Raven callback function
syntax and lazy OfType filtering to main independently of the structural Function
runtime branch. This bounded library change keeps the existing callable and
introspection models. It is complete; the subsequent author direction above selects
the next focus. See the
[query contract and evidence](raven-query-api.md#runtime-type-filtering--2026-09-30-backport).

**Author-directed release — Preview 11 (2026-09-27).**
The current bounded surface is qualified for release with macOS arm64 Raven tools
and a Windows x64 native-runtime ZIP. See [release notes](preview-11-release-notes.md)
and [qualification evidence](preview-11-validation.json). Generic async, shared Task.Run,
text/number foundations, nested JSON and routing samples are included. Windows
Raven SDK/bridge qualification remains separate. The full Linux suite passes; a
validator-only exit-code correction has independent archive-smoke evidence.
This closes release preparation without adding optional Web API capabilities.

**Preceding direction — minimal Web API (author-selected 2026-09-27).** Focus on
serving a useful Web API, nested JSON serialization/deserialization, and a
route parser used within an existing HttpServer handler, including named and typed
parameters. This later author direction defers the earlier separate WebApplication
project and Minimal API infrastructure.
A rudimentary SQL interface with a SQLite provider is an optional follow-on based
on the existing proposal, not a prerequisite or approval of its complete surface.
The [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api) owns
application acceptance; the [bounded plan](web-api-plan.md) proposes the sequence
and records design choices still to validate. The first nested typed JSON slice is
implemented in development with [consumer and HTTP evidence](experiments/json-object-mapping/nested-validation.json).
JSON now matches the 1,024-byte HTTP body budget, with [boundary and case evidence](experiments/json-object-mapping/payload-validation.json).
Typed arrays now support collection payloads; see [collection evidence](experiments/json-object-mapping/collection-validation.json).
The author's route/union refinement selects the next bounded routing slice: direct
RoutePattern/RouteMatch parsing and optional application union dispatch are now
implemented in development; see [the case](experiments/http-routing/README.md).
A [generated attribute-driven mapper experiment](experiments/route-union-mapper/README.md)
validates schemas and reuses compiled patterns. The author subsequently selects
runtime attribute reflection with cached startup mapping. Member/parameter
[attribute data](attribute-introspection.md) is now implemented in development;
the [runtime mapper case](experiments/runtime-route-mapper/README.md) now validates route schemas at startup and retains patterns, conversion bindings and case/carrier constructors for request handling. Source generation remains a future alternative. The mapper ships as tested Preview 11
sample source; it is not an installed SDK mapper API.
Enum, Uuid and Option JSON mapping remain requested and pending.
This explicitly supersedes the previous general useful-library priority; M2–M6
remain candidates. Preview 10's completed POC stays closed.

### Completed POC and preceding library direction

The bounded HTTP POC is [complete on macOS arm64](http-capabilities.md#finish-this-poc),
including independent peers, known-length uploads and the extracted-package check.
Feature scope is frozen; do not automatically start another HTTP feature.

**The POC is released and done for now:**
[Preview 10](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10),
published 2026-09-27. Option, Result
and TaskOutcome use standard Raven unions, and the native reflection boundary uses
the matching Option layout. See [release evidence](preview-10-validation.json).

The [toolchain/release tracker](tracking/toolchain-release.md) owns residual compiler
observations and delivery maintenance. Hosted CI passed all six split jobs at
`1404454e` (slowest 8.88 minutes); later migration corrections passed the focused
checks requested by the author and final extracted-package checks. The released POC feature
scope remains frozen; the new Web API direction is a separate increment. Thematic consolidation is complete.

**Preceding direction: useful library APIs**, selected by the author after release.
The author-selected comparer slice is implemented in development: equality/hash and
ordering policies, callback adapters, StringComparer.Ordinal and a HashMap policy
constructor. See the [library/data sequence and evidence](tracking/library-data.md#active-direction--useful-api-gaps).
The author-selected explicit String comparison modes and matching ignore-case policy
are implemented in development. The [String/System.Text foundation review](design/text-abstraction.md#systemtext-foundation-review--2026-09-27)
is complete: keep the UTF-8/grapheme model, but establish scalar, position, ownership
and conversion-progress contracts before dependent API expansion. The author clarifies
that the immediate System.Text scope is encoding/decoding foundations and possibly
a small builder, with Swift as the closer model for text API shape.
The [library tracker](tracking/library-data.md#string-design-review-before-further-expansion)
owns the boundary experiment, [paired text API sketch](design/text-abstraction.md#consumer-api-sketch-identical-behavior-two-vocabularies)
and the shared Encoding/Decoder integration in both stream adapters. Development
constructors accept encoding selection with UTF-8 defaults and strict ASCII, backed
by focused consumer checks. The [bounded report construction evaluation](design/text-abstraction.md#bounded-report-construction-evaluation--2026-09-27)
passes its contracts but does not justify promoting the managed builder: ordinary
concatenation is faster for the tested report sizes. Keep public builder promotion
deferred; do not start a builder optimization project. The [Encoder acceptance/drain evaluation](design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27)
led to the [public Encoder integration](design/text-abstraction.md#public-encoder-integration-development--2026-09-27):
independent factories, bounded progress and explicit StreamWriter.Finish are implemented
in development with matching API artifacts and focused consumers. The bounded
UTF-8/strict-ASCII encoding foundation is complete. Broader codecs, general writer
completion and further text capabilities need separately scoped consumer work. General
scalar/range APIs and the broader portfolio are not prerequisites. Experimental
types and proposed names are not adopted System APIs.
The subsequent author-selected [casing and Int64 report slices](design/text-casing-integer.md)
are implemented in development: Unicode 17 full default String casing, typed strict
Int64 parsing, decimal formatting and bounds properties. Focused consumer/native/
metadata checks and matching API artifacts cover this bounded work. No new broader
milestone is selected by completing these slices.
The author next selected [Number and concrete primitive parsing](design/numeric-contracts.md).
The bounded development slice is implemented with focused consumer/native/metadata
evidence; all numeric parsers use NumberParseError. A parsing interface is on hold. The author also
identified [general interface capabilities](tracking/runtime-language.md#interfaces-as-a-platform-capability)
as important direction: static members, default bodies and member access control.
Number is a first consumer, not a permanent numeric-only interface model.
The next bounded interface slice adds application defaults and public/private static
helpers through Raven, with [focused evidence](experiments/interface-helpers/README.md).
The [interface limitations table](tracking/runtime-language.md#interface-limitations--development-checkpoint-2026-09-27)
separates native support from Raven import gaps. A subsequent bounded
[explicit class implementation slice](experiments/explicit-interface-implementations/README.md)
adds ordinary methods on application classes/interfaces. Accessors, generic/value-type
import, private instance helpers and broader static/default/accessibility cases remain open.
The author also selected [Result/Task Main integration](experiments/entry-results/README.md),
with direct async Main adoption in relevant samples after focused validation.
This is a bounded startup integration, not a new broader milestone.
The author next selected [Task.Run with shared captured objects](tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
as the canonical work-submission API, with runtime-selected execution. Development
now has native shared captures, typed/completion-only overloads and task unwrapping.
The [original Task.Run compiler integration failures](experiments/task-run/compiler-gaps/README.md)
are corrected: short-name lookup and block-lambda returns use independently tested
Raven fixes integrated into both branches. Direct completion-only await
now passes with the target compiler unit-result fix. Mutable-local
sharing in ordinary async methods is corrected by the independently tested Raven
closure fix. Generic-method capture metadata is repaired in Raven.
[Generic async application import](experiments/task-run/README.md#generic-async-application-import)
now handles the constructed state machine, shared closure and generic holder, with
forced suspension, identity and cancellation checks. Closed ordinary static helpers
and their generated generic types follow normal Raven metadata.
Async methods inside generic classes have a separately reproduced Raven arity bug.
Thread's future public role is open.
M2–M6 remain candidate applications; no complete File Catalog
or additional HTTP feature is required by the comparer slice.

The author next selected [constructor discovery and Reflection execution](reflection-members.md):
argument-based activation, invocation and field access, with Result failures and a
separate website feature page. This is bounded library/runtime work, not selection of
a new application milestone.

The author selected a bounded `System.Runtime.GC` API after Reflection: execution-local
object counters, explicit full collection and lifetime use. See the
[GC contract](runtime-gc.md); generation/byte accounting and collector tuning remain
outside this one-off author-directed slice.

The author selected [value tuple support](tuples.md) on 2026-09-28, using
`System.Tuple` as the neoCLR name corresponding to .NET `System.ValueTuple`.
This is a bounded author-directed addition; it does not replace the Web API direction.
The [runtime tracker](tracking/runtime-language.md#value-tuples--2026-09-28) owns status.

## Theme trackers

Each work item has one status owner. Use that tracker for its status, remaining
scope, next evidence and linked design. Cross-theme dependencies link to their
owner; they do not create a second checklist.

| Theme | Status owner | Scope |
| --- | --- | --- |
| HTTP and networking | [Client/server capability tracker](http-capabilities.md) | HTTP POC finish line, client/server matrix, transport limits and future protocol work |
| Runtime and language | [Runtime/language tracker](tracking/runtime-language.md) | Values, references, GC, async execution and deferred type-system/runtime experiments |
| Library and data | [Library/data tracker](tracking/library-data.md) | Text, collections, files/streams, time, introspection, JSON and general resource contracts |
| Toolchain and delivery | [Toolchain/release tracker](tracking/toolchain-release.md) | Raven integration, diagnostic debt, packaging, CI, API documentation and publication gates |

These are maintenance ownership boundaries, not an instruction to develop four
features in parallel. HTTP owns the application package acceptance decision;
tooling owns packaging procedure and general release checks.

## Milestones

The author now selects the Web API increment after M1. M2–M6 remain candidate
products; their earlier ordering is not approved API scope.

| Milestone | Sample product | Status / scope owner |
| --- | --- | --- |
| M1 — Communicate | Hello Service + Hello Client, now the typed HTTP/JSON exchange | Released in Preview 10; done for now |
| Web API increment | Nested/collection JSON + typed route-parser case | Active direction; [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api); JSON and direct routing slices implemented; generated union mapper experiment validated |
| M2 — Work with data | File Catalog | Candidate; library/data tracker |
| M3 — Handle waiting and failure | Download Queue | Candidate; HTTP, library and runtime contracts must be selected together |
| M4 — Human time and presentation | Activity Report | Candidate; library/data tracker |
| M5 — Explain programs | Assembly Explorer | Candidate; library/data tracker, with tooling integration |
| M6 — Across hosts | Portable Sample Pack | Candidate; toolchain/release tracker |

The detailed earlier product sketches and comparisons are preserved in the
[dated roadmap](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance).
An implemented supporting API does not by itself complete a milestone.

## Working rules

- Select a bounded task from the active milestone and keep its tracker and evidence
  current. A one-off author request does not silently reprioritize all future work.
- Prioritize features over optional optimization. Investigate runtime cost when it
  blocks a selected feature, materially impairs supported use, or fails a release
  criterion. Preserve defects and measurements without making indefinite profiling
  a prerequisite for unrelated work.
- At most one small companion task may accompany a larger feature. It must finish
  independently without introducing a new language, ABI, culture or lifetime contract.
- Run only validation needed for the change, including performance tests when relevant.
  Avoid routine website builds for unrelated work; keep content and API snapshots
  current. Run the full suite only when the change or uncertainty requires it. This reflects the author’s 2026-09-27 validation correction.
- Reuse the relevant .NET/CLR comparisons in linked design notes; deepen research
  when a contract changes. Follow [design research](design-research.md). Proposals
  are exploratory inputs, not specifications or implementation commitments.
- General Raven fixes must be independently validated on a main-based feature
  branch before neoCLR integration. Keep target policies isolated; follow AGENTS.md.
- Distinguish implemented source, tested development artifacts, packaged evidence
  and released behavior. Public APIs require matching reference documentation.
  Runtime release and website publication remain separate operations.

## Maintaining tracking

Put current status and next actions in the owning theme tracker. Put contracts,
alternatives and primary-source comparisons in design notes, commands/results in
experiment records, implemented changes in the changelog, and significant direction
exchanges in the development timeline. Do not append a new “current priority” section
to an older roadmap. Update the existing status row and preserve prior decisions as
history when they matter. Add a new theme only when existing ownership cannot fit it.

The 2026-09-26 issue inventory has been assigned across the theme trackers. It is a
dated inventory, not a fresh GitHub status check. Original issue details and all nine
Raven issue assessments remain in the [archived triage](history/planning-20260927/issue-fix-roadmap.md).
No issues were closed or implementations revalidated by this documentation change.

## Historical entry points

Older links are retained below and lead to dated records. Their former “current” or
“next” headings do not override the current work section above.

<a id="current-checkpoint--finish-this-http-poc-2026-09-26"></a>

- [Current checkpoint — finish this HTTP POC (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-checkpoint--finish-this-http-poc-2026-09-26)

<a id="current-priority--next-release-features-2026-09-26"></a>

- [Current priority — next-release features (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-priority--next-release-features-2026-09-26)

<a id="implemented-checkpoint--http-json-clients-2026-09-26"></a>

- [Implemented checkpoint — HTTP JSON clients (2026-09-26)](history/planning-20260927/platform-roadmap.md#implemented-checkpoint--http-json-clients-2026-09-26)

<a id="issue-driven-priorities--2026-09-26"></a>

- [Issue-driven priorities — 2026-09-26](history/planning-20260927/platform-roadmap.md#issue-driven-priorities--2026-09-26)

<a id="active-direction--sockets-for-a-web-application-2026-09-24"></a>

- [Active direction — sockets for a web application, 2026-09-24](history/planning-20260927/platform-roadmap.md#active-direction--sockets-for-a-web-application-2026-09-24)

<a id="http-client-pipeline-checkpoint--2026-09-24"></a>

- [HTTP client pipeline checkpoint — 2026-09-24](history/planning-20260927/platform-roadmap.md#http-client-pipeline-checkpoint--2026-09-24)

<a id="networking-and-web-release-target--discussion-2026-09-25"></a>

- [Networking and web release target — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#networking-and-web-release-target--discussion-2026-09-25)

<a id="scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24"></a>

- [Scheduling checkpoint before the public socket bridge — 2026-09-24](history/planning-20260927/platform-roadmap.md#scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24)

<a id="author-directed-interface-naming--2026-09-25"></a>

- [Author-directed interface naming — 2026-09-25](history/planning-20260927/platform-roadmap.md#author-directed-interface-naming--2026-09-25)

<a id="authority-and-use"></a>

- [Authority and use](history/planning-20260927/platform-roadmap.md#authority-and-use)

<a id="how-the-plans-fit-together"></a>

- [How the plans fit together](history/planning-20260927/platform-roadmap.md#how-the-plans-fit-together)

<a id="milestones-at-a-glance"></a>

- [Milestones at a glance](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance)

<a id="release-checkpoint--async-and-tasks"></a>

- [Release checkpoint — Async and Tasks](history/planning-20260927/platform-roadmap.md#release-checkpoint--async-and-tasks)

<a id="next-release-validation-efficiency"></a>

- [Next-release validation efficiency](history/planning-20260927/platform-roadmap.md#next-release-validation-efficiency)

<a id="post-release-concurrency-direction--2026-09-23"></a>

- [Post-release concurrency direction — 2026-09-23](history/planning-20260927/platform-roadmap.md#post-release-concurrency-direction--2026-09-23)

<a id="future-networking-and-web-namespaces--consideration-2026-09-23"></a>

- [Future networking and web namespaces — consideration, 2026-09-23](history/planning-20260927/platform-roadmap.md#future-networking-and-web-namespaces--consideration-2026-09-23)

<a id="minimal-http-application-dependencies--consideration-2026-09-24"></a>

- [Minimal HTTP application dependencies — consideration, 2026-09-24](history/planning-20260927/platform-roadmap.md#minimal-http-application-dependencies--consideration-2026-09-24)

<a id="progressive-delivery-before-networking--revised-2026-09-23"></a>

- [Progressive delivery before networking — revised 2026-09-23](history/planning-20260927/platform-roadmap.md#progressive-delivery-before-networking--revised-2026-09-23)

<a id="m1--communicate-hello-service-and-hello-client"></a>

- [M1 — Communicate: Hello Service and Hello Client](history/planning-20260927/platform-roadmap.md#m1--communicate-hello-service-and-hello-client)

<a id="m2--work-with-data-file-catalog"></a>

- [M2 — Work with data: File Catalog](history/planning-20260927/platform-roadmap.md#m2--work-with-data-file-catalog)

<a id="m3--handle-real-waiting-and-failure-download-queue"></a>

- [M3 — Handle real waiting and failure: Download Queue](history/planning-20260927/platform-roadmap.md#m3--handle-real-waiting-and-failure-download-queue)

<a id="m4--human-time-and-presentation-activity-report"></a>

- [M4 — Human time and presentation: Activity Report](history/planning-20260927/platform-roadmap.md#m4--human-time-and-presentation-activity-report)

<a id="m5--explain-programs-assembly-explorer"></a>

- [M5 — Explain programs: Assembly Explorer](history/planning-20260927/platform-roadmap.md#m5--explain-programs-assembly-explorer)

<a id="m6--across-hosts-portable-sample-pack"></a>

- [M6 — Across hosts: Portable Sample Pack](history/planning-20260927/platform-roadmap.md#m6--across-hosts-portable-sample-pack)

<a id="research-products-alongside-the-milestones"></a>

- [Research products alongside the milestones](history/planning-20260927/platform-roadmap.md#research-products-alongside-the-milestones)

<a id="active-progress--2026-09-23"></a>

- [Active progress — 2026-09-23](history/planning-20260927/platform-roadmap.md#active-progress--2026-09-23)

<a id="working-rules-and-immediate-next-step"></a>

- [Working rules and immediate next step](history/planning-20260927/platform-roadmap.md#working-rules-and-immediate-next-step)

<a id="later-exploration-generic-math-interfaces--2026-09-24"></a>

- [Later exploration: generic math interfaces — 2026-09-24](history/planning-20260927/platform-roadmap.md#later-exploration-generic-math-interfaces--2026-09-24)

<a id="http-status-names-and-pattern-contracts--discussion-2026-09-25"></a>

- [HTTP status names and pattern contracts — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#http-status-names-and-pattern-contracts--discussion-2026-09-25)

## Author-directed calendar slice — 2026-09-27

The author requested useful date rendering with Gregorian/Hebrew calendars, Swedish and Hebrew cultures, invariant presentation and system discovery. The [provisional implementation and evidence](calendar-globalization.md) complete this bounded library slice without permanently reprioritizing later milestones. Unified localization stays separate, with shared interfaces and interchangeable providers/sources still to design.

Author follow-up, 2026-09-27: the next slice is the Time API and time-zone handling. DateTime and globalization have separate feature pages. This records sequencing, not implemented zone support.

### Time and named zones — implemented development slice, 2026-09-27

The selected [Time/zone slice](time-zones.md) adds TimeOffset, ZonedDateTime, named IANA
rules and explicit Unique/Ambiguous/Skipped mapping. DateTime is a nominal
parenthesized union of LocalDateTime and ZonedDateTime following the author's
clarification. Time wraps, civil addition carries the date, and elapsed Instant
addition checks overflow. The bounded 1900–2099 named-zone range and pinned 2025b
database are provisional. Broader rules/providers, parsing and scheduling remain open.

### Feature branch organization (2026-09-30)

At the author's request, structural Function work continues on
`codex/structural-types` with this Self-integrated main as its base. The former
`feature/function-types` name is retired after validation and synchronization.
The branch organization does not enable structural types on main.

Read-only callable import checkpoint (2026-09-30): Raven emission now consumes a
metadata definition snapshot without the producer builder graph. C# and Raven native
consumers return 42; [integration scope](raven-cli-bridge.md#read-only-call-imports--2026-09-30)
still excludes native symbol loading and production target registration.

Compiler adapter checkpoint (2026-09-30): the Raven consumer now calls an optional
compiler-owned emitter API with explicit contracts and diagnostics. Native verification
still returns 42; [integration scope](raven-cli-bridge.md#compiler-owned-native-adapter-checkpoint--2026-09-30)
retains the .NET input bootstrap and leaves native loading/target composition pending.

Multi-file adapter checkpoint (2026-09-30): cross-file function calls now execute
in both source-tree orders, with correct later-file diagnostics and no partial output
on validation failure. The native provider and production composition are still pending.

Native-input checkpoint (2026-09-30): a bounded native declaration reader now feeds
a temporary reference-only PE into Raven's existing semantic provider. The original
native dependency runs with all three emitted application variants to 42. See
[scope and replacement plan](raven-cli-bridge.md#native-dependency-input-through-a-reference-projection--2026-09-30);
a native symbol provider and production registration remain pending.

Raven-to-Raven dependency checkpoint (2026-09-30): the end-to-end producer is now a
Raven library, with a separately compiled Raven application consuming its native
metadata through the temporary reference projection. Overload/helper calls execute
to 42 and dependency failure cases are checked. Native symbol loading and wider
source/visibility support remain staged follow-up work.

Transitive runtime checkpoint (2026-09-30): three Raven-compiled native assemblies
load/verify/execute as a dependency chain to 42, in either supplied module order.
The native reader exposes direct exact references; missing and wrong-revision
transitive dependencies fail runtime verification. Runtime acceptance remains required;
this checkpoint is superseded by the direct container gate below; a native compiler symbol provider remains open.

Direct runtime metadata gate (2026-09-30): the same API-produced PE/#Neo library
files now feed Raven reference binding and neoCLR runtime loading. The two-library
chain and single/multi-file applications verify/run to 42. The runtime rejects missing
or altered recognition data and unsupported required execution schemas. This initial
profile carries format-5 JSON inside section 256/schema 1; it does not yet remove text
parsing or claim faster loading. Next evaluate binary native encoding and separate
load/link/verify timings, while staging the native compiler symbol provider. See the
[implementation and limits](design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).

Author-selected first acceptance programs (2026-09-30): Hello World directly in
the entry point, then through an entry-point call to another function. Both now
compile to PE/#Neo, load/verify/run in neoCLR, print exactly one line and exit zero.
The bounded Console string-literal bridge uses an explicit compiler reference
contract; general string signatures and no-result source entry points remain staged.

Binary-loading checkpoint (2026-09-30): execution schema 2 now carries bounded CBOR
and decodes directly into the runtime module model without a JSON intermediate.
Schema 1 remains readable; the experimental Raven PE emitter selects schema 2.
Hello World/function-call and the two-library cases pass. Native indexed tables, a
compiler-native symbol provider and broader signatures remain staged work. Compare
[measured loading phases](experiments/extended-cli-metadata/binary-loading.md) before
inferring performance gains; metadata loading and execution are separate costs.

Class-library consumer direction (author clarification, 2026-09-30): compile the
neoCLR runtime class library and load its symbols into Raven. Reuse present .NET-like
metadata through an explicit temporary projection while allowing native contracts
to diverge. The author proposes translating existing JSON into neoCLR assemblies as
a bootstrap; next test a bounded real class-library slice through that translation
and Raven reference loading. General translation is not implemented by the current
static-Int32 writer. Preserve unsupported information by rejecting it, then grow
coverage from the actual library instead of encoding permanent .NET restrictions.
