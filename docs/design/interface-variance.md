# Interface variance and result-value conversion

Status: runtime metadata and bounded interpreter reference conversions implemented
in development, 2026-10-15. SDK annotation authoring, inspection and CLI/native metadata transport are implemented.
Raven native integration and AOT/JIT execution are being completed. The wider Result
proposal remains unimplemented. Structural-type work stays parked. This follows the
[implemented slicing contract](slicing-capabilities.md).

## Concrete requirement

The author proposed using a `Sliceable<int[]>` as a
`Sliceable<Iterable<int>>`, then directed exploring beyond .NET's restrictions,
while noting that some limits remain useful and supplying Microsoft's variance
guide. Treat this example as a design acceptance case, not a currently supported
assignment. Keep nominal interface conformance explicit.

The array-to-Iterable conversion preserves the same array. The fact that its
elements are integers is irrelevant: the type argument being widened is the
array itself. This does not request `int[]` to `Object[]`, or element covariance.

The method returns `Result<TSlice, InvalidRangeError>`. Therefore a complete
solution must preserve both cases while converting the success payload:

- `Ok(array)` becomes `Ok(iterable view of that same array)`.
- `Error(error)` preserves the error and does not access a success payload.

Merely annotating Sliceable's parameter is insufficient while Result is invariant.
The slice allocation already required by the receiver is distinct from any cost
introduced by widening its returned result.

## Baseline and current gaps

Sources inspected 2026-10-15:

- [Author-supplied C# variance guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/concepts/covariance-contravariance/): direction of producer/consumer conversions and the unsafe-write problem with covariant arrays.
- [C# specification §19.2.3](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces#1923-variant-type-parameter-lists): declaration-site annotations, variance safety and conversion rules.
- [.NET generic variance](https://learn.microsoft.com/en-us/dotnet/standard/generics/covariance-and-contravariance): variant interfaces/delegates and reference-type argument restriction.
- [Scala variance](https://docs.scala-lang.org/tour/variances.html): parameter variance on generic classes as well as producer/consumer roles; invariant mutable containers. This establishes an alternative language model, not a ready-made native value ABI.

C# exposes declared variance on interfaces and delegates. Its conversions use
identity/reference conversions, not arbitrary numeric widening, boxing or
user-defined conversions. A nominal Result class or value type cannot simply be
declared variant in C#. Separate CLI metadata/runtime guarantees from C# syntax
when implementing neoCLR; this investigation does not claim a CLR implementation
experiment has been run.

neoCLR's current metadata builder explicitly exposes invariant generic interfaces
(`tools/metadata/NeoCLR.Metadata.Experimental/Model/InterfaceBuilder.cs`).
`Iterable<T>` produces `Iterator<T>`, which produces T; those contracts are also
currently invariant. `Result<T,E>` is a nominal union implementing
`Propagatable<Result<T,E>,T,E>`, with output extraction through `out` parameters.
Its full inherited and generated member surface needs auditing before variance can
be advertised. Read-only case data alone does not prove a declaration is safe.

## Preserve the safety restrictions

Provisional constraints for the next implementation design:

1. Validate explicit declaration-site intent across all instance members, base
   interfaces and substituted generic signatures. Compose nested input/output
   polarity. Reject contradictory uses; do not trust annotations from a frontend.
2. Writable fields, setters, replaceable array elements and ordinary ref aliases
   remain invariant. MutableSequence/List must not inherit a producer conversion
   that permits writes of a wider type into narrower storage.
3. Keep interface conformance nominal. A variant view derives from an explicitly
   implemented interface and a validated conversion; matching member names is
   insufficient.
4. Initially permit only identity and admitted nominal reference upcasts at the
   leaves. Do not fold numeric conversions, downcasts, user code or automatic
   boxing into variance. Value-type arguments remain exact unless a separately
   validated value conversion applies.
5. Preserve exact generic type identity and reflection. A conversion must not
   declare two constructed Result types identical or reinterpret different layouts.
6. Out/ref need separate treatment. Do not reinterpret a pointer to a wider slot as
   a narrower slot. If output conversion is supported, an adapter uses a temporary
   exact output, converts it and writes back under a specified success/failure rule.
   Aliasing and failure semantics must be tested before admitting this feature.

Contravariance reverses the admitted input conversion, for example an explicitly
implemented consumer of a base reference used where a consumer of a derived
reference is expected. It does not authorize writing arbitrary bases into derived
containers. Nested combinations need polarity checking, not a simple rule based
only on whether a parameter appears in the outermost return type.

## Alternatives and proposed staging

| Approach | Benefit | Cost or limitation |
| --- | --- | --- |
| .NET-style interface/reference variance only | Familiar safety model and narrower runtime work | Does not satisfy Sliceable while Result remains invariant |
| Explicit result mapping plus an explicit Sliceable adapter | Expressible with existing nominal interfaces; visible conversion cost | Boilerplate and potentially an adapter allocation; does not make the original assignment implicit |
| Validated result-value widening plus interface call adapters | Supports the requested case without changing Result into an interface or erasing cases | Metadata, conversion planning, dispatch, GC and backend work; no blanket zero-cost claim |
| Treat generic values as layout-compatible aliases | Superficially cheap | Reject: different payload layouts, output slots and mutable state can make this unsound |

Proposed direction: establish ordinary nominal interface variance first, then add
explicitly validated conversion of immutable nominal result values. An interface
view whose return requires value conversion needs a conversion-aware call adapter;
its metadata must describe the actual implementation signature and required
conversion. Reuse an existing reference where possible, but measure and disclose
adapter allocation, payload-copy and dispatch costs. Decide source syntax only
after the metadata and safety model are precise.

A general mechanism is preferable to privileged treatment of the names Result and
Sliceable, but Result is the first bounded acceptance case. Constructor/static
factory parameters are not writes into an already widened instance; distinguish
these from mutable instance members during the audit. No blanket covariance for
all nominal unions is authorized by this proposal.

Ordinary Raven/.NET must remain valid on its backend. General diagnostics and
polarity checking can be developed on Raven main; native conversion behavior goes
to codex/neoclr deliberately under explicit Runtime Contracts. If the .NET backend
requires explicit mapping/adapters, document that difference rather than accepting
source it cannot safely emit. This is a proposal for a target capability, not a
request to restart structural-type work.

## Acceptance and next bounded slices

1. Inventory native generic metadata, importer/emitter flags and runtime type
   relationships; specify variance metadata and malformed-image rejection.
2. Validate declaration polarity and reference-only interface conversions, including
   inherited/nested producer and consumer contracts and negative writable cases.
3. Specify Result's generated/inherited member safety and value conversion plan;
   test both cases, default behavior, differing layouts, exact generic identity and
   output-slot rules. Do not simply set a variance flag on the current declaration.
4. Execute the author's Sliceable conversion against a separately compiled library
   in interpreter, AOT and JIT. Exercise empty/success/error results, exact receiver
   identity, reference retention, faults and collector cleanup.
5. Verify reverse conversions and writable views are rejected, overload selection
   prefers exact matches and reports incomparable candidates, and unsupported .NET
   cases get a clear diagnostic or an explicitly selected safe adapter.
6. Compare direct versus widened calls and explicit mapping with allocation counts
   and throughput before making efficiency claims. Keep each validated slice and
   its changelog/API documentation separate.

The initial bounded task was the metadata/conversion inventory and a small
reference-only interface proof. Runtime progress and remaining backend work are
recorded below; the slicing API and implicit value conversions are unchanged.

## Runtime metadata slice (2026-10-15)

Native runtime TypeDef metadata now accepts `generic_variance`, either omitted/
empty (all invariant) or an arity-aligned vector of `invariant`, `covariant`, and
`contravariant`. NeoIL authors can write `.variance out in invariant` inside a
corresponding interface declaration. This is runtime metadata, not new Raven syntax
or an SDK builder API. The first slice left the C# native-metadata SDK rejecting this field. The later
read-only SDK slice below exposes it without enabling authoring or emission import.

Existing artifacts omit the field and retain invariance. Older schema readers
reject annotated artifacts rather than silently erase their variance. The loader
verifies interface-only admission and nested input/output polarity.
Invariant generic wrappers (including Result), mutable array elements, byrefs and
structural signatures cannot carry a varying parameter. Default bodies, static
contracts, Self and generic methods on variant interfaces remain rejected in this
bounded profile. Invariant existing declarations retain their behavior. Primitive
reference stubs cannot hide variance metadata; native specialization clears the
open parameter annotations together with the parameters on its private closed
shapes. This slice records declarations only; it does not yet change assignment or
dispatch.


## Reference-only interpreter slice (2026-10-15)

Reference assignment, casts, type tests and descriptive assignment queries now
recognize declared interface variance. Exact constructed identities remain distinct;
interface closure/reflection enumeration still reports declared relationships, not
an unbounded set of possible widened views. An exact implementation wins; multiple
other compatible views are rejected as ambiguous. This conservative rule avoids
silently choosing between distinct implementations and is a documented restriction
relative to broader CLR dispatch behavior.

A producer returns a narrower nominal reference; a consumer accepts a wider nominal
reference. Calls resolve the original, explicitly implemented contract before
selecting the body. The interpreter retypes argument slots to the implementation's
signature and returned references to the caller's signature without changing the
referenced object's identity. This matters when a consumer replaces its argument
slot. Closed nominal reference and array-reference arguments compose through other
variant interfaces; differing value arguments, numeric conversion, implicit boxing, mutable
array-element covariance and borrowed receiver conversion remain excluded.
Intrinsic String also remains excluded from *differing* argument conversions: its
payload requires materialization when becoming an Object/interface view. It must
not be admitted as an identity-preserving nominal reference by accident.

The [retained consumer and evidence](../validation/interface-variance/README.md)
cover producer/consumer calls, receiver and payload identity, allocation counts,
collection, malformed declarations, ambiguity and rejected directions. Nongeneric class and already-boxed implementations are included in interface
reachability. Generic implementations execute when closed, but variant call-graph
analysis rejects them until it can enumerate all compatible closed instantiations.
Inferring only the target arguments would miss narrower implementations. An
explicitly boxed receiver reuses its box; variance never inserts a box.

Native specialization currently rejects used variant interfaces with a specific
conversion-aware-dispatch diagnostic before erasing their generic identity. A
focused backend test enforces that boundary. There is no claim of AOT/JIT support.
The C# SDK now reads variance annotations but rejects their emission import and
CLI projection; Raven does not emit these annotations. Existing invariant APIs, including Sliceable and Result, retain
their current contracts. General .NET behavior is unchanged.

Next: retain variance and its concrete implementation mapping through native
specialization, qualify AOT/JIT reference calls and GC, then wire the metadata SDK
and Raven native importer/emitter. Only after those layers agree should standard
interfaces be annotated. Result-value conversion remains a separate stage.


## Minimal current scope and SDK inspection (2026-10-15)

The author requested implementing only the minimum variance needed now. The current
bounded scope remains declared nominal interface variance with reference-only
conversions. Result-value widening, output-slot adapters, automatic Sliceable
widening and structural changes remain deferred. This is a scope reduction for the
current work, not a decision to remove the earlier acceptance case.

The SDK now reads `generic_variance` and exposes an immutable, ordinal-aligned
`TypeDefinition.GenericParameterVariance` list. Missing/empty native annotations
normalize to invariant. `GenericParameterTypeInfo.Variance` exposes the same fact
for introspection. This corresponds to the variance component of .NET generic
parameter attributes; it does not claim CLR conversion or backend parity.
CLI snapshots and authored definitions return null for the list, and introspection
throws when the fact is unavailable, rather than inventing invariant annotations.

Snapshot `Write()` preserves the original image. Importing a variant definition for
emission and projecting an annotated native assembly to CLI both reject explicitly,
since those paths still assume invariance. NeoIL remains the authoring path. The SDK
checks annotation shape, arity and interface ownership; the runtime remains
responsible for signature polarity, verification and executable conversions. No new
SDK assignability algorithm or Raven contract is introduced.

Focused validation: `dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --interface-variance`.
Checks cover all three annotations, absent/empty defaults, immutable copied
snapshots, introspection, byte-preserving round trips, malformed metadata and
projection/import rejection. AOT/JIT qualification and frontend authoring remain
required before advertising general end-to-end variance support.


## Explicit SDK annotations (2026-10-15)

The author subsequently directed completing runtime and Raven-on-neoCLR variance,
and annotating eligible library interfaces. The SDK now provides `SetVariance`,
validates nested declaration polarity at write time, emits native annotations and
CLI generic-parameter flags, and retains annotations through native/CLI readers and
reference projection. This supersedes the earlier read-only slice's projection/import
rejection. Imported annotation facts are retained without inventing them for manually
created references; native bindings compare reference and runtime variance.

Validation uses the focused SDK variance and existing declaration/projection checks.
An SDK-authored NEOX loads and returns 42 in the interpreter. Call-site conversion,
Raven integration and AOT/JIT work are separate slices; metadata transport alone is
not end-to-end qualification. Structural work and automatic Result mapping stay parked.


## Raven interpreter integration (2026-10-16)

Raven's native adapter now opts into interface variance using its explicit emission
capability, preserves annotations on imported type parameters and writes SDK
annotations. The SDK also checks reference-only call/return conversions and accepts
explicit annotations from the compiler's resolved external contracts. A separate
producer/consumer library and consumer return 42 under the interpreter; reverse,
invariant and value-argument conversions reject. Unsafe declaration emission fails
without writing an output image. See [bridge integration](../raven-cli-bridge.md).
Native specialization and library annotations remain outstanding in this slice.

## Native execution qualification — 2026-10-16

Superseding the earlier rejection boundary, private native shapes retain verified
source interface identity, argument types, annotations and contract-slot identity.
Selection retains all reached compatible peers, including closed generic
implementations, and reuses nominal exact-first/unambiguous resolution. The proof
is not serializable metadata and introduces no artificial inheritance or allocation.
The [native evidence](../validation/interface-variance/README.md#native-qualification--2026-10-16)
qualifies separately compiled Raven producer/consumer calls in interpreter, AOT and
managed JIT, including native GC cleanup. Library annotations and guest introspection
remain follow-ups; value-result widening remains deferred.

### Generic metadata preservation requirement (2026-10-16)

Author direction: match .NET's reified generic type model. Native specialization
may change executable shapes, but must not erase observable generic definitions,
arguments, constraints or variance. Complete source type declarations and closed
arguments are retained in the current native source catalogue; a build-time report
alone is not runtime introspection support. The current open-definition native
reflection restriction remains a correctness/coverage gap to close, not intended
neoCLR semantics. This requirement does not resume structural experiments.

## Class-library annotations (2026-10-16)

The following nominal interfaces opt in after signature-polarity validation:

| Annotation | Interfaces |
| --- | --- |
| `out T` | Iterator, Iterable, Collection, Sequence, AsyncIterator, AsyncIterable, ConvertibleInto |
| `in T` | Comparer, EqualityComparer, ComparableTo, EquatableTo |

These roles correspond to .NET's producer/enumerator/read-only collection and
comparison variance. AsyncIterator's tasks contain bool or unit, not T; only its
Current getter exposes T. Concrete classes remain invariant, but their declared
interfaces can supply compatible reference views. Array element storage stays
invariant. MutableSequence/List, Set/MutableSet, Map/MutableMap, Stack/Queue,
Sliceable, Closable and Propagatable remain invariant because they consume and
produce parameters, use invariant value wrappers, or rely on static/Self contracts.
Result/Option remain invariant nominal values. No result mapping or boxing is
implied by the annotations; `Sliceable<int[]>` does not thereby become
`Sliceable<Iterable<int>>`.

The retained [library consumer](../validation/interface-variance/library/Main.rvn)
exercises imported read-only views and comparison/conversion contracts. Generic
metadata is retained independently of executable specialization.

### Native generic metadata retention follow-up (2026-10-16)

The native source catalogue now retains method `genericConstraints` as well as
`genericParameters`. Constraints remain in declaration form: a method bound on
its owner's `T` retains that owner-relative parameter instead of being replaced
by one specialization's argument. Each closed row retains its distinct arguments
and the shared definition identity, original type parameters, variance and bounds.
A focused catalogue regression checks two constructions and constructor/type bounds.

This is build-time evidence shared by native preparation, not guest introspection
completion. The assembly-enumeration consumer still fails native preparation at
`neoCLR.Runtime.AssemblyTypes: unsupported value member contract`. Native support
needs metadata-only, owner-bound handles for definitions and their parameters,
separate from executable layouts, plus explicit discovery retention. The .NET-like
requirement to preserve observable generic metadata remains unchanged.

### Class-library generic argument sequences (2026-10-16)

All TypeInfo providers now build `GetGenericArguments()` using the existing count,
indexed handle and FromHandle operations. This preserves the same parameter
classification while removing the separate native TypeGenericArguments binding
requirement from the class-library path. One result array is reserved and filled;
no extra list or adapter is introduced. Public signatures and Runtime Contract
configuration are unchanged. The closed consumer checks indexed/sequence parity
and a nongeneric empty result; the open-definition interpreter consumer checks
all three variance annotations. Native open-definition discovery still needs the
metadata-only handle model described above.

### Descriptive native handle foundation (2026-10-16)

.NET distinguishes a generic type definition from its constructions and describes
a parameter by the type or method that declares it plus its zero-based position
([definition](https://learn.microsoft.com/en-us/dotnet/api/system.type.isgenerictypedefinition?view=net-10.0),
[position](https://learn.microsoft.com/en-us/dotnet/api/system.type.genericparameterposition?view=net-10.0)).
neoCLR already uses declaring-definition identity and ordinal in its interpreter
TypeDescriptor. Native preparation now has a separate compiler-owned descriptor
table and private `$metadata.token` instruction for open definitions and their
parameters. These preserve that semantic identity without synthesizing a nominal
class, closing a generic declaration or retaining an executable layout.

The table is not serialized. A serialized opcode without its compiler-owned table
is rejected by verification, and supplying the private table as an input field is
rejected. Only the in-repository native bridge populates it from resolved source
definitions; nongeneric definitions keep ordinary type tokens. Cloning, selection
and private nominal renaming preserve the table. Native lowering assigns image-local
integer handles in a disjoint range from ordinary closed-type tokens; no object or
adapter allocation is required to acquire a descriptive handle.

This belongs to the runtime/compiler metadata boundary, not a foreign library:
source identities and verified IR are required, and no external ABI is introduced.
The tradeoff is a second internal token producer and a table-preservation obligation
for native rewrites. Guest TypeInfo APIs and Runtime Contract configuration are
unchanged. This foundation alone does not enable assembly discovery, open generic
member substitution or invocation. Interpreter identity/forgery tests, selection
and rewrite checks, AOT emission and synchronous macOS ARM64 JIT execution qualify
the handle foundation; guest discovery/query wiring follows separately.

### Descriptive native queries (2026-10-16)

The native query binder now handles retained open-definition and parameter tokens
for names, generic-argument counts, indexed argument identity, source metadata
tokens and shape flags (including all three variance states). It closes the token
inventory over parameters returned by argument queries, even when no explicit
parameter token occurs in the caller. Shape facts and metadata tokens come from
the interpreter's source metadata rules. Unknown selectors and argument indices
retain their faults. Mixed open/closed token dispatch is qualified as well.

Open-source or open-target assignability/subclass queries explicitly fault in this
native subset; no result is fabricated while open substitution semantics remain
unqualified. The guest assembly/module discovery path is still unfinished, so this
is backend query support rather than a claim that AssemblyInfo.GetTypes already
works in native programs. Tests execute the generated query bodies in the
interpreter and macOS ARM64 JIT and emit AOT objects without retaining the open
interfaces as executable types. The focused reflection-metadata suite passes eleven
checks. No Raven syntax, emitted source metadata or Runtime Contract option changes.

### Native discovery recipes (2026-10-16)

The compiler bridge can now obtain assembly-wide or logical-module type snapshot
recipes through the interpreter's discovery queries. Like .NET Assembly.GetTypes,
the result includes generic definitions as definitions, alongside nongeneric types;
it does not substitute an arbitrary constructed instance or discard generic facts.
Metadata order, owner-bound arguments and variance are preserved. Module discovery
requires an exact assembly identity and an explicit declaration-module catalog;
empty declared modules return empty arrays and foreign module names fault.

These are build-time recipes, not executable reflection permissions. Native guest
GetTypes binding and explicit discovery retention are still pending. Focused tests
cover mixed generic/nongeneric catalogs, variance, exact identity, empty modules,
legacy metadata rejection and foreign-module rejection. No public class-library
API, Raven lowering, Runtime Contract configuration or CLI encoding changes.

### Explicit native discovery binding (2026-10-16)

Native reflection-root schema 6 adds `typeCatalogs`, an array of exact assembly
identities, alongside the existing `types`, `moduleCatalogs` and `moduleFunctions`
arrays. Unlike `moduleCatalogs` (names only), this explicitly retains complete type
snapshot recipes for AssemblyInfo.GetTypes and ModuleInfo.GetTypes. The binder
validates exact InternalCall signatures, returns empty arrays for retained empty
modules and faults for unretained assemblies or foreign module scopes. It neither
grants constructors/member invocation nor changes existing schema 1–5 behavior.

Snapshot factories emit descriptive handles for open definitions and their
owner-bound parameters. Nongeneric/closed types keep ordinary type tokens, avoiding
a second identity for types also obtained through typeof or object introspection.
Consequently ordinary discovered types still require supported executable type
shapes; eliminating that layout dependency is not part of this slice. Partially
open constructions remain unsupported. This is explicit AOT retention, rather than
a change to Raven syntax, Runtime Contract settings or CLI/native metadata encoding.
Focused catalog tests qualify retention admission, exact query contracts, empty
results, unretained faults and snapshot token identity. Full Raven native discovery
qualification remains a separate acceptance gate.

The original `VarianceIntrospection` consumer was retried with schema 6 against
`target/variance-argument-sequence-20261016` (Raven `eef1ed456`). This exposed and
fixed a source-recipe contract mismatch: assembly discovery now propagates the
source profile to TypeInfo snapshots instead of selecting legacy System.Type.
The retry passes that stage but stops at `$aot_linked_65: unsupported value member
contract`; no AOT/JIT success is claimed. Nine backend catalog checks and the
runtime catalog regression pass, including an explicit TypeInfo recipe assertion.

### Native display names and namespaces (2026-10-16)

The native descriptor binder now handles TypeDisplayName selectors 0 (display
name) and 1 (namespace) through shared interpreter source-metadata queries. This
covers the Namespace property used by the original variance discovery consumer.
Open definitions and owner-bound parameters use their retained descriptors; closed
types are queried through their original semantic types before native renaming.
Unknown selectors fault, and the reserved query requires its exact two-argument
InternalCall signature. This follows the existing .NET-aligned introspection
contract without changing Raven lowering or runtime type identity.

Eleven focused metadata tests pass, including nonempty namespaces, open parameter
names, mixed open/closed queries, invalid selectors, interpreter execution, AOT
object emission and macOS ARM64 JIT execution. No public API or CLI encoding changes.

Retrying the original consumer now passes the unbound native query and reaches
`static owners cannot be used as values or instance receivers`: a discovered static
type's ordinary token is incorrectly checked as value storage. This is the next
native qualification blocker; the full consumer is not yet AOT/JIT qualified.

### Static-owner token validation (2026-10-16)

Native `ldtoken` now accepts a supported static owner as type identity, separately
from value storage. This matches .NET's ability to reflect a static type without
constructing an instance. It uses the ordinary token inventory, so discovery and
explicit tokens agree; it does not introduce a second descriptor identity.
Static-owner locals and instance values remain rejected. Open or unsupported types
still pass through the existing token checks rather than receiving a blanket bypass.

The focused static-token regression passes interpreter and macOS ARM64 native
execution and AOT emission, with a negative static-local-storage check. Raven
lowering and Runtime Contract configuration are unchanged.

### Native Raven discovery qualification (2026-10-16)

The [open-definition consumer](../validation/interface-variance/introspection/README.md)
now passes interpreter, AOT and managed JIT against the matching
`variance-argument-sequence-20261016` bundle (Raven compiler `eef1ed456`). It checks
all three variance states, parameter sequence/indexed identity, namespace filtering,
concrete closed arguments and module/assembly definition identity. Native hosts
verify absence of faults and GC/root cleanup; AOT uses UBSan/bounds checks. The
runner retains commands and artifact hashes in `native-results.json`.

This supersedes the discovery blockers above. Runtime support comes from native
binding and token validation; there is no compiler workaround or new source/CLI
encoding. Explicit type-catalog retention remains required. Open member and
relationship operations, method generic-definition discovery and intrinsic String
variance materialization remain gaps. Structural types remain parked.

### SDK variance across imported class bases (2026-10-16)

A focused regression exposed a mismatch between ordinary SDK reference conversion
and its variance argument check: a local `Derived : ExternalBase` could be used as
ExternalBase directly, but `Producer<Derived>` could not widen to
`Producer<ExternalBase>`. The SDK now reuses the explicit imported-base ancestry
proof, including local transitive inheritance. No declaration or interface
conformance is inferred from matching names. Dependency facts remain explicit and
the native loader still verifies the actual linked definitions.

The focused `--interface-variance` regression failed on the direct widening before
the fix and passes afterward. Tests cover direct/transitive covariance,
contravariance and nested variance, native metadata writing/reading, and invocation
of the emitted conversion functions on .NET using null reference arguments. Reverse
and unrelated-class conversions still reject. Existing metadata/projection checks
also pass. This restores .NET-aligned nominal reference variance; it does not add
adapters, boxing, generic class-base support or String materialization. Runtime
call dispatch is unchanged; these tests qualify the SDK admission fix, not a new
Raven/AOT consumer. Runtime Contract configuration and metadata encodings are unchanged.

### Raven imported-base variance qualification (2026-10-16)

The [separately compiled consumer](../validation/interface-variance/imported-base/README.md)
now closes the source/native qualification gap above. Raven `9854031b8` was rebuilt
against SDK fix `2c98cbd1b`, using the unchanged
`variance-argument-sequence-20261016` class-library bundle. The library owns Base and
the producer/consumer contracts; the consumer owns Derived, Leaf and implementations.
Widening, narrowing and nested variance calls preserve actual object identity and
mutations of inherited storage. One emitted consumer returns 42 in interpreter,
AOT and managed JIT. Three reverse/unrelated conversions produce RAV1503 diagnostics.
Native host fault/root/collection checks and AOT UBSan/bounds checks pass. Commands
and hashes of both images, sources, compiler, SDK and runtime/backend are retained.

No source compiler change, Runtime Contract setting, metadata encoding, adapter,
boxing or expanded variance rule was needed beyond the SDK ancestry correction.
Generic class bases and intrinsic String variance materialization remain gaps;
Result and Sliceable remain invariant. Structural work stays parked.
