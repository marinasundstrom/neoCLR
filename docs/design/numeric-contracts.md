# Numeric contracts and parsing

The development class library provides basic arithmetic capabilities in
System.Numerics, the ordered System.Number aggregate, and concrete parsing methods.
The eight fixed-width integer types, Single and Double implement Number. Char
represents a grapheme and is not a numeric implementer.

## Arithmetic capabilities

| Interface | Static operator |
| --- | --- |
| AdditionOperators | `+(left: Self, right: Self) -> Self` |
| SubtractionOperators | `-(left: Self, right: Self) -> Self` |
| MultiplyOperators | `*(left: Self, right: Self) -> Self` |
| DivisionOperators | `/(left: Self, right: Self) -> Self` |

These are nominal interfaces: types explicitly declare conformance and provide the
required operator. An operator with a matching shape alone does not establish
membership. Generic algorithms can require only the capability they use.

Number inherits all four interfaces and ComparableTo<Self>, and declares Zero,
One and its arithmetic operators. It does not require parsing. Native
[Self](../self-types.md) denotes the implementing type in operands and results.

[.NET generic math](https://learn.microsoft.com/en-us/dotnet/standard/generics/math)
also separates operator capabilities from numeric aggregates. NeoCLR uses Self
instead of a repeated self-type parameter. This simplifies same-type contracts but
does not express .NET's independent operand/result types. Mixed-type contracts,
separate identity interfaces, checked default operators and a NumberBase-like
aggregate remain unimplemented. Native-sized integers, Decimal, transcendental
functions and numeric conversion modes are outside this minimal surface.

Ordinary integer arithmetic wraps; invalid integer division faults. Floating
arithmetic and existing CompareTo semantics are retained, including NaN ordering
distinct from IEEE equality. Custom operators define their domain and failure
policy. Operator conformance does not promise recoverable errors; the existing
Result-returning Int32.Divide and proposed recoverable arithmetic APIs are separate.

## Compiler and runtime support

Raven's NeoCLR target emits native generic method bounds and Self signatures.
Closed nongeneric nominal interface bounds and static Self calls work in the
interpreter, managed JIT and AOT. Source-local generic functions, class-contained
static methods and imported versions of both are qualified. Imported references
carry bounds from compiler symbols through explicit SDK contracts; no dependency
reader fallback supplies missing semantic information.

Primitive arguments retain their selected nominal owner's declared interfaces.
JIT/AOT specialization checks conformance against original metadata before resolving
static Self calls to concrete implementations. Constructed interface method bounds
and instance/borrowed Self specialization are not covered by this capability path.
Ordinary Raven/.NET continues to use .NET contracts and delegate/runtime conventions.
See the [integration contract](../raven-cli-bridge.md#nominal-numeric-constraints)
for ownership and representation details.

## Complex demo

The [library consumer](../validation/numerics-interfaces/LibraryConsumer.rvn) defines
a Complex struct with double components and explicit arithmetic capabilities.
It does not implement Number or invent an ordering. This follows the distinction in
[.NET Complex](https://learn.microsoft.com/en-us/dotnet/api/system.numerics.complex),
which implements arithmetic interfaces and INumberBase rather than ordered INumber.

The demo covers finite, moderate inputs. Scaled division, zero-divisor policy,
NaN/infinity/extreme magnitudes, equality/hash, formatting and performance remain
unqualified. Complex is a model test, not a production class-library numeric type.

## Concrete parsing

Parsing uses whole-text, culture-independent ASCII grammar. All numeric types admit
an optional sign; unsigned negative nonzero values overflow, while -0 is zero.
Single/Double admit decimal mantissas and exponents plus exact NaN and Infinity
spellings. Numeric floating overflow is an error; underflow can round to signed zero.
Boolean accepts ASCII case-insensitive true/false. Parsers do not trim, decode bytes
or accept grouping. Whole grammar validation precedes range validation.

Numeric parsers return Result values with NumberParseError (InvalidFormat/Overflow).
Boolean uses BooleanParseError (InvalidFormat only). Unlike
[default .NET Double.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.double.parse?view=net-10.0),
this contract has no ambient culture or grouping, uses typed errors rather than
exceptions, and rejects numeric overflow instead of producing infinity. The benefit
is predictable configuration/protocol parsing; the cost is less convenient human
input and no .NET parsing compatibility promise. UTF-8 storage does not dictate
numeric grammar or require a byte-oriented Parse signature.

### Proposed numeric parsing capability

A separate numeric parsing interface remains on hold. Its purpose would be to let
a generic algorithm require a static Parse operation returning
Result<T, NumberParseError>, independently of arithmetic. Its final name and Self
shape are undecided; it is not an available API.

Compared with [.NET IParsable](https://learn.microsoft.com/en-us/dotnet/api/system.iparsable-1?view=net-10.0),
a fixed numeric error contract avoids exceptions/out parameters but cannot describe
arbitrary parsers with different errors. Rust's
[FromStr](https://doc.rust-lang.org/std/str/trait.FromStr.html) demonstrates an
associated error type. A future contract needs focused generic parsing and import
validation before adoption; existing arithmetic support does not establish it.

## Validation and reference

The [numerics qualification](../validation/numerics-interfaces/README.md) records
ten-primitive arithmetic, Number-bound forwarding, Complex, separate-provider calls,
invalid conformance rejection and interpreter/JIT/AOT results. The reference
signature checks cover 218 assertions. The [API reference](../../api-docs/text-numbers.md)
and native documentation snapshot describe the public types and members.
