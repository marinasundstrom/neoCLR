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
