# Globalization API proposal

Proposal recorded 2026-09-16. Globalization is separate from the Time API and from
the later localization/resource API. No culture-sensitive formatting, parsing or
collation behavior is implemented by this proposal.

## Model and boundaries

The preferred model separates culture identity, resolved cultural data and the source
of contextual culture:

```text
CultureId ──resolve──> Culture ──> NumberFormat / DateTimeFormat / Collation
                              └──> DefaultCalendar (from System.Time)
CultureProvider ────────────> current Culture
```

`Culture` is an immutable description of conventions, analogous in role to .NET's
`CultureInfo` but without a clone-and-mutate expectation. `CultureId` is a compact,
standardized language-tag value (`sv`, `sv-Latn-SE`) and does not imply that CLDR,
OS or bundled globalization data has been loaded. `Language`, `Region` and `Script`
are strongly typed identifier values. `CultureResolver` turns an identifier into a
resolved `Culture` and returns `Result<Culture, CultureError>` when data is unavailable
or invalid.

The proposed namespaces are:

```text
System.Globalization: Culture, CultureId, Language, Region, Script,
  CultureResolver, CultureProvider, NumberFormat, DateTimeFormat, Collation
System.Time: Date, Time, Calendar, Instant, LocalDateTime, ZonedDateTime
System.Localization: resources and messages (later proposal)
```

`Calendar` remains a Time-domain semantic type. A culture may select a default calendar,
but dates and times do not carry a culture. Formatting is a projection of a temporal or
numeric value through a culture, not a property of the value itself.

## Context and ergonomics

Applications may inject `CultureProvider` in the same way they inject `Clock`:

```text
interface CultureProvider { Current: Culture }
SystemCultureProvider, FixedCultureProvider
```

The ambient convenience `value.ToString()` may use an execution-context-local current
culture, while `value.ToString(culture)` remains explicit and deterministic. A default
facade must not become a mutable process-global replacement mechanism. `Culture.Invariant`
is useful for stable human-facing output, but protocol and serialization formats must
define their own canonical grammar rather than relying on invariant culture accidentally.

## Formatting and linguistic operations

`NumberFormat` describes separators, signs, decimal precision, percent and optional
currency conventions. `DateTimeFormat` describes date/time patterns, names, separators,
first-day-of-week and the selected calendar. Exact format-string compatibility with .NET
is not selected; format values and explicit format identifiers remain alternatives.

`Collation` makes linguistic comparison visible and separate from ordinal comparison:

```text
Collation.Compare(left, right, options) -> Ordering
Collation.Equals(left, right, options) -> Bool
Collation.GetSortKey(text, options) -> Result<SortKey, CollationError>
```

Existing ordinal String operations remain deterministic and do not silently become
culture-sensitive. Culture-sensitive parsing and formatting should use typed Result
errors (`DateParseError`, `NumberParseError`, `CultureError`); `Option` remains for
absence, not malformed input.

## .NET baseline, alternatives and tradeoffs

The .NET baseline combines culture identity, resolved data, mutable/customizable format
objects and ambient thread/async context through `CultureInfo`, with formatting and
collation APIs distributed across numeric, date/time and globalization types. That is
familiar and comprehensive, but it leaves identity, data resolution and contextual
selection coupled. The proposal adapts the useful behavior while making those boundaries
explicit. The .NET encoding/globalization guidance also demonstrates that Unicode text,
formatting culture and temporal representation are separate concerns.

| Alternative | Benefit | Cost / limitation |
| --- | --- | --- |
| Adopt `CultureInfo`-like objects | Maximum .NET familiarity and broad API precedent | Mutable customization, ambient-state coupling and overloaded identity/data responsibilities |
| Keep only explicit culture arguments | Deterministic and simple runtime behavior | Verbose ordinary application code; no contextual provider model |
| Use immutable `Culture` plus `CultureId`/resolver/provider | Clear identity/data/context split, testability and target-specific data backends | More concepts, resolver/data-version compatibility work and an ambient facade to specify |
| Put localization/resources in Culture | Convenient discovery | Couples presentation conventions to message catalogs; rejected for this proposal |

The current proposal is informed by .NET's `CultureInfo` model and by the need for
execution-context injection, but no claim is made that a new globalization runtime
service is required. Bundled CLDR data, OS facilities and a constrained target may
provide different implementations behind the same library contract.

## Decision, uncertainty and validation

Prefer immutable `Culture`, lightweight `CultureId`, explicit `CultureResolver`, and
injectable `CultureProvider`, with ambient current-culture syntax as an ergonomic
facade. Keep Time and Localization separate. This improves conceptual separation and
test isolation, at the cost of data distribution, versioning, collation and formatting
surface area. The proposal is provisional.

Before implementation, specify BCP 47 parsing/canonicalization, fallback and aliases,
CLDR/OS data provenance and versioning, execution-context propagation, custom immutable
format values, calendar interaction, collation options/sort keys, invariant behavior,
thread safety, allocation limits and missing-capability errors. Validate Swedish and
invariant number/date formatting, language/script/region combinations, culture fallback,
parsing failures, collation versus ordinal comparison, concurrent contexts and a target
with no bundled culture data. Compare results with a pinned .NET toolchain; do not claim
globalization correctness from names or sample output alone.
