# Repository workflow

## Raven integration work

- Keep work in the Raven repository isolated on a feature branch. Verify its branch
  before editing; do not make experiment changes on Raven's `main` branch.
- General fixes benefiting Raven, including .NET Framework and NanoFramework targets,
  belong on Raven `main` even when discovered through neoCLR: extract
  and test them independently, then integrate them. Keep neoCLR-specific policies
  and target experiments on their feature branch; never merge that branch wholesale.
- Do not integrate neoCLR-specific code, configuration or tests into Raven main yet.
  General fixes must stand independently on CLI metadata contracts. Reconsidering
  emission architecture or another backend is future evaluation, not this stabilization scope.
- Review mixed changes by behavior and dependencies. A general metadata/emission fix
  is not permanently experimental just because neoCLR exposed it. Record deferred
  general candidates explicitly until they can be validated independently.

## Sample readability

- Prefer readable samples over compact formatting. Expand block expressions and
  statements across lines when that makes their contents easier to follow; there is
  no requirement to fit a complete block on one line.

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
