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


### Remaining sample/test review — 2026-10-14

Reviewed Raven sources in `docs/experiments`, `runtime/raven/tests`, and their
embedded Python/C# fixtures for wrapper unions, implicit error converters and
MapError/stringification at composition points. The strong current candidate is
[JSON mapping orchestration](experiments/json-object-mapping/Main.rvn): independent
stream, JSON and schema-mapping failures now propagate as an ad-hoc union, with
executable checks for all three. MappingError retains its domain cases.

| Reviewed area | Decision |
| --- | --- |
| JSON mapping RoundTrip/ReadReport | Preserve StreamError, JsonError and MappingError; remove the lossy seek-to-schema mapping. |
| HTTP report AppError and mapping MappingError | Keep nominal schema/body rules and intentional domain conversion. |
| RouteMappingError, BoundaryError, SelectionFailure and codec failures | Keep named validation states, context and distinct domain outcomes. |
| HTTP status conversion/boxing and outcome-operator tests | Keep converters and mappings: those behaviors are the subjects under test. |
| Runtime test framework TestFailure | Keep assertion descriptions and expected/actual values; an operational error is not an assertion result. |
| Deferred async/union ABI and async-forward-return fixtures | Keep their nominal types: they test retention, representation or binding, not error aggregation. |
| File-transformer and early JSON/byte-copy prototypes | Retain the older CLI checkpoint. File-transformer is a future modernization candidate, but changing its union signatures alone would break its legacy importer; its synchronous stream calls and sample-owned JSON API also predate current contracts. |

Single-error pipelines do not need an ad-hoc union merely to replace a mapper with
a larger type. Historical artifact reports and published samples are not rewritten
as if they used the new representation. This is a sample/style change; existing
.NET comparisons and the temporary native bridge contract remain unchanged.


## Opt-in Error convention (2026-10-23 development)

Author direction: Error participation is opt-in for authors and a platform
convention. Result and general propagation remain unconstrained. The first slice
adds System.Error, ContextError<E>, explicit WithContext and bounded FormatError;
FileReadError, NumberParseError and StreamError adopt the interface. Cause and
StackTrace use Option rather than nullable references; the typed wrapper cause
remains required. Stack capture is explicit in both Debug and Release. See the
[member contracts](../api-docs/error-context.md) and
[design rationale](proposals/error-propagation-and-context.md#bounded-recommendation--2026-10-23).
Native default interface bodies and structural-union generic constraint satisfaction
remain separate gaps; this library slice does not change propagation or Runtime
Contract configuration. Other platform errors remain migration work.
