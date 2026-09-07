# Checked arithmetic error contracts

The canonical library methods use ordinary typed Results:

- `System.Math.Abs(Int32) -> System.Result<Int32,System.OverflowError>`
- `System.Int32.Divide(Int32,Int32) -> System.Result<Int32,System.IntegerDivisionError>`

Abs fails only for minimum Int32, whose positive magnitude cannot be represented.
OverflowError is a single-purpose, fieldless ordinary type with a constructor and
ToString returning `Overflow`. It does not require a union or inherit from System.Error.

IntegerDivisionError is a non-generic ordinary carrier with directly nested fieldless
DivisionByZero and Overflow cases. It has constructor overloads, IsDivisionByZero and
IsOverflow properties, checked GetDivisionByZero/GetOverflow accessors and ToString.
Zero divisors produce DivisionByZero, including zero divided by zero. Minimum Int32
divided by -1 produces Overflow. Valid quotients truncate toward zero.

The Result uses its existing nested companion wrappers. Example success extraction:

```text
ldc.i4 84
ldc.i4 2
call System.Int32::Divide(Int32,Int32)
call instance System.Result<Int32,System.IntegerDivisionError>::GetOkCase()
call instance System.Result.Ok<Int32>::get_Value()
```

General code preserves the Result, branches on get_IsErrorCase and extracts the error
through GetErrorCase and System.Result.Error<System.IntegerDivisionError>.Value before
inspecting its case. Mismatched extraction is a Fault; formatted text is not a discriminant.

Both methods are platform IL with no added runtime services or union-specific instructions.
The existing explicit carrier storage remains in use. The raw `div` instruction still
Faults on zero division and minimum Int32 / -1. Other unchecked arithmetic keeps its
existing wrapping semantics. These library contracts do not introduce exception handling
or implicit propagation.

This changes Abs's error parameter and replaces Divide's bootstrap Result return.
Reassemble applications and System together. Replace old ldcase extraction after Divide
with ordinary calls, and use the new error arguments in signatures. There are no parallel
Typed methods. The format number remains 3; that is not a library API compatibility promise.
