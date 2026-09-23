# Object and Value

**Development reference after Preview 9.** neoCLR distinguishes reference types from
value types. Assigning a class instance shares its reference; assigning a value
copies its fields. A reference field inside a copied value still refers to the same
object. Neither assignment performs a deep clone.

## Object's current API

[System.Object](xref:System.Object) supplies
[GetType()](xref:System.Object.GetType), returning System.Introspection.TypeInfo for
the concrete runtime type. Supported object, base and interface views retain that
type. Arrays, strings and boxed values have tested runtime paths too. Calling it on
null produces a terminal NullReference fault.

This reference intentionally does not advertise all .NET Object methods as implemented.
The compiler reference contains Equals, GetHashCode and ToString declarations, but
the executable Object library does not yet supply those methods. A normal implicit
base-constructor call is recognized by the importer; direct Object construction is
not a demonstrated application API. These are implementation gaps, not usable stub
implementations. Public reference coverage will expand with the implementations.

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

The intended baseline is .NET-compatible reference/value semantics. The proposed
first Object extension is an overridable ToString with a runtime-type-name fallback.
Equality and hashing should then be designed together: default reference identity
for ordinary classes, value behavior for value types and consistent custom overrides.
These additions are not implemented by this review. See Microsoft's
[Object contract](https://learn.microsoft.com/en-us/dotnet/api/system.object?view=net-10.0)
for the comparison baseline; neoCLR does not yet provide that entire surface.

System.Value remains until its dependent carriers and runtime boundaries have a
verified replacement. Shallow cloning, finalization and broad implicit boxing are
not part of the next small Object slice.
