# Explicit primitive equality — development, 2026-10-20

The author requested EquatableTo where applicable to primitive types. All fourteen
scalar types now explicitly declare EquatableTo<T>: signed/unsigned/native-width
integers, Single, Double, Boolean and Char. Int32 already declared it; Char already
provided typed canonical equality. The other types gain a public Equals(T) method.

This follows .NET's IEquatable<T> primitive contracts and typed Equals semantics.
[.NET Double.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.double.equals)
treats NaNs and signed zeros as equal; IEEE equality operators remain unchanged.
neoCLR's deliberate Char divergence remains canonical grapheme equality rather
than UTF-16 code-unit comparison. Equality matches the existing Object adapters
and hash normalization. No new operator, automatic interface synthesis, box layout,
ABI or compiler contract is introduced.

Direct typed calls borrow intrinsic value storage without a new box. Converting a
value to an interface or Object still boxes, as for other value types. Integer and
Boolean equality compares the stored value; native-width integers use the same
exact-width conversion services as CompareTo (the current native metadata bridge
does not admit direct native-width equality operands). Float equality handles NaNs
explicitly without converting to text or allocating. Char reuses its existing
runtime canonical comparison; original UTF-8 spelling is preserved.

The executable consumer covers all fourteen typed/interface contracts, unequal
values, narrow and unsigned boundaries, exact generic identity, NaNs, signed zero,
Object/hash agreement, canonical graphemes and references surviving GC. The JSON
report records interpreter/JIT/AOT results and matching artifact hashes. Reference
metadata, XML summaries and native API pages are refreshed with the source library.

Broader primitive Formattable coverage is separate: Int32/Int64 currently implement
the bounded D/F/N contract; Boolean and Char do not acquire numeric formats.

Qualification completed on macOS ARM64: interpreter, JIT and AOT all pass.
The four native API snapshot checks and the 261-type source inventory check pass.
Website validation builds and checks 5008 pages; publication is separate.
