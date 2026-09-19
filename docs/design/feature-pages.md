# Feature pages for the next preview

Planning direction recorded **2026-09-19**. The author proposes dedicated pages
that explain individual features or APIs in depth, with more substantial samples.
The first Introspection guide is now implemented locally at
`website/features/introspection/index.html`; it has not been published by this slice.
No release number or date is assigned.

## Purpose

Help evaluators understand a feature's purpose, run it, see its limits and give
specific feedback. Release notes remain a concise account of changes; API reference
remains the contract lookup. Feature pages provide the worked explanation joining
the two, with links to current reference and design documents.

## Page outline

1. **Problem and current status.** Describe a concrete use case, supported target and
   whether the feature is shipped, in development or proposed.
2. **Runnable walkthrough.** Start small, then develop one coherent example. Include
   prerequisites, the run command or VS Code task, expected output and explanation.
3. **API and behavior.** Explain relevant types, ownership, errors, collection
   capabilities and other observable contracts through that example.
4. **Design choices.** Reuse the feature's primary-source .NET/CLR comparison and
   explain deliberate differences, benefits and costs. Link deeper research.
5. **Preview limits.** Separate working behavior from unsupported cases and future
   plans. Include migration notes where evaluators may have tried an older snapshot.
6. **Feedback prompts.** Ask concrete questions about naming, ergonomics and missing
   use cases; do not present provisional choices as final commitments.

Use tested Raven samples as the source of code excerpts. Keep an executable sample
and expected-output check for the whole walkthrough, and verify page snippets stay
in sync when APIs change. Label development-only samples separately from a published
release's supported API; preserve published release notes.

## First candidates

| Page | Walkthrough | Basis and limits |
| --- | --- | --- |
| Introspection and RuntimeContext | Acquire TypeInfo from typeof and Object.GetType; traverse ExecutingAssembly, direct references, modules and types; inspect members with Sequence and match the sealed member hierarchy; pair MetadataToken with Module | [Current contract](../introspection-design.md), [discovery sample](../experiments/raven-target/samples/library-assembly-info.rvn), [member sample](../experiments/raven-target/samples/library-reflection.rvn). Discovery is retained loaded metadata; dynamic loading, invocation and emit remain future work. |
| Strings and runtime text behavior | A byte/text boundary example that explains the selected units and error behavior | Follow the later String story. [Existing text model](../text-model.md) and [String APIs](../raven-string-api.md) are the starting point; do not invent the next API, Encoding hierarchy or Utf8String. |

Introspection is the assistant's suggested first page, not an additional API feature
or authorization to expand reflection. Other page candidates should follow actual
preview capabilities and evaluator needs.

## Delivery slice

When website work begins, choose the site's navigation and feature URL structure,
implement the Introspection page and sample extraction/checks, then verify rendered
code, links, narrow-screen reading and copy/run instructions. Keep that website slice
separate from the completed Introspection API commits. Publishing remains a distinct
release action; this plan does not publish the site.


## First implemented page — 2026-09-19

The Introspection guide follows one saved Raven program through acquisition,
assembly/module discovery, metadata tokens, Sequence results and sealed member
matching. All excerpts and downloadable source derive from that program; the saved
project gate checks the expected-output file rendered on the page. The homepage now
links to the guide and replaces obsolete class/Type.Info migration descriptions.

The static builder supports nested feature pages and validates cross-page links,
anchors and sample downloads. Desktop and narrow layouts are checked locally.
String pages remain future work; no dynamic loading or API expansion is included.
