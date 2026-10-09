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

- Develop shared Raven compiler fixes and target contracts on the shared line.
  Native Self integration is authorized. Keep structural Function experiments on
  feature branches in both repositories until native metadata/compiler support is ready.
- Keep target-specific mappings and capability checks behind explicit contracts;
  ordinary .NET behavior remains the default. Document temporary bridge encoding,
  its limitations, validation and eventual native metadata/backend replacement.
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

- Use `await` by default for asynchronous work, including async `Main`. Reserve
  explicit `OnCompleted` for a documented adapter, outcome-observation or callback-test purpose.

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
- Prefer private `var`/`val` for ordinary storage; Raven emits these as fields.
  Reserve explicit `field` for intentional field declarations (public or private) or compatibility.
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

- Treat .NET as the ergonomic target, not an exact API or implementation template.
  Learn from other frameworks and account for neoCLR’s UTF-8 platform contracts;
  justify claimed improvements with concrete benefits and costs.
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

## Focused validation

- Run only the checks necessary to establish that the changed behavior works.
  Prefer focused contract tests and a relevant executable consumer; reuse unaffected
  evidence instead of repeating broad suites or platform matrices. Run a full suite
  when the change’s impact or unresolved uncertainty makes it necessary.
- Include performance tests when performance is part of the change, a suspected
  regression materially affects supported use, or a release criterion requires it.
  Do not add benchmarks to routine API work without a relevant performance question.
- Do not routinely build the website for unrelated work. Run a website build only
  when needed to verify the change or explicitly requested; honor task-specific skip
  directions. This author direction, clarified 2026-09-27, supersedes earlier routine
  build requirements. Keep website/API content and necessary snapshot checks current.

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
- Validate the API snapshot before committing API changes; local website builds
  follow the focused-validation rule above. Follow the
  [maintenance procedure](api-docs/README.md); publication remains a separate manual operation.

## Temporary Raven bridge behavior

- Document neoCLR CLI bridge changes with native semantic intent, temporary CLI
  representation, lost/restricted information, owners, validation and the native
  metadata/codegen replacement. See `docs/raven-cli-bridge.md` and Raven's
  `docs/compiler/neoclr-cli-bridge.md`. Bridge limits are not permanent platform rules.
- Identify runtime feature branches and tested bundle revisions explicitly. Raven
  compiler support on main does not imply the corresponding neoCLR feature is on main.
