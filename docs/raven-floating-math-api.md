# Raven Double Math APIs

The experimental bridge admits Double literals (`ldc.r8`), value parameters/returns,
locals, and concrete `Double.CompareTo` receivers. It projects all 15 existing Double
Math methods: Abs, Min, Max, Sqrt, Pow, Floor, Ceiling, Truncate, Round, Exp, Log,
Log10, Sin, Cos and Tan. With the Int32 subset, all 20 methods currently declared
in runtime System.Math are callable through this bounded profile.

The [existing Math contracts and .NET comparison](math.md) apply unchanged. These
methods return Double directly; floating domain/overflow outcomes use NaN/infinity,
not newly invented error Results. Round uses ties to even. The runtime's existing
Double comparison contract orders NaN below numbers and compares NaNs equally.

## Example

Copy the [floating Math sample](experiments/raven-target/samples/library-floating-math.rvn)
into a fresh [prepared project](experiments/raven-target/README.md), then run
**neoCLR: Run saved project**. It prints comparison results: nineteen zeros followed
by -1. The checks exercise all 15 functions, ties-to-even rounding, NaN propagation,
a floating overflow call, and NaN ordering, through local and parameter receivers.
The comparisons use exact, simple cases; they are not a cross-platform accuracy test
for transcendental functions.

Console does not yet have a Double output overload in the runtime library. This
example therefore prints comparison results using the existing Int32 overload.
It does not silently borrow host formatting or Console APIs.

## Boundaries

The compiler emits ordinary CLI literal and call instructions. The importer maps
those into existing neoCLR instructions and library calls, with no Raven compiler
or runtime change. Literal text is emitted with invariant culture and round-trip
precision. Argument addresses remain bounded to admitted concrete primitive receivers;
this is not general constrained/interface dispatch.

This does not add Single, Decimal, numeric conversions, floating arithmetic operator
lowering, formatting, nullable numerics or the wider .NET Math overload set. Existing
host-dependent transcendental precision and signed-zero/NaN contracts remain as
specified in the runtime Math documentation. The sample does not independently prove
signed-zero selection or NaN payload preservation.

Saved-project tests verify sample output alongside earlier integer, Result and
collection examples. Signature checks reject wrong Math argument types and validate
Double receiver mapping. Editor checks cover the admitted Math method names. SDK and
VSIX assets have not been refreshed; use freshly generated declarations.
