# Raven target support before library migration

Recorded 2026-09-14. This assessment supersedes the immediate migration priority in
[the runtime API plan](runtime-api-plan.md). It is a source review and focused test
checkpoint, not approval to merge the entire Raven experiment branch.

The author directed us to pause Raven authoring of System classes, evaluate the Raven
changes, integrate suitable general fixes, and replace accumulated bridge behavior
with consistent target support. Raven should compile for both .NET and neoCLR through
its normal compiler and project model. Retaining a loader/importer is compatible with
that goal; relying on it to repair language semantics is not.

The author clarified on 2026-09-14 that the remaining stabilization fixes are to
precede the next release. Runtime-library migration from neoIL to Raven is deferred
until after that release, rather than being a release prerequisite.

The intended endpoint is a reusable Raven target model with a small set of explicit
framework-contract mappings, such as Iterable versus IEnumerable. Review each special
case as a general compiler defect, configurable framework contract or genuine runtime
difference. Reusable mechanisms belong on main; experimental neoCLR configuration and
unresolved semantics remain separate. This is a direction, not a claim that the current
integration already meets that endpoint. For now, neoCLR is expected to share the
CLI structure closely. Future differences such as runtime nullability may require
semantic-model changes (types, conversions, flow and diagnostics), not merely name
or metadata mappings; do not anticipate those unresolved semantics in these fixes.
NeoCLR-specific code, mappings and tests remain excluded from Raven main for now.
A possible alternative emission backend is future evaluation, outside this scope.

[Preview 6 is published](preview-6-release-notes.md). The earlier
[release work order](release-stabilization.md) is historical; remaining general
compiler reviews continue before the generic-library authoring probe and migration.

## Indexer member access and completion — 2026-09-14

Raven main includes `ac4901f6b`. The author showed completion treating
`typeof(int).GetProperties().Item.` as if the indexer were an ordinary property and
requested the C# experience. The general reproduction used .NET `List<string>`,
`IList<string>`, `IReadOnlyList<string>` and a Raven-declared indexer, without neoCLR
configuration. The initial focused run had 16 failures and 16 passes.

Indexer symbols now report `CanBeReferencedByName = false`. Named lookup excludes
them; semantic-model fallback follows the same rule. Consequently `.Item` is invalid
for an indexer, dot completion omits indexers, and `[index].` provides element members.
Ordinary properties named Item remain valid, and metadata enumeration retains the
indexer symbol. This aligns with [C# indexer access](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/indexers/using-indexers)
without changing CLI property/accessor metadata or neoCLR's runtime contract.

All 440 completion/indexer/property/semantic-model checks passed, including cold and
already-bound queries, valid indexed access and imported interface indexer execution.
The main-based branch was integrated, pushed and removed. Installed experimental
tools have not been refreshed; synchronization and packaging are still needed to
expose this fix in the author's existing VS Code installation.

The inspected client log recorded completion at `54:37` on the sample at
2026-09-14T15:16:37.821Z, completing in 71 ms with 13 items. The matching server log
recorded startup and normal shutdown, with no request-level failure. These logs
establish successful request transport, not correctness of the answer. Compiler
regressions reproduce and validate the semantic fix independently of the editor.

Next reviews remain the separate bare type-pattern failures and array-factory work
listed below. Keep neoCLR target policy experimental and library migration paused.

## Imported union emission follow-through — 2026-09-14

Raven main includes `43f288b05`. Independent ordinary-CLI regressions confirmed the
remaining member-union method-signature and pattern-local problems from `04c953d67`.
Target-core emission now derives closed carrier and variant locals from semantic
symbols rather than temporary call proxies. Constructed generic method references
retain definition `!n` parameters, encode concrete primitive signatures correctly,
and preserve by-reference/modifier wrappers. Actual metadata void returns are kept
separate from unit-valued returns.

Scope normalization now runs after proxy replacement: earlier removal of apparently
unused scopes produced invalid TypeRef tokens. A further independent Raven-library
execution test found that case accessors used the logical carrier instead of the
case's actual metadata container (`Outcome` versus `Outcome<T,E>`); this is corrected.
All 291 focused metadata/pattern/by-reference checks and the .NET 10/.NET 11 build/run
matrix passed. The main-based branch was integrated, pushed and removed.

This does not integrate all of `04c953d67`. Bare type-pattern probes exposed separate
failures: `choice is Choice.Ok<int>` produced invalid code under ordinary and target-
core emission, while `result is Outcome.Ok<int>` was rejected with RAV2102 in both
modes. These probes were separated from destructuring coverage; the declaration-pattern
local hunk and independent boxing optimization remain deferred pending their own
binding/emission review. The array-factory review (`de872fa34`) also remains open.

The author's subsequent indexer-completion report takes the next focused slice.
Experimental Raven remains at `246d697bf`, unsynchronized; installed tools and Preview 6
artifacts are unchanged. Runtime-library migration remains paused. These tests prove
modern .NET execution, not execution on .NET Framework or NanoFramework.

## Metadata core-library identity follow-through — 2026-09-14

Raven main includes `11e5c57ec`. The reduced .NET 10/.NET 11 regression confirmed
that Raven's imported symbol recognized a struct while its metadata reflection type
reported a class. This reproduced independently of unions and neoCLR configuration.

Metadata setup now selects the supplied assembly that directly defines the root
`System.Object`, before adding host fallback paths. It uses the full assembly identity,
including version: selecting only the name could pick another framework version from
fallback paths populated by earlier compilations. A facade that only forwards Object
is not selected. If no supplied reference defines the root type, the host fallback
remains. No new target configuration or neoCLR-specific policy was added to main.

Eight new cases cover imported generic classes and structs with .NET 10/.NET 11
references under ordinary and target-core emission. They check exact core identity,
agreement between symbol and reflection classification, emitted return/parameter/local
types, referenced assembly scope, and execution against the separate library
implementation. The 25-check prior focused baseline passed; after the fix, all 33
focused checks, the full Raven baseline (5,515 reported passes) and the .NET 10/.NET 11
build/run matrix passed. This is broader compatibility evidence, not proof of .NET
Framework/NanoFramework execution or completion of the full Raven release gate.

The main-based branch was fast-forwarded, pushed and removed. The experimental branch
remains separate and has not been synchronized in this slice. Published Preview 6
artifacts and the installed experimental SDK/extension were not changed.

The core-library prerequisite identified below is resolved for the tested general
path. Next, re-evaluate the remaining pattern-local, method-signature and normalization-
order changes from `04c953d67` against this corrected baseline; do not assume every
old workaround is still necessary. The array-factory review (`de872fa34`) follows.
The independent boxing optimization remains deferred. Synchronization and the generic-
library capability probe still precede migration of the runtime library to Raven.

## Imported member-union binding review — 2026-09-14

Raven main includes `b60e3635f`, independently extracted from the binding portion of
`04c953d67`. Imported member unions now support `.Case(...)` by unique variant name
and infer closed generic variant arguments for an in-scope `Case(...)` pattern.
The leading-dot form needs no case import; the bare generic type name still does.
Qualified variant patterns continue to work. This uses the ordinary CLI union
constructor/extraction/deconstruction contract, with no neoCLR import configuration.

The independent probe initially failed six of seven cases: four shorthand binding
cases failed in both emission modes, while target-core qualified and imported Raven
union cases failed during emission or execution. With the binding correction, all
three member-pattern spellings executed correctly through the default .NET path.
The integrated fixture additionally verifies the import requirement, the leading-dot
form without imports, inactive cases, and repeated matching through a mutating
value-type deconstructor. The carrier retains its value across those matches.

The 14-test metadata/pattern baseline and the separate 210-test pattern/union baseline
passed. All 219 final focused checks and the .NET 10/.NET 11 build/run matrix passed.
This is focused validation of the binding fix, not a full Raven release gate or proof
of .NET Framework/NanoFramework execution. The main-based branch was fast-forwarded,
pushed and removed. No experimental target settings or tests were integrated.

### Target-core prerequisite exposed by the review

Temporary application of the candidate's pattern-local metadata fixes let assemblies
write, but all four target-core probes still failed on .NET. Inspection showed
imported struct carriers and variants emitted with `CLASS` instead of `VALUETYPE`
in parameters and locals. Method references also exposed incorrectly encoded
primitive/void signatures. This was reproduced with standard .NET reference assemblies
and without the experimental metadata import options; it is broader than union
shorthand binding. The temporary emission edits were reverted before integration.

Source review points to core-library identity alignment as the first prerequisite:
`Compilation.Setup` creates its metadata context around the host core library, while
the supplied target reference set defines core types in its own reference assembly.
Selecting an emission target identity does not itself align that metadata context.
A reduced imported-struct regression should confirm the correction independently,
including primitive signatures, actual value-type classification and .NET execution.
Do not work around the disagreement by rewriting union IL or importing the entire
experimental configuration. Any reusable selection/configuration mechanism must be
validated as general CLI support before integration into main.

At this checkpoint, the work order was (the first item is now completed above):

1. Resolve and test metadata core-library identity alignment on Raven main's general
   target path, retaining ordinary .NET behavior and attribute-emission coverage.
2. Revisit the remaining pattern-local, imported method-signature and normalization-
   order changes from `04c953d67` against that corrected baseline.
3. Review `de872fa34` for its general array-factory behavior. Keep the independent
   boxing-avoidance optimization from `04c953d67` deferred until separately justified
   and validated; it was not required for the proven binding fix.
4. Synchronize reviewed main fixes into the experimental branch and run the generic-
   library capability probe. Runtime-library migration remains paused.

Published Preview 6 artifacts remain unchanged. The experimental branch was not
synchronized by this binding slice; its existing implementation is not evidence of
independent validation of these general fixes.

## Interface implementation metadata follow-through — 2026-09-14

Raven main includes `f8f7568a1`, independently reviewed from `000ed511e`.
The ordinary CLI regression compiled against a reference-only library and ran with
its separate implementation. Default .NET emission passed initially; target-core
emission failed while writing an imported generic interface declaration. Replacing
that declaration exposed a second failure during .NET execution: primitive generic
arguments had been encoded as value-type tokens rather than CLI primitive elements.

The fix normalizes imported `MethodImpl` declarations through the existing semantic
reference mechanism, replaces temporary declarations before writing, tracks new
member references during scope normalization, and preserves primitive element codes.
Both implicit and explicit implementations are covered, including generic interface
returns, property getters, closed owners, definition-relative generic parameters,
assembly scopes and actual interface dispatch. No neoCLR names, target policies or
experimental import options are used by these regressions.

This review independently extracted the general member-reference tracking helper and
primitive type-reference encoding from the mixed `04c953d67` candidate as dependencies.
Its pattern changes, imported method-signature corrections (including Unit handling)
and normalization-order change remain to be classified and tested separately; the
mixed candidate has not been integrated wholesale.

All 15 prior focused metadata checks passed before the new regression. After the
fix, all 54 combined interface/metadata/generic/attribute checks and the .NET 10/.NET 11
build/run matrix passed. This is focused compatibility evidence, not a full Raven
release gate or proof of .NET Framework/NanoFramework execution. The main-based
branch was fast-forwarded, pushed and removed. The neoCLR experiment remains at
`246d697bf`; these reviewed main changes have not yet been synchronized there.
Published Preview 6 artifacts remain unchanged.

At this checkpoint, the mixed union-pattern/target-signature (`04c953d67`) and
array-factory (`de872fa34`) reviews were next. The binding review and newly identified
target-core prerequisite are recorded above; runtime-library migration remains paused.

## Application generic metadata follow-through — 2026-09-14

Raven main includes `b5ce4023b`, independently reviewed from the `cbd87efa8`
candidate. Four ordinary CLI regressions initially failed: target-core emission
mixed metadata definitions with source TypeBuilders, while default .NET emission
could not resolve fields on those unfinished generic constructions. The review
corrected both paths rather than retaining only the target-core workaround.

The target path uses signature-only generic construction where reflection contexts
cannot be combined. Source type references stay module-local; unlike the original
candidate, the scope check also requires a source assembly, so a matching name alone
does not identify an external metadata assembly as the current module. Default
emission maps generic fields from their definition through Reflection.Emit.

The tests compile against a reference-only library and execute its separate real
implementation. They cover classes, structs, arrays and nested generic arguments,
checking member identities, absence of self assembly references, calls and preserved
reference identity. The prior 17-test metadata/attribute baseline passed; after the
fix, all 28 combined metadata/attribute tests plus 14 other generic regressions
passed. The .NET 10/.NET 11 build/run matrix passed. These results do not certify
.NET Framework or NanoFramework execution or constitute a full Raven release gate.

The main-based temporary branch was removed after fast-forward integration and push.
The neoCLR experimental branch remains separate and unchanged in this slice; its
existing candidate implementation is not evidence that these new main changes have
been synchronized or validated there yet. No samples or published release artifacts
were modified.

At this checkpoint, interface implementation references (`000ed511e`) were next;
that review is now complete as recorded above. Remaining mixed reviews and the
generic-library capability probe still precede runtime-library migration.

## Delegate metadata follow-through — 2026-09-14

Raven main includes `35a9df494`, independently extracted from `5f274c063`, on top
of the attribute-regression correction. The normal-reference delegate fixture
failed before its fix; it contains no neoCLR names or import options. Delegate
normalization now retains its metadata constructor token instead of mixing
reflection contexts. The initial candidate passed 26 focused checks and the target
matrix; after the stability audit and correction, all 55 combined metadata,
attribute, delegate and generic-call checks passed. Both temporary branches were
removed. Main fixes remain separate from the active neoCLR experiment.

At that checkpoint the next reviews were mixed application/metadata generics and
interface implementation references. The application-generic review is now complete
as recorded above. The broader audit below retains its original commit scope.

## Raven main stability correction — 2026-09-14

The author's stability request prompted a broader audit of main `ea6f3383b`:
5,493 baseline tests and 173/172 standalone builds/runs passed, but the project
corpus exposed four NanoFramework attribute-emission failures. Corrected the
runtime/metadata type boundary for custom-attribute serialization in Raven main
`5a67d5d4c`. Both new regressions failed before correction; 39 focused checks,
four NanoFramework rebuilds and the .NET 10/.NET 11 matrix passed afterwards.
All 38 eligible project executables passed. The separate MacCatalyst host build
requires Xcode 26.6 rather than the installed 26.2; the full build gate remains
non-green until that prerequisite is satisfied. No sample or exclusion was changed.
This audit caught a regression the earlier focused integration checks missed.

## Generic field metadata follow-through — 2026-09-14

Raven main includes `ea6f3383b`, independently extracted from `4af98e7c1`.
Closed generic fields retain their declaring type and generic definition signature
without mixing host types with target metadata. The ordinary-reference regression
failed before the fix. Extended coverage verifies instance reads, static reads and
writes, and removal of temporary proxies. All 22 focused checks and the repository
.NET 10/.NET 11 build/run matrix passed. No .NET Framework or NanoFramework execution
is claimed; the author's direction explicitly includes fixes useful to those targets
on main. Both repositories now record that policy in their instructions.

The temporary branch was removed after integration. The neoCLR experiment remains
separate and already contains the implementation. Generic delegate-constructor
metadata (`5f274c063`) is the next independent review.

## Generic method metadata follow-through — 2026-09-14

Raven main includes `e14d23d32`, independently extracted from `11e9964f2`.
Closed generic method calls preserve MethodSpec arguments, definition signatures
and target assembly scopes. The normal-reference regression failed before the fix
with a MetadataLoadContext mismatch; all 27 focused checks and the repository
.NET 10/.NET 11 build/run matrix passed afterwards. No experimental metadata-import
option was used. The completed integration branch was removed; the active Raven
experiment already contains the implementation and remains separate.

Closed generic field metadata (`4af98e7c1`) is the next independent review.
Remaining stabilization work still precedes release, and runtime-library migration
remains deferred until afterwards.

## Constructor metadata follow-through — 2026-09-14

Raven main includes `f6b4748e6`, independently extracted from `995a4c982`.
Constructing reference-only nested generic cases and carriers no longer mixes
compiler-host types with MetadataLoadContext types. Temporary constructor tokens
are replaced with the original target signatures and removed from the final PE.
The adapted normal-reference test failed before the fix; all 26 focused checks
and the repository .NET 10/.NET 11 build/run matrix passed afterwards.
The temporary integration branch was removed after integration. The active
experiment already contains the implementation and remains separate. Closed
generic method metadata (`11e9964f2`) is the next independent review; the runtime
library migration remains deferred until after release stabilization.

## Closed-generic metadata follow-through — 2026-09-14

Raven main includes `031b9aaaa`, independently extracted from `17c9f8b82`.
Reference-only generic types remain in metadata during target-core emission;
member definitions retain their generic parameter signatures and by-reference
parameter shapes. The ordinary-reference regression failed before the fix and
passed afterwards, without experimental metadata-import options. All 18 focused
checks and the repository .NET 10/.NET 11 build/run matrix passed.

The experimental Raven branch already contains the implementation; it remains
separate and has not been synchronized with the latest two main commits. Constructor
emission (`995a4c982`) is the next independent review. This is a stabilization
checkpoint, not completion of the remaining fixes or the release gate.

## Pointer emission follow-through — 2026-09-14

Raven main now includes `521711bec`, independently extracted from experiment commit
`3df1b54b0`. Native pointer signatures are preserved when reconstructing method
references for target-core emission. The original default-options test already
passed on main; adding its existing EmitOptions retargeting path reproduced an
unsupported `Unit*` failure. After the fix, all 53 focused metadata/pointer tests
passed. No neoCLR target policy or experimental import option was added to main.

The experiment already contains this implementation and remains on
`codex/neoclr-namespace-metadata` at `5d1022ced`; it has not yet been synchronized
with this new main commit. The remaining generic metadata-emission candidates below
still require independent review. Runtime-library migration remains paused.

## Main integration — 2026-09-14

The author directed that general Raven improvements must be integrated into Raven
main while experimental neoCLR support remains on a separate branch. Raven main is
now `8fa59a967`: the numeric/pointer and binding/dispatch batches plus general
namespace-metadata/completion fixes. The namespace regression uses normal framework
references and default compilation options. Forty-seven focused namespace tests and
the 5,490-test broader baseline passed with no failures or skips.

Main was fast-forwarded from the reviewed integration branch, not merged from the
neoCLR experiment. The experiment was synchronized from main at `5d1022ced`; 50 focused
regressions passed and compiler source was unchanged by the merge. Main is now an
ancestor of the experimental branch. Remaining general metadata-emission candidates below are
queued for independent dependency review; they are not permanently categorized as
experimental. Target-specific array/Void/protocol policies have not entered main.
The earlier checkpoints below record the review state before integration.

## Follow-through checkpoint — 2026-09-14

General compiler fixes were extracted onto Raven `codex/compiler-fixes-integration`,
based on refreshed upstream `d92b02812740ae052f277c23151e9cc208f7672d`. Commits
`c4febb094` and `961041ae4` cover numeric/pointer behavior and binding/dispatch;
`a689ceab3` records validation. New regression tests failed before the fixes
(12 numeric/pointer failures and nine binding/dispatch failures), then the focused
suites passed (122 and 32 tests). The broader compiler/support/editor baseline passed
5,489 tests. These commits are pushed for review, not merged into Raven main.

A separate Raven branch, `codex/neoclr-target-contracts`, adds project-owned emission
core selection. It also prevents the service and driver from reintroducing host
references or host-derived defaults into explicit metadata targets. Seventy targeted
configuration/metadata tests and a separate compiler-driver regression passed, as did
the .NET 10/11 target matrix. NanoFramework was not tested in this checkpoint.

neoCLR now accepts an independently compiled assembly through `--import`; the normal
Raven compiler can produce it without runner-supplied EmitOptions. Five end-to-end
checks cover ordinary output, string filtering, propagation and invalid target inputs.
All 63 saved-project checks also pass. See [the compilation workflow](raven-target-compilation.md)
for commands and the required source-tool/project update.

This completes the first reviewed general-fix batches and the emission-core portion
of target selection below. A versioned target-pack description remains open. Importer
identities, generic library bodies, namespace-function projection and removal of the
Void adapter remain prerequisites to evaluate before resuming library migration.
Installed tools and demos are unchanged.

Application type/field/function naming now follows
[assembly/signature identities](raven-import-identities.md), with original names in
source-map sidecars. This removes public application row-token naming; separate
general signature/dispatch importing remain open. A subsequent
[static-library checkpoint](raven-library-import.md) admits explicitly supplied
nongeneric static bodies and intra-library namespace-function calls through their
existing CLI metadata. Direct consumer wildcard discovery initially reported RAV0103; the subsequent
namespace-metadata slice supplies the missing TopLevelAttribute and fixes Raven
metadata-first marker lookup and imported-member completion. Library-owned nongeneric instance types are now covered by the same workflow,
including cross-assembly inheritance/interface calls and delegates. Generic bodies
remain open.

## Initial assessment evidence

Raven branch `codex/neoclr-target-resolution` at `854cd4d3d8c2fa4ed2f82e834c65cfef4371ebe3`
contains 37 commits beyond local `main` / merge base
`d92b02812740ae052f277c23151e9cc208f7672d`: 71 changed files, including compiler,
workspace/editor, tests and documentation. These are local comparison revisions;
upstream must be refreshed before preparing integration branches. No Raven changes
have been merged by this assessment.

The Math pilot is paused, not adopted. Its source, generated fragment, importer mode
and tests are preserved locally in git stash commit
`32cd5a95dec6e10cdf34db536d629598d7bf8311`, named
“Paused Raven scalar library migration pending compiler and target-contract evaluation”.
This is a local recovery reference, not a published source artifact. It demonstrated
three nongeneric integer functions, not general System class authoring. The published
source checkpoint remains `milestone/raven-library-baseline-2026-09-13`.

A reported `List<string>.Where(x => x == "2")` failure was reproduced in a disposable
project. The failing operation was `ceq` with String operands inside the lambda;
collection interface conversion had succeeded. The reference String declaration
omitted equality operators, allowing Raven's reference-equality fallback. Adding
those declarations and mapping the operator calls to existing String value equality
made the exact query print `2`, without changing Raven. This is a missing target
library contract, not evidence that collection lowering must be rewritten.

## Separate the Raven changes before integrating

These are review batches, not a claim that each commit is independently cherry-pickable.
Inspect dependencies and include each fix's tests and documentation. Keep work isolated
on Raven feature branches and retain the experimental branch until replacements land.

| Batch | Representative commits | Required review and evidence |
| --- | --- | --- |
| General semantic correctness | `bc0ec8046` implicit operator applicability; `09cf60417`, `809aef0fe`, `e51da7a48`, `26907410f`, `406962312` numeric conversions/comparisons; `c1431bea1` pointer substitution | Check .NET runtime outcomes and diagnostics, signedness and boundary cases; suitable first integration candidates independent of neoCLR naming. |
| Binding, dispatch and editor correctness | `854cd4d3d` inherited indexers; `55c0f7ef5` constructor queries; `3142f2f13`, `62105de24` delegate/virtual flags; `9b269f9d0`, `6ab473fbc`, `a843844e4` receivers/enum context | Preserve ordinary .NET behavior, semantic APIs and completion. Test before proposing merges. |
| General cross-target metadata emission | `17c9f8b82`, `995a4c982`, `11e9964f2`, `4af98e7c1`, `5f274c063`, `cbd87efa8`, `000ed511e` (pointer fix `3df1b54b0` subsequently integrated; see above) | Preserve declaring identities, closed generic signatures and receiver modes without loading target assemblies into the host. Review token-proxy normalization as an emitter implementation, not a permanent target-specific API. |
| Reusable target configuration | `1d7341fa6`, `37ae97304`, `5b773ae35`; `c28657859`, `1e3f7ff07`; `1a52d9464` | Explicit references and core identity, iteration and propagation contracts must use the same configuration in CLI, projects, editor and emission. Defaults must continue to target .NET. |
| Deliberate neoCLR differences | `0fad44881`, `22cea6fa1`, `8e0f6cb7d`; `f2a4af608`, `ec88c4474`, `e127e1c49`, `6072dcf4b` | Keep inhabited Void versus no-result returns, invariant mutable arrays and generic array shape explicit and opt-in. Do not integrate by silently changing CLR defaults. |
| Mixed changes requiring closer separation | `04c953d67` imported union patterns and target signatures; `de872fa34` array factory dependency | Separate general metadata/pattern correctness from runtime assumptions; exercise .NET plus an explicit alternative target. |

The inventory covers themes rather than an exhaustive line-by-line approval. The
existing framework suite alone is insufficient: its current filter ran only seven
tests. Explicit metadata, iteration, propagation and array-policy suites must also be
selected. A final merge needs tests on the proposed integration branch, not merely on
the combined experiment branch. NanoFramework is a useful independent target check;
this checkpoint does not claim it was tested.

## Validation in this checkpoint

- The screenshot reproduction prints `2` with fresh target metadata and the corrected
  bridge; installed user source and tools remain unchanged.
- The query suite passed 30 checks, including the new string predicate case,
  constructed equal strings, captured predicates, inequality and UTF-8 text.
- Target metadata signature checks passed 118 checks, including both String operators.
- Raven's framework-and-targeting feature suite passed seven tests. Explicitly
  selected metadata-import, target-emission, iteration, propagation and array-policy
  tests passed 63 tests on the existing experiment branch.
- These checks establish bounded current behavior. They do not certify the 37 commits
  for integration, generic library body importing, or NanoFramework compatibility.

## Intended boundary

| Layer | Responsibility |
| --- | --- |
| Target reference pack | Truthful type/member/operator signatures, interfaces, generic constraints and attributes. Versioned with the executable library. No invented fallback host APIs. |
| Raven target selection | Resolve a single coherent configuration before binding. Select iteration/propagation symbols and intentional policies; preserve .NET defaults. |
| Raven binding and lowering | Diagnose missing contracts; resolve operations against selected symbols; generate valid target behavior, including return/stack conventions. |
| Raven emission | Share metadata/CIL emission where semantics match. Preserve target identities and signatures; avoid host framework dependencies leaking into artifacts. |
| neoCLR importer/loader | Validate metadata, dependency closure, CIL and capabilities, then map supported constructs consistently into runtime representation. Reject unsupported constructs with source/IL context. |
| neoCLR library/runtime | Implement the advertised APIs, GC, dispatch and type rules. Intrinsics and deliberate ABI differences need explicit, documented contracts. |

Prefer a versioned target-pack description that composes the existing options, rather
than another set of unrelated flags or hard-coded `if neoCLR` cases. Exact descriptor
format is open. Binding names belong in the pack; rules that affect compilation belong
in compiler options derived from it. The same resolved configuration must reach the
editor, build driver and emitter. The compiler must not depend on neoCLR's per-method
binding catalog.

A stable importer can initially remain C#/Cecil. Rewriting it in Rust or adding a Raven
backend does not itself solve contract drift. Move from selected-method catalogs to
metadata-driven type/member resolution incrementally, retaining explicit intrinsic
bindings where the runtime requires them. Do not reinterpret CLI reference `ceq` as
string value equality to compensate for absent operator metadata.

## Ordered next slices and exit criteria

1. **Compiler stabilization.** Review and propose the general fix batches first, with
   their regression tests on ordinary .NET. Evaluate dependencies before cherry-picking.
   Run the target framework matrix for cross-target emission fixes. Do not merge the
   experiment wholesale or claim tests of one profile prove another profile works.
2. **Coherent target selection.** Consolidate explicit metadata imports, target core
   identity, iteration, propagation, arrays and Void policy into a validated target
   description shared across CLI/project/editor. Test missing and inconsistent packs,
   no host fallback, and unchanged default .NET projects.
3. **Importer contract.** Preserve stable assembly/type/member identities, validate
   library declarations against implementations, replace application-token identities
   and special per-API translations with general signature/dispatch rules. Keep normal
   CLI void returns distinct from inhabited Void in value positions. Document each
   remaining adapter's purpose and removal condition.
4. **Library capability probes.** Before resuming migration, import a nongeneric class
   with constructor/fields/properties, a generic type and method, an interface call and
   a namespace function; compile a separate consumer against the resulting reference
   surface. Include direct neoIL consumers, malformed metadata, GC and callback tests.
   Demonstrating calls to an existing generic API is not demonstrating body importing.
5. **Resume migration only after that evidence.** Choose a small ordinary implementation
   and compare outcomes, faults, allocations and API metadata with the current library.
   Bootstrap from a clean checkout and execute a packaged consumer.

Namespace functions should eventually receive stable namespace/function identities in
neoCLR. Raven's CLI container remains a reasonable intermediate representation for
shared emission; automatic projection must be based on a documented metadata contract,
not guessed container names. Reflection and overload identity need tests.

`System.NotImplementedException` is currently a metadata-only extension-marker
compatibility dependency in `CoreDeclarations.cs`; executable construction is rejected.
Removing it is deferred emitter/reference-pack cleanup, not a proposal to add guest
exception classes. Do not migrate it into Raven-authored System APIs.

## Comparison and tradeoffs

.NET separates reference metadata from executable implementations; reference assemblies
are compiler inputs, not runtime implementations. Keep this useful distinction, but
verify neoCLR's projected declarations against its library so target code does not
bind to accidental signatures. See [Microsoft's reference assembly documentation](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies)
(accessed 2026-09-14). Existing [binary-profile research](raven-binary-profile.md) and
Raven's metadata-import/iteration/propagation design documents remain the baseline.

The string fix restores the familiar value comparison contract described by
[System.String equality in .NET 10](https://learn.microsoft.com/en-us/dotnet/api/system.string.op_equality?view=net-10.0)
(accessed 2026-09-14). neoCLR continues to use its existing UTF-8 String implementation;
this does not add nullable string support or change CLI reference equality.

Continuing per-API adapters is quick for demos but duplicates contracts and hides
missing capabilities. A second compiler backend offers control but duplicates emission
and increases Raven divergence. Shared emission plus explicit target contracts and a
metadata-driven importer is the preferred direction: more initial validation work,
less dependence on a growing list of special cases. Full .NET compatibility is not
promised; Result/Option error flow, Void value positions and array invariance remain
intentional differences to validate separately.

## Branch housekeeping — 2026-09-14

At the author's request, removed completed Raven integration branches
`codex/compiler-fixes-integration`, `codex/general-pointer-emission` and
`codex/general-generic-metadata` locally and remotely after checking main ancestry.
Removed superseded `codex/neoclr-target-contracts` and
`codex/neoclr-target-resolution` locally and remotely after checking that both are
ancestors of the active `codex/neoclr-namespace-metadata` experiment. The neoCLR
local branch `codex/raven-neoclr-target` was already merged into neoCLR main and
was also removed. Active worktrees, the experiment and unrelated branches remain.
These are ref deletions; the retained branches preserve all of this history.


### Generic array empty-value API direction (2026-09-14)

The author proposes `System.Array<T>.Empty`, probably a static property: the closed
array type supplies the element type. This differs from .NET's
[`System.Array.Empty<T>()`](https://learn.microsoft.com/en-us/dotnet/api/system.array.empty)
factory on its non-generic Array base class. It fits neoCLR's existing generic array
shape without adding a method type argument. The cost is another API shape that
compiler target mapping must recognize if it wants to use the shared empty array.

The property, caching policy and experimental Raven mapping remain to be implemented
and validated. Keep the general Raven array-factory review independent: an empty
collection expression must not assume a factory absent from the target metadata;
a zero-length array allocation is a possible fallback. Do not add a hardcoded
neoCLR property lookup to Raven main.
