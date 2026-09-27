# DateTime, time offsets and time zones

Date and Time describe civil fields. Instant identifies a point on the timeline,
Duration an elapsed amount, and Clock supplies Now. **The time-zone and DateTime union
APIs below are provisional development additions beyond Preview 10.**

## Local or zoned

DateTime is a nominal parenthesized union: `DateTime(LocalDateTime | ZonedDateTime)`.
An existing local or zoned value converts directly into it. Match the type to decide
what information is available; there are no extra wrapper cases or .NET-style Kind flag.
An uninitialized union is inactive, rather than an implicitly local date.

```raven
{{DATETIME_UNION_SAMPLE}}
```

LocalDateTime has no zone. ZonedDateTime retains Instant, Zone, LocalDateTime and
Offset. TimeOffset is a fixed whole-second displacement from UTC; TimeZone contains
rules that can change that displacement over time. No OffsetDateTime type is included.

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

Time.Add(Duration) wraps modulo one day. LocalDateTime.Add carries into the date,
returning an error outside years 1–9999. Instant.Add advances elapsed ticks and checks
overflow. Adding 24 elapsed hours can produce a different local hour across DST;
civil addition does not apply zone rules.

Time.ToString uses the current culture; an explicit culture gives repeatable output.
[Globalization](../globalization/) covers culture, calendar selection and Hebrew/Latin
rendering. Format a zoned value's LocalDateTime through those same formatters.
The DateTime union's generated ToString is diagnostic, not a serialization format.

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

Parsing, serialization, richer resolvers, custom rule providers, broader zone ranges
and scheduling remain future work. Resource localization remains separate from culture.

[API reference →](../../docs/namespaces.html)
· [Detailed contracts and evidence →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/time-zones.md)
