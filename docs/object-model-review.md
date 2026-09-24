# Object, values and erased storage — consistency review

Reviewed 2026-09-23. The author directs harmonizing with .NET where useful,
particularly reference-type and value-type semantics. This review supersedes the
older proposal that Object inheritance should leave every type value-like, and
that all Object equality should describe fields independently of type category.
The opening review records the initial state. Subsequent implementation sections below
cover class identity/display, record classes, boxed Int32 and the first
[struct/record-struct slice](#struct-object-slots-and-record-structs--2026-09-24).

## Current layers

| Concept | Current neoCLR behavior | Consequence |
| --- | --- | --- |
| Class instance | Nominal reference type; assignment copies the reference | Aliases share mutation; a base/interface view retains the allocation |
| Value instance | Assignment copies fields; reference fields still share their targets | Value copying is shallow, not deep cloning |
| `System.Object` | Abstract Raven class with GetType, ReferenceEquals and virtual equality/hash/display | Common reference view, not an arbitrary unboxed payload slot |
| `box T` | Copies a supported value into GC-owned storage exposed through Object/interfaces | Allocation, shared boxed identity and copy independence are observable |
| `System.Value` | Intrinsic, explicitly erased complete payload; pack/is/unpack operations | Carries exact type and value-copy behavior; not .NET ValueType or Object |
| `System.ValueType` | Compiler/reference metadata role | Not a replacement name for System.Value and not a complete executable .NET ValueType implementation |

Source evidence: [Object](../runtime/raven/src/System/Object.rvn),
[Option](../runtime/raven/src/System/Option.rvn),
[Result](../runtime/raven/src/System/Result.rvn),
[TaskOutcome](../runtime/raven/src/System/Tasks/TaskOutcome.rvn),
[class semantics](class-semantics.md) and [boxing](boxed-interface-values.md).
The raw/legacy value-and-managed-byref profile is not proof of the Raven class
contract; preserve that distinction in tests and documentation.

### A reference declaration is not implementation evidence

CoreDeclarations supplies Object Equals, GetHashCode and ToString stubs to the
reference/compiler environment. At review time the generated Object library only supplied GetType. The subsequent
class-display slice implemented ToString; later equality/hash and struct slices below
replace those particular stubs with bounded implementations.
These stub bodies must never be executed or advertised as working implementations.
The subsequent author-directed abstract Object choice permits base-constructor
chaining but deliberately rejects direct construction. Keep this gap explicit in the on-site docs.
Do not remove compiler scaffolding without checking compiler synthesis dependencies.

## .NET baseline and preferred adaptation

Sources retrieved 2026-09-23, targeting .NET 10 public contracts:

- [Object](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
  supplies GetType, virtual Equals/GetHashCode/ToString, static equality helpers,
  protected MemberwiseClone and finalization support. This is a library surface;
  C# assignment rules, CLI boxing and GC identity are separate layers.
- [Equals](https://learn.microsoft.com/en-us/dotnet/api/system.object.equals?view=net-10.0)
  defaults to reference identity for Object, while
  [ValueType.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.valuetype.equals?view=net-10.0)
  compares value contents. Overrides can define domain equality. Adopt this distinction
  as the design baseline; do not force structural equality on ordinary mutable classes.
- [GetHashCode](https://learn.microsoft.com/en-us/dotnet/api/system.object.gethashcode?view=net-10.0)
  must agree with equality. Equal objects require equal hashes; distinct instances
  need not have distinct hashes. Implement equality and hashing together. Hashes
  are not persisted identities, addresses or guaranteed stable between processes.
- [ReferenceEquals](https://learn.microsoft.com/en-us/dotnet/api/system.object.referenceequals?view=net-10.0)
  compares identity without virtual equality, including two null references. neoCLR's
  existing `ref.eq` also compares managed interior locations; retain that low-level
  operation separately from an Object-level API. Intrinsic String currently lacks
  canonical object identity, so a universal identity helper needs that case resolved.
- [ToString](https://learn.microsoft.com/en-us/dotnet/api/system.object.tostring?view=net-10.0)
  has a useful type-name default with overridable display text. This is a good first
  Object extension, provided calls through Object actually dispatch to overrides.
  Do not add a nonvirtual lookalike merely because it is easier to import.
- [MemberwiseClone](https://learn.microsoft.com/en-us/dotnet/api/system.object.memberwiseclone?view=net-10.0)
  is shallow copying, not resource duplication. Defer it until a concrete caller
  and derived-layout/readonly/resource rules are tested. GC finalization likewise
  must not be introduced as a substitute for deterministic resource cleanup.

The [executable .NET baseline](experiments/object-baseline/README.md) checks aliasing,
shallow value copies, boxing, equality/hash consistency, null identity and formatting.
It makes no performance claim and does not claim neoCLR already matches every case.

## Keep, revisit and remove

**Keep:** reference/value category distinctions, typed `Equatable<T>` contracts,
explicit boxing and GetType's concrete-type behavior. An Object base method must
not silently change ordinary value assignment or introduce boxing in generic storage.
Maintain current Result/Option ergonomics independently of payload representation.

**Revisit:** Object metadata versus executable members, override dispatch through
Object, defaults for Equals/GetHashCode, String identity and boxed-value dispatch.
Choose CLR-compatible semantic outcomes where possible; internal layout need not
copy CoreCLR. Identity hashes must survive collection without exposing pointers.
Value equality needs a deliberate policy for reference fields, floating point,
cycles and custom Equatable implementations rather than raw byte comparison.

**Retain temporarily:** System.Value and its instructions. Option, Result, TaskOutcome,
native return protocols and host input/inspection still depend on this representation.
Replacing every erased field with Object today would change copying to box aliasing,
introduce observable identity and leave exact extraction/unboxing gaps. Replacing it
with Void* instead requires ownership, tracing and copy/release machinery for managed
payloads. Neither is a rename. Keep the prior retirement goal, but require a measured,
tested migration before removing the type, instructions or artifact encoding.

Rust's [Any](https://doc.rust-lang.org/std/any/trait.Any.html) provides exact runtime
type inspection/downcasting independently of universal equality. It is a useful
comparison for separating responsibilities, not a proposed neoCLR replacement:
borrowing, owned boxes and lifetimes differ. No alternative .NET library was selected
as a replacement for Object/CLI value semantics; this review concerns foundational
runtime contracts rather than a library performance problem. A deeper source-pinned
runtime/issue review is required when implementing hashing or changing storage.

## Bounded implementation order

1. **Review completed:** record actual behavior and gaps; cover Object.GetType and Value
   in the API reference; correct obsolete direction notes. No runtime API expansion.
2. **Class-display slice completed (bounded scope below):** a small Raven object-display sample with a base reference and derived
   override. Implement/test Object.ToString with a type-name fallback, ordinary
   virtual dispatch and honest importer diagnostics. Check class, boxed value,
   String and null paths before declaring the surface general. Keep Console object
   overloads and automatic interpolation integration out until this dispatch works.
3. **Class identity/equality slice implemented (limits below):** Object.ReferenceEquals
   and the Equals/GetHashCode pair, with equal
   payload/distinct identity, null, boxed values, custom overrides, mutable fields
   and GC tests. Assess typed collection equality alongside this contract.
4. **Separately:** inventory and migrate one Value-backed carrier with nested managed
   fields, copy/alias tests, GC, host/native boundaries and artifact migration evidence.
   Do not remove Value merely because Object exists.

This order is an assistant recommendation under the author's consistency direction,
not approval of an entire .NET Object clone. Storage cleanup, suspension and scheduling
remain open follow-ups; this review does not pull networking forward.

## Class display implementation — 2026-09-23

Object.ToString now has a Raven body returning GetType().FullName and a virtual slot.
The importer preserves ordinary core Object ancestry for application classes, so a
ToString override has an actual inherited contract. Reference calls retain the CIL
call/callvirt distinction: explicit base calls must not re-enter the override.
Rootless nominal classes and managed arrays already have an admitted Object view;
they use the default Object slot when they have no declared Object ancestry.
Only declared ancestry contributes overrides; no arbitrary same-named method is
promoted into an override. No new opcode, native formatting service or universal
base requirement is added to the raw runtime profile.

Boxed-value and intrinsic-string virtual dispatch remain unsupported and are tested
as failures rather than silently returning a type name. GetType still handles those
views. This is a deliberate bounded gap from .NET, not a preferred semantic divergence.
The [sample](experiments/object-display/README.md) covers class fallback, override,
base-reference calls and explicit base calls. Broader formatting remains contingent
on proper payload receiver dispatch. Equality/hash and Value migration are next
independent slices, not implemented by the display change.

### Author-directed abstract Object

The author proposes abstract Object, with a dedicated synchronization concept if
locking is later needed. The implementation adopts that direction: abstract in
Raven/reference metadata and the runtime declaration, with a protected reference
constructor and an empty runtime base-constructor entry. Derived construction chains
through it; direct allocation must fail in both source and raw IL. This differs
from .NET's concrete Object and removes its use as a generic instantiable token.
The benefit is a narrower base-class role; the migration cost is replacing `new
Object()` idioms with a domain type. No lock API is introduced or promised.

Migration: application classes now retain Object as their metadata base. A class
that supplies ToString should declare an override; same-name hiding is rejected by
the current importer/runtime profile. Use matching reference/library artifacts.


## Identity prerequisites — 2026-09-23

This bounded follow-up characterizes the next API slice before adding public
ReferenceEquals/Equals/GetHashCode methods. A caller needs to distinguish a shared
object from a different object with equal contents, including after casts and GC.
The six raw-runtime tests in [object_identity_contract.rs](../tests/object_identity_contract.rs)
roundtrip metadata through an artifact and check class/interface/Object aliases
through mutation and collection, distinct boxes, array aliases, typed nulls, and
execution-local diagnostic identities. They also record a concrete string gap:

- Converting a String payload to Object allocates a wrapper on each conversion.
  Reusing a stored Object wrapper preserves its identity; repeating the conversion
  from the same String does not. Casting back to String loses the wrapper identity.
- The .NET comparison preserves String identity across these conversions while
  independently allocated equal strings remain distinct. String content equality
  and equal hashes do not imply reference identity.
- neoCLR's existing `ref.eq` is usable for class/array/box handles and nulls, but its
  support for managed byref locations is a different contract. Object.ReferenceEquals
  must not expose stack/interior-location equality as object identity.
- Heap allocation IDs are stable and never reused within an execution. They restart
  in another execution, so comparing diagnostic IDs alone is not a host-level
  identity test. Current handle equality also checks the owning allocation.

### .NET layers and implementation choices

Sources checked 2026-09-23: [.NET 10 Object source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Object.cs)
implements ordinary class Equals using identity, static Equals with identity/null
shortcuts followed by virtual dispatch, and GetHashCode through RuntimeHelpers.
The [CoreCLR helper](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/System.Private.CoreLib/src/System/Runtime/CompilerServices/RuntimeHelpers.CoreCLR.cs)
checks an existing hash before entering its runtime slow path. These are library
and runtime layers; the signatures alone do not provide their behavior. The
[ReferenceEquals contract](https://learn.microsoft.com/en-us/dotnet/api/system.object.referenceequals?view=net-10.0)
and the expanded 14-assertion .NET baseline distinguish identity from overridable
equality. The baseline requests compacting GC but does not prove relocation.

Rust's [Rc::ptr_eq](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.ptr_eq)
likewise separates shared-allocation identity from payload equality. Its allocation
model does not establish a safe address-based hash for a future moving managed heap.
The earlier alternative-library review remains applicable: a library alone cannot
repair identity lost by a runtime String conversion. No new .NET ecosystem library
or API-review issue is claimed as evidence for changing Object's contract.

Prefer .NET semantic outcomes over promoting wrapper identity into a public promise.
The next implementation should keep ReferenceEquals nonvirtual and implement default
class Equals/GetHashCode together, with override tests. An execution-local allocation
ID can seed a stable 32-bit identity hash without exposing an address; the hash is
not a unique ID, a serialized value or a cross-execution contract. Mixing/folding
and service placement still require implementation and tests. Hashing mutable fields
for default class equality is rejected because mutation would change an identity
hash; automatically structural class equality would violate the selected baseline.

For strings, compare a stable managed identity retained by payload copies and Object
views against making String fully heap-backed. The former touches intrinsic/host
representations; the latter also changes allocation budgets, roots and native
boundaries. Interning by text would incorrectly merge distinct strings. Retaining
today's wrapper behavior is cheapest but fails the desired cast/alias semantics.
No representation change is selected by these characterization tests. Resolve this
explicitly before describing Object identity as general; a first class/array/box-only
API must state its String exclusion. Boxed value Equals/hash also needs payload
receiver dispatch and a deliberate value policy; identity alone does not supply it.

Typed collection equality remains a separate review item: an Object override must
not silently replace existing Equatable<T> contracts. The follow-up adds no public
methods, hash algorithm, collection change or Value migration.


## Class equality/hash implementation — 2026-09-23

The next bounded implementation adds nonvirtual Object.ReferenceEquals and virtual
Object.Equals(Object)/GetHashCode with ordinary class overrides. The Raven bodies
call narrowly bound runtime identity services; no new opcode or collection policy
is introduced. Both aliases and null handling are separate from field equality.
Arrays use the same default identity contract. ReferenceEquals compares box identity
without promising boxed value equality. String payloads and String wrappers are
explicitly rejected by the identity services, including self-comparison, so the
current conversion defect cannot become an accidental API promise.

The default hash mixes the execution-local allocation ID into 32 bits using wrapping
integer arithmetic. The ID is stable across mutation and collection; the hash does
not expose a native address. Collisions and repeated hash values in separate runs
are permitted. No cryptographic, persistence or performance claim is made. Custom
Equals/GetHashCode implementations remain responsible for equal values having equal
hashes; automatic verification of that invariant is not supplied.

The native identity comparison validates live references and is nonvirtual. Instance
Equals separately requires a non-null receiver, including raw direct calls; it does
not inherit static ReferenceEquals(null, null)'s true result. All three imports
report the existing ManagedHeap service. The abstract Object declaration now owns
its static and instance methods together in generated library fragments. Importer
adapters retain direct/virtual call distinctions for Equals and GetHashCode as for
ToString. Explicit base calls use defaults; virtual calls reach class overrides.

The [class sample](experiments/object-equality/README.md) compares mutable Cell
identity with Key equality through Object parameters. Its Key uses a private field
and getter: general readonly field/init-only admission is not added by this slice.
The raw tests cover GC, array views, separate boxes, nulls, override/base dispatch,
unsupported String/boxed paths and exact native signatures/service reporting.
Static Object.Equals, boxed virtual equality/hash, String identity and Value storage
migration remain unimplemented. Existing typed Equatable contracts are unchanged.

### Record syntax as the end-to-end acceptance case

The author identifies record semantics with record syntax as the ultimate test.
This becomes a concrete follow-up gate, rather than calling handwritten equality
sufficient record support. The first probe is
[RecordProbe.rvn](experiments/object-equality/RecordProbe.rvn): `record class
Key(Number: int)` must retain class aliasing and distinct allocation identity while
its generated typed/Object equality, operators and hashes agree on components.
The pinned .NET baseline now includes the corresponding fifteenth assertion.

The [.NET record baseline](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/records)
(checked 2026-09-23) distinguishes compiler-generated value equality from the type's
underlying class/value assignment semantics. Record class assignment still aliases;
record structs copy their fields. Neither promises recursive deep equality or copying.
Extend the initial integer case with String and nested reference fields, generated
display/deconstruction, and the Raven-supported value-record/copy forms after the
basic gate runs. Check actual Raven syntax and semantics rather than assuming every
C# record feature exists.

Current probe outcome: source reaches emission, then fails with `Failed to resolve
EqualityComparer<T>.` on the matching target bundle. Inspection of Raven's
`SynthesizedMethodBodyFactory.Records.cs` also identifies System.HashCode.Add<T>
and ToHashCode dependencies for generated hashing. These are actual target contract
integration gaps; the ordinary Object methods alone do not supply them. The emission
failure is reported as an exception rather than a friendly missing-contract diagnostic.
A general diagnostic improvement is a separately validated Raven candidate; it must
not move neoCLR policy into Raven main.

Next compare a bounded typed comparer/hash library contract with Runtime Contract
configuration that maps record synthesis to neoCLR's existing Equatable conventions.
Do not introduce universal structural Object equality to make records work: generated
record members should express their component policy through normal contracts.
No compiler change, comparer/hash API or record support is claimed in this slice.


### Author clarification after the probe

The author directs making Raven support neoCLR record semantics and implementing
System.HashCode. This selects compiler/library integration as the next work, beyond
merely documenting the absent .NET comparer. Preserve ordinary .NET record behavior
when the target contract is unconfigured; keep neoCLR policy and target fixtures on
the isolated branch. Start with the small record-syntax acceptance source and expand
only with explicit component equality/hash rules and executable evidence.


### Integer records and HashCode — 2026-09-24

The author-directed follow-up now has a passing [record sample](experiments/records/README.md).
Raven uses the opt-in target contract for System.Equatable<Record> and System.HashCode;
integer components use value comparisons without boxed comparers. Class allocation
identity, typed/Object equality, operators, hashes, display and Deconstruct agree.
Normal .NET synthesis retains its existing path. The first target contract rejects
record structs, generic/inherited records and other components with RAVT004.

[HashCode](hash-code-design.md) supplies Add(int/string), ToHashCode and Combine(int,int)
as a mutable Raven value type. Generic/comparer overloads and a hardened hashing
algorithm are not implied. The importer recognizes private readonly backing fields
and init-only property signatures, checks write contexts, and imports integer output
parameters. Init-only assignment remains a compiler rule; runtime field flags and
application-property reflection remain gaps. See the [integration contract](experiments/raven-target/README.md#record-and-hashcode-integration--2026-09-24).

A same-compilation generic provider fixture exposed a separate reentrant lookup-cache
issue in Raven. The accepted tests use a referenced provider assembly, matching neoCLR's
boundary. That general compiler candidate must be reduced/tested independently on main.

### String and nested record components — 2026-09-24

The next slice extends the checked record sample to Person and Entry. It preserves
Equatable<Record>, uses content equality for strings, and typed equality/hash/display
for nested same-compilation record classes. Output deconstruction now handles strings
and application references. Nullable components, metadata-only record components,
boxed equality and record structs remain open. See the
[component comparison and tradeoffs](hash-code-design.md#string-and-nested-record-components--2026-09-24).

Nullable record references now pass source construction, null/present equality,
hashing, display and deconstruction. This reuses existing reference storage and does
not settle intrinsic nullable String or nullable value representation. Boxed-value
equality remains the next bounded review before admitting record structs.

### Boxed Int32 equality and hash — 2026-09-24

Scenario: two Object references containing separately boxed 42 values must compare
equal without sharing identity; changing the source integer must not change its box.
The [.NET 10 Int32 implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Int32.cs)
compares the exact runtime type and stored integer, and returns that integer as its
hash (reviewed 24 September 2026). The pinned .NET baseline now has 22 assertions.
General ValueType equality and user struct overrides are broader contracts than this
primitive case and must not be approximated by comparing arbitrary interpreter data.

The bounded implementation recognizes the exact virtual Object.Equals(Object) and
Object.GetHashCode slots originating in the System library for an Int32 box. It reads
the validated heap payload directly. Equality with null or another concrete type is
false; hash is the stored integer. This intrinsic adds no heap allocation, native
pointer exposure, unboxing API or System.Value dependency. ReferenceEquals and explicit
Object base calls still use allocation identity. Ordinary class dispatch is unchanged.
Non-System lookalike Object slots do not opt into the intrinsic.

Alternatives: waiting for a complete ValueType hierarchy would postpone a simple
compatibility case; generic structural fallback could ignore custom reference-field
Equals and give incorrect hashes. An interpreter intrinsic is intentionally narrow:
it lacks a public value override mapping and adds a runtime-special slot. Named struct
and other primitive behavior, reflection-visible overrides, constrained calls and
non-interpreter backends require separate design. Reachability remains conservative
through Object's default target and already reports ManagedHeap; no new service is
introduced. Boxed ToString remains unsupported. Record structs remain gated.

Validation covers exact type/value checks, signed boundaries, source-copy independence,
GC, retained explicit base equality, existing class/array behavior and lookalike slots.
The Raven Object sample adds four boxed-integer checks without compiler changes.


## Struct Object slots and record structs — 2026-09-24

**Author direction:** continue the slices until structs and record structs are
implemented; skip further VS Code builds. Ordinary non-generic struct construction,
fields and value copying already existed. The missing pieces were boxed Object
slots, exact boxed type tests/unboxing, and configured record-struct synthesis.

**Baseline and choice.** ECMA-335 sixth edition (June 2012), I.8.2.4 and III.4.6,
4.32–4.33 distinguish copied boxing, boxed type tests and copied versus address
unboxing ([specification](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
consulted 2026-09-24). C# record structs generate component equality and hashing
while retaining value copying ([Microsoft record documentation](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/records),
consulted 2026-09-24). The pinned SDK 10.0.100/.NET 10.0.0 baseline now has 26
assertions, including record-struct copied boxes, exact-type Object equality,
interface equality, matching hashes and default integer fields.

The interpreter now adapts explicit Object overrides on rootless named value
records to a managed reference into the boxed payload. It does not add reference
storage ancestry or a universal structural comparison of internal Values. Return,
parameter and output contracts remain exact; readonly receivers are restricted.
Non-generic targets enter the dispatch reachability graph. `isinst T` keeps the
matching box; `unbox.any T` returns a copy. Null faults with NullReference; a different
boxed type faults with InvalidCast. No widening integer conversion occurs.

A new universal ValueType-style fallback could make ordinary struct equality more
.NET-compatible, but would require decisions about every field type, recursive
comparison and hash policy. Explicit overrides plus compiler-generated record
members reuse the existing method contracts and keep these questions visible.
Special-casing each record in the VM would duplicate language policy, while treating
value types as reference subclasses would undermine copying. The chosen adaptation
costs a box allocation at Object/interface conversion and dispatch work, preserves
GC rooting through the existing managed reference, and makes no speed claim. Future
AOT/JIT implementations need the same receiver adaptation. Application reflection
and generic target discovery remain bounded; no new reflection members are claimed.
The prior broader Object review remains the comparison context; no independent
library or other-platform mechanism is needed for this CLI interoperability repair.

Raven's opt-in RuntimeRecordContract now synthesizes struct typed equality without
reference guards, and Object equality with a type test, unboxed copy and component
comparison. Hashing, display, operators and deconstruction reuse the existing typed
component contract. The importer admits struct interfaces and emits virtual modifiers
before byref/readonly modifiers. The experimental branch keeps this target policy
separate from ordinary Raven/.NET synthesis; no neoCLR policy is merged to main.

Evidence: `tests/object_equality.rs` covers copied boxes/unboxing, exact types,
readonly protection, shared box mutation, GC and invalid signatures;
`docs/experiments/records` compiles and runs ordinary Counter plus Coordinate,
NamedCoordinate and OwnedCoordinate record structs. The compiler fixture checks
boxed null/type rejection on .NET. Source Object.Equals currently has a non-null
Object parameter, so a literal-null call is rejected by Raven; this does not remove
the runtime guard. Class and boxed-interface regressions remain required.

Bounds: no general ValueType fallback, generic structs in the application importer,
nested struct record components, nullable-value boxing, address-returning unbox,
reference-type unbox.any, or .NET null-string default semantics. The same record
component gate remains Int32, non-null String and supported same-compilation record
classes (optionally nullable). Those are follow-up capabilities, not required to
claim the checked first struct/record-struct implementation. System.Value retirement
remains a separate migration.


### Nested record-struct components — 2026-09-24

The next composition slice extends the existing typed-component contract to
same-compilation, non-generic record structs, in both record classes and structs.
Reuse the .NET/CLI baseline above: nested assignment and deconstruction copy values;
reference fields still retain references. The pinned .NET comparison now checks
nested Rectangle values, boxed/interface equality, hash agreement and display.
This is compatibility work, with no performance or deep-copy claim.

Synthesis calls typed Equals/GetHashCode using an addressable value receiver; it
adds null guards only for reference components. Existing typed display already
supports value receivers. The importer admits exact stores into declared instance
output parameters, enabling nested Deconstruct without permitting arbitrary byref
stores. Alternatives—boxing each component or reflecting over fields—would add
allocation or duplicate the component policy; neither is needed here. No runtime
instruction, library member or RuntimeRecordContract configuration is added.

Evidence: the compiler execution test covers class/struct containers, forward type
declarations, copy independence and reflected deconstruction. `Nested.rvnproj`
checks target Object/interface calls, recursive zero initialization, nested display
and independent deconstruction copies. It is separate from the larger Records
sample to remain within the unchanged importer method limit. Nullable struct
components still report RAVT004. Syntax, semantic symbol shapes and editor grammar
are unchanged; component properties remain ordinary typed properties.


### Default reference components — 2026-09-24

The next probe found that `default(Payload)` correctly contained null reference
fields, but generated hashing passed its string to HashCode.Add(string), reaching
Utf8Encode with null and raising RuntimeError. The runtime already had String null
defaults (`tests/string_defaults.rs`); the gap belonged to configured record synthesis.

Reuse the .NET comparison above and Raven's nullable type specification: annotations
do not replace zero initialization with constructor calls. The pinned .NET baseline
now has 30 assertions, including a non-nullable string field that is null in a default
record struct and still supports equality/hash/display. neoCLR now guards generated
string component equality, uses Add(int 0) for null hashes, and substitutes empty
text only for display. Stored/deconstructed values remain null. Class-reference
components already use corresponding guards. Returning empty strings from initialization
would lose the distinction between null and empty; making all String APIs null-tolerant
would expand unrelated contracts. This compiler adaptation needs no runtime opcode,
layout change, new allocation contract or API signature change. HashCode.Add(string)
continues to require non-null input. Ordinary Raven/.NET synthesis is unchanged.

Evidence: configured compiler execution checks null/empty equality, hashing, display
and reflected deconstruction. Defaults.rvnproj checks the target through typed, boxed
Object and Equatable calls, including default class-reference fields. It reproduces
the old failure and is included alongside the other checked website samples. Explicit
nullable-string declarations, nullable-value boxing and Object.Equals parameter
annotation alignment remain separate follow-ups; non-null declarations are not a
runtime proof that default reference fields contain an instance.


### Nullable Object arguments — 2026-09-24

The author directs retaining nullable annotations at least for reference types to
get compatible Raven behavior, without committing to a future metadata format.
The existing .NET Object baseline above already tests null comparison arguments;
this slice corrects the source/reference declarations to match that existing runtime
contract. No new null representation or value-nullability policy is introduced.

Object.Equals now accepts Object?; both ReferenceEquals arguments are Object?.
The bootstrap runtime-service declarations admit the corresponding null inputs.
Reference assemblies use existing CLI nullable annotations, scoped to this surface.
The importer extends its existing typed-null adapters to core Object as well as
application references. Null locals and call arguments are materialized with typed
initobj; evaluation order and argument positions remain preserved. No runtime
instruction or virtual signature changes. Making every Object argument nullable
would obscure receiver requirements; retaining the misleading non-null signatures
would prevent valid Raven calls. Explicit annotations express the implemented API
contract while leaving the final platform representation undecided.

The Object source sample checks null/null, null/instance, instance/null, nullable
locals, default/class-override/boxed equality, and a negative non-nullable assignment.
The source library and bootstrap snapshots are regenerated, and the on-site reference
shows the corrected signatures. Direct generated record Equals annotations remain
a separate compiler issue; calls through Object use the corrected base contract.


### Absence model clarified — 2026-09-24

The author clarifies the previous provisional-nullability direction: neoCLR favors
Option<T> for modeled absence for both value and reference types. Keep reference
annotations for Raven compatibility and existing null-based contracts. Nullable
structs/Nullable<T> and nullable-value boxing are explicitly deferred, not the next
implementation slice. They may be reconsidered later; no final metadata format is
selected.

The .NET/Raven comparison above distinguishes reference annotations from a nullable
value carrier. Retaining the former does not require implementing the latter.
Option provides an explicit union case for absence across both categories; the cost
is that its operations and generated-record integration need their own validated
contracts. This direction neither changes default reference initialization nor
claims that Option components already work in configured records. It supersedes
earlier suggestions here that nullable-value representation was a prerequisite for
continuing the bounded Object/record work. No runtime or compiler code changes.

### Record Object equality and nullable-value diagnostics — 2026-09-24

The generated Object.Equals override now preserves its inherited nullable parameter
annotation. A general Raven fix was developed and tested independently on a main-based
branch before integration; .NET reflection/import checks retain the annotation and
execution checks cover equal, unequal, null and unrelated objects. neoCLR's record
sample covers nullable Object locals passed directly to record class and struct
Equals. The separately generated typed Equals contract remains unchanged.

The author also requests an opt-in Raven diagnostic for nullable value declarations.
The general AllowNullableValueTypes option retains true as Raven's .NET default;
neoCLR selects false through its shared build props. RAV0407 reports "Value types
can't be declared as nullable". This keeps source diagnostics consistent with the
current absence model without implementing a nullable value carrier. Reference
annotations remain available; Option record components remain separate work.

### Typed record-class equality — 2026-09-24

The remaining concrete annotation gap was generated Equals(Record): nullable record
locals fell back to Equals(Object?) rather than selecting typed equality. The existing
body already handled null. The selected repair emits Equals(Record?) for classes and
keeps Equals(Record) for structs; changing record structs to Nullable<Record> would
conflict with the selected absence policy and is unnecessary.

Microsoft's [record reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/record)
(retrieved 2026-09-24) describes the typed class equality contract. The pinned .NET
10.0.100/.NET 10.0.0 baseline now checks nullable/literal-null arguments, interface
dispatch and reflected class-versus-struct parameter annotations (32 assertions).
This is compiler metadata and overload alignment, not a new runtime null mechanism.
Keeping only the Object fallback would leave misleading type information and choose
the less specific overload. The cost is correctly preserving dispatch and component
lookup after the annotation changes.

A general Raven main-based regression checks typed overload selection, emitted and
reimported metadata, null/equal/different values, interface calls, generic class
construction and preservation of explicit Equals declarations. It exposed an emitter
mismatch: binding ignored reference annotations for interface matching but emission
did not, causing TypeLoadException. Emission now matches top-level nullable reference
parameters and returns without erasing nullable value wrappers. The integration branch
also resolves nested record-class components by their underlying typed parameter while
retaining null guards. Equatable<T>'s public contract is unchanged.

neoCLR's sample adds absent/present Key? locals and literal-null comparisons. Generic
record support on .NET is compiler regression coverage, not new neoCLR component
support. Option components, nullable strings and broader signature matching remain
separate questions; no public library API, runtime instruction or metadata format is
added by this slice.

### Record comparison operators — 2026-09-24

The next annotation gap was generated class `==`/`!=`: both parameters were marked
non-null even though null comparisons already worked. The .NET comparison above
and the pinned baseline now check both operator parameters and null/value symmetry
(34 total assertions). Generated record-struct operands remain values. This is
reference contract alignment, not nullable-value support or a change to Option policy.

Annotating the operands exposed an internal lowering bug: a generated null guard
resolved overloaded `==`, recursively calling the operator until stack overflow.
The general fix uses Object.ReferenceEquals for these guards and preserves explicit
operators with nullable, non-nullable or mixed operands. Tests check runtime behavior,
emitted/reimported metadata and misleading custom operators. It was independently
validated on Raven's main-based feature branch and integrated as `1394aca99`.

The target branch additionally uses identity guards for record-class components in
equality, hashing and display (`d2a583386`, after general cherry-pick `da9481e50`).
String guards retain existing intrinsic lowering; this does not add String identity.
The focused target suite passes 66 tests. The general suite passes 61 tests plus a
separate custom-operator guard regression. No RuntimeRecordContract configuration or
public library signature changes are required. Existing generic/external record and
nullable string/value restrictions remain.

### Boxed Boolean equality and hash — 2026-09-24

Selected bounded follow-up: Boolean values passed through Object should retain
value equality and matching hashes, as Int32 already does. The pinned
[.NET 10 Boolean implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Boolean.cs)
(reviewed 2026-09-24) checks the exact Boolean type and returns 1/0 for true/false
hashes. A boxed integer 1 is not equal to true. Distinct boxes retain distinct identity.

Extend the existing exact System.Object virtual-slot intrinsic, rather than inventing
structural equality for every value or waiting for a complete ValueType hierarchy.
This keeps the implementation small and allocates nothing beyond ordinary boxing,
but retains the limitation that these overrides are runtime special cases, not
reflection-visible Boolean methods. Typed Boolean APIs, boxed display, other
primitive contracts and other execution backends remain separate work. No compiler,
metadata-format, nullability policy or public signature change is needed.

Validation compares the pinned .NET baseline with raw runtime tests and the Raven
Object sample: both values, wrong type/null, copied payloads, identity, explicit base
calls and survival through collection. Existing class/struct/Int32 checks must remain
valid; unsupported primitive dispatch must still reject rather than guess equality.

### Library-wide consistency checkpoint — 2026-09-24

The author now directs using these contracts throughout the runtime class library.
The foundation is bounded, not complete: primitive boxed ToString and most primitive
Object equality/hash remain missing, and typed methods alone do not establish virtual
Object dispatch. A source audit identifies these concrete next consumers:

| Area | Current evidence | Next bounded check |
| --- | --- | --- |
| Storage.Path | Typed Equals(Path) compares text; ToString is not an override; no GetHashCode | Make typed/Object equality, hash and display agree for separately parsed equal paths, null and unrelated values |
| HashMap | Constructor requires equality/hash callbacks and uses them for lookup | Demonstrate existing callbacks using a complete value-object contract, then evaluate a default comparer without removing custom policies |
| HashCode | Add(int) and Add(string) only; strings are hashed from UTF-8 bytes | Define a shared string hash contract before introducing general Object or generic inputs |
| Int32, Char, time/calendar values | Typed equality/display methods do not uniformly supply Object overrides | Add per-type typed/boxed/interface comparison matrices and close gaps incrementally |
| Error objects and Console | Error display methods and typed Console formatting exist | Check virtual display first; evaluate Object formatting overloads after supported values agree |

Proposed next implementation: Path is an existing value-like API with a concrete
storage use case and a visible typed/Object inconsistency. Preserve its current
ordinal logical-path semantics; do not add Windows normalization or provider identity
as part of this repair. Compare .NET value-object equality/hash rules and System.IO.Path's
static helper role before implementation. Then exercise Path through HashMap callbacks
to prove library reuse. A universal comparer or automatic structural fallback is not
yet selected: either could silently give identity semantics to value-like types or
allocate boxes in generic hot paths. Keep custom equality policies available.

Boolean/GC validation outcome: 43 focused Object/display/identity/boxing regressions
pass, plus a separate reference-field tracing regression. The latter clears the
source struct, retains its box under a three-object heap limit, allocates temporary
boxes to force collection, then reads the child through a virtual override. Final
collection must reclaim the box and child after returning a scalar. Existing tests
cover shared mutable box aliases, readonly overrides, identity, heap limits and
primitive copied payloads. This is evidence for these root paths, not proof of every
GC path or a change to the current non-moving collector. The Raven sample, 37 pinned
.NET assertions, refreshed 385-item API reference and 14-page website build pass.

### Path integration and wider library audit — 2026-09-24

The author requests continuing Object semantics in the library, including Path,
and investigating other classes. Path now implements Equatable<Path> and overrides
Object.Equals, GetHashCode and ToString. Equality compares the exact accepted text;
hashing feeds that same text to the existing HashCode accumulator. Separate parsed
objects retain separate reference identities. Provider resolution, native formats,
normalization and equality operators are unchanged. The importer now retains Object
ancestry and constructor chaining for library reference classes declaring overrides,
and admits Path's reference/interface conversions.

**Explicit operand policy:** the author objects to automatically nullable equality
operands, particularly because nullable value types are excluded. Equatable<T> stays
`Equals(other: T)`, not `T?`. The initial assistant implementation added Path? to the
typed overload; it was corrected before commit. Equals(Path) requires Path. The
existing Object.Equals(Object?) override is the explicit null-aware reference
boundary and returns false for null or another type. No interface change, Nullable<T>
or implicit Option is introduced. The sample checks both Equatable<Path> and
Equatable<int>; API absence remains a separate Option design concern.

The .NET comparison remains useful at the contract level:
[Object.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.object.equals?view=net-10.0)
and [GetHashCode](https://learn.microsoft.com/en-us/dotnet/api/system.object.gethashcode?view=net-10.0)
require equality/hash consistency. [.NET Path](https://learn.microsoft.com/en-us/dotnet/api/system.io.path?view=net-10.0)
is a static helper surface, not this validated value object. Keeping Path as an
immutable class avoids an invalid default struct at the cost of allocation. Its
hash is not persistent, collision-free or compatible with .NET string hash values.
The current string hash path encodes UTF-8 on each call; caching and generic comparer
automation are not part of this correctness slice. Sources reviewed 2026-09-24.

The [executable fixture](experiments/path-object/README.md) checks typed/interface/
Object consistency, null/wrong types, Unicode/case/root distinctions, map duplicate
keys, replacement, collisions and rehashing. An Object-keyed generic map currently
fails importer admission; the tested map uses Path keys with Object-dispatch callbacks.
This remains an explicit importer coverage gap, not a runtime map equality limitation.

#### Audit findings and follow-up order

| Area / source evidence | Finding | Proposed action |
| --- | --- | --- |
| Introspection/Descriptors.rvn: RuntimeTypeInfo.Equals and FromHandle | Typed equality uses handles; each query can create a wrapper. No Object overrides. The probe reports equal typed results, unequal Object results and unequal hashes for the same type. | Next bounded repair: preserve represented-type identity through Object equality/hash/display, with null and generic-type distinctions. |
| Introspection/AssemblyInfo.rvn, ModuleInfo.rvn, Descriptors.rvn member wrappers | Wrappers retain identities, handles, tokens or owner context but do not override Object contracts. | Define assembly/module/owner identity before equality; do not compare names or tokens without their scope. Audit repeated-query wrappers after TypeInfo. |
| String.rvn | Intrinsic String has typed equality but no source Object equality/hash/display overrides; boxed/string reference handling is special. | Separate representation-aware slice; align content hash with equality without inventing String reference identity. |
| Storage/FileSystem.rvn: FileSystem, LocalFile, LocalDirectory | Provider/root/path context and possibly aliases; no Object overrides. | Keep identity now. Lexically equal paths across providers do not establish equal storage items; do not reuse Path equality automatically. |
| IO stream/readers/writers, Tasks/Promise/TaskQueue, Concurrency Thread | Mutable progress, ownership or resource state. | Identity remains appropriate unless a specific contract says otherwise; add targeted alias/lifetime tests when these APIs change. |
| Collections ArrayList/HashMap and Array | Mutable contents with no sequence-equality contract; HashMap already consumes explicit callbacks. | Keep identity and custom callbacks. Evaluate a default comparer only after its typed/Object dispatch and boxing policy are explicit. |
| Date, Time, Instant, Duration; LocalDateTime | First four have typed equality; boxed overrides/hash/display are incomplete. LocalDateTime has no typed equality yet. | Incremental exact-type boxed/typed matrices; define component semantics before default structural behavior. |
| Char, numeric primitives, error structs and union carriers | Typed display/equality varies; most do not override Object. Int32/Boolean boxed equality/hash are special runtime paths. | Close supported primitive dispatch, then diagnostic display. Do not infer universal structural equality for Result/Option/errors. |

The [TypeInfo comparison](https://learn.microsoft.com/en-us/dotnet/api/system.type.equals?view=net-10.0)
is represented-type equality across typed/Object overloads in .NET. neoCLR should
compare its own handle identity, including context, rather than blindly copy wrapper
identity or CLR runtime caches. This is a selected investigation, not implemented
TypeInfo behavior in this commit. Broader reference conversion/importer coverage
must be verified alongside each consumer; source declarations alone are insufficient.

Compiler diagnostic gap: the probe `value: Equatable<Path>; value.Equals(null)`
currently compiles, even with explicit non-nullable reference metadata. Typed
implementations still require T; null handling is not promised by that acceptance.
Track generic-argument nullability checking as a separate Raven investigation.

Runtime follow-up exposed by Path ancestry: reachability incorrectly tried virtual
dispatch for nonvirtual class callvirt (Object.GetType). It now records the static
callee, matching execution's receiver-null-check behavior. A small independent
base/derived regression and the existing Object-default reachability case cover it.

### Introspection consumer slice — 2026-09-24

The author directs the next slice toward introspection types. RuntimeTypeInfo now
preserves represented-type equality through Object, with null/wrong-type rejection
at that boundary. Typed Equals(TypeInfo) and Equatable<TypeInfo> remain non-nullable.
GetHashCode feeds FullName to HashCode; ToString returns FullName. The internal wrapper
layout remains one opaque handle, and neither type identity nor native factory layout
changes. The importer changes from the Path slice preserve its Object base and
constructor chaining automatically.

This follows .NET's distinction between represented type and wrapper identity, using
neoCLR's existing TypeIdentity rather than CLR caches. FullName hashing is a bounded
library-only choice: equal types share a hash, but equal names from distinct definitions
can collide. Hashing allocates through the current UTF-8 implementation and is not a
persistent ID. A native identity hash could improve collision distribution/cost later;
it is not needed to establish correctness. TypeInfo and MemberInfo now have generated
reference coverage and an on-site guide; remaining introspection interfaces remain
explicit documentation gaps.

The [fixture](experiments/introspection-object/README.md) checks allocation versus
represented identity, interface/Object views, boxed GetType versus typeof, constructed
generics, arrays and a type-keyed map under GC pressure. It requires multiple collections
and full reclamation after completion. No generic nullable operand is introduced.

Further source investigation distinguishes these follow-ups:

- RuntimeAssemblyInfo carries the loaded full assembly identity; RuntimeModuleInfo
  carries that identity plus the module name. These are the next bounded equality/hash
  candidates within one loaded program. Future load contexts would require scope in
  their identity; neither short names nor tokens alone are sufficient.
- RuntimeFieldInfo, RuntimeMethodInfo and RuntimePropertyInfo carry a declaring type
  plus a definition index. Equality must include the descriptor kind and closed owner,
  and test inherited versus declared queries. Existing snapshots do not cache wrappers.
- RuntimeParameterInfo currently carries a parameter token, module, position and type,
  but no declaring member identity. Tokens can be absent; matching position/type is
  not enough. Establish that owner contract before changing parameter equality.

These findings are investigations, not newly implemented equality for those wrappers.
The immediate next bounded consumer is AssemblyInfo/ModuleInfo; member/parameter
identity follows after their ownership checks. Mutable resources and collections retain
identity semantics; this is not a move to universal structural equality.


### Assembly and module Object contracts — 2026-09-24

The next bounded slice implements the catalog keys identified above. Assembly equality
uses exact StoredIdentity; module equality uses (StoredIdentity, StoredName). The
metadata-origin validator rejects duplicate full assembly identities and duplicate
module names within one assembly. No fields or native layouts changed. Both wrappers
override Object equality/hash/display, reject null and other kinds, and retain separate
allocation identity. HashCode consumes those same strings; display returns assembly
FullName or module Name. Public interfaces gain no nullable typed equality requirement.

Comparison: .NET 10's [Assembly implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Reflection/Assembly.cs)
and [Module implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Reflection/Module.cs)
default Equals/GetHashCode to Object, with virtual dispatch for runtime implementations;
Assembly displays FullName and Module displays ScopeName. neoCLR reuses descriptive
display and the equality/hash contract, but its current catalog is not the CLR loader.
Full identity strings are unique only within the loaded program. Future independent
load contexts must participate in equality; comparing display names across contexts
would be incorrect. This is a provisional scope constraint, not a universal .NET
Assembly equality rule.

The benefit is stable descriptor-keyed lookup across fresh query wrappers without
adding caching or native identity machinery. Costs include string comparison and
encoding allocations in hashing; hashes are neither unique nor persistent. The
introspection fixture checks map reuse and GC, while a catalog regression distinguishes
same short-name assemblies and same-name modules, including two modules within one
assembly. Member/parameter identity remains the next ownership investigation.


### Declared member Object contracts — 2026-09-24

FieldInfo, MethodInfo and PropertyInfo now compare descriptor kind, closed declaring
type and definition index. Existing native snapshots already carry these components;
no layout, native factory or type-identity changes are needed. Hashing combines kind,
owner FullName and index; display returns Name. ReferenceEquals retains wrapper
identity, and Object.Equals rejects null/unrelated objects. ParameterInfo is unchanged:
its snapshot lacks declaring-member identity, so position/type/token equality would
be unsound. This scope deliberately leaves owner representation for the next slice.

.NET 10 primary sources, reviewed 2026-09-24:
[RuntimeFieldInfo](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/System.Private.CoreLib/src/System/Reflection/RuntimeFieldInfo.cs)
formats field type and name; [RuntimeMethodInfo](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/System.Private.CoreLib/src/System/Reflection/RuntimeMethodInfo.CoreCLR.cs)
compares its method handle, declaring type and reflected context and formats a full
method signature. neoCLR retains declared identity but does not model ReflectedType
or generic method instantiations. Queries currently enumerate declarations only;
inherited traversal is not implemented, even when DeclaredOnly is absent. Earlier
documentation implying inherited selection was corrected. This is a narrower contract,
not a claim of full System.Reflection equivalence.

Alternatives were retaining allocation equality, caching wrappers, or introducing a
new native member handle. Reusing existing owner/index data enables map lookups across
queries without a cache or storage migration. Costs are string hashing/encoding
allocations and possible collisions; owner-name hashes never replace type equality.
Simple Name display is readable but cannot distinguish overloads; full signature
formatting remains future work. These indexes and hashes are not persistent IDs.

The bridge's shared-member projection previously cleared virtual flags on all base
methods. It now normalizes only property getters, preserving the explicit Object
ToString override. The inherited base dispatch is checked by the member fixture.
Reference metadata and runtime library fragments are regenerated together; no Raven
compiler or target-policy changes are required. Public field/method/property APIs now
have generated documentation; parameter reference coverage remains explicit work.

Validation covers repeated queries, wrong kinds/null, different closed generic owners,
different definitions, property/accessor agreement, and map retention through GC.

### Parameter owner identity — 2026-09-24

Parameter snapshots now retain the closed declaring TypeInfo, owner kind and definition
index, alongside their existing position. Object equality compares that key and
position; it rejects other objects and null. Method parameters and property index
parameters are distinct, even when a property reuses an accessor's Param token. Two
properties sharing one accessor are distinct owners. Names/types/tokens do not establish
identity; source Param rows may be absent (token zero). Hashing uses owner FullName,
kind, index and position; Name display may be empty. Typed nullable operands are not
introduced. These hashes/indexes are not persistent keys.

Primary comparison, reviewed 2026-09-24: .NET 10
[ParameterInfo](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Reflection/ParameterInfo.cs)
exposes Member and Position, uses them for serialization-era parameter reconstruction,
and formats type plus name. It does not define owner-based Object equality in that
base class. neoCLR deliberately uses declaration identity across fresh snapshots;
its simple Name display and lack of a public Member/return-parameter API are narrower
than .NET. This is not a claim of full reflection parity.

Alternatives: retain allocation identity; retain entire member snapshots; or expose a
Member getter that scans all declarations. Allocation identity breaks repeat-query map
keys. Whole member retention risks cycles or recursive native snapshot construction.
Scanning adds unrelated allocations and can fault on unsupported sibling metadata.
The bounded choice is an internal key with no public Member property yet. Future public
owner resolution should target the exact declaration directly, preserve closed type
scope, and avoid recursively materializing member/parameter graphs. No new service or
runtime scheduling/handle model is introduced here.

The Raven native snapshot adds one TypeInfo and two integer fields; its importer layout
check changes in lockstep. The archived System.Type/value-descriptor profile keeps its
original fields. This costs additional per-parameter allocation/tracing; it is not a
performance improvement claim. Development runtime/library/SDK builds must be updated
together. ParameterInfo and BindingFlags now have generated member documentation.

Validation includes token-zero parameters, differing positions and method definitions,
closed generic owners, separate declaring types, property/accessor distinctions and
two properties sharing an accessor. The compiled Raven fixture checks Object dispatch,
null/wrong-type rejection, repeat-query map keys and retained owner data through GC:
894 allocated/reclaimed objects, peak 72, 25 collections, zero retained objects.
