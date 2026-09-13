# Mutable array invariance

neoCLR's mutable managed arrays are invariant. `arrayref<Foo>` is not assignable or
castable to `arrayref<Base>`, even when Foo derives from Base. This also excludes
`arrayref<Int32>` to `arrayref<Object>` and applies recursively to jagged arrays.
The reverse conversion between different array types is disallowed as well.
An exact array cast preserves identity; an exact cast back from an interface view
remains valid when the underlying allocation has that array type.

This is an intentional difference from [CLR/C# array covariance](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/arrays#176-array-covariance).
C# permits covariance between reference-element arrays, with runtime checks when
storing through the wider view. It does not permit int[] to object[] through array
covariance. neoCLR also rejects the reference-element case: a mutable Base[] view
would expose a setter that accepts values a Foo[] allocation cannot hold.

The benefit is an exact element contract for mutable array access, including element
addresses. The cost is source/IL incompatibility with code relying on CLR reference
array covariance. Such code must use an explicitly copied array of the desired
element type, or eventually a supported read-only projection. Existing element,
bounds, null, lifetime and storage checks remain; invariance is not a claim that
all runtime write checks can be removed.

## Enforcement and language projection

The verifier and interpreter reject casts between statically different array types,
including typed nulls. The interpreter additionally checks the concrete allocation
when casting an interface view back to an array. Ordinary assignment, parameter and
return validation continue to require the same array type. The Raven importer rejects
both implicit and explicit array widening with an element-type diagnostic.

Raven's ordinary CLR compiler semantics remain unchanged. Consequently, its language
service can still accept CLR-covariant source that the neoCLR target importer rejects.
Target-aware editor diagnostics are a follow-up, not part of this runtime contract.
No new opcode, array type flag or read-only array syntax is introduced here.

## Read-only projections: direction, not implemented variance

The author's exception is a genuinely read-only array view or interface. If a view
only produces T values and cannot replace elements or expose writable element
references, covariance can be sound: read a Foo as a Base. Contravariance describes
an input-only contract and is not the relationship wanted for array reads.

A readonly binding, a readonly reference to an array slot, or a getter-only property
returning an ordinary mutable array does not establish a read-only element contract.
A read-only view also need not mean immutable storage: another mutable alias may
change elements, while the read-only view remains unable to write them. Mutability
of objects returned as elements is a separate concern.

Before implementing this projection, decide its API and generic variance metadata,
validate every member's use of the element type, and define runtime assignability,
dispatch, array back-casts and writable-byref restrictions. Iterable<T>/Iterator<T>
currently remain invariant; their names alone do not enable variance. This leaves a
path to read-only covariance without restoring mutable-array covariance.

## Validation

On 2026-09-13, 72 focused runtime/verifier tests, 57 Raven saved-project checks and
28 query checks passed; clippy was clean. Coverage includes reference inheritance,
value-element arrays, both conversion directions, jagged arrays, typed nulls,
interface round-trips and interface-mediated attempts to change the element type.
The Raven project tests reject both implicit and explicit MethodInfo[] to MemberInfo[]
conversions before producing an executable.
