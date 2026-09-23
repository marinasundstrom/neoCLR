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

## Information structure

| Location | Purpose | Required distinction |
| --- | --- | --- |
| `website/about/index.html` | Project background, goals, relationship to .NET, current scope and participation | Intent versus implemented capability |
| `website/index.html` | Product overview, release/download status and entry points | Published capabilities versus development-only examples |
| `website/features/<name>/index.html` | What currently works, a useful example where appropriate, limits and feedback | Implemented behavior versus “Where we’re heading” |
| `website/try/index.html` | Reader-facing installation, .rvnproj workflow, expected output and troubleshooting | Published bundle instructions versus development-only API availability |
| `website/raven/index.html` | Introduce the Raven language, examples and the .NET/neoCLR target distinction | Language capabilities versus target-specific library support |
| `website/proposals/index.html` | Brief summaries of ideas across the platform, linked to original proposals and maintained design records | Proposals are not promises or a release checklist |
| `docs/` and release notes | Detailed contracts, research, verification and historical release behavior | Preserve published release notes; record corrections under Unreleased |

Feature pages currently cover [Introspection](../../website/features/introspection/index.html)
and [Strings](../../website/features/strings/index.html), with additional notes for
[Option/Result](../../website/features/outcomes/index.html),
[arrays](../../website/features/arrays/index.html),
[collections/queries](../../website/features/collections/index.html),
[dates/clocks](../../website/features/time/index.html) and
[UTF-8 files](../../website/features/files/index.html) and the development
[Tasks and async PoC](../../website/features/tasks/index.html). Introspection is an in-depth
walkthrough; the second deliberately shows one working conversion example. Scale
detail to what helps an evaluator. Do not add an API just to fill a page outline.

A short **Where we’re heading** section can explain likely extensions, open
questions and deferred work. Link the relevant proposal-overview anchor and design
record. State benefits and costs against the .NET/CLR baseline; reuse existing
research. Avoid presenting a possible direction as an approved implementation or a
commitment to the next release. Names, signatures and behavior may change.

## With each feature change

1. Review the homepage, relevant feature page and proposals overview alongside the
   implementation. Update affected descriptions, status, limits and migration notes
   in the feature's documentation slice. A new page is optional; an accurate update
   to an existing page may be enough.
2. When a proposal becomes executable, update its status and link to the current
   implementation. Keep unresolved parts labeled as proposals. Preserve the original
   supplied proposal texts and explain subsequent decisions in maintained notes.
3. Source displayed code from executable samples through `scripts/build-website.py`.
   Verify changed samples with their matching toolchain and expected-output checks.
   Do not duplicate an untested version in HTML. Small implementation notes need not
   reproduce every test case or every API member.
4. Run the website tokenizer tests, page build and link tests documented in
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
