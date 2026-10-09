# Website structure and feature maintenance

Direction recorded **2026-09-19**. The site describes the product's current state
and provides a place to discuss its direction. A feature page may be a short
implementation note; it need not be an exhaustive guide. Keep working behavior,
development-only behavior and proposals visibly separate.

## Editorial direction — 2026-09-23

Present neoCLR as an **experimental application platform**: application APIs, runtime,
language integration and development tools together. Keep “Experimental” visible
beside the header name; “preview” describes individual releases.
Use a technical, approachable tone. Explain the current implementation, useful
features, limits, relevant differences from .NET and the intended direction. Prefer
subject headings and concrete behavior over slogans, sales copy or unqualified
claims of improvement. Separate published releases, development experiments and
unimplemented proposals. Derive sequencing from the authoritative platform roadmap.

General information must be available on the website. Link between site pages for
background, feature explanations, roadmap summaries and setup. Reserve GitHub docs
for optional depth: exact contracts, research, implementation evidence and historical
records. Readers should not need those links to understand the project or a feature.
GitHub source, issues and pull requests remain appropriate participation destinations.

Make participation accessible to a potential community: questions, sample reports,
documentation, tests, design comparisons and code all count. Keep the design open to
criticism without inventing an established contributor community, governance process
or commitment to proposed APIs. The overview page explains background and goals;
the homepage provides current status and feature entry points.

## Current-state presentation — 2026-09-27

At the author's direction, feature pages describe current behavior, useful examples
and limits. Remove superseded experiments, development chronology and routine preview
migration notes. Keep only a breaking-change note that is necessary to use an
available artifact; preview churn does not need a page-by-page history. Brief possible
directions are welcome, clearly distinguished from implementation. Setup/download
pages retain accurate package availability. Preserve changelog, published release
notes and development history outside the feature narrative.

## Installation and language navigation — author direction, 2026-10-07

Give installation one short path: prerequisites, download, install the editor extension,
open a sample and run it. Put optional verification and troubleshooting after those steps;
keep source-build qualification and bootstrap internals in linked maintainer docs.
Keep Raven in the main navigation. Its page links directly to the Raven language website
and playground, explaining that the playground is for the language rather than neoCLR's
runtime libraries. The site's experimental identity supplies general context; repeat only
concrete limitations or availability differences that affect the reader's next action.

## Case-based samples — author direction, 2026-09-27

This is a general website convention, especially for feature pages: explain a
feature or API through a small realistic case and show its essential code directly
on the feature page. A reader should be able to recognize what the feature does
and how to use it from that example. Link to a dedicated case page for the complete
project or additional context; a link alone does not replace the on-page example.
Apply this when developing and reviewing feature pages, without inflating a simple
sample into a large application.

Present samples as coherent application cases, especially paired HTTP clients and
servers. For Web, introduce a general HttpClient sample and its capabilities first;
then show a server followed by the client that connects to it (author clarification
on the same date). Lead with the use case and exchange, then show shared models/payloads,
server behavior, client calls, expected responses and failure behavior. Include
complete downloadable projects and on-site run instructions tied to the matching
release or development toolchain. Extract displayed code from tested sources.
Individual API snippets can support the walkthrough but should not substitute for
it. Give a selected use case a dedicated `website/content/cases/<case>/index.md`
page, a descriptive “Case:” title and navigation from its feature guide and the
cases list. The author emphasizes that these can stay small: a real or realistic
purpose and understandable context matter more than size or tutorial depth. Do not
expand a sample into a large application merely to justify calling it a case.
Evolve the report case into the selected Web API story as nested JSON and
WebApplication become implemented; do not show planned endpoints as runnable code.

## Website and repository audiences — author direction, 2026-10-09

The website helps visitors understand, evaluate and use neoCLR. The homepage must
explain the product immediately: runtime, class library, Raven and tools, with a clear
path to installation and examples. Use the tone of a software project page. Avoid
manifestos, repeated methodological qualifications and claims of superiority.

Repository documentation serves contributors and implementers: exact contracts,
design alternatives, compiler/backend boundaries, validation commands, artifact
revisions and development history. Website content must not echo that documentation
in the same detail or structure. Explain the user-facing model on site and link to
GitHub for optional implementation depth. Keep public API usage contracts in the
on-site API reference; this audience split does not remove API documentation duties.

Maintain current explanations instead of appending progress reports. Remove resolved
limitations and stale checkpoints from feature narratives. Preserve their evidence in
repository records and Git history. Keep exact availability in setup/download pages,
with local exceptions only where needed to use a particular API or versioned sample.
The shared development notice must distinguish the site from published packages.

A feature guide normally needs a purpose, a tested example, important behavior,
current limitations and links to reference material. Length follows the reader's
need, not a fixed word quota. Dedicated benchmark reports keep measurements alongside
methodology and caveats; feature introductions link to them without retelling the
optimization history. Preserve useful public anchors when reorganizing pages.

## Information structure

| Location | Purpose | Required distinction |
| --- | --- | --- |
| `website/content/comparison/index.md` | Maintained neoCLR/.NET comparison with primary sources and practical tradeoffs | Runtime versus language/library policy; development versus published availability |
| `website/content/about/index.md` | Project background, goals, relationship to .NET, current scope and participation | Intent versus implemented capability |
| `website/content/index.html` | Product overview, release/download status and entry points | Published capabilities versus development-only examples |
| `website/content/features/<name>/index.md` | What currently works, a useful example where appropriate, limits and feedback | Implemented behavior versus “Where we’re heading” |
| `website/content/cases/<case>/index.md` | Small realistic use cases, tested code, run instructions and observable results | Working case versus future extensions; link to the general feature/API guide |
| `website/content/try/index.md` | Reader-facing installation, .rvnproj workflow, expected output and troubleshooting | Published bundle instructions versus development-only API availability |
| `website/content/raven/index.md` | Introduce the Raven language, examples and the .NET/neoCLR target distinction | Language capabilities versus target-specific library support |
| `api-docs/namespaces.md` → `/docs/namespaces.html` | Current namespace contents and links to reference pages and guides; extend as APIs land | Implemented namespaces versus proposals; overview versus member coverage |
| `website/content/proposals/index.md` | Brief summaries of ideas across the platform, linked to original proposals and maintained design records | Proposals are not promises or a release checklist |
| `docs/` and release notes | Detailed contracts, research, verification and historical release behavior | Preserve published release notes; record corrections under Unreleased |

Feature pages currently cover [Introspection](../../website/content/features/introspection/index.md)
and [Strings](../../website/content/features/strings/index.md), with additional notes for
[Option/Result](../../website/content/features/outcomes/index.md),
[Console](../../website/content/features/console/index.md),
[arrays](../../website/content/features/arrays/index.md),
[collections/queries](../../website/content/features/collections/index.md),
[dates/clocks](../../website/content/features/time/index.md) and
[UTF-8 files](../../website/content/features/files/index.md) and the development
[Tasks and async PoC](../../website/content/features/tasks/index.md). Introspection is an in-depth
walkthrough; the second deliberately shows one working conversion example. Scale
detail to what helps an evaluator. Do not add an API just to fill a page outline.

A short **Where we’re heading** section can explain likely extensions, open
questions and deferred work. Link the relevant proposal-overview anchor and design
record. State benefits and costs against the .NET/CLR baseline; reuse existing
research. Avoid presenting a possible direction as an approved implementation or a
commitment to the next release. Names, signatures and behavior may change.

## With each feature change

1. Review the homepage, relevant feature page and proposals overview alongside the
   implementation. Update affected descriptions, status and limits
   in the feature's documentation slice. A new page is optional; an accurate update
   to an existing page may be enough.
2. When a proposal becomes executable, update its status and link to the current
   implementation. Keep unresolved parts labeled as proposals. Preserve the original
   supplied proposal texts and explain subsequent decisions in maintained notes.
3. Source displayed code from executable samples through `scripts/build-website.py`.
   Verify changed samples with their matching toolchain and expected-output checks.
   Do not duplicate an untested version in Markdown or HTML content. Small implementation notes need not
   reproduce every test case or every API member.
4. Run the RavenDoc publisher tests, full page build and link checks documented in
   [the website README](../../website/README.md). Inspect rendered changed pages;
   check desktop and narrow layouts when changing layout or CSS.
5. Include the website status in release review. Confirm downloads, instructions,
   examples and claims match the released product. If the public site also shows
   development work, label it explicitly and keep it separate from the download.

## Publication is a separate action

Relevant pushes and pull requests build and validate the website. They **do not
publish it**. The `Project website` workflow deploys only when manually dispatched
on `main`. Its deploy job alone has Pages/OIDC write permissions. Build runs from
pushes cannot cancel an in-progress manual deployment through its concurrency group.

Before publishing, review the selected revision and its release-status text. In
GitHub Actions, open **Project website → Run workflow**, select **main**, and run it.
Check the build and deployment result and inspect the public page. A successful
local build or push is not publication; record the deployment revision when it occurs.
No deployment is triggered by this documentation change.

## How the structure evolved

The author's initial direction was in-depth feature guides for the next preview.
The Introspection walkthrough was implemented first. The author then clarified that
minimal working APIs and short current-state pages are enough, with design left open
for input. They requested a proposals overview, feature-specific future directions,
a durable record of this structure and a website review with feature/release work.
The workflow now separates continuous validation from manual publication.
See [the development conversation record](../development-timeline.md).

Homepage feature boxes link to their corresponding feature page as the primary
next step. Repository samples and technical contracts belong as supporting links
on the feature page, rather than replacing that explanation.

Keep user-facing explanations and setup instructions on the website. Repository
links are optional implementation/research references, never a required step to
understand a feature or try a published bundle. Prefer .rvnproj as the evaluator
entry point. Host complete sample downloads on the site. Update prerequisites,
package names, build/run commands and expected output with each release; do not
present local developer installations as public downloads.

When describing prerequisites, distinguish runtime execution from development tools:
neoCLR and its guest programs do not require .NET. Raven compilation, MSBuild, the
import bridge and Raven Language Server use .NET; the VS Code extension connects
to that server. Do not imply the extension itself is a .NET application.

## Natural Raven examples

Website samples should show idiomatic Raven as developers would write it, following
[the code conventions](../raven-conventions.md#lambda-signatures). Avoid redundant
type annotations and explanatory syntax when the operation and surrounding code
already make the meaning clear. Keep annotations that current compilation requires
or that clarify an otherwise unclear contract, and compile the exact displayed
example. Do not add annotations merely to teach or over-explain the language.


## RavenDoc site and landing page — 2026-09-24

The author selected one RavenDoc site, replacing the bespoke website renderer and
separate DocFX reference. Adapt content to the documentation structure rather than
mechanically carrying over the old landing page. Feature/API guides remain separate
pages alongside generated namespace, type and member reference. The documentation
entry point links both, and guides link their corresponding reference.

The homepage is an HTML body fragment with a hero on its own background and feature
boxes beneath. It has no enclosing article panel or “On this page” outline. Raven
and CloudShell's landing pages supplied layout inspiration; neoCLR uses a restrained
slate/blue palette. The author accepted the feature boxes and requested a less
intrusive development notice. Keep that notice compact but visible: documentation
may precede a runtime release. Publication never implies release availability.

RavenDoc accepts Markdown and HTML body fragments with optional scalar front matter:
`title`, `layout: docs|landing`, and `toc: true|false`. Use the shared header,
release notice and footer; the landing page is not a second website. Long guides
and generated reference retain the outline when useful. Site settings live in
`website/site.json`; presentation lives in `website/custom.css`.

The portable RavenDoc archive, immutable source revision, checksum and upstream
source revision are maintained in `tools/ravendoc`. Follow its README to
update the build, then verify the whole site before publication. CI validates the
same repository-contained build without a sibling Raven checkout or DocFX restore.


Author direction, 2026-09-24: Networking and Web each deserve their own feature
page and homepage box once the corresponding POC is working. Networking starts
with the hostname/TCP echo client; Web remains gated on an executable HTTP POC.
Keep protocol-level Web information separate from transport and hostname contracts.

## Maintaining the .NET comparison

Review `website/content/comparison/index.md` when a change affects a compared
contract, compatibility boundary, implementation limit or release availability.
Update the relevant row and review date after checking evidence. Link to the
feature page that owns details; avoid duplicating a full capability inventory.
Keep primary .NET references, costs and benefits, and development labels current.
About and Guides link to this canonical comparison instead of maintaining copies.

## Guides navigation (2026-10-09)

`website/guides-toc.yml` owns the Guides article hierarchy; `website/toc.yml`
contains only API overview/reference navigation. The build stages guide sources
under an explicit RavenDoc navigation section while preserving their published
`/features/`, `/docs/` and other article URLs. This keeps saved links stable and
lets Guides use the same visible article sidebar across folders. The API browser
continues to list library symbols separately. A page's `toc: false` disables only
its in-page outline, not the Guides sidebar.
