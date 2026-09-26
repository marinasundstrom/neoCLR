# Library and data tracking

**Consolidated 2026-09-27.** [Platform priorities](../platform-roadmap.md) govern work.
This page owns general library scope. Further additions are deferred while the HTTP
POC is prepared for release; a candidate below does not authorize another feature automatically.

## Recorded checkpoints and remaining scope

| Theme | Recorded implementation/evidence | Next boundary when selected |
| --- | --- | --- |
| Text and encoding | [String sequence](../experiments/string-sequence/README.md), [ordinal text](../ordinal-text.md), [UTF-8 chunks](../experiments/utf8-chunks/README.md) | Coherent System.Text, casing/comparison policy and equality/hash consistency; do not confuse HTTP ASCII names, Unicode folding and culture-sensitive collation. |
| Collections | [Collection contracts](../collection-contracts.md), [generic arrays](../generic-managed-arrays.md), [filtering](../arraylist-filtering.md) | Extend for an executable consumer; preserve mutability/capability and GC boundaries rather than reopening a complete interface sweep. |
| Storage, streams and console | [Storage provider](../experiments/storage-provider/README.md), [file streams](../experiments/file-streams/README.md), [console streams](../experiments/console-streams/README.md) | Synchronous sources are useful but are not general async streams. Select async ownership, cancellation and error composition before new adapters. HTTP content ownership is tracked with HTTP. |
| JSON and data mapping | [JSON DOM](../experiments/json-dom/README.md), [flat mapping](../experiments/json-object-mapping/README.md) | Richer model shapes/options remain bounded future slices; existing size/shape limits are not erased by HTTP integration. |
| Introspection/reflection | [Descriptors](../introspection-design.md), [execution fixture](../experiments/reflection-execution/README.md) | Metadata discovery and execution have separate contracts. Do not infer general invocation from parameterless construction/property access. |
| Time and presentation | [Date/time design](../date-time-design.md), [globalization proposal](../globalization-design.md) | Existing injectable clocks provide a foundation; time zones and cultural formatting remain separate selected scenarios, not an immediate broad framework. |

## Deferred issue inventory

This preserves the 2026-09-26 issue inventory, not current GitHub status. The
[original library analysis](../history/planning-20260927/issue-fix-roadmap.md#then-ownership-and-useful-library-gaps)
retains detailed rationale and .NET comparison links.

| Issue | Known boundary and bounded next action |
| --- | --- |
| [#3 comparers](https://github.com/marinasundstrom/neoCLR/issues/3), [#8 comparisons](https://github.com/marinasundstrom/neoCLR/issues/8), [#11 Text API](https://github.com/marinasundstrom/neoCLR/issues/11) | Start with a deterministic comparison consumer; specify ordering, equality/hash agreement and Unicode/culture differences before broad overloads or namespaces. |
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
