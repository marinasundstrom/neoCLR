# Object and Value

neoCLR distinguishes reference types from
value types. Assigning a class instance shares its reference; assigning a value
copies its fields. A reference field inside a copied value still refers to the same
object. Neither assignment performs a deep clone.

## Object's current API

[System.Object](xref:System.Object) is abstract and supplies
[GetType()](xref:System.Object.GetType), returning System.Introspection.TypeInfo for
the concrete runtime type. Supported object, base and interface views retain that
type. Arrays, strings and boxed values have tested runtime paths too. Calling it on
null produces a terminal NullReference fault.

## Object display text

`ToString() -> string` is a virtual class method. Its default is the concrete
runtime type's FullName. A derived override is selected through an Object or base
reference; an explicit base call uses the type-name fallback. Rootless nominal
classes and arrays use the default Object slot. A null receiver faults.

The [display sample](/samples/object-display.zip) prints Plain, Named instance,
Named instance and Named. The last line comes from an explicit base call, showing
that bypassing the override still describes the concrete object.

Virtual ToString dispatch supports named structs with explicit overrides and boxed
Int32, Int64 and Boolean values. Integers produce culture-independent decimal text;
Boolean produces True or False, matching .NET spelling. Char returns its full grapheme text. String through Object returns its unchanged text. Other primitive boxes remain unsupported; use typed formatting where available. It does
not add Console.WriteLine(Object), serialization, culture/format overloads or string
identity. GetType's existing string/boxed paths remain supported.

This reference intentionally does not advertise all .NET Object methods as implemented.
Equals and GetHashCode have the bounded implementation described below. Object has a
protected parameterless constructor used when a derived instance is initialized.
It cannot be instantiated directly; both Raven and raw runtime construction reject
it. This differs from .NET's concrete Object. A future synchronization API should
provide a purpose-specific type rather than an empty Object as a lock token.

## System.Value is a different facility

[System.Value](xref:System.Value) stores a complete payload with its exact erased type.
It is temporary interpreter support used by Option, Result, TaskOutcome and native/host
boundaries. Applications should use typed values and those higher-level contracts.
It is not the .NET System.ValueType base class, nor an alternative spelling of Object.

The low-level operations are `value.pack T`, `value.is T` and `value.unpack T`.
Packing preserves the complete value; type tests compare the exact closed type;
unpacking returns a value copy and faults on mismatch. There is no implicit null or
default payload. A copied class-reference payload still aliases its object. Packing
allocates host storage in the interpreter; it is not an allocation-free optimization.

Explicit boxing instead creates GC-owned storage for a supported value and exposes
an Object/interface reference to it. Aliases can share that box. Replacing Value
with Object would therefore require a storage migration with copy, identity,
extraction and lifetime tests, rather than a type rename.

<a id="identity-equality-and-hashing-development"></a>

## Identity, equality and hashing

`Object.ReferenceEquals(left, right)` is nonvirtual. It compares the allocation
behind class, array and boxed-value references: aliases compare equal, distinct
allocations do not, two nulls compare equal and exactly one null compares unequal.
It does not compare fields, invoke overrides or compare managed byref locations.

For ordinary classes and arrays, virtual `Equals(Object)` defaults to this identity
comparison, and `GetHashCode()` returns an identity hash. Aliases share a hash that
survives mutation and collection. Hash collisions are allowed; never persist a
hash, treat it as a unique ID or expect a stable value between executions.

A class can override Equals and GetHashCode together. Equal objects must have equal
hashes, even when their allocations differ. The
<a href="/samples/object-equality.zip">checked sample</a> contrasts a mutable Cell using
identity with a Key using its Number for both equality and hashing. Calls through
Object select the overrides. ReferenceEquals remains an identity comparison, and
explicit base calls retain the base implementation. Existing EquatableTo contracts
are unchanged.

String identity follows the shared immutable text owner. Object/interface wrappers
may be separate allocations while still referring to the same String. ReferenceEquals
and explicit Object base Equals/GetHashCode use that owner; virtual String equality
and hashing use canonical Unicode 17 text identity. Separate constructions can be
canonically equal without sharing bytes or reference identity. Explicit String.Intern
shares exact stored spellings within one execution; literals are not automatically interned. Virtual
boxed-value Equals/GetHashCode support Int32, Int64, Boolean, Single, Double, Char and explicit named-struct overrides, described below.
ReferenceEquals can compare box identities, but does not supply boxed value equality.
Null instance receivers raise NullReference; default Equals accepts a null argument
and returns false. Static two-argument Object.Equals is not yet available.

## Direction under review

The intended baseline is .NET-compatible reference/value semantics. Class display,
reference identity and class equality/hash have bounded implementations. General
boxed-value dispatch still has coverage gaps. Raven record classes support integer,
string and nested components with generated equality, hashing, display and deconstruction. Generic/inherited records and
nullable string/value and arbitrary component types are not supported by this target contract. See Microsoft's
[Object contract](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
for the comparison baseline; neoCLR does not yet provide that entire surface.

System.Value remains the explicit erased-payload carrier described above. Shallow
cloning, finalization and broad implicit boxing remain outside the supported surface.

Application classes retain Object as their metadata base. A class
that supplies ToString should declare an override; same-name hiding is rejected by
the current importer/runtime profile. Use matching reference/library artifacts.

<a id="hashcode-and-records-development"></a>

## HashCode and records

[System.HashCode](xref:System.HashCode) is a mutable value accumulator. Add(int),
Add(string), ToHashCode() and Combine(int, int) are available. Copies retain independent
state. String hashing uses canonical Unicode 17 identity without allocating a
normalized string or a temporary byte snapshot. Original UTF-8 bytes are unchanged. Generic values and custom comparers are not
supported. Hashes can collide and must not be persisted; this implementation has no
randomized seed and does not reproduce .NET's hash values.

The <a href="/samples/records.zip">checked record sample</a> uses `record class
Key(Number: int)` and a two-component Pair. Separate instances compare equal by
components while ReferenceEquals distinguishes their allocations. Equals through
Object, generated operators and hashes agree. Display prints `Key { Number = 42 }`;
deconstruction retrieves the components. Person(Name: string, Age: int) adds string
content equality, and Entry(Owner: Person, Number: int) compares a nested record via
typed Equals and hashes its value. Nested display calls ToString; deconstruction
preserves the nested reference. Components may be Int32, non-null String or supported
record classes declared in the same compilation, including nullable record references.
Two null components compare equal; null and present values differ. Null contributes
zero to hashing, prints an empty component value and survives deconstruction. Nullable
string/value components and externally compiled record components remain unsupported; this is not general graph equality.

Raven's target configuration selects System.EquatableTo and System.HashCode. Unsupported
record shapes report RAVT004. Init-only property assignment remains a compiler rule;
the importer recognizes the IsExternalInit metadata marker and permits readonly
backing-field stores only in declaring constructors or recognized init accessors.
Application-property reflection and a runtime init-only field flag remain gaps.

<a id="boxed-primitive-equality-development"></a>

## Boxed primitive equality

Through an Object view, a boxed Int32 compares equal to another boxed Int32 with the
same value. Null, another integer type, strings and application classes compare false.
Its virtual GetHashCode returns the stored integer, including negative values and
Int32 limits. Boxing copies the value, so changing the original variable has no effect.
ReferenceEquals still distinguishes separately allocated boxes. Explicit Object base
calls retain allocation equality/hash rather than dispatching to the integer behavior.

A boxed Int64 compares its complete 64-bit copied value, only with another Int64.
Its hash XORs the lower and upper 32-bit halves, matching .NET 10. Different values
can share a hash; equality still compares every bit. Null, Int32 and Boolean operands
compare unequal. Separate boxes retain separate identities. Hashes are not persistent keys.

A boxed Boolean compares by its copied true/false value, only with another Boolean.
Null and boxed integers (including 0 and 1) compare unequal. GetHashCode returns 1
for true and 0 for false. Separate Boolean boxes retain distinct reference identities.

Boxed Single and Double use exact-type value equality: Single never equals Double,
even at the same magnitude. All NaNs of the same type compare equal; positive and
negative zero compare equal. This follows .NET Object equality. Floating `==` still
uses IEEE comparisons, so NaN is unequal to itself with that operator.

Hashing normalizes signed zero and NaN payloads before hashing the bit pattern;
Double XORs its upper/lower halves. NaN and positive infinity may share a hash but
remain unequal. These rules allow explicit Object comparer callbacks to find NaN
and signed-zero map keys reliably. Hashes are not persistent identifiers. Floating
boxed ToString and source-declared typed Equals/GetHashCode remain outside this slice.

[Char](xref:System.Char) compares canonical Unicode 17 grapheme identity, rejects other
concrete types and null, and hashes the same canonical sequence. Canonically equivalent
composed and decomposed spellings compare equal while preserving their bytes. Object ToString returns
all stored text, including combining marks and emoji joiners. Unlike .NET Char,
neoCLR Char is not a UTF-16 code unit and does not use .NET's code-unit hash.

String and Char expose matching typed and Object overrides in the development
library. Their native dispatch preserves the same source bodies. The remaining
primitive policies use bounded intrinsics for the selected Object slots. This does
not add typed GetHashCode members for other primitives, general struct equality, nullable
boxing, or general primitive formatting. Int32, Int64 and Boolean boxed ToString
use the bounded display contract above. Other primitive types still require
explicit implementation. Named structs dispatch their explicit overrides. System.Value is not involved in this dispatch.
The [Object equality sample](/samples/object-equality.zip) demonstrates the behavior.


<a id="structs-and-record-structs-development"></a>

## Structs and record structs

Ordinary non-generic application structs support fields, construction, instance
methods and value copying. Copying reference fields retains their references;
there is no deep clone. Assigning a struct to Object or a supported interface boxes
a copy. Two aliases of that box share its payload, while the original local remains
independent. Explicit Equals, GetHashCode and ToString overrides dispatch against
that payload. A struct without an override has no automatic field-based Object
implementation in this profile; this differs from .NET's ValueType fallback.

The checked sample includes:

```raven
record struct Coordinate(X: int, Y: int)

let first = Coordinate(42, 7)
let same = Coordinate(42, 7)
let value: Object = first
let comparable: EquatableTo<Coordinate> = first
```

Generated typed/Object/interface equality compares components, GetHashCode combines
their hashes, ToString produces `Coordinate { X = 42, Y = 7 }`, and deconstruction
returns X and Y. Object equality rejects null and other concrete types. Separate
boxes remain distinct for ReferenceEquals. `default(Coordinate)` initializes both
integers to zero. The sample also checks constructed non-null strings and nullable
record-class fields, including default absent references.

The configured contract admits the same component types for record structs as for
record classes: Int32, non-null String, supported same-compilation record structs and record-class
references. Nullable values/strings, generic records and
external record components remain outside this first implementation. Default values retain null in reference fields, even when the property is declared
non-nullable. Generated record equality, hashing and display guard these values;
this does not make ordinary calls on a null reference safe.

At the instruction layer, value `isinst` preserves a matching box or returns null;
`unbox.any` copies an exact value payload. Null unboxing faults with NullReference,
and a different concrete type faults with InvalidCast. Reference-type unbox.any,
nullable boxing and address-returning unbox remain unsupported.


### Nested record structs

The record sample also includes a separate `Nested.rvnproj`:

```raven
record class Drawing(Bounds: Rectangle)
record struct Rectangle(Start: Point, End: Point)
record struct Point(var X: int, var Y: int)
```

Rectangle compares Point components with typed equality and combines their hashes.
Display includes each Point's display text. Construction and deconstruction copy
Point values: changing the original point or a deconstructed copy does not modify
the rectangle. Drawing demonstrates a record class containing a record struct.
Default Rectangle initializes its nested integer fields to zero. Nullable struct
components and externally compiled component records remain unsupported.


### Default reference fields

`Defaults.rvnproj` demonstrates `default(Payload)` where Payload contains a string
and a record-class reference, both declared non-nullable. Default initialization
bypasses construction and leaves those fields null. Equality treats two absent
components as equal and distinguishes an absent string from an empty string.
A null component contributes zero to HashCode; display prints an empty component
value. Deconstruction preserves null rather than substituting an empty string or
creating a record instance.

This follows the .NET distinction between nullable annotations and runtime default
initialization. HashCode.Add(string) still requires a non-null argument; generated
record code checks first and uses Add(int) for the null contribution. Explicit
nullable-string record components and nullable-value boxing remain unsupported.
Object.Equals and ReferenceEquals now advertise their nullable reference arguments.


<a id="nullable-object-arguments-development"></a>

## Nullable Object arguments

`Equals(Object? other)` accepts null as the value to compare. The receiver must still
be an instance. `ReferenceEquals(Object? left, Object? right)` accepts null on either
side: two nulls compare true; one null and one instance compare false. The checked
Object sample exercises literal nulls, nullable locals, class overrides and boxed
integers. Non-nullable Object declarations still reject null in Raven.

These reference annotations use the existing CLI nullable metadata consumed by
Raven. They support current type checking and compatible API use; they do not commit
neoCLR to a final nullability model or metadata representation. Runtime signatures,
reference storage, virtual slots and Fault behavior are unchanged. Use matching
reference artifacts to make the corrected annotations visible to the compiler.
Generated record-specific Equals signatures are a separate compiler follow-up.


## Modeling absence

neoCLR favors `Option<T>` when an API or domain model needs to express absence, for
both value types and reference types. Nullable reference annotations remain useful
for Raven compatibility and existing null-based contracts such as Object equality.
They do not make nullable values the platform's preferred absence model.

Nullable structs (`Nullable<T>`) and nullable-value boxing are deliberately deferred;
they may be reconsidered later. Existing reference nulls and default initialization
remain unchanged. This direction does not yet add Option-valued components to the
configured record contract, and the final metadata representation remains open.

### Development compiler policy for absence

The neoCLR target disables nullable value declarations through Raven's
RavenAllowNullableValueTypes option. A known value type declared with `?`, such as
`int?` or a record struct, produces RAV0407: "Value types can't be declared as
nullable". Reference annotations such as `Object?` remain valid. Prefer `Option<T>`
for intentional absence in both value and reference API models.

Generated record Object.Equals preserves Object's nullable comparison parameter;
generated typed record-class Equals also accepts a nullable reference to its record
type and compares null as false. Record-struct typed parameters remain non-nullable
values. The EquatableTo&lt;T&gt; interface contract is unchanged. This does not add Nullable&lt;T&gt;,
nullable-value boxing or Option components to the supported record component set.

Generated record-class `==` and `!=` also accept nullable operands in this preview.
Two absent references compare equal; an absent and present reference compare unequal;
present records compare by components. Record-struct operator parameters remain
values. Explicitly authored operators retain their own contracts.


<a id="mixed-object-map-keys-development"></a>

## Mixed Object map keys

Supported generic API signatures admit Object, including `HashMap<Object, Object>`.
Supply explicit callbacks: `(left, right) => left.Equals(right)` and
`key => key.GetHashCode()`. Paths and type descriptors use their represented-value
contracts; supported boxed Int32/Int64/Boolean/Single/Double/Char values use exact-type value equality, while
ordinary classes retain allocation identity. Equal keys must have equal hashes.

The compiled sample checks duplicate keys, replacement, deliberate collisions,
growth and reference-preserving Object values returned through `Option<Object>` under
GC. The String slice also supports content keys through the existing Object conversion.
Default comparers, nullable-key policy and other boxed primitive keys remain
outside this supported surface.


<a id="string-through-object-development"></a>

## String through Object

[String](xref:System.String) supports Object equality, hashing and display through
the existing intrinsic-text wrappers. Equality compares exact contents with another
String; null and other concrete types compare false. The UTF-8 content hash agrees
for equal text. ToString returns the full text unchanged. Empty text, embedded NUL,
combining sequences and emoji are preserved. No normalization or collation is implied.

String remains a reference type. Conversion currently creates a GC-owned wrapper
around shared immutable UTF-8 text; this is not value-type boxing. ReferenceEquals
and explicit Object base equality/hashing follow that text owner, preserving identity
through casts and independent wrappers. Hashes remain stable across collection and
host retention, but can collide and must not be persisted. Virtual GetHashCode uses
the content hash instead.
No automatic literal interning, nullable String storage, stable hash numbers or source-declared String
Object overrides are added. The reference-only scaffold constructor is excluded:
create strings with literals, `String(Sequence<char>)`, Concat and text APIs.
String implements `Sequence<char>`; its Count is explicit and Length remains public.

The content contract follows .NET String Object equality and display. neoCLR uses
UTF-8 rather than .NET UTF-16 and does not adopt .NET's randomized hash algorithm;
its current FNV-1a hash is provisional, non-cryptographic and not collision-attack
resistant. Hashes are not persistent identifiers. Default comparers remain future work.


### Rust host String payloads

`Value::String` holds the immutable `StringValue` host type. Construct it with
`Value::String(owned_text.into())` or `Value::String("text".into())`. Cloning
retains the text owner. `as_str()` borrows text; `into_owned()` returns a Rust
String, copying only when other owners remain. A retained host Value keeps its text
alive after heap collection. This host representation is not a new guest type and
does not enable String ReferenceEquals. Array budgets still count text bytes per
logical occurrence; GC object counts exclude these text allocations.

### Rust host memory-view payloads

Development-only `Value::MemoryView(MemoryView)` is an opaque interpreter payload.
`MemoryView` exposes no public constructor, fields or memory-access methods; the
runtime creates it internally. It implements `Clone`, `Debug` and `PartialEq`.
Rust hosts with exhaustive matches on `Value` must handle the added variant.
Cloning preserves the owner/window and access capability. Host equality compares
view kind, owner, bounds and readonly capability, not element contents; this is
not a guest equality operator. `Value::ty()` reports the intended closed generic
Slice/ReadOnlySlice/Memory/ReadOnlyMemory type, but does not register a guest type.

Borrowed views reject heap/field/array storage and erasure, including empty views
and views over heap-owned arrays. Persistent views can be stored; normal VM tracing
follows their owner. Holding a Rust Value does not itself root its managed owner.
Host publication validates the execution heap, and expired owners fault. Immutable
UTF-8 views share the String owner and are borrowed capabilities for storage checks.

Nominal metadata admission and interpreter suspension rules are implemented.
Public Raven APIs are implemented; native execution currently covers a bounded array-backed byte-view subset. This payload is not a
supported host factory; checked guest factories are documented in the
[memory-view reference](memory-views.md). See
the [implementation status](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-memory-views.md).


With the `native-metadata` feature, the Rust compiler bridge exposes `retain_memory_view_shape`
(target TypeDef, source Module, source index, lowered type arguments) and
`memory_view_element(&TypeDef) -> Option<&Type>` for private specialization IR.
Retention validates the source opaque declaration, matching view kind and unary
arity; errors return Fault. Ordinary types are unchanged. Callers supply verified
source IR and lowered arguments. The retained element is remapped by
`restore_interface_proofs` alongside other private type facts. These facts are
omitted on serialization and rejected if supplied as metadata fields. They grant
no native ABI or execution rights on their own. Selection and value profiling now
admit exact array-backed byte-view shapes with a dedicated owner/offset/length layout;
other view shapes remain rejected.


### Private native byte-window helpers

`runtime/native/memory/memory-views.h` declares the provisional `neoclr_byte_view`
carrier (`owner`, `offset`, `length`, `kind`) and these internal C entry points:

- `neoclr_byte_view_create_v1(context, array, start, count, kind, output)`.
- `neoclr_byte_view_create_all_v1(context, array, kind, output)`: validates the owner and borrows its entire length.
- `neoclr_byte_view_slice_v1(context, view, start, count, output)`.
- `neoclr_byte_view_convert_v1(context, view, kind, output)`.
- `neoclr_byte_view_get_v1(context, view, index, outputByte)`.
- `neoclr_byte_view_set_v1(context, view, index, byte)`.
- `neoclr_byte_view_copy_v1(context, source, destination)`.

They return Int32 native fault codes: zero on success, IndexOutOfRange (8) for
invalid windows/indexes, RuntimeError (3) for invalid storage/capabilities or
uninitialized reads. Outputs and data remain unchanged on failure. Kinds are
Slice, ReadOnlySlice, Memory and ReadOnlyMemory; conversion only restricts access
or borrows a persistent view. Start/count/index use signed 32-bit inputs.
The owner is a mutable byte-array allocation base from the live supplied context.
These are trusted compiler/runtime helpers, not a stable host ABI or public guest
factory. They do not root values, allocate, collect or authorize borrowed escapes.
Callers publish the owner across GC boundaries; empty nonnull windows also retain
it. Arena mode trusts allocation-base provenance; GC mode validates live ownership.
Copy transfers the entire source into the destination prefix with overlap-safe
snapshot semantics and no temporary allocation. It preserves the destination
suffix and both descriptors. Both views must be valid, the destination writable
and long enough, and every source byte initialized before any write occurs.
Only copied destination bytes become initialized. Short destinations return 8;
readonly destinations and uninitialized sources return 3. Empty copies still
validate both descriptors and destination capability. This is a private byte
kernel, not a public Raven CopyTo member or a generic reference-element copier.
The [native implementation status](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-memory-views.md#native-byte-window-kernel-2026-10-19)
records layout costs, tests and the bounded generated-code binding for persistent byte views.


Development String also exposes `Trim()`, `TrimStart()` and `TrimEnd()`. They remove
Unicode 17 White_Space scalars at the requested edges, preserving all retained
UTF-8 bytes. A combining mark is retained even when its preceding leading space is
removed; trimming is scalar classification, while indexing remains grapheme-based.
Unchanged results reuse the original owner. Changed nonempty results copy only the
retained text through the checked runtime service, without temporary byte or Char
arrays. These methods must not replace protocol-specific whitespace validation.
