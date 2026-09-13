# Raven recoverable integer division

`System.Int32.Divide(int dividend, int divisor)` returns
`Result<int, IntegerDivisionError>` in the experimental target. The existing runtime
method produces a quotient on success, `DivisionByZero` for a zero divisor, or
`Overflow` for `Int32.MinValue / -1`. Signed integer division truncates toward zero.

```swift
import System.*
func DivideInput(dividend: int, divisor: int) -> Result<int, IntegerDivisionError> {
    let value = Int32.Divide(dividend, divisor)?
    return Result<int, IntegerDivisionError>(Result.Ok<int>(value))
}
```

Typed matches can extract `Result.Ok<int>.Value` or inspect
`Result.Error<IntegerDivisionError>.Value.IsDivisionByZero` and `.IsOverflow`.
Propagation returns before following code executes on either error.

## Contract and comparison

This projects the existing library method without changing arithmetic instructions
or fault behavior. As the [API policy](api-policy.md) explains, Divide is an
experimental neoCLR helper, not a corresponding .NET Int32 member. Its final API
location remains open. Returning an ordinary Result lets callers handle expected
arithmetic failures explicitly; migrating conventional division code requires choosing
this helper and handling its result. The compiler does not automatically rewrite `/`
into this API, and this slice does not add operator lowering.

The [error-value projection](raven-error-api.md) additionally exposes case constructors,
checked accessors and ToString.

## Trying it

Prepare a fresh Raven collections project using the
[integration instructions](experiments/raven-target/README.md). Copy
[the division sample](experiments/raven-target/samples/library-division.rvn) into its
`Main.rvn` and run **neoCLR: Run saved project**. It covers positive and negative
quotients, zero, the minimum Int32 value, division by zero and overflow.

The saved-project suite verifies exact output, including absence of the success
message after either failure. Signature checks reject mismatched argument and return
types. `verify_editor.py --parsing` includes Divide completion. This source bridge
slice does not rebuild the installed SDK or extension; use fresh target metadata.
