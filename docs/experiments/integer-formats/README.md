# Bounded integer formatting — development, 2026-10-20

The author requests a **good-enough** globalization foundation. This follow-up adds
common signed-integer formatting, without expanding the culture catalog or adding
custom providers. Int32 and Int64 declare Formattable and expose
`ToString(format)` and `ToString(format, culture)`; composite formatting uses the
same implementation. Parameterless ToString and parsing remain invariant.

| Format | Behavior |
| --- | --- |
| empty, G/g, G0/g0 | Ordinary signed decimal, no grouping |
| D/d | Decimal; precision is the minimum digit count, excluding sign |
| F/f | Fixed-point; precision is fractional zero count, default 2 |
| N/n | As F, with groups of three and culture-specific separator |

D/F/N accept zero to two ASCII precision digits (0–99). Unknown/custom formats,
nonzero G precision and larger precision strings fault before output allocation.
The current profiles use ASCII hyphen-minus and leading negative signs. Swedish
uses U+00A0 grouping and comma fractions; other built-ins use commas and points.
These are the existing NumberFormat data, not a new locale table. Other integers,
floating-point, currency, exponential and hexadecimal formatting remain gaps.

## Contract and implementation comparison

[.NET standard numeric formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings)
define D minimum digits and F/N fractional precision. neoCLR follows those meanings,
including signs outside decimal padding and no decimal separator at precision zero.
Unlike modern .NET, precision is capped at 99 for bounded output; G significant-digit
rounding/exponents are unsupported and rejected rather than silently approximated.
This is a documented coverage/resource limit, not a claim of superior semantics.
The existing concrete Culture parameter remains the Formattable provider contract.

One Raven helper handles both widths. Signed division/remainders avoid negating
Int64.MinValue. General decimal delegates to the existing native integer text
service. Formatted output uses one exactly sized managed byte buffer and a borrowed
UTF-8 view to create the result string: no per-digit strings, intermediate decimal
text, group arrays, builder or NumberFormat instance. The output buffer is temporary;
a future measured optimization could write directly to caller-owned storage.
Interface/composite arguments can still box. No throughput or allocation-free claim.

## Validation

The consumer covers all five cultures, signed minima/maxima, default/lowercase
formats, grouping, precision zero and 99, direct/interface/composite dispatch, the
current-culture overload and GC. A separate fault consumer covers unsupported
specifiers, malformed/excessive precision and unsupported significant-digit formats.
Evidence records interpreter/JIT/AOT execution and binary hashes. Documentation
mirrors the new methods and interface in the compiler reference and native snapshot.
No runtime ABI or compiler contract change is intended.
