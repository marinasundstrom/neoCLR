---
uid: N:System.Time
---
## Dates, clocks and time zones

Date, TimeOfDay and LocalDateTime represent calendar and local-clock values;
Instant and Duration represent timeline positions and elapsed amounts. Clock and
SystemClock supply time, while calendars, TimeOffset, TimeZone and ZonedDateTime
support the implemented calendar and zone operations.

See [dates and clocks](/features/time/), the [DateTime union](/docs/date-time-union.html)
and [local-time mappings](/docs/time-zone-mappings.html). Local time can be ambiguous
or skipped at a zone transition; inspect the mapping result rather than assuming
there is always one instant.

## Development: civil parsing

Date, TimeOfDay and LocalDateTime now provide `Parse(value)` and
`Parse(value, culture)`, returning `Result<T, DateTimeParseError>`. Use
Culture.Invariant explicitly for portable input. The default overload follows
Culture.Current, matching the existing formatting convention.

Invariant forms are `yyyy-MM-dd`, `HH:mm:ss[.fffffff]` and their combination with
`T`. Fractions may contain one through seven digits. Swedish uses a fractional
comma and a space between date and time; HebrewIsrael uses `dd.MM.yyyy` and a
space. All use Gregorian dates. InvalidFormat and OutOfRange are recoverable cases.
No whitespace trimming, timezone suffixes, extra fractional precision, general
format patterns or Hebrew-calendar parsing are supported in this slice.
