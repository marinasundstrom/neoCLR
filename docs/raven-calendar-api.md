# Raven date, time and calendar APIs

The development [calendar/globalization contract](calendar-globalization.md) extends
Preview 10 with Date arithmetic and display, LocalDateTime.Create, Calendar and
minimal Culture/Language/DateTimeFormat/provider contracts. These are provisional;
use matching compiler reference and generated runtime artifacts.

Date remains an absolute Gregorian day number. Gregorian and Hebrew calendar
policies convert between fields and that day, so conversion works in both directions
without formatting/parsing strings. Hebrew supports complete years 5344–5999.
Invalid construction, projection and arithmetic return Result with InvalidDateError.

[The tested formatting sample](experiments/raven-target/samples/library-date-formatting.rvn)
renders the same day as Gregorian, Hebrew alphabet and invariant Hebrew fields.
[The contract consumer](experiments/raven-target/samples/library-globalization.rvn)
also exercises arithmetic, provider discovery, fallback and Hebrew leap-month rules.
Run verify_globalization.py as described in the calendar guide.

Time retains validated tick-based time-of-day construction. LocalDateTime combines
Date and Time without an offset. Instant/Duration and injectable Clock/SystemClock
remain separate; see [date/time design](date-time-design.md) and
[the instant sample](experiments/raven-target/samples/library-instants.rvn).
The older GetLocalNow/UtcOffsetSeconds projection is historical, not the current API.

Localization is independent of culture: future unified interfaces can use different
providers and JSON/resource sources. No resource-loading implementation is included.
