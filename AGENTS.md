# Repository workflow

## Changelog required for every commit

- Include a staged `CHANGELOG.md` update in every commit, including code, tests,
  documentation, tooling and maintenance commits.
- Make development updates under `Unreleased`, grouped by the current date.
  Revise or extend a related entry from the same date instead of duplicating it.
  Add an entry for a different subject or a later date.
- Keep published changelog sections and published release notes unchanged.
  Record any correction under `Unreleased`, referring to the affected release.
- Describe implemented behavior accurately; label plans as plans and include
  material compatibility or migration notes. Do not invent a release version/date.
- Before committing, review the staged changelog alongside the changes and run
  `git diff --cached --check`.

See [the changelog workflow](docs/changelog.md) for consolidation and release handling.

## Research-backed platform design

- Frame every roadmap capability and substantive improvement to existing features
  as a comparison with .NET/CLR APIs, behavior and implementation layers.
- Follow [design research](docs/design-research.md): use primary sources, distinguish
  shipped behavior from proposals, compare alternatives and record tradeoffs.
- Do not call a divergence an improvement without explaining its benefit and costs.
  Reuse existing research for routine fixes; deepen it when contracts or assumptions change.
- Record provisional choices and validation needs. This process is not an additional
  approval requirement and does not override authorized work.

## Record significant development conversations

- Maintain [the development conversation record](docs/development-timeline.md) for
  significant exchanges about project direction, questions, proposals and decisions.
  Its purpose is to show how the author works with AI and approaches software
  development through neoCLR, preserving concrete exchanges in a minutes-like record.
- Record what the user asked or directed, what the assistant proposed in response,
  subsequent user decisions/corrections, actions taken, outcomes and what remains open.
  Attribute each part; distinguish proposals from performed actions and link available
  evidence. Mark reported or unknown outcomes rather than inventing completion.
- Record the entry date; do not invent dates or missing replies. Label retrospective
  author-side notes when corresponding assistant messages are unavailable.
- Preserve prior positions when thinking changes. Quote only available original words;
  distinguish quotations, summaries and assistant-reported implementation outcomes.
- Keep routine changes in the changelog. A routine "continue" need not receive its own
  entry; do not infer approval of every implementation choice from continuation or silence.
