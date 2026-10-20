# Primitive numeric formatting and culture parsing — development, 2026-10-20

The author requested Formattable on the remaining numeric primitives, then explicit
culture parsing, and directed using the built-in test framework instead of standalone
applications where possible. Public API tests live in
[`numeric-formatting`](../../../runtime/raven/tests/numeric-formatting/Tests.rvn).
The seven-test [`globalization`](../../../runtime/raven/tests/globalization/Tests.rvn)
suite checks all five profiles, UK/US clock boundaries, preference resolution and
isolation of returned calendar/grouping arrays. The existing primitive-parsing suite
is the invariant regression. All three use discovery,
Result assertions and the standard runner in interpreter, JIT and native modes.

## Contracts

All twelve numeric primitives declare Formattable: signed/unsigned integer widths,
IntPtr/UIntPtr, Single and Double. Boolean and grapheme Char remain nonnumeric.
The shared integer formatter now handles the entire UInt64 range as well as signed
minima. Existing G/G0, D, F and N semantics and the 0–99 precision bound are unchanged.

Single/Double support empty/G/G0 and R with shortest round-trip digits, G precision
for significant digits, F/N fractional digits (default two), and E fractional
mantissa digits (default six). N groups in threes; E uses a three-digit exponent,
G/R scientific notation a minimum two-digit exponent. Case controls exponent case.
R ignores valid precision. Non-finite symbols are NaN, Infinity and -Infinity for
all bounded profiles. Negative zero is preserved. Explicit precision rounds ties
to even. Parameterless display is invariant, including boxed display; one-argument
formatted display resolves current culture, and explicit-culture/composite calls
use the supplied culture. Unsupported/custom formats and precision above 99 fault.

Every numeric primitive has Parse(text, culture). Native-width integers also gain
invariant Parse(text), with a conversion-back range check. Culture parsing accepts
ASCII signs/digits and correctly placed groups of three; float parsing additionally
accepts decimal fractions, exponents and the existing non-finite spellings. Swedish
uses comma decimals and U+00A0 grouping, not ordinary spaces. Other built-ins use
point decimals and comma grouping. Mixed punctuation, whitespace and malformed
groups return InvalidFormat. Existing overflow classification is preserved. Integer
parsing does not accept decimal fractions, even fractional zeros. One-argument
parsers retain strict invariant grammar.

## Comparison and limitations

[.NET standard numeric formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-numeric-format-strings)
are the semantic model for G/R/F/N/E and midpoint rounding. A differential check
compares fourteen representative invariant Double cases with .NET 10, including
scientific thresholds, round-trip boundaries, rounding and negative zero.

[.NET numeric parsing](https://learn.microsoft.com/en-us/dotnet/standard/base-types/parsing-numeric)
and [Double.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.double.parse)
provide the comparison for culture and error behavior. Explicit-culture parsing is
deliberately narrower than .NET NumberStyles/provider
APIs. It accepts grouped integers so N0 output can be read directly, unlike default
.NET integer styles; it validates grouping strictly instead of accepting misplaced
group separators. This catches ambiguous/accidental punctuation at the cost of
rejecting some text .NET accepts. There is no whitespace trimming, custom grouping,
localized digits, configurable non-finite symbols or arbitrary NumberStyles. The
existing invariant overload is retained for protocol text. Future provider/styles
work must preserve that explicit distinction. Floating overflow continues to return
Overflow, whereas modern .NET may produce infinity for an out-of-range value.

## Implementation and allocation

Raven owns format validation, culture selection and integer output. Floating
conversion is shared Rust core code compiled into the interpreter and the no_std
native text kernel. Bounded stack buffers hold conversion output; native execution
allocates only the final managed String through the existing rooted text allocator.
No locale-sensitive libc formatting, temporary managed string/array, per-digit
string or replacement box is needed for float output. Interface conversion still
boxes. Integral decorated output retains one exact byte buffer and its final String.

Culture parsing first validates borrowed UTF-8 bytes. If punctuation needs no
translation, the original String is passed to the existing parser. Translated input
currently needs one exact byte buffer and normalized String; the result follows
ordinary Result carrier rules. This is not allocation-free parsing. A future
borrowed-byte parser could remove the normalization String; no throughput claim is
made here.

The two floating InternalCalls exposed the metadata producer's ownerless-function
cap. Producer and reader now use the existing 4096-function module bound, leaving
nominal owners at 256 methods. A focused test covers 257-function roundtrip and
4096 admission/rejection. No compiler semantics, Runtime Contract option, source
primitive layout, public ABI or metadata encoding changes. The compiler remains
4c051e332-dirty-record-policy with a separately hashed rebuilt metadata library;
native documentation uses renderer 8da3a45f8. Native-width static methods now use
the same verified primitive-owner projection as the other scalar primitives; the
shared primitive classification replaces a duplicated list. Native interpreter returns
now normalize exact storage values to evaluation-stack values just like managed
returns (including UInt64); a focused unsigned-return comparison regression protects
this common boundary. The float adapter uses the existing 32-bit Boolean call ABI.
See the bridge integration notes.

## Validation

The built-in suite covers numeric widths, direct/interface/composite dispatch,
UInt64 maximum and Int64 minimum, all five cultures, floating subnormals/extremes,
NaN/infinity/signed zero, bounded precision, boxed display, GC retention and parsing
failures. Rust unit tests additionally check 4096 deterministic bit patterns for
each floating width and midpoint/boundary formatting. Reports retain exact artifact
hashes and execution outcomes. Only macOS ARM64 is qualified by this work; other
platforms and full .NET format/provider parity remain outside this evidence.

## Good-enough boundary

This increment is complete when the five built-in cultures, supported primitive
numeric formats and explicit-culture parsers pass the built-in suites across
interpreter, JIT and native execution, alongside the focused conversion/metadata
regressions and matching public API reference. This establishes a bounded foundation,
not general locale compatibility. Custom numeric/date patterns, arbitrary providers,
additional locales, localized digits and configurable signs remain deferred.

Author-directed sequence: enum Parse/TryParse is next, then resume release-readiness
work. That follow-up should reuse enum metadata and ordinary value semantics, with
names, numeric values, comma-separated names and explicit case handling tested in
the built-in framework. No enum parsing API is claimed by this increment.

Final evidence: [built-in suites](validation-20261020.json) pass all 23 tests in each
of the three engines (69 executions). [Core regressions](core-validation-20261020.txt)
pass two floating conversion tests, fourteen native-binding/integer tests and the
metadata module-count check. The [fourteen-case .NET comparison](dotnet-comparison-20261020.json)
also passes. API snapshot validation passes four checks, the source type inventory
matches, and the website build checks 5032 pages. Website publication and replacement
release qualification remain separate. Raven integration notes: commit 6bd12a225.
