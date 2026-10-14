# Structural tuple migration (development)

Author direction, 2026-10-14: continue with structural tuples and remove nominal
tuple types. This changes the earlier choice of `System.Tuple<T...>` as the neoCLR
representation. It does not direct changes to ordinary Raven/.NET ValueTuple output.

## Contract and rationale

The target is a value type whose identity is its ordered sequence of element type
identities, without a Tuple TypeDef, generated class name or generic carrier.
Position, duplicates and nesting matter: `(int, string)` differs from `(string, int)`,
`(int, int)` keeps both elements, and a nested tuple is not flattened. A one-element
metadata shape remains distinct from its element; this does not introduce source
syntax for single-element tuples. Empty source `()` retains the existing unit rule.

The assistant's migration defaults preserve mutable positional elements and value
copying, with label aliases excluded from type identity. Nominal interfaces still
require declared conformance; tuple shape does not implicitly implement arbitrary
interfaces. Synthesized positional access and deconstruction are runtime operations,
not proof of nominal interface conformance. Boxing, equality, interface contracts,
layout and tuple descriptor factories need explicit implementation and validation.

The [C# tuple reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-tuples)
(checked 2026-10-14) documents mutable value tuples, positional access and names
ignored in assignment/equality. The existing [tuple research](tuples.md) records the
ValueTuple library implementation. Preserve those useful value semantics while
replacing the nominal representation by author direction. The benefit is direct
signature identity and introspection without an arity-specific nominal family; costs
include runtime storage/GC/verifier work, synthesized member handling, backend work
and binary migration. No performance benefit is claimed.
The one-to-256-element bound is an experimental metadata resource limit, not a
new permanent source-language arity rule. Older native readers do not recognize
the new Tuple node; use matching tools rather than expecting binary equivalence
with the existing nominal tuple family.

## Sequenced migration

1. **Native signature foundation (implemented):** `Type::Tuple(Vec<Type>)` survives
   native JSON and schema-3/4 binary envelope roundtrips without nominal declarations.
   Type/method/Self substitution traverses elements and retains the depth budget.
   Scope binding preserves order, duplicates and nesting, with one to 256 elements.
   The later host identity slice resolves admitted closed signatures.
2. **Runtime values (bounded interpreter slice):** opt-in construction, positional
   reads/writes, managed element addresses, default initialization, copying and
   nested/reference elements now use structural signatures. Native memory layout,
   synthesized Deconstruct reflection and broader element forms remain pending.
3. **Introspection and authoring (host identity implemented):** resolved tuple
   identities and ordered host element descriptors are available. Guest `TupleTypeInfo.ElementTypes` exposes recursive ordered snapshots. Native C#
   signature authoring is implemented below; reader/import and synthesized member reflection remain.
4. **Raven:** carrier-independent tuple symbols and native lowering/import, preserving
   labels and .NET target behavior behind explicit capabilities. No CLI approximation
   should be mistaken for native tuple identity.
5. **Removal:** migrate tuple source/runtime consumers, compiler mappings, generated
   artifacts, API snapshots and examples; remove nominal neoCLR Tuple declarations
   and legacy admission after the replacement consumer passes. Explicit nominal
   constructor uses must migrate; do not silently reinterpret old binaries.

Native executable admission requires `experimental-tuples` for tuple signatures,
including wrappers. Enabling `experimental-compositions` alone does not authorize
tuple execution. The interpreter slice admits closed scalar, nominal and nested
structural tuple elements. It rejects Void, erased Value, RuntimeTypeHandle, open
parameters, byrefs, pointers, arrays, functions and compositions as direct elements;
these are bounded implementation gaps, not permanent type-system restrictions. Decoding a native
metadata document alone does not validate storage or executable bodies. Old nominal
Tuple signatures remain distinct and usable until stage 5. Existing installed VS
Code kit binaries remain the validated union/intersection snapshot.

## Focused evidence

`tests/structural_tuples.rs` covers declaration-free JSON/binary roundtrips, ordered
signature equality, nested type/method/Self substitution, missing arguments, depth
and arity limits, and explicit executable rejection. Use
`cargo test --test structural_tuples --features experimental-compositions`.
That configuration tests the default execution gate. With
`--features experimental-tuples,experimental-compositions`, the same target also
checks construction, nested fields, copy mutation, managed element addressing,
reference-element survival across GC and invalid element/index/store rejection.
The guest descriptor slice below adds element reflection; Raven lowering and nominal-family removal remain pending.

Metadata foundation validation (2026-10-14): all six structural tuple tests pass
with the composition feature enabled; the existing binary/JSON metadata equivalence test passes;
`cargo check -p neoclr-codegen` and the API snapshot check pass. No runtime tuple
execution was claimed by that metadata-only slice. The follow-up interpreter
configuration passes 12 checks; the default gate still passes six checks.

## Interpreter storage slice

`tuple<Int32,String>` in neoIL produces a structural signature. Existing `newobj`
aggregate construction and indexed `ldfld`/`stfld`/`ldflda` operations work against
synthesized public positional members (`Item1`, `Item2`, ...), without a nominal
constructor or field declaration. `Module::instantiated_fields` returns those
synthesized host field views. The interpreter uses its existing value aggregate
storage, retaining structural type identity in the value. Reusing storage is not
mapping the signature to a nominal Tuple family.

Copies recursively copy value aggregates; managed reference elements retain object
identity. Typed verification checks element order/types and ordinal bounds. Existing
managed slot access enforces readonly borrowing. GC traces element values through
its existing aggregate traversal. Default initialization recursively initializes
supported elements. The resource limits from the signature stage remain in force.
JIT/AOT selection explicitly rejects tuple signatures, including nested uses. Native
layout and compiler lowering remain separate work; guest TupleTypeInfo is covered below.

The [native value example](experiments/structural-tuples/values.neoil) constructs a
pair, modifies a copy, and returns 42 from the original. Run it with a runtime built
using `--features experimental-tuples`; the ordinary runtime rejects the signature.
This is a native runtime example, not yet a Raven source sample.

The interpreter [validation record](experiments/structural-tuples/results.json)
records 12 opt-in tests, six default-gate tests, one native-selection rejection test
and the example's expected exit code 42. API snapshot validation also passes.


## Resolved host identity slice

`TypeIdentity::Tuple` retains the ordered resolved keys of each element, preserving
nested shape and repeated elements. It roundtrips to the native signature without a
nominal definition or metadata token. `TypeDescriptor::tuple_elements` exposes the
ordered element descriptors independently of generic arguments and union/intersection
constituents. The existing executable admission gate applies to host resolution too.
This prepares guest introspection without claiming a guest provider is implemented.

Focused validation: the host identity unit test checks descriptor shape, recursive
identity roundtrip, absent definition/token, reordered and flattened alternatives,
repeated elements and the distinction between a single-element tuple and its element.
The existing 12 structural tuple checks continue to pass with both experiment flags.


## Guest descriptor slice

The runtime selects `System.Introspection.TupleTypeInfo` for native tuple handles;
`ElementTypes` returns ordered element snapshots, preserving duplicates and nesting.
Shape query 16 identifies tuples internally, independently of nominal and composition
classifications. Both source-backed and fallback descriptor-provider dispatch use
`RuntimeTupleTypeInfo`. A nominal type cannot be passed to the tuple element query.

The `verify-type-info.py` consumer runner builds Raven code, substitutes one explicit
native tuple token, and checks recursive descriptor dispatch repeatedly. This is a
fixture for runtime introspection; it does not implement Raven tuple emission.
The source runtime library uses validated compiler revision
`793219d855b17f01fb805e215968b78871eb3804`; the current experimental compiler rejects
existing `(flags & flag)` gzip code as an intersection type. That parser regression
is a deferred general candidate, not a tuple semantic restriction. The consumer uses
the experimental branch compiler. All binaries must be rebuilt together.


Validation (2026-10-14): the Raven consumer exits 0 with `Tuple descriptors passed`
after 180 iterations; two host identity/query tests pass, including rejection of a
nominal handle by the tuple element query. Six default-feature tests preserve
explicit opt-in rejection. The matching API reference snapshot check
passes. [Artifact hashes and results](experiments/structural-tuples/type-info-results.json)
record the tested source-library bundle, runtime, fixture and consumer.


## Native C# authoring slice

The metadata builder now creates ordered native tuples through
`CreateExperimentalTuple`, exposes immutable `TupleSignature.ElementTypes`, checks
exact tuple identities in typed bodies and emits native tuple nodes. It copies input
arrays, validates closed supported elements/ownership and resource limits, rechecks
the graph on write, and rejects CLI projection. The focused authoring fixture includes
nested/repeated elements, a closed generic nominal element, an exact-signature echo
and a tuple type token; its native binary runs with expected result 42.

Native C# reader/import, tuple construction/element-operation builder APIs, and Raven
source lowering remain pending. This completes the signature authoring portion of
stage 3, not the complete compiler path. See the manual host API reference for the
16-level tooling limit; it is narrower than the native metadata nesting budget.

[Authoring validation](experiments/structural-tuples/authored-results.json) records
the native image and test-source hashes. Existing authored composition checks and
the API snapshot check also pass. No website build is needed for this host API slice.


## Native C# reader/import slice

`ReadExperimentalTupleAssembly` preserves tuple shapes in immutable declaration
snapshots without enabling compositions. `ImportExperimentalTupleReference` explicitly
imports nongeneric native free functions, retaining ordered tuple signatures and
closed generic nominal element identities. Ordinary read/import paths remain closed,
including after import-cache population. External nominal definitions resolve through
explicit matching snapshots. Malformed/open tuples reject; bodies stay opaque until
runtime verification. Tuple-bearing array/byref/function imports remain unsupported.

The separate consumer returns 42 through an imported producer method and includes
native tuple-forwarding bodies. Host checks cover snapshot image preservation,
nesting/multiplicity, re-emission, closed generic elements, missing/resolved dependencies
and malformed/default-gate rejection. This is signature and linkage validation;
C# tuple value construction/element operations and Raven source lowering remain next.

The [import validation record](experiments/structural-tuples/import-results.json)
contains the producer/consumer hashes and commands. Existing composition reader
regressions and the API snapshot check pass.
