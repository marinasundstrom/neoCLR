# Casing and decimal reporting

<a id="casing-and-decimal-reporting-development"></a>

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


<a id="number-and-concrete-parse-development"></a>

## Number and concrete Parse

[Number](xref:System.Number) supplies static `Zero`, `One`, the binary
`+`, `-`, `*`, `/` operators, and inherited `CompareTo`. Implementers are SByte,
Byte, Int16, UInt16, Int32, UInt32, Int64, UInt64, Single and Double, plus the
development native Decimal value type described below. Boolean and
Char are not numbers. There is no Parsable interface; numeric-specific parsing
remains a future design question.

Integer addition/subtraction/multiplication wrap at the destination width. Integer
division truncates toward zero; zero division and signed Int32/Int64 minimum divided
by -1 raise terminal Fault. Narrow signed division follows promoted integer
arithmetic and wraps when narrowed. Floating operations retain IEEE behavior;
CompareTo orders NaN below finite numbers and compares NaNs equal, while operators
retain their IEEE rules. Zero/One are properties, not compile-time constants.

The legacy CLI importer supports closed static application functions constrained solely
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


<a id="ordinal-replacement-development"></a>

## Ordinal replacement

[String.Replace(oldValue, newValue)](xref:System.String) returns immutable text after
replacing exact, non-overlapping matches. Empty replacement deletes; unchanged text
retains its reference. Both arguments must be non-null and the search must be
non-empty; empty search faults with RuntimeError. Matching uses UTF-8 without culture
or normalization. Unlike .NET's nullable replacement overload, use empty text for
deletion. The shared service has interpreter and experimental ARM64 coverage; full
native core bootstrap remains work in progress.


<a id="minimal-arithmetic-capabilities-development"></a>

## Minimal arithmetic capabilities

System.Numerics provides four public nominal interfaces in System.Runtime:
AdditionOperators (`+`), SubtractionOperators (`-`), MultiplyOperators (`*`) and
DivisionOperators (`/`). Each has one static operator with signature
`(left: Self, right: Self) -> Self`. Self means the implementing type; both operands
and the result have that type. Implementations must explicitly declare conformance
and supply the exact operator. Supporting an operator alone does not establish
interface membership. These contracts do not provide ordering, parsing or conversions.

Number inherits all four and retains `ComparableTo<Self>`, Zero, One and its existing
operator declarations. The eight fixed-width integer types, Single and Double gain
these capabilities through Number; their arithmetic behavior is unchanged. Integer
addition/subtraction/multiplication wrap and invalid integer division faults. Floating
operations keep their existing semantics. Custom types choose and document their
operator domain/failure policy. No interface here promises recoverable Result errors.

For example, the validated native consumer uses:

```raven
static func Add<T>(left: T, right: T) -> T where T: AdditionOperators => left + right
```

The [consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/numerics-interfaces/LibraryConsumer.rvn) also forwards
Number-constrained parameters to Add and implements the capabilities on a Complex
demo without imposing ordering. Native interpreter/JIT/AOT support closed nongeneric
interface bounds and static Self dispatch, including imported top-level functions and static class methods.
Mixed operand/result types, checked default interface operators, instance Self
specialization and production Complex semantics are outside this initial surface.
Rebuild consumers against the matching library/compiler bundle. Ordinary Raven/.NET
continues using the platform's System.Numerics APIs; these interfaces are NeoCLR APIs.

## Floating special values — development

Single and Double each expose constant fields `NaN`, `PositiveInfinity`, and
`NegativeInfinity`, following .NET. They require no initialization or storage.
Use `IsNaN(value)` to recognize NaNs, not `value == NaN`: floating equality
operators still treat NaNs as unequal, while typed Equals treats NaNs as equal.
`IsInfinity`, `IsPositiveInfinity`, `IsNegativeInfinity`, and `IsFinite` classify
the same primitive value without allocation. Finite includes both signed zeros.
NaN sign/payload is not part of the public constant's contract; classification
accepts every NaN representation. Formatting and parsing keep the existing invariant
NaN/Infinity spellings. These are current development APIs, not published-release claims.

## Decimal (development)

`System.Decimal` is a Raven value type with a 96-bit unsigned coefficient, a sign,
and a stored scale from 0 to 28. It implements the existing `Number`,
`EquatableTo<Decimal>` and `Formattable` contracts. It has no NaN or infinity.
There are no additional numeric capability interfaces in this increment.

Like .NET Decimal, arithmetic uses base-ten values and rounds discarded digits to
nearest, ties to even. Equivalent scales and signed zeros compare equal and have
matching hashes. General formatting preserves the stored scale: `1.00` remains
`1.00`. This is a logical representation contract, not a CLR binary-layout guarantee.

The native reference documents constructors from integers and coefficient words,
`Zero`, `One`, `MinusOne`, `MinValue`, `MaxValue`, `Scale`, arithmetic and comparison
operators, `CompareTo`, `Equals`, `GetHashCode`, `IsZero`, `IsNegative`, `Abs`,
`Negate`, `Round`, `Floor`, `Ceiling`, `Truncate`, and `GetBits`.
`GetBits(value)` allocates four independent words in low/middle/high/flags order;
`GetBits(value, destination: Slice<int>)` writes those words into caller storage.

`Parse(text)` returns `Result<Decimal, NumberParseError>` and accepts an optional
ASCII sign and decimal point. It rejects whitespace, exponents and grouping.
The explicit-culture overload uses the existing strict numeric grouping policy.
`TryParse` accepts nullable input and resets its out value to zero on failure.
Excess fractional precision is rounded; an unrepresentable magnitude is `Overflow`.

`ToString()` is invariant. Format overloads use current or explicit culture and
support G/G0 and F/N/C with precision 0–99. C is a lower-level convenience that uses the culture’s default currency. Custom patterns, exponential formatting,
float conversions and integral extraction conversions are outside this foundation.
Invalid format/scale, arithmetic overflow and division by zero use neoCLR's current
terminal `System.Fail` contract, rather than recoverable CLR arithmetic exceptions.

Arithmetic scratch storage is an ordinary value type; it does not create byte or
limb arrays. Formatting allocates output storage and a String. Interface/Object
conversion may box; explicit-culture parsing may allocate normalized text. No
claim of zero allocation for the complete API is made.

The caller-storage `Slice<int>` overload currently requires interpreter execution:
native/JIT slice lowering supports byte views only. The allocating `GetBits(value)`
overload is available across the execution modes. This is a backend gap, not a
permanent Decimal restriction.


## Monetary values (development)

Prefer `System.Globalization.MonetaryAmount`, a record struct with readonly
`Amount: Decimal` and `Currency: Currency` components. Construction preserves all
Decimal precision. Equality and hashing include currency identity and numerical
Decimal equality; positional deconstruction is supported. Named `let MonetaryAmount(amount, currency) = value` patterns currently require an `else` branch. The default is zero
with `Currency.Unspecified` (XXX). No implicit conversion discards currency.

`MonetaryAmount.Parse(text, currency, culture)` and
`currency.ParseAmount(text, culture)` return
`Result<MonetaryAmount, NumberParseError>`. An explicit expected currency is required:
ambiguous symbols never select a denomination. Parsing accepts the expected symbol
or uppercase ISO code, exact locale spacing and direction marks, and valid groups
of three. Bare numbers, mismatched symbols and malformed grouping are rejected.
Overflow is reported separately; fractional input retains Decimal precision rather
than rounding to the currency's display digits.

`Format(culture)` uses stored currency identity and default display precision;
`Format(precision, culture)` selects 0–99 digits. `Formattable.ToString(format,
culture)` accepts empty or C/c with optional precision. Unsupported formats fault.
Parameterless `ToString()` retains the generated record diagnostic representation.
Display rounding is ties-to-even and never changes the record. Culture affects
presentation, never currency conversion. No cross-currency arithmetic, exchange
rates or cash rounding are supplied.

`Currency` is an immutable identity with `Code`, `DecimalDigits`, `GetSymbol(culture)`,
typed/boxed equality, hash and code `ToString`. The bounded catalog is SEK, ILS,
GBP, USD, EUR, JPY and Unspecified/XXX. `Currency.Parse(code)` requires exact uppercase
codes and returns `Result<Currency, CurrencyError>` (`UnsupportedCode` otherwise).
JPY defaults to zero fractional display digits, others to two. Lower-level
`Currency.Format(decimal, culture)` and its explicit precision overload remain
available, as does Decimal C formatting.

`NumberFormat` exposes Currency, CurrencySymbol, CurrencyDecimalDigits,
CurrencyDecimalSeparator, CurrencyGroupSeparator, CurrencyGroupSizes,
CurrencyPositivePattern, CurrencyNegativePattern, CurrencyNegativeSign and
CurrencyDirectionMark. Group sizes are fresh snapshots. Pattern indexes follow
.NET numbering (positive 0/3, negative 1/8). Swedish uses NBSP and U+2212;
Hebrew currency output includes U+200F and negative U+200E. Ordinary numeric signs
remain ASCII. These bounded CLDR-derived policies are not full NumberFormatInfo,
NumberStyles.Currency, accounting formats or customizable culture providers.

`Language.IsRightToLeft` is true for Hebrew and false for English, Swedish and
Undetermined. This is default writing direction, not a Unicode bidi layout engine.
