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


## Development: duration and offset parsing

`Duration.Parse(value)` returns `Result<Duration, DateTimeParseError>` for invariant
`[-][d.]hh:mm:ss[.fraction]`, with 1–8 day digits and 1–7 fractional digits.
Clock fields require two digits; hours are 0–23 and minutes/seconds 0–59.
The full signed 64-bit tick range is accepted; overflow returns OutOfRange.
`TimeOffset.Parse(value)` returns `Result<TimeOffset, DateTimeParseError>` for `Z`
or `±HH:mm[:ss]`, using the existing inclusive ±18-hour range and whole seconds.
Negative zero means zero. Both reject whitespace and incomplete/non-ASCII fields.
Unsupported syntax returns InvalidFormat. General parsing patterns and inline zone identifiers remain development gaps; see [the contract](/docs/api/System/Time/Duration/method_Parse.html)
and [offset reference](/docs/api/System/Time/TimeOffset/method_Parse.html).


## Development: explicit-offset instant parsing

`Instant.Parse(value)` returns `Result<Instant, DateTimeParseError>` for
`yyyy-MM-ddTHH:mm:ss[.fraction]` followed by `Z` or `±HH:mm[:ss]`. It preserves
1–7 fractional digits and uses whole-second offsets within ±18 hours. Input must
be complete ASCII with uppercase T/Z; omitted offsets never select the host zone.
Local years are 1–9999. Offset conversion may place UTC outside those civil bounds,
within Instant's wider signed tick domain. The source offset is not retained.
Unsupported syntax returns InvalidFormat; invalid fields/ranges return OutOfRange.
[Instant.Parse reference](/docs/api/System/Time/Instant/method_Parse.html).


## Development: zoned parsing with an explicit zone

`ZonedDateTime.Parse(value, zone)` returns `Result<ZonedDateTime, DateTimeParseError>`.
The input uses Instant.Parse's explicit-offset grammar; resolve a named zone with
TimeZone.Find and supply it separately. The stated offset must match its rule at
the parsed instant. `OffsetMismatch` rejects skipped local times and incorrect offsets;
matching offsets select either overlap candidate. UTC and local years must be
1900–2099. No inline zone identifier or host-zone inference is performed.
[ZonedDateTime.Parse reference](/docs/api/System/Time/ZonedDateTime/method_Parse.html).
