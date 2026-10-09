# Native JSON through introspection and reflection — 2026-10-09

The author directs native JSON serialization as the next HTTP showcase priority:
"We need to get JSON serialization working. That requires introspection and reflection
support." This takes precedence over further general hosting/reload work. Keep the
current serializer and its interpreter contract as the semantic reference.

## Consumer and scope

`docs/experiments/native-json` is a typed round-trip probe using the public
`JsonSerializer.Deserialize<Report>` and `Serialize(Object)` APIs. Its string, Int32
and Boolean properties exercise UTF-8 text, type identity, property metadata,
construction, boxing and getter/setter invocation. The interpreter prints
`{"Name":"Café","Count":3,"Active":true}`. Native execution is **not yet admitted**.
The HTTP follow-up must use the same mapper through JsonContent/client JSON APIs,
not a separate serializer or hand-written JSON strings.

The existing mapper first validates the complete input tree, then constructs and
assigns the model. Preserve its checks for public instance properties, constructor
availability, receiver/value compatibility, unsupported shapes, depth/item limits,
null/polymorphism restrictions and terminal user-code faults. Test real setter effects
and ensure invalid input does not run constructors/setters. Existing nested-class and
typed-array support must remain a visible coverage obligation after the flat-model gate.

## .NET comparison and provisional implementation choice

Microsoft's [reflection/source-generation comparison](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/reflection-vs-source-generation)
and [source-generation guide](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/source-generation)
(reviewed 2026-10-09) describe reflection-based metadata discovery and generated
metadata/serialization paths. System.Text.Json's Native AOT use requires source
generation because its reflection path needs APIs unavailable in that environment.
This does not mean Native AOT has no reflection of any kind.

For neoCLR, first retain a bounded closed-world metadata set and generate checked
invocation dispatch for the existing introspection/reflection facades. This is a
provisional native backend design, not a new public reflection API. It preserves
one mapper and enables other metadata consumers, but costs retained metadata,
boxed values and dispatch machinery. Source-generated per-model codecs remain an
alternative with potentially smaller metadata and less dispatch; they would add a
second serialization path and do not supply the requested general introspection
foundation. No performance advantage is claimed without measurements.

## Native reflection architecture — author clarification

On 2026-10-09 the author clarified: "We should implement reflection support for
native compilation in a way that makes sense." Treat reflection as a reusable
platform capability. JSON is the first acceptance consumer; JSON member names,
model names and serializer-specific dispatch must not enter the backend.

The following is the implementation direction, not a claim of completed support:

- **Semantic metadata:** capture validated source identities, closed type arguments,
  UTF-8 names, visibility, inheritance, property signatures and accessor/constructor
  identities before specialization erases metadata. Keep these separate from native
  object offsets, synthetic boxing tags and compiled-function indices. Generated
  private helpers must never appear as user reflection members.
- **Retention analysis:** maintain separate requirements for type identity, member
  discovery and invocation. A type token alone must not retain every method body.
  Invocation requirements add executable roots before specialization and selection;
  dependencies of those bodies then follow ordinary reachability. Iterate until
  both metadata and executable requirements stop growing. Report why each item
  was retained. A metadata-only consumer must work without invocation support.
- **First retention mechanism:** prototype explicit build-time roots keyed by source
  identity, with member categories and invocation requirements. This avoids silently
  guessing requirements from calls to the JSON library. The configuration syntax is
  not settled. Later type-flow inference can supply the same requirements; Raven
  annotations are a separate integration decision, not a prerequisite for the
  first native implementation. Unknown flows must receive actionable diagnostics.
- **Runtime representation:** emit immutable image-owned descriptor tables. Canonical
  type identities must unite `typeof(T)`, `Object.GetType()` and property type queries;
  primitive boxing tags map back to the primitive descriptor. The current token
  ordinal is a private foundation, not the final metadata schema or a cross-image ABI.
  Managed descriptor wrappers remain ordinary traced objects; immutable descriptor
  storage has image lifetime. Image unloading/reload needs an explicit ownership
  contract before handles can cross image boundaries.
- **Execution:** generate typed invocation adapters for retained constructors and
  accessors. Share normal argument lowering, allocation, GC roots, dispatch and fault
  propagation. Call real accessors, preserving their effects and virtual dispatch;
  do not substitute raw field writes. Validate visibility, receiver, argument count,
  nullability and exact boxing/assignability before entering user code. Retaining
  a body does not grant access to it.
- **Failure semantics:** admitted reflection preserves the existing Introspection and
  Reflection API contracts. Missing required retained metadata is an admission error,
  never a fabricated empty member list. Queries for genuinely absent members and
  invalid runtime receivers/values keep the existing error results. Unsupported
  invocation shapes fail compilation without publishing an executable. Discovery
  of a member does not automatically promise that its invocation is supported.

The first independent reflection consumer must enumerate a model's properties,
construct it, invoke a setter with an observable effect and read through its getter,
without importing JSON. Compare it with the interpreter on macOS ARM64 and Windows
x64. Include inherited/closed-generic identities, inaccessible or missing accessors,
wrong receivers/boxed values, metadata-only retention, constructor/accessor faults
and collection during invocation. Then run the unchanged JSON mapper and HTTP cases.
This separates runtime correctness from serializer policy such as nesting limits.

### Retention alternatives and research

Microsoft's [trimming guidance](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/fixing-warnings)
uses member-preservation requirements for known types and propagates them through
call chains. Retaining all members increases size and makes their dependencies
reachable. This motivates separate discovery/invocation requirements and an
inspectable retention report; no claim of better .NET ergonomics is made.
[GraalVM's reachability metadata](https://www.graalvm.org/latest/reference-manual/native-image/metadata/)
provides another shipped AOT approach combining build-time inference with explicit
configuration for dynamically accessed elements. Its missing-registration diagnostics
reinforce the need to distinguish omitted metadata from absent members. Sources
reviewed 2026-10-09; these are design comparisons, not dependencies or copied ABIs.

Keeping all source metadata and bodies would simplify initial discovery but defeats
bounded native selection and exposes unsupported bodies unnecessarily. Aggressive
inference alone would require reliable type-flow analysis before the first consumer.
Explicit roots are the initial tradeoff: predictable behavior and auditable scope at
the cost of configuration. Source-generated serializers remain a library alternative,
but do not implement general reflection. Broader .NET ecosystem comparisons and
measurements remain open before settling public preservation APIs or size claims.

## Bounded slices and admission

1. **Implemented foundation:** preserve opaque RuntimeTypeHandle storage through
   specialization; close generic `ldtoken` operands, retain token-only type shapes
   and emit image-local identity. Handles pass through locals, parameters/results
   and fields; equality distinguishes closed types. They are not GC roots or native
   pointers. Numeric conversion remains rejected. This is not TypeInfo discovery.
2. **Next:** make the primitive object representation needed by the mapper complete
   and retain source type/property/constructor/accessor metadata before private AOT
   projection removes it. The current specialization path clears property records;
   simply accepting `ldtoken` cannot restore that metadata later.
3. Bind the exact type-identity/shape/property-list services used by TypeInfo.
   Keep metadata roots separate from ordinary call reachability. Missing metadata
   must produce an explicit admission diagnostic, not silently empty property lists.
4. Supply checked property get/set and parameterless construction dispatch over
   selected bodies. Preserve access checks, boxing/type checks, fault sites and GC
   ownership. No runtime-generated code or arbitrary dynamic loading is required.
5. Pass the flat JSON round trip and negative cases, then the typed HTTP client/server
   consumers on macOS ARM64 and Windows x64; extend to nested/array mappings.

Explicit metadata-root configuration, metadata size budgets and representation of
revisions/image ownership still need validation. Runtime handles from separate images
must not be compared, persisted or used across reload. Keep native Windows ARM64
qualification separate. Ordinary .NET/Raven behavior and Runtime Contract settings
remain unchanged; this work is in neoCLR's native selection/backend layers.

## Implemented source metadata catalogue — 2026-10-09

The explicit-load-set linker now snapshots validated source metadata before
canonicalization and specialization. Its private `sourceMetadata` report includes
selected nominal declarations with original module/revision identities, closed
property signatures and accessor references, and declared method identity/access
information. Closed generic instances retain their shared declaration identity and
distinct type arguments. The original declaration remains available alongside the
closed view; native synthetic type names do not replace semantic names.

This is build-time preservation and inspection only. It adds no executable roots,
immutable runtime tables, metadata-root configuration or reflection bindings. An
accessor appearing in this catalogue can still be excluded from native code. The
catalogue covers selected nominal types, not every primitive or metadata-only type;
explicit retention closure and runtime type identity integration remain next steps.
The implementation reuses the source verifier and does not loosen access checks.

Validation: two focused `native_source_metadata` tests compile generic and
non-generic consumers, preserve original member IDs/private access, close property
and accessor types, and assert unused accessor bodies remain excluded. The two
`native_type_tokens` tests still pass, including native/interpreter executable
identity parity. The unchanged typed JSON probe reaches its existing Boolean-boxing
rejection; this catalogue does not claim that reflection or JSON executes natively.

## Implemented type-equality service — 2026-10-09

The reference-arena native profile now lowers the exact reserved
`neoCLR.Runtime.TypeEquals(RuntimeTypeHandle, RuntimeTypeHandle) -> Boolean`
InternalCall to token identity comparison. This is the existing service used by
`TypeInfo.Equals`, not a new public API. It allocates nothing and does not compare
display names or grant reflection invocation rights. Source verification remains
mandatory; altered signatures, bodies and receiver/generic contracts are rejected.

A standalone native consumer compares primitive, nominal and closed generic tokens,
including stored handles, against interpreter execution. Exact-contract rejection
checks and the sanitized macOS consumer pass. The portable type-token validator now
has a `--runtime-equality` mode included in both platform jobs. See the
[service validation evidence](native-type-equality-validation.json). Runtime property
queries, object-to-type mapping, invocation and native JSON remain unfinished.

## Executable native descriptor queries — 2026-10-09

The reference-arena profile now binds the exact TypeName and TypeArgumentCount
InternalCalls for primitive and closed nominal token producers. Names come from
validated source declarations/origins; two generic instantiations keep the same
semantic definition name while retaining different identities. Generic arity comes
from closed type arguments, not the spelling of a private native shape.

The first backend implementation generates a bounded dispatch body over known token
producers, using existing token comparisons and ordinary String literals. This
composes existing allocation/root/fault lowering and avoids a separate descriptor
pointer ABI at this stage. Dispatch is linear in the retained token set and code size
grows with it; no performance improvement is claimed. Shared immutable descriptor
tables remain the next representation step as query coverage grows. Names alone
never authorize access or retain executable accessor bodies.

The `type-descriptors.neoil` consumer obtains names for Int32, Account and two Model
instantiations, checks generic arity and distinguishes the Model identities. Its
standalone native host checks GC cleanup and buffer bounds; its output matches the
interpreter. Run `scripts/validate-native-type-tokens.py` with `--descriptor-queries`
to build and execute it on macOS ARM64 or Windows x64. The validator retains the
executable and hashes its source/artifacts. The Windows job runs the same fixture. Both platform jobs pass at `ba871e0d`
in action `37980244735`; all six reports and source/artifact hashes are verified.
See [descriptor evidence](native-type-descriptor-validation.json) and
[runnable instructions](experiments/aot-console/type-descriptors.md).

This is a closed descriptor-query subset, not general TypeInfo or JSON admission.
Array/function signature queries reject explicitly for now, as do missing source
metadata and malformed reserved contracts. All admitted token producers are covered;
foreign or forged handles are unsupported and the generated fallback faults instead
of inventing metadata. Object.GetType, generic argument retrieval, property queries,
explicit metadata roots and checked invocation remain unfinished. The existing .NET
and GraalVM retention comparisons above apply; these services implement existing
neoCLR contracts without changing Raven emission or Runtime Contract configuration.

## Native type-shape inspection — 2026-10-09

The exact TypeShape(RuntimeTypeHandle, Int32) service now reads the preserved source
facts for primitive and closed nominal tokens. It implements all existing selectors:
array/byref/pointer/readonly flags (false within this admitted subset), interface,
abstract, enum, value type, open/closed hierarchy, union, public visibility, nominal
and function classification. Visibility includes generic arguments and imported
source visibility; native storage layout and synthetic helper names are irrelevant.
`IsOpen` retains the existing extensible-hierarchy meaning, not generic-parameter
openness. Discovery still grants no execution/access rights.

An invalid selector produces RuntimeError with the interpreter's message and caller
frame, not UserFault or an artificial generated-helper frame. This error mapping is
restricted to the generated descriptor-query bodies; ordinary guest faults keep their
existing behavior. Arrays, pointers, byrefs and function tokens remain unadmitted
for descriptor queries rather than being reported as nominal types.

Validation compares every selector plus -1 and 14 against the interpreter for ten
primitive/nominal shapes (160 native executions), including interfaces, enums,
abstract/sealed classes and a generic type containing an internal argument. Focused
metadata tests cover closed unions and imported visibility. The ordinary UTF-8 line
output/user-fault regression and the descriptor consumer pass. The portable consumer
now checks representative shape/visibility facts on both platform jobs. See
[shape validation](native-type-shape-validation.json). Full TypeInfo wrappers,
property discovery, invocation and native JSON remain unqualified. This implements
the existing neoCLR reflection contract; the earlier .NET/retention comparison and
provisional dispatch-size tradeoff remain applicable.

## Primitive object values — 2026-10-09

Native Boolean boxes now have a distinct tag and immutable scalar snapshot, alongside
Int32 boxes. Exact `isinst` preserves the original reference on a match and returns
null otherwise. `unbox.any` checks null and exact primitive identity before loading
the scalar payload; it preserves NullReference/InvalidCast, interpreter messages and
caller fault frames. String/reference unboxing follows existing reference casts.
The private native fault protocol adds InvalidCast code 11 without changing context
layout. Boxed Boolean Object.ToString remains explicitly unadmitted.

The portable primitive-box consumer checks false/true, Int32.MinValue, identity,
wrong primitive and String boxes, null unboxing, GC cleanup after success/fault,
root-frame balance and buffer canaries. The interpreter/native regression and the
existing Int32 display/snapshot/limit regression pass on macOS. The unchanged JSON
probe now advances to the native interface-array admission gap required by metadata
collections. This is existing CLR-style exact boxed-value behavior, not a new public
API or a claimed improvement over .NET; the existing reflection comparison applies.

## Evidence

The original typed probe rejects native specialization with
`specialization requires closed reference-free local value types: RuntimeTypeHandle`.
The type-token slice removes that rejection; subsequent native admission still rejects
Boolean boxing in `ObjectMapper.ReadValue` at instruction 110. The subsequent
primitive-box slice removes that rejection. Interface-valued native arrays now
use initialized managed-reference slots and invariant closed generic backing views.
Executable macOS tests cover reserved/default arrays, interface dispatch, aliasing,
collection and interpreter-matched faults. Unlike CLR reference-array covariance,
this reuses neoCLR’s invariant array contract; it adds no runtime covariant store
checks or new public collection API. The next probe rejects the missing
`ObjectTypeHandle` runtime service (implemented in the next slice below). Further
metadata service and reflection dispatch gaps remain.

`type-tokens.neoil` checks primitive identity, distinct nominal types, distinct closed
generic shapes and a handle stored in a generic class. The Rust regression compares
native/interpreter execution with the same supplied System module and checks that
handles are absent from GC trace slots. Invalid numeric casts fail before an object
is emitted. `scripts/validate-native-type-tokens.py` runs the same C consumer on
macOS and Windows. See [validation evidence](native-type-token-validation.json).

## Interpreter/native benchmark gate — author direction, 2026-10-09

The author directs: "JSON serialization across interpreted and native compilation
will be a benchmarking case once it works." This is a deferred acceptance workload,
not a request to benchmark the currently rejected native consumer. Complete semantic
parity first, including negative cases and reflection side effects.

The proposed benchmark protocol uses the same Raven serializer, model definitions
and immutable payload corpus in both execution modes, with matching library/compiler
revisions. Validate output bytes and deserialized values outside measured regions;
consume results so the work cannot be discarded. Measure serialization and
deserialization independently, then the round trip. Cover the admitted flat model,
UTF-8/escaping and payload sizes first; add nested models and arrays when qualified.

Report cold process startup separately from warmed repeated-operation throughput
and latency. Keep reflection metadata initialization/cache effects visible in first-use
measurements. Record allocations/collections and peak process memory where reliable
instrumentation exists; label unavailable counters rather than estimate them. Include
native binary/metadata size, toolchain and optimization settings, machine/OS,
iterations, warmup policy, repeat count and variability. Run modes on the same
machine for each platform; do not conflate macOS ARM64 versus Windows x64 differences
with interpreter versus native differences. A direct serializer benchmark precedes
HTTP end-to-end measurements so networking does not obscure serialization cost.

The primary comparison is neoCLR interpreted versus native execution. A future .NET
System.Text.Json baseline should identify reflection versus source-generated mode
and match behavior/payloads explicitly; it is not silently interchangeable with
neoCLR's existing mapper. No speedup or memory improvement is claimed in advance.
This protocol is an assistant proposal supporting the author's benchmark direction;
the harness and results are not implemented yet.

## Native object type identity — 2026-10-09

The exact ObjectTypeHandle service now maps concrete source class tags, empty
record boxes and Int32/Boolean boxes to the same image-local tokens as `typeof`.
Intrinsic String object views map to String, without allocating another box.
Abstract/static declarations and private array backing records are excluded from
the dispatch table. Compound object shapes remain unsupported: querying one faults
explicitly rather than exposing a backend storage type. Null preserves the existing
NullReference payload and caller location. Native diagnostic rendering still uses
the raw fault message; interpreter diagnostics can render a canonical category text.

This provides a bounded subset of CLR Object.GetType semantics, with linear image
local dispatch instead of a general runtime method-table descriptor. There is no
new public API or performance claim. The macOS executable test compares primitives,
String and distinct closed generic classes with interpreter execution. A portable
C consumer checks roots, collection, null diagnostics and heap bounds; it is included
in both macOS/Windows action jobs. The public Raven probe confirms GetType/typeof
equality on macOS, with the documented temporary explicit Object cast.

The unchanged JSON probe now reaches `ReflectionConstructionCheck`. Explicit
metadata/invocation retention, property discovery and checked invocation are still
required before the end-to-end JSON milestone can be claimed.

## Explicit roots and checked construction — 2026-10-09

The private AOT CLI and project driver accept `--reflection-roots roots.json`:

```json
{
  "schemaVersion": 1,
  "types": [
    {
      "definition": { "module": "Models", "revision": "r1", "index": 0 },
      "construct": true
    }
  ]
}
```

Use the definition identity from the matching source artifact, not a backend type
index or display name. The source load set must contain that module/revision/index.
The first schema admits one to 64 distinct nongeneric declarations; unknown fields,
stale identities and unsupported schemas fail before native artifact publication.
The project driver hashes the supplied configuration as a build input. This is a
private development mechanism, not a new public preservation API or Raven annotation.

Construction checks use original constructor/type access metadata. They retain the
existing unsupported-type, inaccessible-constructor and missing-constructor statuses.
`construct: false` preserves the availability check without adding a constructor
execution root. `construct: true` adds an eligible public parameterless constructor
root before specialization, even if only its availability is queried. Dependencies
follow normal selection; the report records the exact source constructor identity
and requested retention. Denied or missing constructors are never synthesized.

The exact ReflectionConstruct binding dispatches to ordinary `newobj.ctor` and
returns its Object view. It executes real initializer effects, uses ordinary roots
and allocation, and preserves the interpreter adapter's constructor fault frames.
An invocation omitted from the configuration faults explicitly. A type outside the
configured metadata set also faults explicitly, rather than pretending that a
constructor is absent. This bounded profile currently requires explicit registration
of every type supplied to these construction services; broader inferred retention
and compound/generic construction remain unimplemented.

This compares with CLR Activator's runtime constructor lookup, but uses a closed
source-identity dispatch table and explicit code retention. Predictable retained
bodies and reuse of the current Result wrapper are the benefits; configuration,
linear dispatch and limited shape coverage are the costs. No speed claim is made.

The independent Raven introspection project now calls public `CreateInstance()`,
checks a constructor-initialized property through a normal call, and verifies
GetType/typeof identity. Interpreter/native macOS execution both print `Report` and
exit zero. Focused tests cover metadata-only versus invocation retention, stale
registration, access/missing/abstract statuses, real constructor effects and exact
throwing-constructor diagnostics. A portable GC/canary/fault consumer is added to
both platform jobs. Property discovery and reflective getter/setter invocation are
still the next required work; this does not complete native JSON.

Re-running the unchanged typed JSON assembly with its Report construction root now
reaches the missing `ReflectionArrayCreate` service. The mapper's array branch is
statically reachable even for the flat-model showcase; compound reflection and
property discovery/invocation remain explicit gaps rather than stubbed successes.
