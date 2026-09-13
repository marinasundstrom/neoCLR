# Managed array shapes in the Raven target

The importer accepts vectors of existing defaultable primitive/calendar/error values
and ordinary reference shapes, including reflection classes, interfaces and nested
arrays. Array creation, length, typed loads/stores and element addresses share that
mapping. Array.ForEach uses the same element catalog and the existing Func<T,Void>
callback algorithm. Reference arrays preserve identities; value elements are copied.

[The array sample](experiments/raven-target/samples/library-array-shapes.rvn) exercises
long callbacks, Boolean stores and short-circuit reads, Type callbacks, boxed interface
elements, unsigned byte/UInt16/Char reads and jagged arrays. Signedness and element-token
mismatches are rejected by the signature checks. Raven's unsigned-load fix is needed
for high-bit Byte/UInt16/Char values; it also corrects ordinary .NET execution.

This follows CLI newarr, ldelem/stelem and ldelema behavior within the admitted type
catalog. Boolean storage remains the interpreter's canonical Boolean representation;
loads normalize to CLI integers before branch joins. Array covariance and rectangular
arrays remain outside this importer. Ordinary arrays still require valid defaults;
ArrayList uses its separate reserved-capacity mechanism for non-defaultable unions.
No arbitrary application types or general pointer element arrays are claimed here.
