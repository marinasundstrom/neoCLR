# neoCLR platform roadmap

**Updated 2026-09-27.** This is the authoritative default for work priorities,
milestone sequencing and scope. Explicit author directions take precedence.

## Current work

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
checks requested by the author and final extracted-package checks. HTTP feature
scope remains frozen. Thematic consolidation is complete.

**Next direction: useful library APIs**, selected by the author after release.
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
now passes with a bounded writer consumer and explicit final-output checks. Next
promote the factory/progress contract and integrate explicit StreamWriter finishing,
with matching API artifacts and focused regression checks. This is not yet a public
Encoder API. Broader codecs remain separate follow-ups. General
scalar/range APIs and the broader portfolio are not prerequisites. Experimental
types and proposed names are not adopted System APIs.
M2–M6 remain candidate applications; no complete File Catalog
or additional HTTP feature is required by the comparer slice.

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

Retain the application-led milestone sequence. M2–M6 remain candidate products;
reassess their ordering after M1 rather than treating the list as approved API scope.

| Milestone | Sample product | Status / scope owner |
| --- | --- | --- |
| M1 — Communicate | Hello Service + Hello Client, now the typed HTTP/JSON exchange | Released in Preview 10; done for now |
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
