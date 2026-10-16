# Casing and decimal reporting (development)

The native **text model is Unicode text**; **UTF-8 is the native storage representation**.
String exposes grapheme and scalar views. Char represents one grapheme cluster, potentially
containing several scalar values. Scalar APIs currently use uint and UnicodeScalar helpers;
a dedicated rune type is not implemented. Byte represents one encoded byte.

Use matching runtime, System library and compiler references.

[String](xref:System.String) provides `ToUpperInvariant()` and `ToLowerInvariant()`.
Both return String using **Unicode 17 full default casing**. They can expand text
and change byte, scalar and grapheme counts. Uppercasing `Straße ﬃ` yields `STRASSE FFI`;
lowercasing `ΟΣ` yields `ος`, and `İ` yields `i` plus a combining dot. Context uses
the original text. There is no normalization, locale selection or titlecasing.

The familiar .NET names do not promise identical results: full expansions and final
sigma differ from the tested .NET 10 invariant methods. Casing is also separate from
[StringComparer.OrdinalIgnoreCase](xref:System.StringComparer), which retains its
simple-fold equality/hash policy. Existing String references remain unchanged.
Grapheme segmentation still uses its documented Unicode 16 data.

[Int64](xref:System.Int64) provides:

| Member | Contract |
| --- | --- |
| `Parse(value: string)` | `Result<long, NumberParseError>`; optional ASCII sign and one or more ASCII digits. Leading zeros and negative zero are allowed. |
| `ToString()` | ASCII decimal, negative sign only when needed, no grouping or leading zeros; round-trips every Int64 value. |
| `MinValue` | Static read-only property: -9223372036854775808. |
| `MaxValue` | Static read-only property: 9223372036854775807. |

The bounds are properties, not compile-time constant fields. No format strings,
styles or culture providers are supported. Unlike default .NET parsing, whitespace
is rejected. Separators, prefixes, NUL and non-ASCII digits are also invalid.

[NumberParseError](xref:System.NumberParseError) is a standard Raven union:
`InvalidFormat` means the whole input does not match the grammar; `Overflow` means
valid decimal text lies outside the range. Format validation comes first, so an
oversized number followed by `x` is InvalidFormat. Match cases through Raven patterns.
An uninitialized error union has no active value; Parse returns active errors only.

The executable report consumer is maintained in
`docs/experiments/casing-integer/Main.rvn` in the repository.


## Number and concrete Parse (development)

[Number](xref:System.Number) supplies static `Zero`, `One`, the binary
`+`, `-`, `*`, `/` operators, and inherited `CompareTo`. Implementers are SByte,
Byte, Int16, UInt16, Int32, UInt32, Int64, UInt64, Single and Double. Boolean and
Char are not numbers. There is no Parsable interface; numeric-specific parsing
remains a future design question.

Integer addition/subtraction/multiplication wrap at the destination width. Integer
division truncates toward zero; zero division and signed Int32/Int64 minimum divided
by -1 raise terminal Fault. Narrow signed division follows promoted integer
arithmetic and wraps when narrowed. Floating operations retain IEEE behavior;
CompareTo orders NaN below finite numbers and compares NaNs equal, while operators
retain their IEEE rules. Zero/One are properties, not compile-time constants.

The current importer supports closed static application functions constrained solely
by `Number`, specialized for these ten concrete types. It does not yet admit
arbitrary user-defined numeric types, generic classes, additional constraints or
open runtime generic dispatch. General static/default/access-controlled interface
support is further platform work, not a restriction inherent in the API design.

Every listed numeric type has concrete Parse; Boolean has Parse only. All numeric parsers use
[NumberParseError](xref:System.NumberParseError), with InvalidFormat/Overflow.
[BooleanParseError](xref:System.BooleanParseError) has InvalidFormat only.

Integers accept an optional ASCII sign and one or more ASCII digits, including
leading zeros. Unsigned `-0` succeeds; negative nonzero values overflow. Floating
parsers accept an optional sign, decimal mantissa (`1`, `1.`, `.5`, `1.5`) and optional
`e`/`E` exponent with optional sign and required digits. Exact `NaN`, `Infinity`,
`+Infinity`, `-Infinity` are accepted; numeric overflow is an error and underflow may
round to signed zero. Single rounds to its own precision. Boolean accepts ASCII
case-insensitive `true`/`false`. All parsers reject whitespace, grouping and locale
formats; numeric grammar validation precedes range errors.

These rules deliberately differ from default .NET parsing: they have typed Result
errors, strict whole-text grammar and numeric floating overflow errors. No culture,
style options, implicit byte decoding or general formatting interface is introduced.


Development Number is nongeneric. Its displayed compiler-reference signatures use
[the Self transport marker](xref:System.Runtime.CompilerServices.Self); Raven source
writes `Self` for the concrete implementing type. This marker has no constructible
runtime value. Change old `where T: Number<T>` constraints to `where T: Number` and
rebuild against matching references. The numeric importer retains native Self
dispatch while specializing its currently supported closed helper functions.


The separate native metadata/compiler integration branch now emits the unchanged
Number interface and imports it into a separately compiled ordinary struct and
consumer. Direct operators, identities and ordering execute. This is a contract
integration checkpoint, not completion of source-built numeric primitives or generic
Number emission on that newer path; the older bridge's closed specialization evidence
above remains distinct.


## Ordinal replacement (development)

[String.Replace(oldValue, newValue)](xref:System.String) returns immutable text after
replacing exact, non-overlapping matches. Empty replacement deletes; unchanged text
retains its reference. Both arguments must be non-null and the search must be
non-empty; empty search faults with RuntimeError. Matching uses UTF-8 without culture
or normalization. Unlike .NET's nullable replacement overload, use empty text for
deletion. The shared service has interpreter and experimental ARM64 coverage; full
native core bootstrap remains work in progress.


## Minimal arithmetic capabilities (development)

System.Numerics provides four public nominal interfaces in System.Runtime:
AdditionOperators (`+`), SubtractionOperators (`-`), MultiplyOperators (`*`) and
DivisionOperators (`/`). Each has one static operator with signature
`(left: Self, right: Self) -> Self`. Self means the implementing type; both operands
and the result have that type. Implementations must explicitly declare conformance
and supply the exact operator. Supporting an operator alone does not establish
interface membership. These contracts do not provide ordering, parsing or conversions.

Number inherits all four and retains ComparableTo<Self>, Zero, One and its existing
operator declarations. The eight fixed-width integer types, Single and Double gain
these capabilities through Number; their arithmetic behavior is unchanged. Integer
addition/subtraction/multiplication wrap and invalid integer division faults. Floating
operations keep their existing semantics. Custom types choose and document their
operator domain/failure policy. No interface here promises recoverable Result errors.

For example, the validated native consumer uses:

```raven
static func Add<T>(left: T, right: T) -> T where T: AdditionOperators => left + right
```

The [consumer](../docs/validation/numerics-interfaces/LibraryConsumer.rvn) also forwards
Number-constrained parameters to Add and implements the capabilities on a Complex
demo without imposing ordering. Native interpreter/JIT/AOT support closed nongeneric
interface bounds and static Self dispatch, including imported top-level functions and static class methods.
Mixed operand/result types, checked default interface operators, instance Self
specialization and production Complex semantics are outside this initial surface.
Rebuild consumers against the matching library/compiler bundle. Ordinary Raven/.NET
continues using the platform's System.Numerics APIs; these interfaces are NeoCLR APIs.
