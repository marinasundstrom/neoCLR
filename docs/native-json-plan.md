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
`{"Name":"Café","Count":3,"Active":true}`. The original flat case now passes both modes; later sections track the expanded corpus.
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

## Vector descriptors — 2026-10-09

TypeName, TypeArgumentCount and TypeShape now admit closed vector tokens, using
source element identities. A vector is an array and not a nominal type; when the
source module supplies a nominal vector backing, its semantic name, generic arity
and declaration flags are preserved, without replacing the vector's identity.
Unbacked vectors keep their signature name and zero generic arguments. Visibility
follows the element type. Byrefs, pointers and function descriptors remain outside
this query subset.

Native/interpreter tests cover 102 vector selector/arity cases: primitive elements,
private elements and closed generic class elements, each with/without backing
metadata, including invalid selector faults. The portable descriptor consumer adds
vector names, array classification and visibility checks. These metadata queries
do not yet supply reflective array creation/access or property invocation.

The preceding explicit-root construction gate is qualified on macOS ARM64 and
Windows x64 in [action 37985103063](https://github.com/marinasundstrom/neoCLR/actions/runs/37985103063);
all twelve reports and their source/artifact hashes were verified. See
[construction evidence](native-reflection-construction-validation.json).

## Checked vector reflection — 2026-10-09

The native backend now binds ReflectionArrayCreate, ReflectionArrayGet and
ReflectionArrayLength to ordinary checked array adapters. Explicit vector token
producers retain the vector inventory; supported elements are Int32, Boolean, String
and reference classes/interfaces. Exact primitive boxes, invariant array views,
reference aliasing, default/null slots and bounds checks reuse the existing runtime
contract. Retained vectors return their semantic vector token from Object.GetType,
never their private storage-class identity. Unregistered vectors fail explicitly.

Compared with CLR Array.CreateInstance/GetValue, this preserves the existing neoCLR
vector-only API and invariant views, using closed dispatch instead of open runtime
layout discovery. The cost is explicit token retention and linear generated dispatch;
there is no performance claim or public API change. RuntimeTypeInfo's existing handle
getter is an explicit support root when creation is selected; user bodies are not
retained by that support root. Source verification remains required.

Native/interpreter tests cover Int32/Boolean/String/class values, exact bad/null boxes,
negative indices, invalid/null providers and nonarray receivers/types. They check
adapter/caller fault frames, root balance, collection and heap canaries. The public
Raven sample creates Int32 and Report vectors, checks GetType, observes a shared
Report mutation and constructs an empty vector; both modes print Report and exit zero
on macOS. The portable --reflection-arrays consumer passes locally and is included
in both action jobs; Windows qualification is pending.

The unchanged typed JSON assembly now reaches ReflectionPropertyGet admission.
Property discovery, checked accessor dispatch and the end-to-end JSON milestone
remain unfinished. The vector service binding is a private backend implementation
of existing APIs and requires no Raven compiler or public API snapshot change.

## Checked property invocation — 2026-10-09

The private roots schema now also accepts version 2, whose type entries require
Boolean construct, properties, getters and setters policies alongside definition.
Version 1 remains supported without property retention. Getter/setter roots require
properties=true; metadata checks and executable roots remain independent. The first
property adapter subset is nongeneric reference owners with nonindexed instance
Int32, Boolean or reference-valued properties. It invokes real accessors with ordinary
virtual dispatch and never substitutes raw user-field writes.

The existing private property services now preserve access/missing/index/receiver/
value checks, exact boxed primitives, nullable reference values and terminal accessor
faults. Configured metadata without invocation rights can be checked without retaining
accessor bodies; attempting unretained invocation fails explicitly. Original source
visibility and source member identities determine access. The generated adapter/caller
frames match interpreter execution. This implements the current Reflection extension
contract, not a new public capability API; the [pre-stable review](introspection-design.md#pre-stable-api-evaluation--2026-10-09)
tracks its design questions independently. CLR PropertyInfo access is the baseline;
neoCLR retains its bounded Result-validation/terminal-fault split, with explicit
code retention and linear dispatch costs.

Focused tests cover real setter effects, denied/missing accessors, invalid/negative
property indices, null receivers, wrong/null value boxes, user getter/setter faults
and metadata-only body exclusion. The portable property consumer passes on macOS
and is added to both action jobs; Windows qualification is pending. JSON admission
now reaches TypeElementType; TypeProperties and descriptor materialization are still
required before JSON can run. No public API or compiler contract changed.

The preceding vector gate and all six earlier gates pass on macOS ARM64 and Windows
x64 in [action 37988417870](https://github.com/marinasundstrom/neoCLR/actions/runs/37988417870).
All fourteen reports and source/artifact hashes are verified in
[array evidence](native-reflection-array-validation.json). This also qualifies the
vector descriptor changes present in the earlier successful action 37986218456.

## Public descriptor snapshots and first flat round trip — 2026-10-09

TypeProperties now creates ordinary traced runtime descriptors for explicitly rooted
property metadata. A feature-gated Rust bridge, private to the in-repository native
compiler, obtains recipes from the interpreter's metadata queries. It preserves
source member/accessor identities, metadata tokens, index and accessor parameters,
visibility and optional module records. Standard Option case constructors and the
existing runtime provider constructors materialize the snapshots; the backend does
not invent a second JSON-specific descriptor format. Source metadata identities
are assigned before canonical relocation so recipe handles remain resolvable.

Original load-set access is verified first. The backend then gives internal access
only to the exact scoped runtime snapshot constructors and mutable identity fields
needed to finish these freshly allocated descriptors. The report lists these factory
projections; user model fields/accessors are not promoted. Source origins remain
unchanged. This is a temporary private materialization bridge, with constructor/layout
coupling and generated code/allocation costs; image-owned descriptor tables remain
the intended later representation. No public Raven API or compiler bridge change is
required. The installed runtime does not enable the Rust native-metadata feature.

Element lookup returns the existing Option<TypeInfo> for explicitly produced closed
vector tokens. Nonarrays return None; an unknown vector fails explicitly. Generic
flows that produce additional vectors still need retention-closure work. Property
queries apply current public/nonpublic and instance/static flags and preserve the
existing declared-only enumeration behavior; invalid flag bits fail explicitly.
Fresh descriptor wrappers do not grant invocation rights.

The expanded public Raven consumer discovers its property, reflects its getter/setter
signatures and setter parameter/module metadata, performs GetValue/SetValue, checks
filtering and resolves an array element. Interpreter/native macOS runs both print
Report and exit zero. This exposed inherited interface implementations on descriptor
providers: native selection now reuses VM conformance-anchor resolution for class
inheritance. A focused test covers inherited and re-declared interface conformance.
A separate generic-unboxing fix closes unbox.any method arguments before native box
projection, validated through an executable generic helper and the typed serializer.

The unchanged flat JSON round trip now executes natively on macOS with matching
UTF-8 output and exit code. The console project driver now enables the existing
checked stack budget and integer-text services consistently on both platforms,
allowing the recursive mapper with the synchronous console host. Expanded tests add
syntax/missing/null/wrong-type errors, validation before a faulting model constructor,
escaping and Int32.MinValue. The serializer's canonical control-byte spelling is
\\u00xx; comparing it with short input escapes was a test expectation error in both
execution modes, corrected without changing serialization behavior.

## Expanded author-directed milestone — 2026-10-09

The author explicitly requests nested objects, arrays and collection types, and
selects **typed lists/sequences and string-keyed maps** for this milestone. The flat
round trip is therefore a foundation, not completion. Qualify the existing recursive
object/vector paths natively, then define collection shape/construction policy in
the shared mapper and validate both modes. Preserve element order; string-keyed maps
map to JSON objects. Arbitrary custom collection construction, non-string keys,
polymorphism and null/optional mapping are not silently implied by those choices.

After that correctness gate, the author directs JSON benchmarking and publication of
data, with .NET/other-framework comparisons later. Use the measurement protocol
above, publish reproducible workload/toolchain/hardware details and retain raw data.
No benchmark result, speed claim or publication is implied by the working flat probe.

The low-level property gate and seven preceding gates pass on macOS ARM64 and
Windows x64 in [action 37989124785](https://github.com/marinasundstrom/neoCLR/actions/runs/37989124785).
All sixteen source/artifact reports are verified in
[property evidence](native-reflection-property-validation.json). This is separate
from the new public Raven JSON project workflow, whose Windows result is pending.

The author additionally requires representative JSON documents to be deserialized
and serialized both as JsonValue trees and typed objects. Use the same documents
where shapes are supported; assert field/element values after round trips and test
malformed documents. Compare semantic values, allowing documented canonical escape
spelling. Include empty and nested containers, UTF-8, integer boundaries and ordering.

The representative document consumer now passes on macOS ARM64 in both modes:
two nested Bulletin/Report documents use both DOM and typed paths, with field-level
checks after deserialization and a second round trip. A mixed DOM document covers
arrays, null, empty containers, Unicode and decimal token preservation. Trailing
array commas and wrong nested property types are rejected. See the
[corpus](experiments/native-json/README.md#representative-document-checks). Typed
collection coverage remains open. Test helpers use local propagation bindings to
avoid the pinned compiler's expression-boundary limitation; no compiler change is
claimed.

## Typed vector admission follow-up — 2026-10-09

Native specialization now accepts closed vector generic arguments and lowers their
reference unboxing through the existing checked cast. Element descriptors are
retained from explicit generic call arguments and rooted property signatures, as
well as direct tokens. This avoids requiring redundant typeof expressions in JSON
consumers. It is still a bounded source inventory, not arbitrary runtime discovery;
indirect generic flows and jagged native array storage remain separate work.

The public corpus adds Int32, Boolean, String and Report array deserialization,
including an empty Report array, value checks and integer/object array serialization.
Native macOS and interpreter output match. A focused generic-unboxing consumer
covers the vector argument alongside existing scalar/fault cases. No JSON-specific
backend serializer or public API is added. CLR generic array arguments are the
behavioral baseline; the temporary native compiler specializes closed vectors.

The first Windows project action (37991032994) stopped at profile admission: the
console flag check still rejected integer-text services although the host already
links the shared implementation. The check now admits those services, preserving
rejection of file/path/task/socket/character services. Windows execution must still
qualify the complete project; this admission correction alone is not that evidence.

The expanded project also cross-compiles to Windows x64 COFF with the corrected
console gate. [Evidence](native-json-document-validation.json) records source,
build and object hashes separately from macOS execution. Windows execution remains
unverified until the project workflow passes.

## Windows document project qualification — 2026-10-09

[Action 37991724441](https://github.com/marinasundstrom/neoCLR/actions/runs/37991724441)
passes both public projects at `91911ce9`: reflection and the nested/DOM/typed-vector
JSON corpus. Each standalone Windows x64 executable matches the interpreter in
exit code, UTF-8 stdout and empty stderr. All retained artifact/source hashes were
verified; [evidence](windows-native-json-validation.json). This qualifies the
existing corpus, not the still-unimplemented list/sequence/map mapping.

## Generic collection metadata prerequisite — 2026-10-09

Implement native TypeArgument behind the existing TypeInfo.GetGenericArgument API.
Use the same source type identity as the VM, mapped back to retained native shapes;
close argument-token production before generating descriptor dispatch. This runs
after executable/type selection, avoiding a source-wide scan that retained unrelated
HTTP error layouts and unsupported floating-point types in an initial local attempt.
That attempt was discarded, not shipped. Missing argument storage metadata fails
at compilation; unknown descriptors and negative/out-of-range indices fail explicitly.
No constructor, getter or setter is retained merely by requesting generic arguments.

This fills an implementation gap relative to CLR generic type argument inspection,
using the existing neoCLR index-query contract and terminal fault behavior. It does
not introduce a new API or claim an ergonomic improvement over .NET. The costs are
linear generated dispatch and bounded image metadata; general dynamic generic
construction remains outside this slice. The public reflection consumer checks
Sequence<Report> and Map<string, ArrayList<Report>>, while a focused executable
checks identity and index-fault/caller parity. This is a prerequisite for the shared
collection mapper, which remains unfinished.

Validation: the expanded public introspection and unchanged JSON projects pass
standalone native/interpreter parity on macOS ARM64. The focused native argument
query consumer passes valid identity, negative and out-of-range checks with matching
RuntimeError text and caller instruction. Windows qualification of this new query
will follow the project action; prior Windows JSON corpus evidence remains valid.

## Explicit JSON subtrees in typed models — 2026-10-09

The author requests dynamic JSON content within typed neoCLR objects and explicitly
chooses node declarations instead of inferring nodes for Object properties. The
shared mapper now recognizes JsonValue and all six concrete node kinds by type
identity, before ordinary model reflection. JsonValue accepts any kind; concrete
node declarations require a matching kind, otherwise TypeMismatch is returned
during whole-input preflight before user constructors/setters. The existing generic
and TypeInfo overloads support these declarations at roots, properties and vector
elements. Generic collection elements will use this same path once collection
mapping is implemented. Node implementation properties are never serialized.

The parsed node is retained, without cloning or numeric conversion. JsonNull is
explicit JSON data; null references remain unsupported, and Object-typed model
properties do not infer nodes. Serialization embeds the node into the complete
output tree, whose writer still checks total depth, value occurrences, bytes and
cycles. It does not validate each subtree with a fresh, independent limit budget.

Comparison: System.Text.Json supports DOM serialization/deserialization through
JsonNode and concrete kinds ([official DOM guide](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/use-dom),
reviewed 2026-10-09). neoCLR's JsonValue is the whole DOM root, analogous to .NET's
JsonNode rather than its scalar JsonValue class. This slice adopts mixed model/DOM
composition using the existing neoCLR representation; it does not claim a .NET
improvement. Explicit JsonNull preserves the existing missing-versus-null distinction
but differs from .NET nullable-node conventions. Alternatives—Object inference or
reflecting over node implementation fields—were rejected for this slice: the author
chose explicit declarations, and reflection would expose implementation structure
instead of the intended JSON. Retained mutable nodes preserve reference identity;
callers needing an independent copy must explicitly round trip or copy data.

The NodeEnvelope consumer covers object, scalar and array subtrees, number token
spelling, repeated round trips, every root node kind, concrete kind mismatches,
preflight before a faulting constructor, embedded depth overflow and cycles. Native
macOS and matching interpreter runs pass; Windows execution of this new library
slice awaits the project action. Lists/sequences/maps remain the next main work.

The mixed-node macOS report and source/bundle hashes are recorded in
[node evidence](native-json-node-validation.json). The preceding generic query
project also passes Windows action
[37993153131](https://github.com/marinasundstrom/neoCLR/actions/runs/37993153131);
that earlier action does not include this embedded-node mapper change.

## Converter and wire-contract follow-up — 2026-10-09

The author requests eventual System.Text.Json-style converters, inheritance
contracts and Raven union wire formats, then explicitly directs finishing the
current basic JSON support first. Converters are deferred; this does not change
the active nested-object, vector, list/sequence, string-keyed map and explicit-node
milestone. No converter API or polymorphic mapping is implemented by this note.

When that follow-up begins, compare .NET's typed converters and generic converter
factories with its separate contract/polymorphism configuration. Reuse recursive
mapping, limits and retained type identity rather than adding JSON policy to the
runtime. Resolve registration precedence, nested delegation without converter
recursion, null handling, discriminator collisions/unknown cases, and converter
effects during validation before selecting a public API. The current guarantee
of input validation before model constructors/setters must not silently become
a guarantee that arbitrary user converters have no effects.

Primary references reviewed 2026-10-09:
[converters](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/converters-how-to),
[contracts](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/custom-contracts),
and [polymorphism](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/polymorphism).
Use .NET 10 as the initial compatibility baseline; later-version additions in
these evolving pages are separate evaluation inputs, not baseline requirements.

Raven source inspected at 6c90bf2c48d9d60480476433f90ddc4d0cee4a0d
(`src/Raven.Core/Option.rvn`, `Result.rvn`, `UnionJsonConverter.rvn` and
`test/Raven.Core.Tests/UnionTest.cs`) already distinguishes:
Option's payload-or-null representation; Result's `case` plus `value`/ `data`
representation; parenthesized unions' primitive payloads and typed object form;
and opt-in tagged unions using a configurable discriminator (default `$case`).
Generated cases flatten their properties; direct payloads use a naming-policy-aware
`Value` member. Do not collapse these into one universal tagged representation.
These are source-inspected contracts, not a fresh Raven test run or a claim that
neoCLR supports them. Null/inactive-carrier behavior and unknown-case fallback
need explicit compatibility tests before adoption. A cross-target golden-document
corpus should precede any compatibility claim.

## Built-in collection mapping — development, 2026-10-10

The shared mapper recognizes exact source-owned Sequence/List/ArrayList and
string-keyed Map/MutableMap/HashMap families. It recursively maps their elements
using the same scalar, model, vector and explicit-node rules. Reads construct
ArrayList or HashMap with StringComparer.Ordinal; interface declarations accept
the matching built-in implementation. Custom collection implementations and
non-string keys remain unsupported. Map output follows observed key order, but
custom comparer identity/behavior is not serialized.

Compared with System.Text.Json's [supported collections](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/supported-types),
the familiar sequence-as-array and dictionary-as-object shapes are retained.
neoCLR deliberately starts with a smaller construction policy and requires string
keys. This reduces implicit conversion/factory rules; the cost is fewer compatible
models. Existing required-property, exact scalar, explicit-null and document-limit
differences remain. No performance advantage is claimed.

Private adapters bind retained closed element types to ordinary generic Raven
collection methods. The Rust planner is shared by interpreter and native tooling;
the generated native dispatch checks runtime TypeInfo identity before calling the
same helper bodies. JSON names, parsing, validation and recursive mapping remain
in ObjectMapper. The private service/helper names and source scope are a temporary
execution bridge, not a public reflection API or stable ABI. Type tokens, explicit
property roots, concrete generic call arguments and constructed collection owners
supply the bounded inventory.
General arbitrary runtime generic construction is not implied. Direct closed
constructions are conservatively inventoried across the load set, which can retain
more collection adapters than the consumer needs. Constructor inference excludes
callable type arguments whose native metadata is not admitted; it does not turn
unrelated task-queue storage into reflection roots. Native execution adapters are
limited to admitted reference/Int32/Boolean elements; unsupported scalar families
remain classifiable without selecting unsupported boxing bodies. Precise type-flow
retention remains follow-up work. The clone allowance now shares the existing
1,024-function cap instead of stopping independently at 512 clones; the total
function and 512-type limits are unchanged.

Map reads use real TryAdd and map writes use real Keys/Find, preserving ordinary
checks. Keys currently returns a snapshot; repeated indexed mapping therefore has
quadratic copying cost within the 31-entry container limit. Replace that with a
stable enumeration protocol when larger documents/performance work justify it.
Concurrent mutation during serialization is unsupported. The entire JSON input
is preflighted before model constructors/setters, including collection elements.

StringComparer.Ordinal now directly invokes the private UTF-8 content hash
contract, preserving the interpreter's existing FNV-1a result without requiring
general Object.GetHashCode virtual dispatch. Native ordinal/ignore-case kernels
reuse the interpreter's hash/folding implementation and allocate no temporary
byte vector. Hashes remain in-process collection values, not wire identifiers.
Generic reference boxing/unboxing uses ordinary checked reference casts; value
boxing remains restricted to existing admitted value shapes.

The project driver conditionally links the matched allocation-free native text
archive when selected bindings require Unicode comparison/hashing. Source builds
use Cargo; packaged macOS kits carry the archive. Windows uses the same source
kernel, with execution qualification still required for this slice. The JSON
validator builds the current interpreter so new private services are compared
against the matching implementation rather than an older bundled runtime.


The complete reflection/JSON project corpus passes on macOS ARM64 in
`target/native-json-collections-qualified/report.json`, including matching
optimized interpreter results. Follow-up write-only retention checks also pass in both modes;
[recorded evidence](native-json-collection-validation.json) includes source/binary hashes. Windows collection qualification and HTTP integration remain
next; earlier Windows node/array evidence is not collection evidence.
