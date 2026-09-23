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

## Direction under review

The intended baseline is .NET-compatible reference/value semantics. The first Object display slice implements overridable class ToString with a
runtime-type-name fallback. Equality and hashing should next be designed together: default reference identity
for ordinary classes, value behavior for value types and consistent custom overrides.
Equality and hashing are not yet implemented on Object. String-to-Object
conversions currently allocate wrappers: repeated conversions from one String do
not preserve identity, unlike .NET. This is an unresolved representation gap, not
a promised reference-equality contract. See Microsoft's
[Object contract](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
for the comparison baseline; neoCLR does not yet provide that entire surface.

System.Value remains until its dependent carriers and runtime boundaries have a
verified replacement. Shallow cloning, finalization and broad implicit boxing are
not part of the next small Object slice.

Migration: application classes now retain Object as their metadata base. A class
that supplies ToString should declare an override; same-name hiding is rejected by
the current importer/runtime profile. Use matching reference/library artifacts.
