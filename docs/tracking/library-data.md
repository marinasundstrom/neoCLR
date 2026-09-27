# Library and data tracking

**Consolidated 2026-09-27.** [Platform priorities](../platform-roadmap.md) govern work.
This page owns general library scope. After Preview 10, the author selects filling
useful API gaps, including earlier requests. HTTP POC scope remains closed for now.

## Active direction — useful API gaps

Selected 2026-09-27. Earlier author requests include consistent System.Text APIs,
general/string comparers, ToUpper/ToLower-style operations, String methods,
StringBuilder and better time APIs. See the [recorded text requests](../development-timeline.md#2026-09-24--future-systemtext-and-comparer-infrastructure)
and [application discussion](../development-timeline.md#2026-09-24--future-minimal-http-application-namespace-map).
The author subsequently requested implementing comparers. That first slice is now
implemented in development. Later ordering remains an assistant recommendation;
broader culture/time contracts remain design decisions.

| Order | Bounded work | What it enables and completion boundary |
| --- | --- | --- |
| 1 | Comparer infrastructure, beginning with a paired equality/hash policy and an explicit ordinal string comparer used by HashMap | Callers can reuse one coherent policy instead of supplying unrelated callbacks. Demonstrate string-key lookup, duplicates, collisions and equal-value/hash agreement. Keep ordering distinct from equality; select general ordering and primitive defaults only with corresponding consumers. |
| 2 | Useful text construction and operations, including a minimal StringBuilder | Build a text report using the existing collection/stream APIs. Select append/materialization and needed String operations; preserve grapheme, scalar and byte distinctions. Do not make storage optimization or a complete namespace redesign prerequisites. |
| 3 | Explicit casing and comparison policies | Cover the requested ToUpper/ToLower-style behavior with Unicode examples. Specify expanding mappings, normalization and culture policy before advertising case-insensitive equality or hashing. HTTP ASCII casing is not a general text implementation. |
| 4 | Parsing and presentation gaps, then a bounded time/reporting consumer | Add a needed scalar parser or formatter with typed failures; build on existing Date/Time/Instant/Duration and clocks for a concrete report. Time zones and full globalization remain separately scoped. |

**Comparer slice complete in development (2026-09-27):** EqualityComparer<T>,
Comparer<T>, both Delegate* adapters and StringComparer.Ordinal are implemented in
Raven. HashMap accepts an equality policy and retains callback construction. The
[Map design](../map-contracts.md#comparer-policies-development) records signatures,
.NET comparisons, null domains and native UTF-8 ordering. No universal Default,
culture/case-insensitive policy or sorting API is claimed. Next candidate: the
bounded text-construction/report slice above.

[Validation evidence](../experiments/raven-target/comparer-validation.json): six
focused source scenarios, six metadata admission checks and five existing map/GC
runtime regressions pass. Editor discovery, matching API snapshot and the website
build/18 website tests passed before the author asked to skip website builds.
The final library regeneration matches the tested profile. Runtime execution used
the extracted Preview 10 binary with the new library and bridge; no new package or
website was published. The broad metadata probe has separate stale union assertions
tracked with [tooling](toolchain-release.md#integration-and-correctness).

Reuse the Map design’s .NET Dictionary/EqualityComparer comparison and the
[ordinal text comparison](../ordinal-text.md#net-baseline-alternatives-and-decision).
.NET provides reusable comparer policies; neoCLR should offer that convenience
while retaining its selected UTF-8/scalar ordinal ordering rather than silently
adopting UTF-16 ordering. Pairing equality and hashing reduces accidental policy
mismatches but cannot prove a custom comparer obeys the contract. Null-key behavior,
callback adaptation and compatibility with the existing constructor must be explicit.
Use ordinary Raven library interfaces/classes where possible; add runtime services
only for behavior the existing library cannot express.

**Small companion work:** at most one independently useful scalar constant, parser
or enum-flag projection correction may accompany a larger slice. Verify the gap
first against current Raven source and generated references; older issue titles
are not evidence it is still missing. Skip companions that introduce new culture,
ABI or lifetime decisions. Collection removal/enumeration, richer JSON mapping,
reflection execution and general resource cleanup remain consumer-driven follow-ups.

**Acceptance per slice:** run only tests necessary for the changed contract and a
relevant executable Raven consumer; maintain matching reference/XML/snapshot coverage.
Check editor discovery or packaged execution when the projection makes those checks
necessary. Avoid routine website builds for unrelated work. Include performance tests when a
concrete performance question requires them. Reuse unaffected evidence; run a full
suite or platform matrix only when the change or unresolved uncertainty requires it,
per the author’s 2026-09-27 clarification.

## Recorded checkpoints and remaining scope

| Theme | Recorded implementation/evidence | Next boundary when selected |
| --- | --- | --- |
| Text and encoding | [String sequence](../experiments/string-sequence/README.md), [ordinal text](../ordinal-text.md), [UTF-8 chunks](../experiments/utf8-chunks/README.md) | Coherent System.Text, casing/comparison policy and equality/hash consistency; do not confuse HTTP ASCII names, Unicode folding and culture-sensitive collation. |
| Collections | [Collection contracts](../collection-contracts.md), [generic arrays](../generic-managed-arrays.md), [filtering](../arraylist-filtering.md) | Extend for an executable consumer; preserve mutability/capability and GC boundaries rather than reopening a complete interface sweep. |
| Storage, streams and console | [Storage provider](../experiments/storage-provider/README.md), [file streams](../experiments/file-streams/README.md), [console streams](../experiments/console-streams/README.md) | Synchronous sources are useful but are not general async streams. Select async ownership, cancellation and error composition before new adapters. HTTP content ownership is tracked with HTTP. |
| JSON and data mapping | [JSON DOM](../experiments/json-dom/README.md), [flat mapping](../experiments/json-object-mapping/README.md) | Richer model shapes/options remain bounded future slices; existing size/shape limits are not erased by HTTP integration. |
| Introspection/reflection | [Descriptors](../introspection-design.md), [execution fixture](../experiments/reflection-execution/README.md) | Metadata discovery and execution have separate contracts. Do not infer general invocation from parameterless construction/property access. |
| Time and presentation | [Date/time design](../date-time-design.md), [globalization proposal](../globalization-design.md) | Existing injectable clocks provide a foundation; time zones and cultural formatting remain separate selected scenarios, not an immediate broad framework. |

## Supporting issue inventory

This preserves the 2026-09-26 issue inventory, not current GitHub status. The
[original library analysis](../history/planning-20260927/issue-fix-roadmap.md#then-ownership-and-useful-library-gaps)
retains detailed rationale and .NET comparison links.

| Issue | Known boundary and bounded next action |
| --- | --- |
| [#3 comparers](https://github.com/marinasundstrom/neoCLR/issues/3), [#8 comparisons](https://github.com/marinasundstrom/neoCLR/issues/8), [#11 Text API](https://github.com/marinasundstrom/neoCLR/issues/11) | Development comparer slice now covers reusable equality/hash and ordering policies, ordinal strings and HashMap integration. Default selection, case/culture policies and broader System.Text remain separate gaps. |
| [#4 constants](https://github.com/marinasundstrom/neoCLR/issues/4) | Verify missing scalar bounds/Boolean strings against the Raven surface; do not assign UTF-16 bounds to grapheme Char. Suitable companion only when selected. |
| [#5 parsing](https://github.com/marinasundstrom/neoCLR/issues/5), [#10 globalization](https://github.com/marinasundstrom/neoCLR/issues/10) | Int32.Parse already has a typed Result/ASCII contract. Add one needed type; generic parsing/error and culture policy are separate decisions. |
| [#6 enum flags](https://github.com/marinasundstrom/neoCLR/issues/6) | Reconcile historical HasFlag support with the current public projection; test zero/combinations/unnamed bits before closing a gap. No enum-model redesign. |
| [#7 reflection](https://github.com/marinasundstrom/neoCLR/issues/7), [#14 introspection](https://github.com/marinasundstrom/neoCLR/issues/14) | Constructor metadata, argument-binding construction, field access and general method invocation are distinct additions. Select a consumer and test visibility, receiver/value validation, unsupported-provider results and GC. |
| [#13 ConvertibleInto](https://github.com/marinasundstrom/neoCLR/issues/13) | Instance Convert exists. Static Self conversion, ConvertibleFrom and return-directed inference remain different proposals. |
| [#16 resource cleanup](https://github.com/marinasundstrom/neoCLR/issues/16) | General Disposable/Closable and Raven use lowering across return, propagation and await remain broader than explicit HTTP cleanup. Validate a minimal target consumer before an async-disposal family. |

HTTP-specific [#18](../http-capabilities.md#issue-and-dependency-ownership) belongs to
the HTTP tracker. Runtime metadata/GC changes belong to [runtime/language](runtime-language.md).
Public API additions require same-change reference coverage and matching artifacts;
[toolchain/release](toolchain-release.md) owns those procedures, not this task list.

## Candidate products after the POC

File Catalog (M2), Activity Report (M4) and Assembly Explorer (M5) remain candidate
products from the [platform roadmap](../platform-roadmap.md#milestones). Each should
select only the missing library behavior needed by its runnable case. The earlier
[API catalog](../history/planning-20260927/runtime-api-plan.md) and
[product sketches](../history/planning-20260927/platform-roadmap.md#milestones-at-a-glance)
are design inputs, not a second priority list. Reuse their .NET/CLR comparisons and
update the relevant design when a contract changes. Consolidation revalidates no APIs.
