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


## Aggregating existing errors with an ad-hoc union

Development code can return `Result<Config, FileReadError | NumberParseError>`
and propagate either underlying Result with `?`, without declaring a wrapper,
inheriting a shared error type, implementing an interface or writing implicit
converters solely for aggregation. The signature makes both alternatives
discoverable; typed patterns distinguish them after extracting the Result error.
See the [tested configuration loader](experiments/adhoc-error-unions/Config.rvn)
and [validation and temporary bridge](experiments/adhoc-error-unions/README.md).

Prefer this when the caller should see the existing error types. For many public
APIs, a dedicated error type or nominal union is still preferable: it names the
contract and its cases and avoids exposing incidental dependencies. Keep implicit
converters for intentional domain adaptation. This supplements, rather than
replaces, the existing conversion guidance.


### Review existing API aggregation

Author clarification, 2026-10-13: ad-hoc unions (eventually structural unions) may
also improve existing APIs despite the preference for many nominal public error
contracts. Evaluate each API's meaning and compatibility rather than categorically
reserving ad-hoc unions for private code.

A concrete review candidate is `HttpJsonError`: it currently carries either
`HttpError` or `JsonError` and forwards the cause's formatted description. An
ad-hoc `HttpError | JsonError` could expose those alternatives directly and remove
wrapping/converter boilerplate. Costs include changing public return signatures,
existing Http/Json case patterns and their named domain boundary. This is an
assistant-proposed candidate, not an approved migration; the current HttpJsonError
API is unchanged. Errors such as NumberParseError.InvalidFormat/Overflow retain
useful domain cases and are not aggregation-only wrappers.
