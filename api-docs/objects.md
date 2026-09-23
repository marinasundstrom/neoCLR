# Object and Value

**Development reference after Preview 9.** neoCLR distinguishes reference types from
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

`ToString() -> string` is now a virtual class method. Its default is the concrete
runtime type's FullName. A derived override is selected through an Object or base
reference; an explicit base call uses the type-name fallback. Rootless nominal
classes and arrays use the default Object slot. A null receiver faults.

The [display sample](/samples/object-display.zip) prints Plain, Named instance,
Named instance and Named. The last line comes from an explicit base call, showing
that bypassing the override still describes the concrete object.

This first slice does not implement Object virtual dispatch for boxed values or
intrinsic strings. Those calls fail; use typed formatting where available. It does
not add Console.WriteLine(Object), serialization, culture/format overloads or string
identity. GetType's existing string/boxed paths remain supported.

This reference intentionally does not advertise all .NET Object methods as implemented.
The compiler reference contains Equals and GetHashCode declarations, but
the executable Object library does not yet supply those methods. Object has a protected parameterless constructor used when a derived instance is
initialized. It cannot be instantiated directly; both Raven and raw runtime
construction reject it. This deliberately differs from .NET's concrete Object.
A future synchronization API should provide a purpose-specific type rather than
requiring an otherwise empty Object as a lock token. Equality/hash remain gaps,
not usable stub implementations. Public reference coverage will expand with the implementations.

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

## Identity, equality and hashing (development)

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
explicit base calls retain the base implementation. Existing Equatable contracts
are unchanged.

String identity is deliberately unsupported: the current String/Object conversion
creates wrappers instead of preserving an underlying String allocation. Identity
calls on String payloads or their wrappers raise a terminal RuntimeError. Virtual
boxed-value Equals/GetHashCode also remain unsupported; use typed equality APIs.
ReferenceEquals can compare box identities, but does not supply boxed value equality.
Null instance receivers raise NullReference; default Equals accepts a null argument
and returns false. Static two-argument Object.Equals is not yet available.

## Direction under review

The intended baseline is .NET-compatible reference/value semantics. Class display,
reference identity and class equality/hash now have bounded implementations. String
identity and boxed-value dispatch remain representation gaps. Raven record syntax
now passes an end-to-end record-class sample with integer, string and nested components with generated equality,
hashing, display and deconstruction. Record structs, generic/inherited records and
nullable string/value and arbitrary component types are not supported by this target contract. See Microsoft's
[Object contract](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
for the comparison baseline; neoCLR does not yet provide that entire surface.

System.Value remains until its dependent carriers and runtime boundaries have a
verified replacement. Shallow cloning, finalization and broad implicit boxing are
not part of the next small Object slice.

Migration: application classes now retain Object as their metadata base. A class
that supplies ToString should declare an override; same-name hiding is rejected by
the current importer/runtime profile. Use matching reference/library artifacts.

## HashCode and records (development)

[System.HashCode](xref:System.HashCode) is a mutable value accumulator. Add(int),
Add(string), ToHashCode() and Combine(int, int) are available. Copies retain independent
state. String hashing uses non-null UTF-8 contents without Unicode normalization and
allocates a temporary byte snapshot. Generic values and custom comparers are not
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

Raven's target configuration selects System.Equatable and System.HashCode. Unsupported
record shapes report RAVT004. Init-only property assignment remains a compiler rule;
the importer recognizes the IsExternalInit metadata marker and permits readonly
backing-field stores only in declaring constructors or recognized init accessors.
Application-property reflection and a runtime init-only field flag remain gaps.
