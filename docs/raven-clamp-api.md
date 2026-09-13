# Raven integer clamping

The target exposes `Math.Clamp(int value, int min, int max)` returning
`Result<int, InvalidRangeError>`. Values below/above the inclusive bounds become
min/max; values inside the range stay unchanged. Equal bounds are valid. Reversed
bounds return an error.

```swift
import System.*
func ClampInput(value: int, min: int, max: int) -> Result<int, InvalidRangeError> {
    let bounded = Math.Clamp(value, min, max)?
    return Result<int, InvalidRangeError>(Result.Ok<int>(bounded))
}
```

`InvalidRangeError` is the existing ordinary error value, not a union with invented
subcases. Match `Result.Ok<int>` or `Result.Error<InvalidRangeError>` to distinguish
success and failure. The [error-value projection](raven-error-api.md) also exposes its public constructor
and ToString.

The [Math contract and .NET research](math.md) apply unchanged. The inclusive bounds
are familiar, but neoCLR returns an error Result where .NET Clamp throws for reversed
bounds. Callers must adapt error handling. No arithmetic opcode, runtime or Raven
compiler changes are needed to project the existing method.

The [sample](experiments/raven-target/samples/library-clamp.rvn) covers values within
and outside bounds, equal/reversed bounds, and both Int32 extrema. Copy it to
`Main.rvn` in a fresh [prepared target project](experiments/raven-target/README.md)
and run **neoCLR: Run saved project**. The saved-project checker verifies the exact
output and early return after propagation failure. Editor checks include Clamp
completion.

All five existing Int32 Math methods (Abs, Min, Max, Sign and Clamp) now have bounded
Raven projections. The [Double Math slice](raven-floating-math-api.md) projects the remaining runtime
Math methods with bounded primitive support. Installed SDK/VSIX assets are
unchanged; generate fresh target declarations to try the sample.
