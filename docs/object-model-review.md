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
