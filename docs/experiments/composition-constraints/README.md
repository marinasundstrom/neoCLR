# Structural generic constraints — development slice, 2026-10-14

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
