# Repository workflow

## Work priorities

- Treat [the platform roadmap](docs/platform-roadmap.md) as the authoritative default
  for work priorities, milestone sequencing and scope unless the author explicitly
  directs work otherwise. Explicit author directions take precedence.
- Detailed milestone plans, API plans, backlogs and older direction notes support
  that roadmap; they do not independently override its priorities. Proposals remain
  exploratory inputs, not specifications or implementation commitments.
- Select the next bounded task from the active milestone. Keep roadmap status and
  linked evidence current as work completes. Record significant author-directed
  changes in direction without inferring a permanent reprioritization from a one-off task.

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
- Document compiler-affecting integration changes in both Raven's compiler docs and
  neoCLR's integration docs, with changelog updates in both repositories. Include
  Runtime Contract configuration, semantic/emission effects, limitations and validation.
- Review mixed changes by behavior and dependencies. A general metadata/emission fix
  is not permanently experimental just because neoCLR exposed it. Record deferred
  general candidates explicitly until they can be validated independently.

## Raven code conventions

- Follow [idiomatic Raven conventions](docs/raven-conventions.md), grounded in
  Raven’s own style and feature guides. Apply them to all hand-authored Raven code,
  including the runtime library, applications, tooling, tests, experiments, samples,
  documentation and website examples. Prefer inferred local and callback types;
  retain annotations when required by the compiler or an otherwise unclear contract.
  Use `let` for immutable lexical bindings
  and `val` for read-only properties. Compile target examples instead of assuming
  every host Raven/.NET API or syntax spelling is supported.

- Prefer readable code over compact formatting. Expand block expressions and
  statements across lines when that makes their contents easier to follow; there is
  no requirement to fit a complete block on one line.

- Use standard Raven `union` declarations for class-library unions by default,
  including unions with members. Manual carrier implementations are rare exceptions;
  document the concrete constraint and when to revisit it.
- Prefer union patterns or destructuring to extract case payloads, rather than
  case accessors such as GetSomeCase(). Use idiomatic case construction, such as
  Option<TypeInfo>.Some(owner) or Some(owner) when the target is known; avoid
  explicit carrier wrappers in ordinary API implementations and samples.
- Prefer expression-bodied properties (`val Name: string => expression`) when
  a property has only a getter expression.

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

## Website maintenance

- Review website content with each feature change and before release. Follow
  [website structure and maintenance](docs/design/feature-pages.md): homepage for
  the product overview, feature pages for current behavior and short future
  directions, and a separate proposals overview for open ideas.
- Keep pages as small as useful; examples should come from tested samples. Label
  development work and proposals distinctly from published product capabilities.
- Website validation runs on relevant pushes/PRs. Publication is a separate manual
  workflow on main; do not treat a code push or successful local build as deployment.

## API reference documentation

- Public APIs must be covered by the on-site API reference at `/docs/`. Update
  documentation in the same change that adds, changes, renames or removes an API;
  a feature overview alone does not replace type and member documentation.
- Keep signatures, useful XML summaries, parameters/results, errors, limitations
  and examples aligned with the implemented contract. Distinguish development APIs
  from published releases and proposals. Never document a planned API as shipped.
- Keep namespace/type navigation and the API landing page browsable. New public
  APIs must be included in the RavenDoc type selection, with the reference assembly and
  documentation snapshot refreshed from the matching compiler bridge.
- If RavenDoc cannot render a signature, provide a linked manual reference entry and
  record the exact exclusion and reason. Do not silently omit public APIs. Track
  existing coverage gaps explicitly in [API documentation maintenance](api-docs/README.md)
  and close them as those areas are developed.
- Validate the API snapshot and build the combined website before committing API
  changes. Follow [the maintenance procedure](api-docs/README.md); publication remains
  a separate manual operation.
