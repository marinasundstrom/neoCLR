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
