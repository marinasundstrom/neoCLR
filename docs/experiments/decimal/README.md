# Decimal foundation — development, 2026-10-21

Author direction: implement Decimal now using existing contracts, without the proposed
IntegerNumber/FloatingPointNumber hierarchy. This is a bounded addition during release
stabilization, not a reprioritization of the platform roadmap.

## Contract and implementation

The value domain follows [System.Decimal](https://learn.microsoft.com/en-us/dotnet/api/system.decimal?view=net-10.0):
a 96-bit unsigned coefficient, sign and decimal scale 0–28, with no NaN or infinity.
The ordinary Raven struct implements Number, EquatableTo<Decimal> and Formattable.
Equality/hash ignore equivalent scale and signed zeros; general formatting preserves
stored scale. Rounding is nearest, ties-to-even. GetBits exposes low/middle/high/flags;
this is a logical representation contract, not CLR binary interop.

Arithmetic uses seven base-10^9 value-type limbs for exact intermediate coefficients,
including multiplication and scale alignment. It does not approximate through Double
or allocate limb arrays. Formatting uses output byte storage and a String; Object and
interface conversions can box. Culture normalization may allocate text. No whole-API
zero-allocation or throughput claim is made.

Primary references are [.NET Decimal](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Decimal.cs)
and [Decimal arithmetic](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Decimal.DecCalc.cs).
The independently implemented Raven algorithm retains observable zero-result scale
behavior as well as numerical values. Differential tests found and corrected remainder
scale preservation, division-underflow scale and wide zero-product representation.

Raven `d6a035206` lowers decimal literals, built-in operators and integral input
conversions to declared constructors/methods. The default .NET emitter is unchanged.
Native metadata uses ordinary nominal value fields, with no new decimal opcode.
neoCLR `bc5efd883` fixes inherited Self views for boxed nominal implementations using
the runtime's existing interface closure; all 22 specialization regressions pass.

## Supported API and bounds

Integer and coefficient-word construction; implicit integer inputs; arithmetic,
comparison, value equality/hash; identities and limits; Scale/GetBits; Abs/Negate;
Round/Truncate/Floor/Ceiling; invariant and explicit-culture Parse/TryParse; and
G/G0, F/N formatting with precision 0–99. See the
[numeric reference](../../../api-docs/text-numbers.md#decimal-development) and native
RavenDoc member pages for details.

Parsing returns Result errors and TryParse resets output. Invalid scale/format,
arithmetic overflow and division by zero currently use System.Fail, rather than
recoverable CLR arithmetic exceptions. Exponents, floating conversions, integral
extraction conversions, decimal constant-field metadata, custom patterns and broader
serializer/database mappings are not implemented by this increment.

GetBits(value, Slice<int>) is interpreter-only because native/JIT slice lowering is
currently byte-only. The array-returning overload works across the execution modes.
The legacy CLI CoreProbe scaffold does not project Decimal; native RavenDoc supplies
its complete member reference. These are documented gaps, not new platform rules.

## Validation

The common built-in suite is `runtime/System.Runtime/test/DecimalTests/DecimalTests.rvnproj`. It covers
exact arithmetic, representation, equality/hash, literals and conversions, boxing,
generic Number calls, rounding/overflow parsing and Swedish culture integration.
`Oracle.cs` generates 51 successful CLR arithmetic boundary expectations; `Oracle.rvn`
checks them with aggregate diagnostics. The host program supplies oracle data only;
guest execution uses the built-in test discovery, registry and runner.

```sh
dotnet run --file docs/experiments/decimal/Oracle.cs
python3 scripts/test-runtime-library.py --bundle target/decimal-bundle \
  --output target/decimal-tests --suite decimal \
  --mode interpreter --mode jit --mode native
python3 docs/experiments/decimal/check-storage.py \
  target/decimal-bundle target/decimal-storage
```

The separate StorageTests project uses the same built-in framework and verifies that
GetBits writes only the caller's selected slice. Strict backend library Clippy passes.
[Execution evidence](validation.json) records all 21 common built-in executions passing
(seven tests in interpreter/JIT/native), including the 51 CLR vectors in each mode.
[Storage evidence](storage-validation.json) adds one interpreter test. API snapshots
pass; the website builds and validates 5,122 pages, with the local Decimal API HTTP 200.
This is
local development qualification, not a matched release package or benchmark result.

A separate Raven diagnostic candidate was observed while authoring the zero check:
`(left.high | left.middle | right.high | right.middle) == (uint)0` reported
RAV0103 for `left.high`. The final code expresses the intended four zero comparisons
directly. Investigating parenthesized qualified bitwise expressions belongs on Raven
main first; this increment does not claim that compiler issue fixed.
