# neoCLR issue triage and fix sequence

**Reviewed 2026-09-26.** Supporting plan for the authoritative
[platform roadmap](platform-roadmap.md#issue-driven-priorities--2026-09-26).
The author requested investigation of both issue trackers, clarified “Focus is on
neoCLR”, then requested roadmap documentation. The priorities below are assistant
recommendations, not approval of every issue's proposed design or a release schedule.

## Evidence and scope

Read all **22 open neoCLR issues (#2–#23)** and **nine open Raven issues**, including
bodies and returned comments, through GitHub CLI. Neither open set contained comments
at retrieval; Raven had no open pull requests. GitHub status is a dated snapshot.
Compared the issues with neoCLR source at `d5fd1239`, its current platform roadmap,
experiment records and the local Raven resource-declaration specification. No new
runtime/compiler tests were run for this planning review; historical passing checks
are evidence to reproduce, not fresh verification. No issues were edited or closed.

Most issues request capabilities or design investigation rather than report a
reproduced bug. Prioritize by correctness, contribution to the managed HTTP/JSON
application, dependency risk and bounded verification cost—not issue age or API count.
Source implementation, bridge projection, packaged SDK availability and released
behavior are separate completion states.

**Follow-up before committing this plan:** the author reports that compiler work may
now be complete. Source and commit review confirms Raven `13b9105d8` / neoCLR branch
`56083626e` fixed the generic method-group failure, and neoCLR `afcc8c3d` integrated
JSON helpers and generic boxing. The [resolution](experiments/http-json-client-prototype/README.md#resolution--2026-09-26)
and [active fixture](experiments/http-json-client/README.md) supersede the original
blocker below. The separate unresolved-call failure is not claimed resolved. Continue
with sample repeatability and matching-toolchain acceptance, not reimplementation.

The [subsequent focused check](experiments/http-json/repeatability-20260926.md)
reproduces both the managed-pair timeout and the separate unresolved-call acceptance
failure on the updated bundle. Independent client repeats pass. This is fresh
validation beyond the original read-only inventory; the original compiler blocker
and remaining diagnostic defect must not be conflated.

The [array-budget cost follow-up](experiments/http-json/array-budget-cost.md) reduces
temporary host allocation in the per-instruction quota scan. It improves successful
run timings, but baseline and candidate each pass only two of three comparisons.
Keep timeout localization on the active application track; do not close it based
on the optimization or promote further library expansion as if M1 were complete.

The [slot-summary follow-up](experiments/http-json/slot-budget-summary.md) reuses
unchanged payload counts while enforcing quotas at every instruction. All 88 focused
checks pass; both variants pass three new identical-input exchanges, with candidate
client execution 2.3–5.2 seconds versus 5.9–8.1 seconds for the previous walker.
Managed counts remain identical and final live objects are zero. This improves
observed client cost; it does not explain the prior timeout or establish M1 completion.

## First: compiler correctness and a repeatable application

The most urgent reported defects are documented locally but are not represented by
dedicated issues in the retrieved open lists:

| Task | What it means | Completion evidence |
| --- | --- | --- |
| Compiler correctness: partly resolved | The generic method-group defect is fixed; retain its regression. Independently reduce `missing()` compiling into an empty body on the matching compiler. The active JSON fixture now replaces the temporary prototype. | Invalid source fails compilation; supported generic method binding executes the intended converter. Cover ordinary CLI methods, generic extension contexts and source/reference shadowing as relevant. |
| JSON client helpers: integrated | Preserve the implemented generic-only GetFromJson/PostAsJson over existing JsonContent and the Send pipeline. POST returns HttpResponse; keep status, JSON errors and cancellation policy explicit. | Compile and execute value/reference generic cases, including `box T` identity and GC checks; verify immediate and pending task paths. |
| Stabilize the managed HTTP pair | Measure startup, model/reflection work, scheduling and transport phases using matching pinned artifacts. Current source reports successful runs, while an older installed bundle records repeated timeouts. Do not generalize either result to all toolchains. | Repeated managed client/server exchanges with recorded budgets and timings, malformed/error/cancelled cases, independent peers and zero retained resources after completion. A single successful run or increased timeout is insufficient evidence of the cause being fixed. |

See the [JSON prototype](experiments/http-json-client-prototype/README.md),
[installed snapshot limits](local-sdk-snapshot.md#provenance-and-validation-limits),
[mapped application](experiments/json-object-mapping/README.md) and
[HTTP design](http-client-design.md). The end-to-end demo now uses the generic helpers and shared
JsonContent. Generic helpers improve convenience;
they do not justify replacing the execution backend or expanding all reflection APIs.

For Raven, reproduce general defects on a feature branch based on a verified main
baseline, test them independently and integrate the resulting fixes into neoCLR's
experimental branch. Keep neoCLR policies and tests isolated. Compiler-affecting
integration requires both repositories' compiler/integration documentation and
changelogs. Broader diagnostic education is separate from emitting a correct error.

## Then: ownership and useful library gaps

| Priority / issue | Current evidence and bounded next action | Meaning and completion boundary |
| --- | --- | --- |
| Reconcile now: [#18 HTTP enhancements](https://github.com/marinasundstrom/neoCLR/issues/18) | HttpResponse.Request and copied client default headers are implemented. Verify against the intended source/package; split the optional server association question from completed client behavior. | Response origin is discoverable. HttpContext.Request already identifies the inbound request; server-created response association remains a separate policy. Do not claim all server scope complete. |
| Next ownership slice: [#16 deterministic cleanup](https://github.com/marinasundstrom/neoCLR/issues/16) | Disposable and Result-returning Closable<E> already exist. Raven documents use cleanup, including propagation and async scopes, for its .NET contracts; neoCLR target behavior still needs a compiled probe. | Choose one cleanup route, not unconditional Dispose plus Close. Define how close failure interacts with the body result and whether async completion differs from release. Verify reverse order, early returns, `?`, cancellation and lifetime across await. |
| Reconcile, then extend on demand: [#14 introspection](https://github.com/marinasundstrom/neoCLR/issues/14), [#7 reflection](https://github.com/marinasundstrom/neoCLR/issues/7) | TypeInfo.GetFields overloads, parameterless runtime-backed CreateInstance and property GetValue/SetValue already exist. Constructor lookup, general MethodInfo.Invoke, field execution and argument-binding constructors are distinct gaps. | Start with metadata-only constructor discovery if a consumer needs it. Add execution only with visibility, receiver/value checks, unsupported-provider results and GC tests. Current flat JSON mapping already works without general invocation. |
| Application-driven: [#8 comparisons](https://github.com/marinasundstrom/neoCLR/issues/8), [#3 comparers](https://github.com/marinasundstrom/neoCLR/issues/3), [#11 Text API](https://github.com/marinasundstrom/neoCLR/issues/11) | CompareOrdinal and strict UTF-8 infrastructure exist. Specify the missing public/projection surface before adding comparison options, StringComparer, encoding abstractions or StringBuilder. | Start with deterministic protocol needs; keep HTTP's ASCII case-insensitivity separate from Unicode folding and linguistic collation. Equality/hash consistency matters for keyed collections. StringBuilder is a separate buffered-construction slice, justified by a caller or measurement. |
| Follow-up: [#5 parsing](https://github.com/marinasundstrom/neoCLR/issues/5), [#10 globalization](https://github.com/marinasundstrom/neoCLR/issues/10) | Int32.Parse already returns a typed Result with strict ASCII grammar. Expand one required type at a time. Generic Parsable and invariant-culture infrastructure are separate contracts. | Document overflow, full consumption, whitespace and error types. Protocol grammars must not change with ambient culture. Decide how a generic parser represents each type's errors before introducing an interface. |
| Small companion candidate: [#4 constants](https://github.com/marinasundstrom/neoCLR/issues/4), [#6 enum flags](https://github.com/marinasundstrom/neoCLR/issues/6) | Numeric bounds and Boolean strings are recorded follow-ups. Interpret the issue's ValueString spelling as a question to resolve against intended FalseString. HasFlag exists in the historical Neo/runtime path; establish the Raven public gap. | Exact scalar bounds and Boolean strings can be small independent changes; grapheme Char must not inherit UTF-16 bounds. Verify flags zero, combinations and unnamed bits. Do not rebuild the enum model. |
| Reconcile now, defer expansion: [#13 ConvertibleInto](https://github.com/marinasundstrom/neoCLR/issues/13) | The instance Convert() interface is implemented. Static Self conversion, ConvertibleFrom and return-directed overload selection are additional designs. | Explicit instance conversion is already usable; avoid tying it to a new inference rule. |
| Demand-driven: [#17 GC helper](https://github.com/marinasundstrom/neoCLR/issues/17) | Select a concrete diagnostics/test use before defining System.Runtime.GC. | A narrow statistics or collection request may help tests, but must have invocation/heap semantics. It is not deterministic cleanup and need not expose .NET's full GC controls or promise immediate native-resource release. |

Source checkpoints: [HTTP client](../runtime/raven/src/System/Web/Http/HttpClient.rvn),
[introspection](../runtime/raven/src/System/Introspection/Descriptors.rvn),
[reflection extensions](../runtime/raven/src/System/Runtime/Reflection/ReflectionExtensions.rvn),
[common interfaces](common-interfaces.md), [ordinal text](ordinal-text.md),
[parsing](raven-parsing-api.md), [enum contract](enums.md) and
[globalization proposal](globalization-design.md). Historical documents may describe
earlier carriers or Neo-only paths; verify the current Raven consumer before closing
an issue on that evidence.

## Larger changes: separate experiments and decision gates

These are not all prerequisites for each other. Start with the smallest question
that can reject an unnecessary dependency.

| Issue | Recommended order and experiment | Benefit to test / cost to account for |
| --- | --- | --- |
| [#12 importer transition](https://github.com/marinasundstrom/neoCLR/issues/12) | First measure text serialization/parsing versus metadata decoding and execution on the same workload. Compare loading into existing runtime structures with the current path. Treat a new metadata format and Raven CodeGen refactor as later, independently justified work. | Potential startup/toolchain simplification; requires identity, validation, unsupported-feature diagnostics, source mapping and equivalent execution. Removing text does not itself establish faster loading. |
| [#21 Self](https://github.com/marinasundstrom/neoCLR/issues/21), [#22 generic math](https://github.com/marinasundstrom/neoCLR/issues/22), [#2 return-directed overloads](https://github.com/marinasundstrom/neoCLR/issues/2) | Test one generic numeric algorithm with explicit self type parameters and the required operator constraints. Determine static-interface dispatch support before selecting Self syntax. Keep return-directed overload selection separate. | Generic algorithms may remove duplicated numeric code; new dispatch/inference rules affect diagnostics and metadata. Self is not automatically required for generic math, and return-directed overloads do not fix a broken ordinary method-group conversion. |
| [#20 tuple structs](https://github.com/marinasundstrom/neoCLR/issues/20) | Clarify whether the request is syntax, nominal named tuple structs, structural identity or a runtime representation replacing ValueTuple. Compare layout, equality, names and reflection using one pair-valued API. | Potential simpler value APIs; compiler/importer/debugger and interoperability costs vary sharply by interpretation. The issue is too terse to choose a representation. |
| [#23 function types/objects](https://github.com/marinasundstrom/neoCLR/issues/23) | Use the [proposal](proposals/function-types-and-objects.md) for an isolated callable experiment: static and bound methods, escaping captures, generic invocation, reflection and GC. Compare existing nominal delegates, function syntax lowering to them, and a new runtime callable representation. | Could give structural callable contracts and explicit multicast collections. Costs include type identity, variance, equality, callbacks/tasks, bridge compatibility and migration. The issue's replacement direction is stronger than the older delegate review; record it as a proposed architecture, not an already implemented replacement or an HTTP prerequisite. |
| [#15 metadata unions/intersections](https://github.com/marinasundstrom/neoCLR/issues/15) | Try the capability-composition research case already in the platform roadmap. Compare existing nominal interfaces and constraints first. | A new metadata type expression must justify dispatch, assignability, reflection and cross-language costs. Existing nominal Result/Option unions do not establish structural union semantics. |
| [#19 Raven bootstrapping](https://github.com/marinasundstrom/neoCLR/issues/19) | Later application milestone: inventory compiler dependencies, run one compiler component, then a bounded source-to-artifact case on neoCLR. | A powerful platform test, but a large library/runtime demand. It does not require every proposed type-system redesign, nor should the HTTP release wait for self-hosting. |

## Work alongside a larger feature

Keep one main feature and, when useful, one independently reviewable companion task.
A companion should have a fixed finish condition and no new ABI or language decision.
Small source diffs are not necessarily small semantic changes.

| Main work | Useful companion | Stop boundary |
| --- | --- | --- |
| Compiler reproduction/fix | Reconcile implemented issue portions; turn temporary repros into durable fixtures; fix one RavenDoc generic-name or navigation defect | Do not expand into educational analyzer policy or a documentation architecture rewrite. |
| HTTP/JSON stability | One #9 API-discoverability repair, a focused independent-peer fixture, or documentation of package/source differences | Avoid simultaneous timeout-sensitive runs and heavy builds; they confound timing evidence. |
| Stream content/ownership | Compile a minimal use/Dispose target probe and document borrowed versus owned stream behavior | Async close/error-composition design is main work if required, not a spare-time add-on. |
| Loader experiment | Bounded #4 constants or a confirmed #6 public helper gap with consumer/reference coverage | Stop if it needs generic dispatch, storage redesign or a new primitive contract. |

For [#9 RavenDoc](https://github.com/marinasundstrom/neoCLR/issues/9), split the broad
issue: first fix incorrect Result/Option labels and inaccessible navigation; then
missing companion/case/inherited-member coverage; later consider extension-member
aggregation, Markdown overlays and richer hierarchy presentation. A CSS/navigation
repair is a plausible small task; union semantic modeling may not be. General fixes
belong upstream in RavenDoc. Keep manual linked API entries for rendering gaps until
the matching generated reference is verified. Publication remains separate.

## Raven issues through the neoCLR lens

| Open issue | neoCLR priority |
| --- | --- |
| [#1256 interface access modifiers](https://github.com/marinasundstrom/raven/issues/1256) | Highest listed Raven correctness candidate if a minimal neoCLR contract exposes it. Probe valid/invalid access and emitted metadata against C# semantics. It is not yet a demonstrated blocker or a substitute for reducing the two recorded compiler failures. |
| [#1255 type aliases](https://github.com/marinasundstrom/raven/issues/1255) | Later ergonomic work; explicit Result<T,E> already works. Aliases should not require new runtime type identity. |
| [#1220 educational diagnostics](https://github.com/marinasundstrom/raven/issues/1220) | Helpful later; separate optional style advice/code fixes from mandatory compiler errors. Do not turn neoCLR Result/Option preferences into universal .NET target policy. |
| [#1248 build badges](https://github.com/marinasundstrom/raven/issues/1248) | Small upstream maintenance, but little direct neoCLR value; prefer a neoCLR documentation gap. |
| [#1237 container image](https://github.com/marinasundstrom/raven/issues/1237) | Promote only if packaging/reproducibility needs it. A container does not replace native supported-target verification. |
| [#1258 parameter patterns](https://github.com/marinasundstrom/raven/issues/1258) | Defer; binding/symbol/display redesign is not required by the current application. |
| [#1254 binding macros](https://github.com/marinasundstrom/raven/issues/1254), [#1253 macro diagnostics](https://github.com/marinasundstrom/raven/issues/1253), [#1236 clause macros](https://github.com/marinasundstrom/raven/issues/1236) | Defer for neoCLR; no demonstrated M1 dependency. Cleanup correctness should not depend on introducing new macro forms. |

## Comparison basis and validation

Reuse existing feature research for routine fixes; apply [design research](design-research.md)
before adopting substantive new contracts. Primary .NET references checked 2026-09-26:

- [.NET resource scopes](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/using): synchronous/async disposal and reverse-order scope cleanup. neoCLR's Result-returning Close needs an explicit additional failure policy. Raven already documents related lowering; this plan does not claim its neoCLR integration is verified.
- [.NET string comparison guidance](https://learn.microsoft.com/en-us/dotnet/standard/base-types/best-practices-strings): explicit non-linguistic comparisons for symbolic data. neoCLR's native UTF-8 ordinal ordering intentionally differs from .NET UTF-16 ordering; ASCII protocol comparison is not full Unicode ignore-case.
- [.NET generic math](https://learn.microsoft.com/en-us/dotnet/standard/generics/math): numeric interfaces and static abstract interface members provide the baseline. This separates library contracts and dispatch support from a proposed Self spelling.
- [C# interface contracts](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/interface): the baseline for Raven #1256, including body/static/accessibility distinctions. Verify particular cases on the pinned compiler rather than assuming all interface members share one visibility rule.

Loader, callable, structural type and bootstrapping experiments still need their
own pinned implementation/specification comparisons and alternatives review before
production design. Their proposed benefits here are hypotheses, not benchmark results.

Each selected slice needs focused positive/negative consumers, relevant GC/lifetime
checks and matching bridge/reference/library snapshots. Refresh public API coverage
in the same change. Reserve broad integration and the combined website build for
release stabilization or a concrete regression need, following current author direction.
Before release, resolve HTTPS scope explicitly, retain IPv4/HTTP/size limits accurately,
validate supported targets and the installed SDK, and reproduce the managed sample.
These remain release gates even where the open issue list does not mention them.
