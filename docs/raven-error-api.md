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

The first migration is the development HTTP JSON client surface: all GetFromJson
and PostAsJson overloads now return `Result<..., HttpError | JsonError>` inside
Task. HttpJsonError merely labeled those existing types and forwarded ToString;
its extra layer added no distinct failure meaning. GET now propagates transport
and decode errors with `?`; POST preserves the original serialization/HTTP error.
Cancellation remains task cancellation and GET status rejection still precedes
decoding. The nominal HttpJsonError type remains available for explicitly declared
contracts but is no longer returned by these helpers. This is a breaking development
signature change: rebuild consumers and replace wrapper patterns with typed matching.

The review retains HttpError and JsonError: both define domain failures as well
as typed causes. In particular, HttpError.Content and SecureTransport carry the
same StreamError type with different meanings; an ad-hoc union would lose that
distinction. NumberParseError.InvalidFormat/Overflow and TextReadError likewise
name domain outcomes. The report sample retains AppError because its Body case
belongs to the application, and explicitly adapts library alternatives at that
boundary. No further aggregation-only public error wrapper was found in this
bounded review of the current runtime Raven source unions.

See [HTTP JSON validation](experiments/http-json-client/README.md). Relative to
.NET's HTTP JSON conveniences, this changes neoCLR's existing typed Result error
contract, not HTTP status, decoding or cancellation semantics. The established
[JSON design comparison](json-dom-design.md) still applies. Benefits are visible
alternatives and less forwarding code; costs are signature/pattern migration and
exposing both dependencies in the public contract. Native structural unions remain
planned; the temporary System.Union bridge and its limits are unchanged.
