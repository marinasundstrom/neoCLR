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
broader culture/time contracts remain design decisions. The subsequent explicit
comparison slice is implemented in development; the author then directs a stop for
the string design review below before further string API expansion.

Earlier proposed sequence; the review below now precedes further text implementation.

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
culture policy or sorting API is claimed. The subsequent String comparison slice
adds a documented simple-fold policy; further text construction awaits review.

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

## String design review before further expansion

**Review completed (2026-09-27); implementation pause retained.** The author directed
finishing explicit comparison modes and then reassessing strings as a whole. .NET is the ergonomic target;
exact API spelling, behavior and implementation are not mandatory. Learn from other
frameworks and evaluate benefits and costs specific to valid UTF-8 text.

The completed development slice supplies `StringComparison.Ordinal` and
`OrdinalIgnoreCase`, `String.Compare(left, right, comparison)`,
`CompareOrdinalIgnoreCase` and `StringComparer.OrdinalIgnoreCase`. Existing ordinal
operations remain unchanged. [Contract and tradeoffs](../ordinal-text.md#explicit-comparison-modes-development)
cover Unicode 17 simple folding, shared equality/hash rules, .NET differences and
focused validation. [Recorded evidence](../experiments/raven-target/string-comparison-validation.json):
two native folding tests, three Raven scenarios, eleven metadata checks and one
archived-profile ordinal regression pass, with matching library/API snapshots.
Website and full-suite runs are skipped. These additions do not settle the long-term
string design.

Review the current String, Char, scalar/byte views, length/index/slicing, comparison,
search, equality/hash, normalization, casing, text construction and encoding boundaries
as a coherent API. Compare real tasks in .NET and relevant UTF-8/grapheme-oriented
frameworks (for example Rust, Go and Swift), using their primary documentation and
small examples; these are research candidates, not preselected designs. Include
Unicode-version consistency: segmentation/classification currently use Unicode 16,
while this folding table uses 17. Examine whether the familiar OrdinalIgnoreCase
name adequately communicates the selected simple-fold behavior.

**Review deliverable:** a compact keep/change/defer matrix in the existing text
design documents, concrete consumer examples, tradeoffs and unresolved questions,
and a recommended next bounded slice. Distinguish proven usability improvements
from hypotheses; benchmark only performance claims that matter. Do not start a
redesign, StringBuilder, generalized casing or additional comparison overloads
while producing that recommendation. This review supersedes the earlier proposed
text-construction-first ordering; it does not reopen the released HTTP POC.

**Outcome:** the [completed review](../design/text-abstraction.md#systemtext-foundation-review--2026-09-27)
contains the keep/change/defer matrix, framework comparisons, actual five-language
observations, a model-impact assessment and crucial-interface checklist. The model
is viable; its main risks are hidden units/costs, grapheme buffering assumptions and
foreign-text round trips, not UTF-8 itself. These are explicit acceptance concerns
for a modern API, not permission to copy .NET signatures mechanically. The author stresses the System.Text
dependency, then clarifies that we should select what to bring over rather than
copy everything. The [proposed portfolio](../design/text-abstraction.md#what-to-bring-over-from-systemtext)
is scalar support, a small builder, improved UTF-8 conversion and incremental
encoder/decoder roles. General codec registries, fallback class hierarchies, every
encoding and adjacent subnamespaces are not automatic scope.

**Further author clarification:** separate metadata serialization, logical text,
runtime storage and API boundary encodings. String/Char-facing APIs should remain
encoding-independent; UTF-8 is a documented boundary default where selected, not
Char's meaning. The [layering contract](../design/text-abstraction.md#separate-the-text-and-encoding-contracts)
qualifies the review's low-level buffer recommendations. This does not change the
currently selected grapheme unit or any artifact format.

**Next bounded recommendation:** a String/System.Text boundary prototype using a
small delimiter-extraction consumer and split UTF-8 input. Resolve validated scalar
identity, traversal, source-range ownership and progress/error units before dependent
APIs. Unicode-version alignment and explicit simple-fold naming are recommended
changes, not performed migrations. Keep the existing Sequence<char> surface for now;
do not require grapheme buffers underneath every text API. Broader designs remain
provisional and HTTP remains closed. After the boundary decisions, a small append-only
builder need not wait for unrelated search, regex, collation or zero-copy work.

**Review validation:** the corrected Raven observation sample and independent .NET,
Swift, Rust and Go probes pass/run as recorded in
[the evidence](../experiments/text-review/README.md). Source inspection supplies
complexity and ownership observations; these are not performance measurements.
No runtime/compiler/API changes, full-suite run or website build are part of the review.

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
