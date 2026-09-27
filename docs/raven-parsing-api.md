# Raven numeric parsing

Development numeric `Parse(string)` methods return `Result<T, NumberParseError>`
for SByte, Byte, Int16, UInt16, Int32, UInt32, Int64, UInt64, Single and Double.
Int32 and Int64 use the same error union as the other numbers. Boolean has a
separate format-only BooleanParseError. No Parsable interface is exposed.

Match standard Raven union cases; avoid manually wrapping carriers or comparing
error descriptions. This excerpt comes from the [tested consumer](experiments/numeric-contracts/Main.rvn):

```raven
match Int32.Parse("2147483648") {
    Error(NumberParseError.Overflow) => { }
    _ => System.Fault("Int32 shared parse error")
}
```

All parsers consume the complete text and reject whitespace/grouping. Integers use
optional ASCII signs and decimal digits. Floating parsers additionally support a
decimal point, exponent and explicit NaN/Infinity spellings. Numeric grammar errors
are distinguished from overflow; floating underflow can round to signed zero.
See the [API reference](../api-docs/text-numbers.md) for exact grammar and the
[design comparison](design/numeric-contracts.md) for .NET behavior and tradeoffs.

Raven's .NET framework projections stay disabled for this target
(`RavenFrameworkProjections=None`): these are direct typed-result APIs rather than
catch-to-Result projections of .NET Parse. Number's arithmetic constraint is
independent of parsing. The current numeric generic-import limits are documented
alongside the [focused evidence](experiments/numeric-contracts/README.md).

The [archived Neo parser](int32-parse.md) retains its old compatibility carrier;
its type names are not the current Raven API. Rebuild consumers and matching
references when migrating old Int32ParseError/Int64ParseError patterns to
NumberParseError. The existing Result-returning [Int32.Divide](raven-division-api.md)
is a separate error contract.
