# Primitive value interface contracts — 2026-10-20

The integer formatting consumer exposed an existing native gap: a boxed Int32 lost
its declared Formattable interface (interpreter passed, JIT/AOT raised InvalidCast).
The author explicitly requested checking the rest of the primitive family and
using normal value-type contracts, without inventing undeclared interfaces.

Native closed-world selection now retains reachable implementations for boxed scalar
interfaces, just as it retains nominal value implementations. Original verified
conformance is preserved across private scalar-storage projection. The existing
private box tags are mapped to those verified interface views and the ordinary
interface dispatcher calls the existing value method with a borrowed payload address.
No new public type, box representation, native ABI, runtime service or Raven compiler
encoding is introduced. Private box tags do not pretend to be new source classes with
fabricated methods. Source metadata remains authoritative.

The supported scalar receiver projection is shared by signed/unsigned integer widths,
native-width integers, Single, Double, Boolean and Char. The runtime neither
adds EquatableTo nor infers structural semantics from empty primitive declarations.
The subsequent [library equality slice](../primitive-equality/README.md) explicitly declares it.
Generic interfaces retain their exact type arguments. String already has a separate
reference-storage adapter and is unchanged. Floating Object display remains a separate
existing limitation; fixing interface conformance does not claim it is implemented.

This follows [.NET value-type boxing/interface behavior](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/types/boxing-and-unboxing): a box retains the value's
interfaces and invoking an interface member uses its value payload. A box still
allocates when reference identity is required; dispatch adds no replacement box or
payload copy. Native internal scalar layout remains neoCLR's existing layout, not a
claim of CLR binary ABI compatibility.

The Raven consumer exercises all fourteen scalar kinds through ComparableTo<T>,
checks direct/interface agreement, exact generic identity, the explicit Int32 equality
contract, the absence of equality for a different primitive type, unsigned high bits, signed extension,
and references retained across GC. The integer-format consumer separately exercises
Formattable and composite dispatch, with a retained pre-fix baseline. JSON reports
record interpreter/JIT/AOT outcomes and artifact hashes.

The native generic-argument admission list also omitted IntPtr/UIntPtr despite their
existing scalar storage and ComparableTo declarations. It now admits those exact
types without changing representation or synthesizing interfaces.

Closed generic specialization now preserves reached scalar interface relationships through
the same path as intrinsic String. Previously only String retained these relationships,
so scalar casts could succeed while generic interface dispatch had no implementation.

Qualification: all fourteen primitive comparison receivers, casts, exact generic identity
and retained boxes pass in interpreter, JIT and AOT on macOS ARM64. The integer
formatting consumer also passes all three engines; the existing borrowed-receiver
native regression and four API snapshot checks pass.

The original validation report predates explicit Byte equality and tested its then
undeclared interface. The consumer now checks a different type argument; subsequent
primitive-equality evidence covers the updated declared contracts.
