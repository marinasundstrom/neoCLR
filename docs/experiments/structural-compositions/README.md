# Structural union/intersection experiment

Status: branch-only interpreter signatures behind `experimental-compositions`,
2026-10-14. Raven source support, JIT/AOT storage, public reflection APIs and a stable
ABI remain unavailable. The earlier probes below retain their narrower evidence.

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
