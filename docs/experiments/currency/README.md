# Monetary foundation — development, 2026-10-21

Historical qualification: the author subsequently renamed MonetaryAmount to Money.
See [deconstruction and Money follow-up](../deconstruction/README.md) for the current
name and completed language integration; the evidence below retains its original names.

Author-directed bounded work after Decimal. MonetaryAmount is the primary API:
an ordinary record struct with Decimal Amount and Currency Currency. Positional
storage, equality, hashing and deconstruction remain compiler-generated; there is
no runtime monetary intrinsic. Decimal C formatting remains a convenience.

## Comparison and decisions

.NET Decimal and NumberFormatInfo keep currency identity outside the numeric value.
neoCLR retains that lower-level facility but adds a denomination-bearing value
object so monetary parsing does not discard identity. This adds a type and requires
callers to extract Amount explicitly for numeric operations; it prevents culture
selection from silently changing a monetary value's denomination. Arithmetic,
exchange rates, cash rounding and an open-ended currency registry remain outside
this increment. Default records represent zero XXX; constructors do not round.

Primary references:
- [.NET NumberFormatInfo](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-globalization-numberformatinfo)
- [Unicode LDML numbers](https://www.unicode.org/reports/tr35/tr35-numbers.html)
- [CLDR numbers and currencies](https://github.com/unicode-org/cldr-json/tree/main/cldr-json/cldr-numbers-full/main)

Manually selected CLDR en/en-GB/sv/he standard currency patterns and symbols provide
the bounded presentation policy. This is not an OS locale dependency or complete
CLDR implementation. Swedish uses NBSP and U+2212. Hebrew adds RLM to the amount
and symbol and LRM to the negative sign. Invariant uses generic currency XXX/¤.
Unlike .NET NumberStyles.Currency's permissive options, parsing requires the
expected symbol or ISO code and exact spacing/marks. This avoids ambiguous currency
inference but is less tolerant of manually entered text; plain Decimal.Parse is
available when the caller already has separate denomination context.

Formatting writes digits and affixes into one output byte buffer without an
intermediate formatted number. Parsing validates borrowed UTF-8 windows and calls
Decimal's numeric parser without substring or normalization arrays. The result is
a value record; formatting still allocates its output and interface use may box.
No performance benchmark or zero-allocation claim is made.

## Native record integration

The native Raven emitter previously rejected any authored record members and
limited explicit component contracts to imported nominal values. It now routes
record methods through ordinary callable lowering and uses declared equality/hash
and ToString contracts for source nominal values too, including Decimal. There is
no monetary special case. Additional record fields/properties/constructors and
explicit interface mappings remain outside this bounded admission. Ordinary .NET
record emission and Runtime Contract configuration are unchanged.

## Validation

`runtime/System.Runtime/test/CurrencyTests/NeoClr/RuntimeTests/Tests.rvn` uses the built-in framework. It covers all
five cultures, six named currencies, exact parsing, monetary record construction,
copy/deconstruction, equality/hash/boxing, Formattable and preservation of precision.
The Decimal suite provides arithmetic and numeric-format regression coverage.
Final evidence is recorded after execution; the initial pre-record currency suite
passed five tests in each of native, JIT and interpreter modes.

Named value deconstruction patterns now share the declared Deconstruct call path
with deconstruction assignment. Matching is bounded to the same non-nullable value
type and simple/no outer capture. The current parser requires an `else` branch for
`let MonetaryAmount(amount, currency) = value else { ... }`; allowing omission for
an irrefutable named record pattern remains a separate general Raven syntax gap.


Qualified monetary execution: [validation.json](validation.json) records six tests
in each of native, JIT and interpreter (18 passes). The matching compiler work is
Raven codex/neoclr `ae1802eef`: three focused PositionalRecordPlanTests pass.
The five source-owned native libraries build. Native and legacy API snapshots
pass, and the website build checks 5,216 pages (124 retained legacy pages).

The library snapshot was built from the working increment based on neoCLR
`2ae82910c` and Raven `d6a035206`, including the record-method/component changes;
its explicit `record-methods-working` provenance is retained. Final guest lowering
also includes deconstruction support committed in `ae1802eef`. This is development
qualification, not a clean release bundle or a release-performance claim.

The final compiler build additionally passes all six monetary tests in interpreter
mode: [final-interpreter.json](final-interpreter.json). This verifies the final
same-value-type pattern admission guard alongside the three compiler regressions.

Decimal regression: [decimal-regression.json](decimal-regression.json) records all
seven tests in native, JIT and interpreter (21 passes), including 51 CLR arithmetic
vectors in each mode. The shared numeric parsing/formatting refactor therefore
retains the previously qualified Decimal contracts.
