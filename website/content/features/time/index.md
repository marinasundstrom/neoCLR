# Dates, times and time zones

Date and TimeOfDay describe civil fields. Instant identifies a point on the timeline,
Duration an elapsed amount, and Clock supplies Now. Named-zone APIs are provisional.

The development API groups these constructs in `System.Time`: use
`import System.Time.*`. `TimeOfDay` replaces the former `System.Time` struct.
Rebuild consumers against the matching library bundle. Formatting policies remain
in `System.Globalization`.

## Choose a specific type

Prefer Date for a calendar date, TimeOfDay for a time of day, LocalDateTime for civil
fields without a zone, and Instant for a point on the timeline. Use ZonedDateTime
when a value must retain its Instant, Zone, LocalDateTime and Offset.
TimeOffset is a fixed whole-second displacement from UTC; TimeZone contains rules
that can change that displacement over time. No OffsetDateTime type is included.

## Duration value equality (development)

Duration compares its complete signed tick count through typed, interface and Object
calls. Equal values have equal hashes; null and unrelated types are unequal. It can
be used in supported record fields and generic pairs. Hashes are not persistent IDs.
See the [API reference](/docs/api/System/Time/Duration/index.html) and
[validated consumer](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/duration-values).

## Mapping a local time

A local clock reading can occur once, twice or not at all. TimeZone.MapLocal returns
Unique, Ambiguous or Skipped through LocalTimeMapping. Applications choose how to
handle overlap candidates and gaps; the library does not silently choose or shift.

```raven
{{TIME_ZONE_SAMPLE}}
```

For Stockholm on October 27, 2024, 02:30 occurs twice: first at UTC+02:00 and then
at UTC+01:00. Both values retain the same local fields and different instants.
TimeZone.AtInstant converts in the other direction without ambiguity.

[Complete executable example →](../../samples/library-time-zones.rvn)

## Arithmetic and display

TimeOfDay.Add(Duration) wraps modulo one day. LocalDateTime.Add carries into the date,
returning an error outside years 1–9999. Instant.Add advances elapsed ticks and checks
overflow. Adding 24 elapsed hours can produce a different local hour across DST;
civil addition does not apply zone rules.

TimeOfDay.ToString uses the current culture; an explicit culture gives repeatable output.
[Globalization](../globalization/) covers culture, calendar selection and Hebrew/Latin
rendering. Format a zoned value's LocalDateTime through those same formatters.
The DateTime union's generated ToString is diagnostic, not a serialization format.

## When an API accepts either local or zoned values

DateTime is an optional nominal union: `DateTime(LocalDateTime | ZonedDateTime)`.
Use it only when the contract intentionally accepts either form; prefer the specific
civil or zoned type otherwise. Existing values convert directly, and type patterns
extract them without wrapper cases or a .NET-style Kind flag. An uninitialized union
is inactive, rather than an implicitly local date.

```raven
{{DATETIME_UNION_SAMPLE}}
```

## Data and limits

Named conversions bundle IANA **2025b** and support UTC/local years **1900–2099**.
DatabaseVersion exposes the version; updating it requires rebuilding the runtime.
It is a pinned snapshot, not a promise of the latest government rule changes.
Find accepts case-sensitive IANA identifiers and aliases. GetSystem discovers the
host's IANA name without falling back silently to UTC. macOS discovery was exercised;
Windows/Linux adapters remain unverified.

Fixed TimeOffset conversions support civil years 1–9999, offsets up to ±18 hours and
second-level historical offsets. Precision is 100 ns; leap seconds are unsupported.
.NET DateTimeOffset instead restricts offsets to whole minutes and ±14 hours, so
interop needs validation. .NET TimeZoneInfo already handles rule conversions; this API
makes the zero/one/two mapping explicit, following the distinction also used by Noda Time.

Parsing covers Date, TimeOfDay and LocalDateTime numeric culture
formats, Duration's invariant day/clock form, and TimeOffset's signed numeric form
or Z. Instant.Parse accepts an invariant numeric timestamp with an explicit offset
or Z, without consulting the host timezone. Parsers return typed Result errors and
preserve 100 ns precision where relevant. ZonedDateTime.Parse takes the timestamp
and an explicitly resolved zone, validating the stated offset and rejecting gaps or
mismatches. Matching offsets select either DST-overlap candidate. The native runtime
supports pinned-zone lookup, offset queries and database version; native MapLocal
and system-zone discovery remain gaps. Inline zone identifiers, serialization,
richer resolvers, custom rule providers,
broader zone ranges and scheduling remain future work. Resource localization remains separate from culture.

[API reference →](../../docs/namespaces.html)
· [Detailed contracts and evidence →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/time-zones.md)

## Standard formatting (development)

Date, TimeOfDay, LocalDateTime and Duration implement
[Formattable](/docs/api/System/Formattable/index.html) with an explicit Culture.
`String.Format(culture, format, ...)` forwards that culture. Default composite calls
resolve one current-culture snapshot when a formattable value is encountered.

| Type | Formats |
| --- | --- |
| Duration | c (also empty/t/T): invariant constant; g: compact; G: days and seven fraction digits |
| Date | empty/G/d: current numeric convention; O/o: invariant Gregorian |
| TimeOfDay | empty/G: culture with seven fraction digits; O/o: invariant; t: hours/minutes; T: hours/minutes/seconds |
| LocalDateTime | empty/G: culture; O/o: invariant roundtrip; s: invariant sortable seconds |

Duration g/G uses a comma for Swedish. Constant formatting supports every signed
64-bit tick count. LocalDateTime never adds a timezone or offset. These are bounded
formats over the current culture/calendar policies; custom patterns, general format
providers and full .NET locale patterns are not implemented. Unsupported specifiers
fault. Existing numeric civil G/d conventions retain their precision and do not claim
.NET's complete culture-pattern behavior.

Development Date, TimeOfDay, Instant, TimeOffset and LocalDateTime now provide
matching typed/Object equality and value hashes. LocalDateTime also implements
EquatableTo and ComparableTo: it compares civil date first, then every tick of the
time of day, without interpreting a timezone. These contracts apply to boxed values
and supported record components as well. Hashes are execution-local, not persistent
identifiers. Ordinary struct default value semantics remain a separate runtime gap.

Development `TimeZone` values compare by their case-sensitive IANA identifier within the fixed bundled rule database; aliases stay distinct. `ZonedDateTime` compares the same instant in the same named zone, with matching hashes through typed, Object and equality-interface calls. Compare its `Instant` when zone identity should be ignored. Zone display returns the identifier; zoned display remains diagnostic type text. See the [API reference](/docs/api/system/time/TimeZone/) and [qualified cases](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/zoned-values/README.md).

## Development: culture formatting foundation

`Culture` now exposes immutable `DateTimeFormat` and `NumberFormat` policies and
`OptionalCalendars`. Invariant, Swedish (`sv-SE`), Hebrew-Israel (`he-IL`), UK English
(`en-GB`) and US English (`en-US`) have bounded built-in profiles. UK and US dates
use day/month/year and month/day/year respectively. UK time uses a 24-hour clock;
US time uses a twelve-hour clock with AM/PM, including midnight and noon. The time policy exposes its
separators and supported short/long patterns; duration and time fractions share
the numeric decimal convention.

Both Gregorian and Hebrew calendars can be selected independently of language.
Returned calendar and grouping arrays are independent snapshots. These are foundational
APIs: full locale patterns, custom providers, currency
formatting and primitive numeric format specifiers remain gaps. See the API references
for [Culture](/docs/api/system/globalization/Culture/),
[DateTimeFormat](/docs/api/system/globalization/DateTimeFormat/) and
[NumberFormat](/docs/api/system/globalization/NumberFormat/).
