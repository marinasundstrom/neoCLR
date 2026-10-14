# Structural union/intersection experiment

Status: branch-only interpreter signatures behind `experimental-compositions`,
2026-10-14. Raven supports explicit native call-only intersection consumers; general
authored compound storage, JIT/AOT storage, public reflection APIs and a stable ABI
remain unavailable. The earlier probes below retain their narrower evidence.

## Branches and reused work

- neoCLR: `codex/structural-union-intersection`, based on main `e2b010683`.
- Raven: `codex/neoclr-union-intersection`, created at the native compiler line
  `caa787a96` (`codex/source-object-metadata-resolution`). The separate
  `raven-union-intersection` worktree selectively carries semantic symbol and
  reference-conversion changes from `02d669acd` and `c4d57d282`; source storage and
  native emission remain pending. The existing Raven checkout is unchanged.
- neoCLR `codex/structural-types` at `3571ab791`: inspected, not merged. Its
  branch-only commit is an old scalar-AOT experiment, not a newer composition
  implementation. Current main already contains the reusable structural signature
  codec and catalog-scoped identity resolver in `tools/metadata`.
- Raven `codex/neoclr-structural-types` at `b92a0d4a7`: native Function transport
  is relevant architectural precedent; it is not union/intersection storage support.
- Raven `codex/intersection-constraints` at `6c112574c`: reusable candidates include
  semantic symbols, normalization, constituent conversion checks, substitution,
  member ambiguity handling and focused tests. Its compiler document explicitly
  rejects source intersection storage with RAV0363 and limits execution evidence
  to modern .NET. Do not merge the branch wholesale or remove that gate merely
  because this probe succeeds. Reassess individual commits against the native line.

## Question and comparison

Can closed nominal reference types compose with either/all membership, preserve
one object through checked projection, and reuse existing signature transport?
The experiment passes if positive and negative memberships, codec roundtrips and
order-independent identity agree, and accepted static conversions preserve runtime
membership over an independent set of object witnesses.

.NET provides nominal interface membership, reference casts and dispatch. This
probe uses `Type.IsAssignableFrom` and `IsInstanceOfType` as a host oracle, then
uses ordinary interface calls to check object preservation. The proposed addition
is a reusable compound signature understood by neoCLR across language boundaries.
A compiler-only erasure alternative can use ordinary object/interface storage but
needs extra signature information to preserve the compound contract. A tagged
carrier alternative records a selected arm; that is observably different from
membership when one object satisfies both alternatives.

Initial primary comparison inputs (retrieved 2026-10-14):
[ECMA-335, sixth edition](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
[Scala 3 unions](https://docs.scala-lang.org/scala3/reference/new-types/union-types.html)
and [Scala 3 intersections](https://docs.scala-lang.org/scala3/reference/new-types/intersection-types.html).
This is a reduced experiment, not the completed metadata/verification design review.

## Executable scope

Run from the neoCLR root:

```sh
dotnet run --project docs/experiments/structural-compositions/StructuralCompositions.csproj --property WarningLevel=0
```

The project references the existing experimental metadata library without modifying
it. It exercises both wire connectives, flattening, duplicate removal, order-independent
identity and distinction between union and intersection. Its private membership layer
supports only closed nominal references and rejects unsupported syntax after the
existing codec/resolver checks (including their depth and size bounds).

Static conversion uses conservative structural rules and the nominal host oracle.
It is not a complete subtype solver: distributive entailment, bottom types,
uninhabited intersections, variance involving compounds and subtype normalization
are deferred. Null is rejected; nullable composition needs an explicit later contract.
Matching methods alone do not establish nominal interface membership. Interface
calls in the probe are explicit constituent projections, not synthesized compound
member lookup or native dispatch.

The shared metadata identity resolver currently preserves a singleton compound after duplicate
removal: `A | A` has a different identity from `A`. The test records this existing
boundary, not a proposed permanent rule. The Rust probe now applies separate semantic
normalization, collapsing singleton unions/intersections to their constituent. This
is the provisional semantic contract for this branch; the shared wire/identity API
is unchanged. Subtype equivalence is not identity normalization in this slice.
Reference indices are catalog-bound; this experiment does not define persistent ABI identifiers.

## Validation and limits

Local SDK: `11.0.100-rc.1.26425.128`, target `net10.0`. The command passes 187
assertions (including reference-table relocation and cross-assembly identity checks), including the finite witness conversion matrix. The matrix demonstrates
soundness only for its witnesses; it is not a proof of the full algebra. This host probe alone supplies no neoCLR execution evidence. The Rust probe below
adds bounded interpreter/runtime-service evidence, not JIT/AOT, compiler integration,
Windows or performance qualification. No allocation or speed advantage is assumed.

## Next bounded slice

1. Build value-storage binding and native metadata mapping on the selectively reused
   intersection symbol/conversion and source-constraint foundation, behind an explicit
   target capability.
   Keep existing ad-hoc union carriers until structural union policy is selected.
   Compiler symbols remove nominal supertypes; runtime signature normalization
   currently does not. Resolve that distinction before importing/exporting identity.
2. Specify descriptor identity and compiler metadata import/export together. The
   current Rust runtime variants are an experimental execution transport, not an
   assignment of public structural metadata bytes or a CLI bridge encoding.
3. Extend backend admission and GC layout deliberately before enabling JIT/AOT.
   Arrays, generics, fields, byrefs, intrinsic/boxed references and reflection need
   their own focused slices; these gaps are not permanent platform semantics.

The experiment does not reprioritize the platform performance milestone.

## Runtime-service probe

`runtime_probe.rs` is compiled only into the crate's unit-test binary via
`#[cfg(test)]`. Run:

```sh
cargo test --lib structural_compositions_probe -- --nocapture
```

Local validation: all seven Rust tests pass. The existing metadata host probe also
passes all 187 assertions. Only the focused test filter was run.

It admits a private expression descriptor containing closed nominal classes and
interfaces, using the runtime's existing type validator and reference relationship
checks. It rejects value types, generic parameters, byrefs, arrays, missing types,
invalid arity and excessive nesting/node counts. Identity is module-local; descriptors
from different borrowed module instances are not equated. Normalization flattens,
sorts, deduplicates and collapses singletons without subtype or distributive reduction.
Its JSON roundtrip is test transport, not a new production metadata format.

Checked narrowing uses the allocation's concrete type, never its existing interface
view, and validates ownership against the supplied managed heap. It preserves the
original handle and view. The compound descriptor stays external to the value:
there is no compound slot, runtime view tag, synthesized carrier or new GC layout.
This demonstrates that the existing reference representation can support membership
checks; it does not establish a general ABI. Nulls and boxed values are excluded.

The collector test retains the object through the existing root enumeration API and
reclaims an unrelated allocation. An ordinary neoIL consumer checks interface
projection, mutation through a recovered class view and reference equality using
existing verifier and interpreter behavior. A second interpreter consumer returns a
class allocation which the experimental intersection accepts without a wrapper.
These are runtime prerequisites, not compound guest instructions.

**Earlier probe boundary:** this stage deliberately did not change the product
`Type` enum. The following opt-in slice adds that representation with explicit
admission/rejection rules. The subsequent compiler API slice does not introduce
a Runtime Contract switch or source storage.

Website/API review: the manual Union/Intersection family references label the
branch-only execution slice. Published capabilities and generated guest API
snapshots are unchanged; no public guest members were added.


## Opt-in interpreter signatures

The feature flag is a Cargo build capability, not a Raven Runtime Contract option:

```sh
cargo test --features experimental-compositions --test structural_compositions
cargo run --features experimental-compositions -- run docs/experiments/structural-compositions/reference-compositions.neoil --execution interpreter --show-result
```

The saved consumer returns `Int32(42)`; the CLI also uses 42 as its process exit
code. This is expected success for this guest program, not a shell-zero convention.

`union<A,B>` and `intersection<A,B>` are experimental neoIL spellings. The native
format-5 model has `Union` and `Intersection` operand arrays; JSON and the existing
CBOR/NEOX execution transport preserve these variants. No shared structural-signature
codec bytes or published format versions change. Older readers reject unknown
variants; builds without the feature reject compound admission explicitly.

The loader validates constituents before normalization, then flattens like
connectives, sorts, deduplicates and collapses singletons before call binding.
Partial assembler contexts preserve unresolved expressions until definitions are
available, preventing normalization from hiding invalid operands. Every constituent
participates in ordinary dependency and accessibility checks. There is no subtype
reduction or distributive normalization, and the subtype algorithm is conservative.

Supported: closed nominal class/interface compositions in nongeneric managed free
function parameters, returns and locals; ordinary reference assignment; `castclass`
and `isinst`; interface dispatch when guaranteed by the receiver type. A union
must be narrowed before using an interface not guaranteed by every alternative.
A cast checks the concrete allocation, never merely its current view. A compound
view retains the same object handle and uses existing GC root tracing. There is no
arm tag and no wrapper allocation introduced by entry or projection.

**Null refinement:** the earlier probes only tested non-null membership. Executable
reference slots now follow existing reference behavior: typed nulls can be stored,
casts preserve null, and failed type tests return typed null. Null does not prove
that an object satisfies the constituents. Locals still require definite assignment;
this does not introduce default-initialized local variables. A separate enforced
nullable/non-null type distinction remains future work.

**Temporary admission limits:** only nominal class allocations are accepted as
non-null compound values. Strings, arrays, Functions and boxed value allocations
remain unsupported even if an Object/interface view would satisfy a constituent.
Consequently a statically admissible Object/interface assignment may fault on an
excluded concrete representation. This is an experimental completeness gap and
must be removed or reflected in a stronger static capability before production use;
it is not a new permanent union semantic rule.

Rejected: compound fields/static fields, inheritance/constraints, generic arguments,
arrays, Function signatures, managed/unmanaged addresses, reflection tokens and
other unimplemented instruction operands, and compound signatures on instance,
generic, native or InternalCall functions. Reflection construction fails explicitly;
no `UnionTypeInfo`/`IntersectionTypeInfo` API is exposed. Nominal named unions and
Raven's existing `System.Union` carriers are unaffected.

JIT/AOT selection and value layout reject compounds explicitly. The backend gate is
covered by `cargo test -p neoclr-codegen --features neoclr/experimental-compositions
--lib composition_admission_tests`. There is no native execution qualification.

The .NET comparison remains nominal interface casts and dispatch, with ordinary
reference/null behavior. The new runtime expression and signature admission are
neoCLR extensions; reference-only incompleteness, metadata migration, additional
verification paths and backend work are their present costs. No performance benefit
is claimed. Runtime/compiler ownership and pending bridge work are recorded in
[the integration notes](../../raven-cli-bridge.md#structural-reference-composition-experiment--2026-10-14).


### Retained validation for the executable slice

- 10 integration tests pass with the feature enabled: artifact/call normalization,
  parameters/results/locals, guaranteed dispatch, negative narrowing, null handling,
  exclusion of boxed values, GC retention, reference/access checks, malformed input
  and the saved consumer through a NEOX roundtrip.
- 7 test-only Rust runtime-service probes pass with the feature enabled.
- 27 default-build checks pass: class interfaces (7), object identity (6), module
  references (7), scoped types (6), and raw compound admission rejection (1).
- 1 backend admission test passes, confirming explicit compound-storage rejection.
- The saved CLI consumer prints `=> Int32(42)` and exits 42 as designed.
- The metadata host probe's 187 passing assertions are reused; its source is
  unchanged in this executable slice.

Only these focused checks are claimed. No full suite, website build, Windows run,
Raven compilation, published artifact or successful compound JIT/AOT execution.

## Raven compiler API checkpoint — 2026-10-14

The companion branch selectively reuses `02d669acd` (semantic symbols) and
`c4d57d282` (reference conversions). It exposes compiler-created intersections,
order-independent identity, substitution, grouped display and conservative
same-reference conversions. It retains declared-member distinctions and rejects
portable signature storage. The constraint test uses existing comma-separated
constraints; no intersection parser or .NET erasure code was imported.

Fresh and post-change compiler build scripts pass. All 89 focused net11.0 tests
pass (24 new cases, 65 existing controls). The pre-change overload-resolution
suite has one existing inaccessible-metadata-fixture failure out of 415 cases;
the post-change run reproduces the same 414 passes and one failure.
These are compiler API tests using ordinary metadata references, not native
compound import/export or source execution. The companion compiler contract is
`docs/compiler/intersection-types.md`. No runtime implementation changed in this
compiler slice; the earlier interpreter evidence remains applicable.

### Superseded local branches

At the author's direction, after validation the old local experiment branches
were removed and their exact tips retained as annotated archive tags:

| Repository | Removed local branch | Recovery tag | Tip |
| --- | --- | --- | --- |
| neoCLR | `codex/structural-types` | `archive/2026-10-14/structural-types` | `3571ab791` |
| Raven | `codex/intersection-constraints` | `archive/2026-10-14/intersection-constraints` | `6c112574c` |
| Raven | `codex/neoclr-structural-types` | `archive/2026-10-14/neoclr-structural-types` | `b92a0d4a7` |

The historical branch names above identify inspected work. Uncarried commits
remain accessible through the tags. Active experiment branches, main/native
lines and remote branches are unchanged. Archive tags are local and were not pushed.

## Raven source constraint checkpoint — 2026-10-14

Raven commit `620695e16` reuses `08b53d8c4`, `41b684e3f` and `c6a303f0d` from
the archive, following semantic foundation `f4d6f0681`. The runtime checkpoint is
`3d61b17b6`; this source slice changes no runtime implementation.
Raven parses intersection syntax and binds conjunctions in generic constraints to
ordinary nominal bounds. Public semantic queries expose the normalized compound
while `ConstraintTypes` remains flattened. RAV0363 diagnoses value storage;
RAV0364 diagnoses unsupported bounds. No native compound emitter or Runtime
Contract switch is enabled. This is a source/semantic prerequisite, not evidence
of a Raven-generated neoCLR composition program. Ordinary .NET constraint emission
uses CLI generic bounds, with no intersection carrier or new ABI.

The source-slice compiler build script and 1,199 focused net11.0 tests pass,
including syntax/incremental recovery, cold constraint queries, storage rejection,
ordinary CLI constraint metadata, separate compilation consumers and two .NET
execution cases returning 42. The broad pre-change baseline stops at two existing
documentation-ID failures after 1,346 passes. Its 93-case failing batch reproduces
the same 91 passes/two failures after the change; no full-green baseline is claimed.
No runtime code changes were needed for this compiler-only slice.

## Opt-in native declaration reader — 2026-10-14

The host metadata reader now has an explicit `ReadNativeAssembly(image, true)`
profile for bounded compound free-function signatures. The default reader remains
closed to them. `SignatureType.Composition` preserves either/all membership and
snapshot-owned bounds, flattens same-kind nesting, deduplicates operands and
collapses singletons. It does not remove nominal supertypes, distribute mixed
compositions or define a new cross-catalog identity. Bodies remain opaque.

This is a prerequisite for the native compiler mapping, not that mapping itself.
Authored builder signatures, callable import and CLI projection reject compounds;
loaded `Write()` preserves original bytes. Raven still uses the default reader.
The [manual host API reference](../../../api-docs/experimental-metadata.md#read-only-reference-compositions-experiment-2026-10-14)
records public signatures, limits, errors and exclusions. No guest APIs or snapshot
selection changed. Future native emitter admission needs an explicit capability,
compiler/runtime normalization agreement and executable consumer validation.

Validation: `dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --reference-compositions`
passes schema-2/schema-3 NEOX reading, same/mixed nesting, order-independent
equality/hash, snapshot identity, singleton collapse, immutable bounds, original
image preservation, default/CLI/import rejection and malformed/over-budget inputs.
The complete metadata harness reports 170/172 passing versus 169/171 before the
change; the same two existing failures are `Native Object root authoring` and
`Int32 local ownership initialization and execution`. No new failure is introduced.
`python3 scripts/build-api-docs.py --check` passes. Runtime and Raven compiler
behavior are unchanged, so their earlier executable evidence is retained.

## Canonical metadata inspection — 2026-10-14

`ReferenceCompositionTypeInfo` now exposes admitted signatures through the host
metadata facade. Reordered bounds reuse a context-owned view; nominal bounds reuse
the same catalog identities. Separate contexts remain distinct, constituent lists
are immutable, and mixed kinds stay nested. Missing snapshot/dependency resolution
fails explicitly. No subtype reduction, member synthesis or compiler admission is
added. Unlike .NET reflection, this native-only view can preserve a compound
signature; the cost remains an explicit import/emission boundary.

The focused `--reference-compositions` check passes for both binary schemas,
including canonical function result discovery, reordered bounds, context isolation,
immutable bounds and mixed nesting. Public XML documentation builds and the guest
API snapshot check pass. Runtime evidence above is unaffected. Raven still uses
the default reader; semantic import and emission remain open.

## Raven semantic import — 2026-10-14

The companion compiler at Raven `50037e711` now provides an explicit
`NeoClrMetadataReference.ReadExperimentalIntersectionAssembly` API. It maps native
intersection views to compiler-owned symbols for cold/warm semantic queries and
inferred local types. Default readers/project loading remain closed. Normalized
structural unions are rejected, including those nested inside intersections.
Native descriptors retain bounds that compiler nominal-supertype reduction removes.

A focused probe exposed an emission hole for a reduced `A & Derived` signature:
the compiler could previously emit using only the nominal symbol. The new
reference-wide NEOMETA001 guard rejects native emission whenever a reference
contains compound callables, even if unused or normalized to a nominal type.
Both emit entry points preserve output on rejection. This is a deliberate temporary
restriction, not a compound ABI or an executable Raven composition program.

Validation: Raven `NeoClrMetadataProbe --intersection-symbols <core.dll>` passes
schema-2/schema-3 inputs, cold/post-diagnostics queries, reordered canonical identity,
nominal reduction with raw descriptor retention, insufficient-bound rejection,
union/default-reader rejection and emitter output preservation. The existing
`--nullable-symbols` probe passes before/after, using this repository's
`api-docs/reference/NeoCLR.CoreProbe.dll` and metadata checkpoint `97bbcf76b`.
Native storage/call emission is the next capability; structural union symbols and
receiver merging remain open. Runtime evidence above is unchanged.

## Native callable import and emitted consumers — 2026-10-14

The host builder now has a separate `ImportExperimentalCompositionReference` API
for nongeneric native free functions. It preserves the original normalized bounds,
including nominally redundant bounds, while importing their identities into the
consumer. Native call serialization retains Union/Intersection operands. Stack
validation supports exact compound identity (order-independent bounds), and rejects
union-to-intersection mismatches. No new nominal/compound conversion proof rules
are claimed for the host verifier. Ordinary import, authored storage and compound
CLI declaration emission remain closed. This preserves the native contract rather
than choosing a .NET carrier or an erased nominal signature.

Reproduce the focused host check and interpreter executions:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --composition-imports /tmp/neoclr-composition-import-check
cargo build --features experimental-compositions --bin neoclr
for case in Union Intersection RedundantIntersection; do
  target/debug/neoclr run /tmp/neoclr-composition-import-check/${case}Consumer.neox --module /tmp/neoclr-composition-import-check/${case}Provider.neox --execution interpreter --show-result
  test "$?" = 42 || exit 1
done
```

All three callers pass a live Both allocation through imported factory/identity
functions and return `Int32(42)` (exit 42). The redundant intersection retains
`A & Both`, rather than silently reducing its native callable contract to Both.
The provider fixtures amend native signature JSON because general compound
authoring remains closed; the consumers are emitted entirely through the host API.
These results demonstrate native callable transport/execution, not Raven source
emission. Raven's existing `--intersection-symbols` probe still passes and its
reference-wide NEOMETA001 gate remains in place.

The complete metadata harness reports 171/173 passing, adding the import case to
the prior 170/172 result. The same pre-existing `Native Object root authoring` and
`Int32 local ownership initialization and execution` failures remain. Focused
reader/import checks, public XML documentation build and API snapshot check pass.
The next compiler slice must preserve raw descriptors through lowering and call
encoding before admitting executable source use.

## Raven-generated native call consumers — 2026-10-14

Raven `dc750c659`, validated against neoCLR `930ddf60`, now offers
`NeoClrEmitOptions(..., allowExperimentalIntersectionCalls: true)`
for native JSON emission through `NeoClrCompilationEmitter.Emit`. Combined with
`ReadExperimentalIntersectionAssembly`, it admits direct calls such as:

```raven
func Main() -> int => Consume(Echo(Produce()))
```

Produce/Echo/Consume are imported nongeneric free functions with matching native
intersection contracts. The portable call plan carries semantic intersections
without allowing them as general storage types. The native adapter selects the
original callable by token within its exact semantic snapshot and imports the raw
signature. Both artifact-only and explicit snapshot dependency bindings work.
Redundant `A & Both` bounds remain in call operands after compiler normalization.

Four generated programs (nested calls and discarded results, each with ordinary or
redundant bounds) return `Int32(42)` in the opt-in interpreter. The compiler fixture
also verifies default and PE/#Neo rejection, inferred-local rejection, unchanged
output on failure. The follow-up below closes the initial exact-identity gap for
flat intersection call arguments, including contracts that simplify to the same
nominal compiler type.

From the Raven companion worktree, generate the programs with:

```sh
dotnet run --project tools/NeoClrMetadataProbe -p:NeoClrMetadataProject=/Users/robert/Projects/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj -p:WarningLevel=0 -- --intersection-calls /Users/robert/Projects/neoclr/api-docs/reference/NeoCLR.CoreProbe.dll /tmp/raven-intersection-calls
```

Using the feature-enabled runtime built in the preceding section, run from neoCLR:

```sh
for name in IntersectionCallsConsumer ReducedCalls; do
  for suffix in '' Discard; do
    target/debug/neoclr run /tmp/raven-intersection-calls/${name}${suffix}.neox --module /tmp/raven-intersection-calls/${name}Provider.neox --execution interpreter --show-result
    test "$?" = 42 || exit 1
  done
done
```

The existing Raven `--intersection-symbols` and `--nullable-symbols` controls pass,
as do 24 intersection symbol/conversion tests against the rebuilt net11.0 compiler.
No CLI carrier or erasure is introduced. The option defaults off, is not exposed
as a project/CLI switch or Runtime Contract setting, and does not enable PE output,
locals/spills, authored compound signatures, structural unions or JIT/AOT. Imported
cross-dependency bounds requiring a resolver and broader reference conversions
remain follow-up work. Default .NET behavior and milestone sequencing are unchanged.

## Intersection call entailment — 2026-10-14

The host verifier now accepts a flat reference intersection argument when every
required nominal bound is guaranteed by a source bound: either identical scoped
identity or a declared interface relationship (including transitive interfaces).
For example, with Both implementing A and B, `A & Both` can feed `B & Both` or
`A & B`. The reverse strengthening from `A & B` to `A & Both` rejects regardless
of the concrete object returned by the fixture. Union-to-intersection remains
rejected. Original descriptors survive; this is a verification proof, not erasure.

This rule applies only while consuming call parameters. General storage,
nominal/compound projection, mixed composition proofs, class-base traversal and
cross-dependency resolver support remain gaps. Existing graph and descriptor
bounds apply; no new Runtime Contract or transport encoding is introduced.
The same-reference semantics build on the existing .NET nominal interface relation;
first-class intersection signatures remain the documented native-only divergence.
The metadata toolkit owns the proof, Raven owns admission/diagnostics, and the
runtime independently validates the emitted contracts. Wider native verification
and backend support must replace these bounded admission gaps before expansion.

The focused host probe adds three executable positive cases (equivalent contracts
in both directions and weakening) plus a rejected strengthening. All three return
42, including a transitive interface guarantee. Raven's `--intersection-calls`
now emits `ReducedCallsEquivalent.neox`, which returns 42 with
`ReducedCallsProvider.neox`; its four earlier consumers and default/PE/storage
controls still pass. No compiler semantic conversion rule changed.

## Union and cross-kind call entailment — 2026-10-14

The host's flat reference-composition call proof now follows the interpreter's
existing conservative relation for all four outer-kind combinations. For a union
source, every alternative must satisfy the target; for an intersection target,
every bound must be guaranteed. Intersection-to-union needs one guaranteed source
bound implying one target alternative. Nominal implication still uses scoped
identity and the declared interface graph, including transitive interfaces.

This admits `A | Both` to `A | B`, `A & B` to `A | Both`, and
`Both | OtherBoth` to `A & B` when both classes implement A and B. Three new
host-generated consumers return 42 in the feature-enabled interpreter. Focused
negative cases reject reverse union narrowing, an intersection with no guaranteed
union alternative, and a union where one alternative lacks a required intersection
bound. Checks inspect contracts rather than the provider's known allocation.
The existing host import/storage/CLI controls and Raven intersection call probe pass.

Reproduce with `--composition-imports /tmp/neoclr-composition-import-check` as above;
the new executable pairs are named `UnionWiden`, `IntersectionToUnion` and
`UnionToIntersection`, each with `Consumer.neox` and `Provider.neox` suffixes.
No signature bytes, runtime implementation or compiler semantic rules change.
This extends the same-reference native composition experiment; .NET/CLI does not
provide these first-class signatures. The benefit is forwarding values without
carriers or erasure; the cost remains explicit opt-in and bounded proof support.

Raven structural-union import is still closed. Authored storage, nominal/compound
injection/projection, nested mixed-composition proofs, class-base traversal and
cross-dependency resolver support remain gaps. Native metadata/runtime owners must
retain matching verification before expanding compiler or backend admission.
No Runtime Contract configuration, CLI bridge representation or release behavior
changes. Website proposal wording remains accurate; no published capability changes.

## Extendable intersection bounds — author clarification, 2026-10-14

Intersection operands must be interfaces or non-sealed reference classes. The
native reader and runtime reject sealed classes before normalization, including
`Dog & Dog`, even if Dog already implements every other requested interface.
`Animal & Quacks` is admissible when Animal is abstract or non-sealed and Quacks
is an interface; `Dog & Barks` is rejected when Dog is sealed. This supersedes the
earlier broader experimental admission. Value types remain excluded.

Sealed concrete objects can still satisfy interface intersections, and sealed
classes remain valid union alternatives. The rule constrains the written bounds,
not the concrete object. It is an author-selected restriction beyond general set
intersection semantics, which could otherwise express redundant or impossible
sealed-class combinations; the benefit is keeping written bounds extendable, at
the cost of rejecting those expressions. It does not introduce .NET/CLI signatures.

The bounded host reader now fails closed on external intersection leaves whose
extendability cannot be resolved during reading; cross-dependency admission needs
a resolver-aware follow-up. Local raw metadata is checked before deduplication.
Eleven focused runtime tests and the host composition-import probe pass, including
sealed intersection rejection and accepted sealed union alternatives.

## Working Raven consumer POC — 2026-10-14

Run the committed compiler probe and feature-enabled runtime together:

```sh
python3 docs/experiments/structural-compositions/verify-poc.py --raven /path/to/raven-union-intersection
```

The runner builds the runtime, invokes the compiler host against this checkout's
metadata library, runs four consumers, and writes sources, native images and
`validation.json` under `target/structural-composition-poc`. On the validation host,
Xcode's selected linker and CommandLineTools SDK disagreed; selecting the matching
SDK fixed linking without changing system settings:

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk python3 docs/experiments/structural-compositions/verify-poc.py --raven /Users/robert/Projects/raven-union-intersection
```

The main source consumer is:

```raven
func Main() -> int => Consume(AsUnion(AsIntersection(MakeFirst())))
```

The provider declares two sealed concrete classes, each implementing A and B.
MakeFirst/MakeSecond return `First | Second`; AsIntersection accepts that union and
returns `A & B`; AsUnion accepts the intersection and returns `A | B`; Consume
accepts that union and returns 42. The provider fixture amends native signatures,
while every consumer is emitted from Raven source. Both concrete alternatives,
a direct union-to-intersection call and a discarded union result execute successfully.

Raven uses a distinct structural-union symbol, not the existing nominal System.Union
carrier. The explicit composition reader and emit option preserve the original
native descriptors. Default readers/emission, the narrower intersection-only option,
PE projection, unsafe narrowing and inferred locals reject. All four programs
return 42 in the interpreter. This is a working imported-call POC; source-authored
compound declarations/storage and JIT/AOT are not implemented by it.

Author direction: use understandable admission/normalization rules to control
complexity. The POC admits flat, nongeneric nominal reference bounds; sealed union
alternatives are valid, and written intersection bounds must be extendable.
Duplicate removal and bounded nominal proofs are deterministic. Mixed nesting,
wrappers and member merging remain unsupported at compiler call admission.
**Full generic integration, including open and closed types in both unions and
intersections, is the explicitly requested next step.** Closed generics already
have semantic factory/conversion tests; native nongeneric limits and open-type
restrictions are temporary gaps, not the intended final type system.

Final evidence: [POC validation record](poc-validation-20261014.json) pins Raven
`aa542a6f5` and neoCLR runtime/metadata `b4dceb711`, four program hashes/results,
79 compiler tests, 11 runtime tests and passing host/import controls. The runtime
checkout had only the pending runner/documentation/report changes during this run.


## Generic composition first slice — 2026-10-14

The compiler semantic factory now accepts bound type parameters, open constructed
nominal types and value alternatives in structural unions. Owner and method generic
substitution traverses unions, including unions nested inside generic arguments,
and normalizes duplicate alternatives after substitution. Parameter identity includes
its declaring binder. Unbound generic placeholders and Void are rejected. A union is
reference-only/value-only only when all alternatives have that category; this does
not select a runtime storage representation.

The author examples covered by semantic tests are `T | HttpError | JsonError`,
`Request<T> | HttpError | JsonError`, `Task<T | HttpError | JsonError>` and
`Task<Response<T> | HttpError | JsonError>`. Tests close T over Int32 and exercise
value error alternatives, nested substitution, and duplicate collapse where T itself
is an alternative. Open constructed intersections retain substitution coverage;
written sealed/value intersection bounds remain invalid.

The feature-enabled runtime now admits closed constructed reference bounds in
unions/intersections. It validates generic arity/arguments/constraints using ordinary
runtime type validation, preserves constructed identity, and supports interface
dispatch and runtime casts for those bounds. Value generic arguments are permitted
inside a reference bound. Written sealed constructed intersection bounds are rejected
before normalization, while sealed constructed union alternatives are permitted.

Validation: 88 focused compiler tests, 14 runtime structural-composition tests and
13 ordinary generic-class controls pass. Generic runtime cases are assembled neoIL
consumers; they are not evidence of Raven generic source emission. The existing
four-program Raven imported-call POC was rerun against this slice: First, Second,
Direct and Discard all returned Int32(42).

Compared with .NET, ordinary generic argument identity and substitution are reused;
structural composition signature nodes remain a deliberate native extension because
CLI signatures do not encode these unions/intersections. The benefit is preserving
constituent contracts without carrier identity; the cost is new verifier/storage and
compiler integration work. Runtime Contract configuration and the temporary CLI
carrier representation are unchanged. Raven's existing source union binder continues
to use that carrier. The compiler/native metadata adapter still admits only the
nongeneric call POC. Runtime open parameters, value union storage, compositions as
generic arguments (including Task of a union), and JIT/AOT remain follow-up work.
These are implementation gaps, not final type-system restrictions. Runtime/compiler
owners are the paired feature branches `codex/structural-union-intersection` and
`codex/neoclr-union-intersection`; no main-branch capability is implied.

Compiler slice: Raven `f277f2a47`; runtime slice is the commit containing this
section and the closed-generic composition tests. Website proposal status remains
accurate: this is isolated development work, not a published capability.


## Closed generic composition call integration — 2026-10-14

The native metadata composition reader and Raven imported-call profile now admit
closed constructed reference bounds. Local intersection definitions must still be
resolved interfaces/non-sealed classes; external intersection extendability remains
resolver-limited. Type arguments may be ordinary closed value/reference types or
vectors, but not open parameters, Void, or another composition. Existing nominal
arity/constraint validation applies. Rejection precedes duplicate normalization.

Call entailment now retains invariant constructed argument identity and follows
existing nominal interface relationships. `A<int>` cannot prove `A<bool>`.
Raw native descriptors survive semantic import and call emission; generic functions,
compound source storage, composition arguments such as `Task<Response<T> | HttpError
| JsonError>`, value-union representation and JIT/AOT remain unsupported. The four
author examples from the previous slice still have semantic substitution coverage.

The repeatable POC runner now executes eight consumers: the original four and a
`generic/` variant using `A<Int32> & B<Int32>` and `A<Int32> | B<Int32>`, with two
sealed concrete producers implementing both constructed interfaces. All return 42.
The probe rejects mismatched generic arguments at both host writer and compiler,
open/Void/composition arguments, wrong arity, sealed generic intersection bounds
(including duplicates), default and old intersection-only reader/emitter paths,
PE output, unsafe narrowing and authored storage. Sealed generic union bounds read
successfully. Existing host composition-import controls also pass.

This continues the native signature extension rather than altering .NET generic
identity, Runtime Contract configuration or the temporary CLI carrier encoding.
Reusing nominal construction/import avoids a second generic identity model, while
requiring separate adapter and backend validation. Native metadata/codegen support
for compositions inside generic arguments is the next missing layer for Task of a
union. Owners remain the paired experimental branches; published/main support is
unchanged. No website build is needed: the proposals page still labels this work
as exploratory, and the experimental API reference documents the widened profile.

Validated companion compiler slice: Raven `c996229e5`, paired with the metadata
reader/proof and runner changes in this checkpoint. The runner records exact input
revisions, dirty state, source consumers and artifact hashes in validation.json.


## Runtime composition argument storage — 2026-10-14

The experimental interpreter now accepts a closed reference class instantiated with
reference-only union/intersection arguments. A generic field declared as T can store
such a payload; ordinary generic identity, field write checking, free-function
parameter/result passing and object tracing are reused. For example, an assembled
`Envelope<union<Response<Int32>,Left>>` retains its Response payload across repeated
garbage collections and returns 42 after narrowing and field extraction. This is a
plain test envelope, not System.Task or evidence of async support.

Admission validates instantiated fields as well as generic declarations: T[] and T&
fields cannot bypass current compound array/byref restrictions. Wrappers remain
invariant: Wrapper<A | B> cannot be passed as Wrapper<A & B>. Wrong payload writes
reject in verification and execution. Reference unions/intersections as arguments
round-trip native serialization, including reordered bounds. Raw serialized wrapper
signatures retain the feature gate.

This slice allows the existing field-wise neoIL newobj allocation, field access and
nongeneric free-function calls. Constructor calls, owned generic methods, open
composition parameters, value alternatives, explicit compound field declarations,
static fields, arrays/byrefs and JIT/AOT remain outside the admitted surface. Generic
value/interface wrappers are also still gated. The metadata host adapter and Raven
emitter continue to reject compositions inside generic arguments; the next integration
step is native wrapper signature transport and supported owned-call execution before
attempting real Task consumers.

Compared with .NET generic storage, the experiment preserves invariant construction
and field safety, extending only the admitted argument language. Reusing ordinary
reference payload tracing avoids a new carrier allocation; validating substituted
storage adds admission work and deliberately rejects unsupported wrappers. Runtime
Contract settings and temporary CLI carriers are unchanged. Ownership remains on the
paired experimental branches, with no main/published capability change. Existing
website proposal status remains accurate; no website build is needed.

Validation: 19 feature-enabled composition tests, 13 ordinary generic-class
controls and both default-off serialized-signature tests pass.


## Runtime wrapper constructor/method calls — 2026-10-14

Closed reference wrappers with composition arguments now support managed instance
constructors and nongeneric, nonvirtual instance methods through direct call and
null-checking callvirt. Generic substitution preserves payload signatures for
constructor, getter, setter and forwarding calls. Admission traverses instantiated
bodies, including transitive calls, and rejects compound addresses/type tokens and
unsupported storage after substitution. A visited set handles recursive calls;
the experimental closure is bounded to 256 distinct call references. Virtual/override,
abstract, native and method-generic calls remain gated.

Twenty-four composition tests and thirteen ordinary generic-class controls pass.
Coverage includes union and intersection payloads, reordered signatures, recursive
admission, nested payload retention across GC, incorrect constructor payloads and
receivers, null receivers, and hidden unsupported operations inside generic bodies.
The generic declaration controls for those operations still admit ordinary reference
arguments. Default-off evidence from the preceding storage slice is unchanged.

This reuses .NET-like invariant generic ownership, field safety and callvirt null
checking; the native extension is the composition argument itself. Inspecting closed
bodies costs extra admission work and intentionally excludes virtual dispatch until
all possible implementations can be checked. Runtime Contract and CLI encoding are
unchanged. Host metadata/compiler wrapper transport and real Task execution remain
pending; the consumer is assembled neoIL. The website proposal status is unchanged.


## Author direction: value positions and constraints — 2026-10-14

The author clarified that structural unions have broader everyday uses than
intersections: unions are useful for values/results and also parameters and generic
constraints; intersections are usually useful for inputs requiring several
capabilities and for constraints. This is usage guidance, not a blanket prohibition
on intersection results, storage or generic arguments. Existing intersection result
and wrapper experiments remain valid.

The assistant proposed the explicit constraint interpretation `T : A | B` as an
any-of requirement and `T : A & B` as an all-of requirement (schematic spelling,
not a claim of implemented source syntax). Union constraints must preserve their
grouping; lowering them to two ordinary constraints would incorrectly require both.
Inside generic code, an any-of bound guarantees only capabilities shared by every
alternative until narrowed; an all-of bound guarantees every required capability,
subject to the eventual member-conflict rules.

The existing .NET/CLI conjunctive-constraint machinery is reusable for all-of bounds;
a disjunction needs an explicit native contract and compiler checking. No implicit
CLI weakening or conjunctive fallback is acceptable. Union constraints are required
follow-up work, not implemented by the wrapper slices. Existing Raven intersection
constraint syntax/CLI emission remains separate from native composition constraints,
which the runtime currently rejects. Constraint satisfaction, runtime value layout
and compiler source admission must be tracked independently.

The earlier written-bound rule stands: intersections exclude sealed classes/value
types; concrete sealed types may still satisfy interface conjunctions. Union
alternatives may include sealed types, and eventual value-union support must not be
confused with today's reference-only executable profile. Full open/closed generic
substitution remains the target. Native constraint metadata, argument checking and
negative any-of/all-of tests are explicit follow-up requirements.


## Wrapped composition metadata and Raven calls — 2026-10-14

The opt-in metadata reader now retains compositions inside closed, unconstrained
local reference-class constructions in nongeneric free-function signatures. Snapshot
materialization preserves generic argument identity and the original image. An internal
read/import path constructs these read-only signatures without opening ordinary
builder construction or authored storage. Wrapper definitions must be resolved locally;
interface/value wrappers, wrong arity, excessive nesting, open arguments and array/byref
composition wrappers remain rejected. Compositions inside a bound's own generic
arguments remain outside the flat-bound call profile.

Experimental callable import recursively preserves wrappers and their composition
arguments into output-owned references. CLI rejection and Raven composition detection
now recurse through generic arguments, preventing a wrapper from hiding a union from
default/legacy profiles. Raven's call-only lowering carries the complete wrapper type
and imports the exact native descriptor. Wrapper generic arguments remain invariant;
union/intersection entailment is not lifted into a conversion between wrappers.

The runner now executes twelve consumers: eight preceding flat/closed-generic cases
and four wrapper cases (union/intersection, each from host and Raven emission). All
return Int32(42). A native provider allocates Wrapper<A | B> or Wrapper<A & B> through
a generic constructor, passes it through Echo and invokes its getter inside Consume.
Raven emits `Consume(Echo(Make()))`; it does not author the provider's compound source
signatures. Probe checks cover reordered bounds, immutable image retention, wrong
arity, interface/value/array wrappers, invariant wrapper mismatch, default/old-option
and PE/CLI rejection, ordinary import after experimental import, and authored local
storage rejection. Existing host composition-import controls also pass.

This closes wrapper signature transport for the bounded imported-call POC. Real Task
and async execution, source structural storage, open executable parameters, native
union/intersection constraints, value union representation and JIT/AOT remain pending.
The .NET baseline is invariant generic signature identity; the deliberate native
extension preserves composition nodes where CLI has no encoding. The benefit is
lossless contract transport without a carrier; the cost is recursive opt-in checks
and a separate compiler/runtime profile. Runtime Contract settings and temporary CLI
carriers remain unchanged. Owners are the paired experimental branches; website
proposal status remains accurate and no published/main capability is implied.

The author clarified again that input/constraint prevalence is guidance only:
intersections describe combined traits and are valid return/signature contracts.
The intersection-wrapper return consumer explicitly preserves that capability.

Negative compiler coverage exposed a nominal-name identity fallback that erased
composition kind/bounds in generic arguments. Structural-bearing types now use
structural symbol equality instead of that fallback. Reordered bounds retain
identity; differing kinds/bounds cannot become implicit identity conversions.
This is a correction to the experimental semantic integration, not new variance.

Final checks: 89 focused composition compiler tests and eight ordinary metadata
identity controls pass, alongside twelve interpreter consumers, host import controls
and the API-reference snapshot check.

Companion compiler/probe revision: Raven `30576332a`; pair it with the metadata
reader/importer and runner changes in this checkpoint.


## Runtime reference-array storage — 2026-10-14

The next Task prerequisite is reference-vector storage: the Raven Task implementation
uses Promise<T>, whose result slot is an ArrayList<T> backed by arrayref<T>. The
experimental interpreter now admits arrayref of closed reference compositions,
including substituted generic wrapper fields. It admits newarr, array.reserve,
ldelem and stelem with those elements, reusing checked runtime storage and exact
element identity. New arrays contain typed nulls; reserved slots must be written
before reading. Entering an element preserves its object and GC reachability.

This deliberately retains invariant array element identity, unlike CLR reference
array covariance: the existing neoCLR array policy remains unchanged, avoiding
additional composition covariance rules. Compound element addresses, owned/value
arrays, byrefs, direct authored compound fields and generic interface wrappers
remain gated. Ordinary host metadata and Raven source storage gates are unchanged;
this is runtime neoIL evidence, not compiler-emitted Task or async support. Task
integration still needs generic interface admission and the library's remaining
substituted operations checked; this slice does not claim ArrayList<T> integration.

Focused coverage exercises union/intersection default values, reordered element
identity, invalid writes and mismatched accesses with and without verification,
wrapper method substitution, reserved-slot initialization, bounds, collection with
live array payloads, and serialized default-off admission. Existing reference-array,
reserved-array and generic-class controls accompany these checks. No public API or
Runtime Contract setting changes; the website's planned native-structural-value
status remains accurate. Existing ECMA/array design research and the prior native
composition rationale apply; JIT/AOT and stable ABI work remain deferred.

Validation commands (use the matching local SDKROOT where required):

```sh
cargo test --features experimental-compositions --test structural_compositions --test reference_arrays --test reserved_arrays --test generic_classes
cargo test --test structural_compositions
```

The feature-enabled composition suite passes 29 tests; ordinary controls pass 15
reference-array, three reserved-array and 13 generic-class tests. All three
default-off serialized-signature checks pass.


## Generic interface composition arguments and Task construction — 2026-10-14

The interpreter now admits closed composition arguments in nominal interface types,
including interfaces inherited by constructed reference classes. This lets the real
ArrayList<T> result slot retain its collection interface identities when T is a
reference composition. Reference fields remain checked after substitution; generic
value wrappers remain unsupported. Existing nominal interface closure and invariant
generic identity are reused: reordered composition bounds agree, but different
bounds or union/intersection kinds do not become interchangeable interface arguments.
Calls whose declared interface owner contains composition arguments are explicitly
rejected until instantiated dispatch bodies receive their own validation. This is
not a claim of general virtual/interface dispatch support for composition arguments.

The executable Task probe uses the existing Raven-authored System library and
constructs Promise<Response<Int32> | HttpError | JsonError>, retrieves its Task,
checks GetAwaiter preserves identity and verifies IsCompleted is false. It exercises
the actual Promise/Task/ArrayList constructors and pending-state methods, without
altering or replacing that library. Both error classes are sealed union alternatives.
This establishes construction and pending observation only. A completion probe
reached the existing compound-local-address rejection in the instantiated library;
Complete/GetResult, async suspension, source-authored composition signatures and
open executable type parameters remain follow-up work.

This retains .NET-style nominal generic interface contracts and task identity;
the deliberate extension is native composition type arguments. Reusing interface
closure avoids a second conformance model, but does not remove the need to validate
dispatch and remaining substituted operations. Runtime Contract configuration and
CLI bridge encoding are unchanged. The host/Raven metadata profile still excludes
interface wrappers carrying composition arguments; no corresponding compiler or
published/main capability is implied. Website proposal/planned status stays accurate.

Focused checks cover inherited interface projection, direct concrete method calls,
invariant casts and stores (also without verification), dispatch rejection and the
real Task construction consumer. Ordinary class/interface inheritance and Task
contract tests remain the controls; the default-off composition gates are retained.


Validation: 32 composition tests, eight real-library Task tests, eleven interface
inheritance tests and seven class-interface tests pass. Run with:

```sh
cargo test --features experimental-compositions --test structural_compositions --test tasks --test interface_inheritance --test class_interfaces
```

The Task consumer is
`tests/tasks.rs::structural_union_task_construction_preserves_identity_and_pending_state`.
The nested interface-bound admission assertion is also checked separately after
moving that formerly unsupported shape to positive coverage.

All three default-off serialized composition checks also pass.


## Immediate compound local initialization — 2026-10-14

The interpreter admits the compiler's adjacent ldloca/initobj idiom for a local
whose type contains a reference composition. Both instructions must refer to the
exact same local type, and initobj must immediately consume that local address.
This initializes typed null defaults for compositions, reference arrays and nominal
reference wrappers using existing managed initialization. It does not admit general
compound byrefs, address escape, argument/element addresses or indirect reads/writes.
The verifier and interpreter still check the address target type, including a branch
that enters initobj with an incompatible address rather than executing its preceding
ldloca. Generic body admission applies the same checks after substitution.

This retains the .NET-style managed local initialization idiom already emitted by
Raven; it changes experimental admission, not the CLI bridge or public API. The
bounded allowance avoids requiring a second compiler lowering for reference defaults.
Its cost is a deliberately restricted instruction pattern, to revisit with general
managed-reference support. Runtime Contract and compiler feature gates are unchanged.
This removes the local initialization blocker encountered by ArrayList<T>.Add in the
Task completion path; dedicated real-library delivery evidence follows separately.

Validation: 34 feature-enabled composition tests, nine managed-initialization
controls and three default-off serialized-signature checks pass. Commands:

```sh
cargo test --features experimental-compositions --test structural_compositions --test managed_initialization
cargo test --test structural_compositions
```


## Real Task union completion and result delivery — 2026-10-14

The real Raven-authored Task/Promise/ArrayList library now completes and delivers
Task<Response | HttpError | JsonError> in the opt-in interpreter. The consumer
constructs each alternative, completes its Promise, observes IsCompleted, and calls
GetResult to recover the same object and payload value. A second completion with a
different object returns false and preserves the first result. HttpError and
JsonError are sealed classes here; both are ordinary successful Task payloads,
not task faults or cancellation. This distinguishes application error alternatives
from the Task lifecycle, as for a nominal result carrier on .NET.

The consumer clears its own payload reference and creates allocation pressure before
reading the result again, checking that the Task/Promise/ArrayList graph retains the
payload across collection. Negative checks preserve the library's pending/cancelled
GetResult faults, and reject a value outside the union both in the verifier and when
executing without verification. Existing ordinary Task capability/access, cancellation,
identity and invariant-payload tests remain controls.

This is real library execution from native neoIL, with no special Task implementation,
carrier wrapper or altered System library. The preceding runtime initialization slice
is the implementation change; this checkpoint supplies end-to-end evidence. Raven
source-authored structural signatures, compiler async/await consumption of these
results, TaskOutcome<T> carrier access, native composition constraints, value-union
storage and JIT/AOT are not established by these tests. Runtime Contract/CLI encoding
and the narrower host metadata profile are unchanged; publication remains separate.
The website's planned/native-experiment status is still accurate.

Consumer: `tests/tasks.rs::structural_union_task_delivers_each_alternative`.
Negative lifecycle/storage coverage:
`tests/tasks.rs::structural_union_task_checks_pending_cancelled_and_invalid_payloads`.

```sh
cargo test --features experimental-compositions --test tasks
```

Validation: all ten Task tests pass. After adding explicit GC pressure, the delivery
test was rerun separately and passed for all three alternatives. The preceding
runtime slice also passed 34 composition checks, nine initialization controls and
three default-off checks. No website build or public API snapshot update is required:
this adds gated runtime admission and test evidence, with no public API changes.
