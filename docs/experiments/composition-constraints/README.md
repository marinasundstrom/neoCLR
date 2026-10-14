# Structural generic constraints — development slice, 2026-10-14

Current acceptance scope and consolidated reproduction: [closed-reference POC](../structural-poc/README.md).
The sections below retain historical implementation checkpoints.

The experimental interpreter admits closed reference union/intersection signatures
inside the existing `GenericConstraint.TypeBound` metadata field, for both type and
method parameters. No schema version or nominal carrier is added. JSON and native
signature identity preserve the any-of/all-of grouping.

Compared with ordinary CLR conjunctive nominal constraints, this adds a native
union bound and explicit structural grouping. The benefit is preserving a disjunctive
contract; the cost is native-only metadata and additional proof rules. Existing
nominal bounds and special constraints keep their existing checks. These rules reuse
the relationship research in the structural composition experiment.

Every union alternative supplied as a type argument must satisfy the target bound.
Intersection bounds require every trait; union bounds require membership in an
alternative. A closed concrete class may satisfy interface intersections. Closed
classes cannot be written as intersection operands. Leaf checks preserve nominal
interface/Self conformance and inheritance, without numeric or user conversions.
Closed generic reference operands retain invariant argument identity.

`native.neil` exercises native generic methods with union/intersection constraints;
its result is 42. The runtime tests also inspect generic class instantiation, reject
partial-overlap union arguments, check closed generic bounds, retain metadata on
roundtrip and reject invalid host method specializations without prior verification.
Run `cargo test --features experimental-compositions --test composition_constraints`.
Without the feature, the explicit default-profile rejection test runs instead.

The companion Raven semantic checker now understands structural constraint bounds,
including type/method parameter substitution. Previously an unhandled structural
bound could fall through as satisfied. The new tests cover positive/negative type
and method uses, union coverage and parameter forwarding. Source structural unions
remain selected by RavenNeoClrStructuralCompositions; ordinary .NET behavior stays
nominal. Existing Raven intersection syntax still flattens into conjunctive bounds.

This is not end-to-end Raven native constraint emission. The host metadata builder still has its narrower nominal constraint profile;
the compiler backend rejects unsupported constraint emission. Next work must extend
the authoring/emission profiles together and prove separately compiled
source consumers. Open structural bounds, mixed source formulas, generic code using
all guaranteed members and guest constraint reflection remain incomplete. Composition
storage/generic-method specialization restrictions and JIT/AOT gates are unchanged.

Validation: 4 native composition-constraint tests, 1 default-profile rejection test,
60 existing runtime regression tests and 18 dedicated Raven semantic tests pass.
The earlier compiler run also exercised the existing intersection-constraint suite.
See results.json for exact focused commands and source hashes.

Metadata inspection follow-up: opted-in native snapshots now preserve closed method
TypeBound compositions as full signatures and expose context-owned structural type
views through `GetTypeConstraints()`. JSON and binary images retain their original
bytes. The nominal-interface queries reject structural bounds instead of truncating
them. Dedicated metadata checks pass, including final-class rejection with closed
hierarchy acceptance and the existing nominal method-constraint regression checks.
This is host inspection only; native authoring/import, Raven emission, generic-class
constraint materialization and guest reflected constraints remain pending.

Native method authoring follow-up: `MethodBuilder.AddExperimentalStructuralConstraint`
now emits closed, local structural bounds directly as TypeBound signatures. Nominal
inheritance/interface checks and closed construction identity support any-of/all-of
proofs with complete source-union coverage. Calls are checked again at serialization,
so late constraint changes cannot bypass admission. External bounds, open forwarding,
generic trait calls and Raven compiler emission/import remain unsupported. The
`AuthoredStructuralConstraintChecks` fixture executes with result 42; focused authoring,
inspection and nominal constraint regression checks pass. This completes the bounded
host method-authoring step, not end-to-end Raven constraint compilation.

Raven emission/import checkpoint: closed local method constraints now compile into
native TypeBound signatures. Source conjunctions become intersection bounds. The
native catalog exposes complete bounds to Raven symbols and detects compositions
that occur only in constraints. Calls use the exact-snapshot constrained import API;
unsupported symbol-only imports cannot erase bounds. The source probe emits a library
and separate consumer, exercises union/intersection and `Response<A> | B` bounds,
rejects invalid imported arguments and runs with interpreter result 42. The ordinary
catalog/backend gates remain closed. This supersedes earlier pending method-emission
and free-function-import notes, but does not implement open structural constraints,
class-parameter constraints, constrained instance imports or generic-body trait use.

Run the Raven `tools/NeoClrMetadataProbe` command `--structural-constraints` with
`target/composition-info-libraries-v2` and `target/source-constraints` absolute paths.
Run `ConstraintConsumer.neox` with `ConstraintLibrary.neox` as a module plus the
matching System runtime seed/module/object root. See results.json for commands.
The existing source composition probe and metadata nominal/composition regressions
pass; there are no syntax or token classification changes in this wiring slice.

Closed-bound method forwarding checkpoint: native call admission now derives a
method parameter’s guarantee from the conjunction of its caller’s declared bounds.
Union-to-union and intersection-to-intersection forwarding pass; separate nominal
interface bounds can jointly prove an intersection. Union-to-intersection and
unconstrained forwarding reject. Nominal callee bounds retain existing Self handling
unless a structural caller requires structural entailment. The metadata fixture
and Raven exported wrappers execute with result 42. This supersedes the blanket
method-forwarding gap above, while open bound operands, owner parameters, external
bound authoring and generic-body trait calls remain outside this profile.

Generic-body interface use checkpoint: an owned nongeneric interface contract may
now be invoked through a method parameter whose structural bound guarantees it.
Raven’s local intersection fixture calls both interface methods and returns 42;
the native test also accepts a union whose every alternative guarantees both traits
and rejects partial-coverage unions. Existing borrowed callself handles the receiver;
nominal leaf/Self checks remain intact. This supersedes the generic-body-trait gap
for this profile only. External/constructed targets, owner-parameter authoring and
open structural bound operands are not expanded by this change.


2026-10-14 imported trait dispatch checkpoint: the explicit native composition
profile now admits completed imported nongeneric interface methods through a
structurally constrained method parameter. The receiver remains a managed parameter
reference, encoded with borrowed `callself`. Every union alternative must guarantee
the requested interface; intersections can supply it through nominal inheritance.
Nominal-only static calls retain their existing validation. The Raven
`--structural-constraints` probe compiles `TraitLibrary.neox` separately, then emits
`ExternalTraits.neox` whose local `LocalA & LocalB` bounds inherit the imported
interfaces. Its two method calls return 40 + 2 = 42 in the native interpreter.
This adds no CLI carrier or metadata schema: default composition rejection remains.
Direct external structural-bound authoring, constructed interface targets under
structural bounds and open bound operands remain pending. The host metadata emitter
owns admission; Raven retains its existing imported-interface emission path.


Closed constructed trait targets (2026-10-14): native structural method bounds now
also authorize imported constructed interface calls when their closed guarantees
prove the exact target construction. `Reads<int>` does not prove `Reads<string>`;
every source-union alternative must prove the requested construction. Existing
nominal/Self constrained calls retain their prior path. Raven's `ConstructedTraits`
probe inherits `ImportedReads<int>` through a local intersection constituent and
calls it using the existing borrowed native dispatch. No CLI encoding is added.
Direct external structural-bound authoring and open operands remain unsupported.


Nominal interface contract (author clarification, 2026-10-14): intersections make
existing nominal interfaces composable. Types must explicitly implement those
interfaces, directly or through declared inheritance. Structural identity belongs
to the union/intersection signature; it does not introduce member-shape matching,
implicit conformance or Rust-style traits. References to “traits” in earlier probe
notes describe guaranteed interface capabilities, not a separate runtime trait
system. A future compiler-level feature simulating trait characteristics is an
exploratory possibility, not an implementation commitment or a runtime contract.
