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

The assistant initially proposed mutable positional elements and value copying,
with labels excluded from identity. The author subsequently clarified that element
names must be real native structural metadata, not attribute-based aliases. Native
name storage and reflection were required follow-up work, now implemented by the
native name and guest query slices below. The author selected
preserved names with positional type compatibility: renaming elements must not
break compatibility or type equality. The author clarified that tuple type equality
follows the ordered element structure; names are metadata, not part of that equality. The positional-only node at that point did not fulfill this contract; the native
name slices below now preserve names through compiler emission/import and guest reflection. Nominal interfaces still
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

1. **Native signature foundation (implemented):** `Type::Tuple(TupleSignature)` survives
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


## Executable metadata-builder tuple operations

`IILGenerator` now supports `NewTuple`, `LoadTupleElement` and `StoreTupleElement`.
The store consumes a tuple and replacement element, returning an updated value copy;
it never implicitly writes a source local. Typed bodies check exact receiver/element
signatures, while operand admission checks ownership and zero-based position bounds.
Native output reuses aggregate construction and indexed fields, not nominal members.

The separate consumer now constructs a nested tuple, calls the imported echo with
that value, reads elements and changes a copy. It returns 42, checking that the
original remains unchanged. Negative checks cover wrong constructors/receivers/stores,
non-tuple and foreign signatures, and invalid positions. Managed element-address
builder APIs and Raven source lowering remain pending. Existing interpreter support
for managed addresses is unchanged.

[Operation validation](experiments/structural-tuples/operation-results.json) records
the producer/consumer and test-source hashes, commands and expected result.


### Source-local tuple lowering — 2026-10-14

The experimental Raven host option `NeoClrEmitOptions.AllowExperimentalTuples`
(default false) admits tuple literals, parameters/results, locals/copies and named
or positional reads for scalar and nested tuple elements. Native output uses Tuple
signatures, aggregate construction and positional loads, without a nominal tuple
TypeDef/constructor/field. The tuple capability does not enable composition calls.
There is no new CLI/project property; Runtime Contract configuration is unchanged.
Ordinary .NET ValueTuple emission is unchanged. Unlike .NET's nominal carrier and
name attributes, the intended neoCLR contract stores names directly in native
metadata while preserving positional type equality and compatibility.

This slice still binds through the configured nominal tuple family. Source-local
names work, but the current native Tuple node stores element types only: preserving
names across metadata import and reflection is required follow-up work, not completed
by this slice. Do not remove the nominal family yet. Native compiler imports,
element mutation/address lowering, non-scalar elements, tuple arrays/byref parameters
and open generic tuple elements remain outside this compiler profile.

Validation: `NeoClrMetadataProbe --source-tuples <bundle> <output>` emits a nested
tuple consumer using named reads and copies; the matching experimental interpreter
returns 42. The probe checks ordered native signatures, absence of a local nominal
tuple declaration, default rejection, and atomic rejection of mutation/open tuples.
The source composition regression also passes. The runtime library fixture is
`target/tuple-info-libraries-v2` from compiler
`793219d855b17f01fb805e215968b78871eb3804`; the emitting compiler is the dedicated
`codex/neoclr-union-intersection` branch, runtime `codex/structural-union-intersection`.

[Source validation](experiments/structural-tuples/source-results.json) records the
compiler revision, fixture hashes and commands. The [tested source](experiments/structural-tuples/SourceTuples.rvn)
includes renamed element access with unchanged positional type compatibility.


### Native names: runtime metadata and host inspection

The next slice implements direct native names in Rust TupleSignature, preserving
legacy unnamed encoding. Type equality/hash and resolved identity ignore names;
per-signature host descriptors retain them. Names survive substitution and native
roundtrips, and renamed parameter contracts execute (42). See the
[API and validation boundaries](../api-docs/tuples.md#native-tuple-element-names-2026-10-14-development).
C# authoring/reading, Raven emission/import and guest name queries still need wiring;
the earlier source-local test does not yet emit names.

[Native-name validation](experiments/structural-tuples/name-results.json) records
the focused test counts, commands and source hashes.


### Native tuple names: C# and Raven emission — 2026-10-14

The C# tuple builder now accepts ordered optional names; native writing, experimental
reading and callable import preserve them recursively. Names remain excluded from
SignatureType/TupleSignature equality and hashing. Per-signature descriptors retain
different names on structurally equal parameters and results; admission sets do not
intern away those names. Unnamed encoding remains compatible, and the explicit
experimental reader/CLI-output gates remain in place.

Raven's existing `NeoClrEmitOptions.AllowExperimentalTuples` host opt-in now emits
element names directly in native Tuple payloads. It emits each bound element's
effective name, including default `ItemN` names. A parameter `(left: int, right: int)`
and result `(x: int, y: int)` retain their respective names and remain type-equal.
No name attributes or nominal carriers are emitted. Runtime Contract settings and
ordinary .NET behavior are unchanged; no CLI/project option is added.

The C# separate producer/consumer and Raven source-local consumer both execute with
result 42 on the matching `experimental-tuples` runtime. Checks cover recursive name
preservation, rename equality/hash, immutable input, invalid names/serialized shapes,
ordinary-reader rejection and unsupported-source gates. Raven native tuple import
and guest name queries still require separate implementation. Nominal tuple symbols
remain a binding dependency; tuple mutation/open elements remain outside this
compiler profile. The tested library bundle remains `target/tuple-info-libraries-v2`
from compiler `793219d855b17f01fb805e215968b78871eb3804`; emitting compiler and runtime
remain on their dedicated structural feature branches.

[Emitted-name validation](experiments/structural-tuples/emitted-name-results.json)
records both executable outcomes, source/image hashes and commands.


### Raven native tuple import — 2026-10-14

`NeoClrMetadataReference.ReadExperimentalTupleAssembly(image, bootstrap, resolver)`
now reads named native tuple contracts for semantic import. Register the reference,
its matching primitive bootstrap and dependencies explicitly; this does not add a
project/catalog flag or admit compositions. Default native readers still reject
tuples. Emission of this reference requires `AllowExperimentalTuples`; without it
the emitter reports NEOMETA001 before output. Imported nongeneric tuple-bearing free functions use the exact
reader snapshot through `ImportExperimentalTupleReference`, preserving names and
preventing a signature-erasing fallback.

C# `Introspection.TupleTypeInfo` projects ordered element types and names into Raven
tuple symbols. Parameters `(left: int, right: int)` and results `(x: int, y: int)`
retain different names but compare structurally equal. A separate native producer
and consumer exercise nested named tuples, string elements, local reads and calls;
the interpreter returns 42. Ordinary-reader and disabled-emitter rejection are
covered, alongside an ordinary generic function in the same library. Runtime Contract settings and ordinary .NET behavior remain unchanged.

The bounded compiler profile supports scalar/nested tuples with the configured
nominal tuple family still supplying binding symbols. Open elements, tuple mutation,
carrier-independent symbols, project/catalog integration and guest reflection name
queries remain follow-ups. The matching fixture uses `target/tuple-info-libraries-v2`
from compiler `793219d855b17f01fb805e215968b78871eb3804`, with the current structural
feature branches for emitter and runtime.

Deferred general candidate: calling `Compilation.GetAssemblyOrModuleSymbol` on a
new compilation before reference initialization exposed a NullReferenceException.
This slice reads that symbol after the emit pipeline has initialized references;
it does not claim to fix or validate the cold assembly-query API. A general fix
should be validated independently on Raven main before deliberate carry-over.

[Import validation](experiments/structural-tuples/raven-import-results.json) records
the separate consumer, fixture hashes and focused checks.

### Guest tuple name reflection — 2026-10-14

`System.Introspection.TupleTypeInfo.ElementNames: Sequence<string>` now exposes
ordered effective native names, using ItemN for unnamed positions. The runtime
queries the retained signature descriptor, not its name-free resolved identity.
Consequently equal tuple types can report different names; nested descriptors keep
their own names. Snapshots are independent. Identity-only reconstructed handles
report positional defaults. Unlike .NET TupleElementNamesAttribute, these names
live directly in native signature metadata; the cost is carrying descriptive data
separately from canonical type identity. The effective-name guest API intentionally
does not distinguish an explicit ItemN from its default; host APIs retain that detail.

This requires `experimental-tuples` and matching rebuilt source libraries. It adds
no Runtime Contract flag or CLI encoding; ordinary .NET output is unchanged. The
native TypeTupleNames runtime service backs the source provider. CLI reference
bindings exist only for compiler/documentation bootstrap. The guest fixture tests
two equal renamed native tokens, nested names, unnamed defaults and 180 repeated
queries per signature. The fixture isolates reflection from source tuple lowering.

Validation uses `target/tuple-names-libraries-v2` built with compiler
`793219d855b17f01fb805e215968b78871eb3804`, the existing explicit primitive bootstrap,
and the structural runtime branch. See
[guest name evidence](experiments/structural-tuples/name-info-results.json).
Tuple mutation, carrier-independent Raven symbols and project/catalog opt-in remain
separate follow-ups.

### Native local tuple mutation — 2026-10-14

With `AllowExperimentalTuples`, assignments to named or positional elements of a
local tuple now lower to native updated-copy stores. Nested paths rebuild each
containing tuple and write back the root local; earlier copies remain unchanged.
The RHS is evaluated once before reading the root for reconstruction, preserving
any RHS changes to sibling elements. Ordinary .NET emission is unchanged. This
uses native Tuple signatures and StoreTupleElement, with no CLI carrier encoding.

The bounded profile accepts local-rooted paths. Parameter/field/property/array-rooted
mutation, managed element addresses and open elements remain unsupported and fail
before publishing output. Binder mutability rules still apply. Runtime Contract
settings are unchanged; the existing explicit experimental emitter gate is required.

Validation: `NeoClrMetadataProbe --source-tuples` covers direct and nested writes,
named/positional reads, preserved copies and unsupported parameter mutation. The
TupleMutation consumer returns 42 in the interpreter with `tuple-names-libraries-v2`
(compiler `793219d855b17f01fb805e215968b78871eb3804`). The pre-change source tuple probe
passes; focused .NET tuple semantic/emission checks guard the ordinary target.

### Native tuple binding without nominal carriers — 2026-10-14

The explicit neoCLR Runtime Contract now offers
`MetadataImportOptions.UseStructuralTuples` (default false) and immutable
`WithStructuralTuples(bool enabled = true)`. With this option, tuple syntax and
native tuple imports produce structural symbols directly; they never look up or
construct System.Tuple. Native tuple emission now requires both this binding option
and `NeoClrEmitOptions.AllowExperimentalTuples`. Existing experimental hosts must
add `WithStructuralTuples()` to their metadata import options. Non-neoCLR targets
reject this option; CLI emission rejects the structural profile before output.
There is no project/catalog switch yet. Ordinary .NET and the legacy CLI bridge
keep their existing nominal tuple representation.

The native symbol has TypeKind.Tuple, no UnderlyingTupleType, assembly ownership,
metadata declaration name, constructors, inherited nominal members or interfaces.
Ordered TupleElements carry native names; ItemN aliases remain available for source
access. TypeArguments is empty: elements are not generic carrier arguments. Tuple
identity/hash uses ordered recursive element types, ignores names, and remains
distinct from nominal tuples. Substitution preserves the native shape and names.
Semantic lookup/display uses these fields directly; no syntax or TextMate change
is needed. This is the semantic counterpart of the existing native signature API,
not a new CLI transport.

Compared with .NET ValueTuple, the benefit is no arity-specific declaration or
attribute-name dependency. The cost is a dedicated structural symbol and explicit
identity/substitution handling. Native mutation requires a mutable local root;
immutable let roots and nonlocal roots reject before output. A general review of
value-type field writes through immutable bindings belongs on Raven main; this
slice enforces the native writeback boundary without changing .NET policy.

Validation covers carrier-free factory construction with no references, positional
identity/hash (including nested/repeated types), type-parameter substitution, member
names/display, target rejection, CLI rejection, source/import symbols, and separate
interpreter consumers. A nine-element tuple executes and mutates without any
System.Tuple`9 declaration; it returns 42. Nested mutation/copy and imported tuple
consumers also return 42 using `tuple-names-libraries-v2`. Focused ordinary .NET
tuple tests retain the existing carrier behavior.

This removes Raven's nominal backing dependency for the native structural profile.
It does not delete the old tuple declarations from the runtime library or CLI
bootstrap: those remain for existing bridge consumers pending bundle migration.
Open-element emission, nonlocal mutation, managed element addresses and broader
backend support remain separate work.

[Carrier-free symbol validation](experiments/structural-tuples/symbol-results.json) records the focused tests and executable consumers.
