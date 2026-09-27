# Raven error-value APIs

Current Raven error unions use standard union declarations and case patterns,
including NumberParseError, BooleanParseError, FileReadError, FileWriteError,
ConsoleReadError, Utf8SliceError and IntegerDivisionError. These are typed values,
not an Exception hierarchy; their formatted descriptions are not discriminants.

NumberParseError has InvalidFormat and Overflow. All numeric Parse methods use
that union; BooleanParseError has only InvalidFormat. A default union has no
active case. Parsers return active error values, so ordinary callers match cases
rather than constructing wrappers or using legacy carrier accessors. Single-case
ordinary error types retain their individual constructor contracts.

The [numeric consumer](experiments/numeric-contracts/Main.rvn) demonstrates typed
matching, while [API documentation](../api-docs/text-numbers.md) covers the current
members and error grammar. The [error library sample](experiments/raven-target/samples/library-errors.rvn)
provides additional construction and pattern examples. Use matching development
references and runtime library artifacts.

Historical [Neo carrier contracts](runtime-error-contracts.md) are isolated in the
archived bootstrap. Their IsCase/GetCase spellings and initialized-storage rules
do not describe the current Raven union protocol.
