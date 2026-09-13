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
loads normalize to CLI integers before branch joins. Mutable arrays are deliberately
[invariant](array-variance.md); rectangular arrays remain outside this importer. Ordinary arrays still require valid defaults;
ArrayList uses its separate reserved-capacity mechanism for non-defaultable unions.
No arbitrary application types or general pointer element arrays are claimed here.

## Direct array iteration after Preview 5

Raven lowers a `for` loop over a vector to indexed loads and an ordinary CLI `bge`
exit branch. The bridge now admits `bge` and its short form for matching Int32,
Int64 or Double operands, preserving the runtime's existing comparison semantics
and control-flow validation. Preview 5 rejected this instruction even though direct
indexing worked. No runtime opcode or Raven compiler change is needed.

The [array iteration sample](experiments/raven-target/samples/library-array-foreach.rvn)
covers reflected method arrays and empty, singleton and multiple-element integer
vectors. This is development support for the next release; published Preview 5
and existing installed bundles remain unchanged.
