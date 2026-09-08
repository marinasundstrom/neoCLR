# Fundamental Math operations

This bounded slice extends the ordinary System.Math library. It does not introduce
new opcodes or a complete numeric framework.

## Surface and behavior

- Int32 Min and Max return an operand; Sign returns -1, 0 or 1 without subtracting
  operands or overflowing at Int32.MinValue.
- Int32 Clamp(value, min, max) returns Result<Int32,InvalidRangeError>. Valid bounds
  clamp inclusively; min > max returns Error. Abs(Int32) keeps its existing
  Result<Int32,OverflowError> contract.
- Double Abs, Min, Max, Sqrt, Pow, Floor, Ceiling, Truncate, Round, Exp, Log, Log10,
  Sin, Cos and Tan return Double. Log is the natural logarithm. Angles are radians.
- Round(Double) rounds to the nearest integral Double with ties to even. There is
  no digits or rounding-mode overload yet. Floor rounds down, Ceiling up, Truncate
  toward zero. These do not convert the result to an integer.
- Double Min/Max propagate NaN. Min selects negative zero and Max positive zero
  when comparing opposite signed zeros. Abs clears the sign, including negative
  zero. Domain errors in floating functions produce NaN; floating overflow produces
  infinity where applicable, with no fabricated Result error or terminal fault.

Inputs are values, with no retained reference, mutation or heap ownership. Integer
policy and Result construction live in IL. Fifteen signature-validated bootstrap
helpers implement Double operations using host floating facilities and explicit
NaN/zero selection for Min/Max. These declare the MathOperations runtime service;
integer methods do not. Host code exhaustively matching RuntimeService must account
for the new variant. Host functions allocate no managed objects. Their internal
work is not separately instruction-metered. NaN payload bits and transcendental
last-bit results are not cross-host reproducibility guarantees.

## .NET comparison and choices

Primary sources consulted 2026-09-08, targeting .NET 10:

- [Math.Round](https://learn.microsoft.com/en-us/dotnet/api/system.math.round?view=net-10.0)
  supplies the familiar ties-to-even default; using Rust's round-to-away default
  would change results at 2.5 and -2.5, so use round_ties_even explicitly.
- [Math.Min](https://learn.microsoft.com/en-us/dotnet/api/system.math.min?view=net-10.0)
  propagates NaN. The [v10.0.0 library source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Math.cs)
  also defines the signed-zero ordering. Do not directly use host min/max functions
  with different NaN selection rules.
- [Math.Sqrt](https://learn.microsoft.com/en-us/dotnet/api/system.math.sqrt?view=net-10.0)
  and [Math.Pow](https://learn.microsoft.com/en-us/dotnet/api/system.math.pow?view=net-10.0)
  establish floating domain/special-value behavior. Tests cover NaN, infinity,
  negative and zero inputs, and approximate finite transcendental results.
- [Math.Clamp](https://learn.microsoft.com/en-us/dotnet/api/system.math.clamp?view=net-10.0)
  throws when bounds are reversed. neoCLR returns a typed Result instead, consistent
  with its existing integer Abs. This is a deliberate source/API difference: it
  enables ordinary union handling but requires callers to unwrap success too.

.NET already combines library code and runtime intrinsics for numeric APIs. We
preserve that responsibility split at prototype scale; a future JIT can specialize
ordinary calls after checking the same contracts. No performance improvement is
asserted. Decimal, MathF, broader integer overloads, Double Clamp/Sign, rounding
modes, constants and additional transcendental functions remain future extensions.
Double Sign needs an explicit NaN failure contract before implementation.

## Neo projection and validation

Neo now accepts finite Double literals with a fractional part or exponent (`2.5`,
`1e3`, `-0.0`), unary negation and same-type arithmetic/comparison. Integer literals
remain Int32; mixed numeric operations and implicit widening are rejected. Ordered
comparisons against NaN are false, while `!=` is true. These lower to existing
floating IL, including unordered comparisons when negating <= and >=. This is
language policy, not a change in numeric storage. Leading/trailing-dot literals,
suffixes, digit separators and special NaN/infinity literal names remain unsupported.

```sh
cargo run --locked -- run examples/source/math.neo
cargo test --locked --test math_helpers --test math_typed --test arithmetic_errors
(cd docs/experiments/math-dotnet && dotnet run)
```

The sample combines a triangle calculation, ties-to-even rounding, integer clamping
and exhaustive Result matching. Tests include direct calls, serialized artifacts,
source type errors, NaN comparisons, signed zero and typed failures. The pinned
.NET SDK 10.0.100 probe provides executable comparison evidence.
