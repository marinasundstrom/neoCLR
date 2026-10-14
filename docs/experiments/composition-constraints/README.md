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

This is not end-to-end Raven native constraint emission. The host metadata builder
and declaration materializer still have their narrower nominal constraint profiles;
the compiler backend rejects unsupported constraint emission. Next work must extend
those profiles together, preserve reflected grouping, and prove separately compiled
source consumers. Open structural bounds, mixed source formulas, generic code using
all guaranteed members and guest constraint reflection remain incomplete. Composition
storage/generic-method specialization restrictions and JIT/AOT gates are unchanged.

Validation: 4 native composition-constraint tests, 1 default-profile rejection test,
60 existing runtime regression tests and 18 dedicated Raven semantic tests pass.
The earlier compiler run also exercised the existing intersection-constraint suite.
See results.json for exact focused commands and source hashes.
