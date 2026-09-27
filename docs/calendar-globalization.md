# Calendar and globalization foundation

Development slice, 2026-09-27; not part of Preview 10. The author requested useful
date/time APIs, a calendar concept, invariant culture, a Hebrew comparison calendar,
two named cultures, and separate calendar/formatter implementations.

## Selected contract

`System.Date` remains a compact absolute day number with Gregorian convenience
properties, years 1–9999 and the existing zero default. `Calendar.Gregorian` and
`Calendar.Hebrew` interpret that same day. `CreateDate` converts calendar fields to
`Date`; `GetYear`, `GetMonth` and `GetDay` project it back. A Hebrew-created Date's
own `Year`, `Month`, `Day` remain Gregorian. Equality and ordering compare the day,
not the representation. No culture, calendar object, zone or offset is stored in Date.

The namespace remains `System`: moving these types to the proposed `System.Time`
namespace would conflict with the existing `System.Time` type and require migration.

| Calendar | Id | Supported calendar years | Numbering |
| --- | --- | --- | --- |
| Gregorian | `gregory` | 1–9999 | January=1; proleptic Gregorian |
| Hebrew | `hebrew` | 5344–5999 inclusive | Civil/Tishri=1; Adar=6 in common years; Adar I=6, Adar II=7 in leap years |

Hebrew uses the fixed calculated calendar, not observation or sunset-based religious
day boundaries. Its range contains complete years within .NET 10 HebrewCalendar's
supported range; .NET's partial first year 5343 is deliberately excluded. `MinYear`
and `MaxYear` make the limitation discoverable. Every day of year 5999 is supported.
`GetMonthsInYear`, `GetDaysInMonth` and `IsLeapYear` validate their inputs.

`Date.AddDays`, `AddMonths` and `AddYears` use Gregorian rules. The corresponding
Calendar methods use the selected calendar and reject input/output days outside its
range. Days advance the absolute day count. Months clamp the day to the target
month's last day. Years preserve the **ordinal month number**, clamp missing month
13 to 12, then clamp the day, matching .NET HebrewCalendar. Consequently Hebrew
month names can change when moving between leap/common years; this is not a
religious-anniversary resolver. Arithmetic is not invertible after clamping. Extreme
Int32 offsets return errors rather than overflowing or starting unbounded loops.

All invalid dates, calendar components, projection ranges and arithmetic outcomes
return the existing `InvalidDateError` through `Result`. This is deliberately a small
error contract; it does not yet distinguish individual invalid fields. No exceptions,
new native clock services or timezone database are required.

`LocalDateTime.Create(date, time)` combines already valid values. It neither selects
an offset nor converts to an instant. Existing host-local Instant conversion is unchanged.

## Cultures and formatting

`System.Globalization.Culture` exposes immutable `Invariant`, `SwedishSweden`
(`sv-SE`) and `HebrewIsrael` (`he-IL`) selections, with `Name`, `Language`, `DefaultCalendar` and
`DateTimeFormat`. All three default to Gregorian, including Hebrew Israel, matching
the .NET/CLDR distinction between language conventions and calendar choice. Invariant
has the empty name; it is not a real language/region tag. Instance identity is not promised.

`DateTimeFormat.Create(culture, calendar)` and `WithCalendar(calendar)` select the
calendar independently. `WithCalendar` returns a new formatter and preserves the
original. `Culture` and `Calendar` properties expose both selections.

| Formatter | Date | Time | Date/time separator |
| --- | --- | --- | --- |
| Invariant | `yyyy-MM-dd` | `HH:mm:ss.fffffff` | `T` |
| Swedish Sweden | `yyyy-MM-dd` | `HH:mm:ss,fffffff` | space |
| Hebrew Israel | `dd.MM.yyyy` | `HH:mm:ss.fffffff` | space |

These are fixed profiles, not arbitrary .NET or CLDR format strings. Time is 24-hour
with seven fractional ASCII digits. The Israeli Gregorian profile deliberately pads
day/month (the tested .NET culture uses `d.M.yyyy`). HebrewIsrael with Calendar.Hebrew
instead renders Hebrew day/year numerals and month names, including leap Adars,
traditional 15/16 spellings and Unicode geresh/gershayim. The millennium is explicit:
`א׳ תשרי ה׳תשפ״ד`. Invariant with Hebrew renders the same fields as `5784-01-01`.
No bidi controls are inserted; embedding and display direction belong to the UI.
`Date.ToString()` uses Culture.Current; `ToString(culture)` selects explicitly.
These convenience methods use the culture's Gregorian default.

`FormatDate` and `FormatLocalDateTime` return `Result<string,InvalidDateError>` because
a valid Date may lie outside the selected calendar. `FormatTime` returns string.
No method invents an offset or appends `Z`. Invariant formatting with Hebrew selected
is not an ISO Gregorian date: applications must explicitly select Gregorian and define
their own protocol/serialization grammar. Parsing and protocol codecs remain follow-ups.

The executable [consumer](experiments/raven-target/samples/library-globalization.rvn)
compares `2023-09-16` with Hebrew `א׳ תשרי ה׳תשפ״ד`, tests leap-month/year arithmetic,
formats fractional times and verifies unchanged culture defaults.

## Implementation boundary

The public Calendar facade delegates to internal `GregorianCalendar` and
`HebrewCalendar` implementations through a private `CalendarRules` interface.
DateTimeFormat delegates numeric presentation to `InvariantDateTimeFormat`,
`SwedishDateTimeFormat` and `HebrewDateTimeFormat` through `DateTimeFormatRules`.
All are Raven library code, using existing CLI classes/interfaces and dispatch.
The concrete policies remain internal so selecting a culture/calendar does not depend
on construction details or expose an unvalidated extensibility contract. Public
Calendar/DateTimeFormat instances have private constructors and no writable properties.

This separation costs small policy/facade allocations and virtual calls; it keeps
calendar arithmetic independent of formatting and isolates each policy for testing.
There is no performance claim or singleton identity guarantee. Language has Code and
Undetermined (`und`), Swedish (`sv`) and Hebrew (`he`) values. CultureProvider exposes
Current; FixedCultureProvider supports injection. SystemCultureProvider snapshots the
host's preferred locale through a ProcessEnvironment native service backed by
sys-locale 0.3.2. PreferredCultureName preserves the input; Current resolves Swedish,
Hebrew (including legacy `iw`) primary tags or falls back to Invariant. IsFallback
also reports approximation of a different regional tag. FromPreference provides
repeatable resolution without changing OS preferences. Culture.Current reads a fresh
system snapshot; there is no mutable thread/async-local culture.

macOS discovery was executed (en-US, invariant fallback). Windows/Linux adapters are
provided by the dependency but were not exercised here. Preferred language discovery
does not promise separate OS regional overrides, user-customized patterns or a full
BCP 47 resolver. CultureId, richer language/region/script values, number formatting
and collation remain proposals.

### Localization boundary

The author clarified that localization must be independent of culture APIs, with
shared localization interfaces across feature areas. Providers and sources may differ:
JSON and resource files are examples. Culture supplies language and date/number
preferences; it must not own resource loading or require a date-specific localization
interface. This slice supplies built-in formatting data only. Unified localization
interfaces, source precedence, missing-key behavior and resource implementations remain
future work, to be designed together rather than introduced piecemeal here.

## Comparison and evidence

Primary sources checked 2026-09-27:

- [.NET 10 Calendar](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.calendar?view=net-10.0)
  exposes calendar fields and arithmetic as library policy. We retain the separation
  from cultural presentation, but use Date instead of DateTime and Result instead of
  out-of-range exceptions. CLI layout/dispatch needs no new temporal primitive.
- [.NET 10 HebrewCalendar](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.hebrewcalendar?view=net-10.0)
  supplies the executable boundary/month/arithmetic comparison. Our complete-year
  range is narrower, and no era or two-digit-year inference is exposed.
- [.NET InvariantCulture](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.cultureinfo.invariantculture?view=net-10.0)
  supplies the stable explicit-culture precedent. Our fixed profiles are a small
  subset, not a replacement for CultureInfo's full formatting/data contracts.
- [.NET DateOnly/TimeOnly API review #49036](https://github.com/dotnet/runtime/issues/49036)
  records the approved separate civil types; it is design history, not evidence that
  .NET lacks civil values. We keep Date's existing day identity rather than changing
  layout in this slice.
- [Noda Time 3.2 calendars](https://nodatime.org/3.2.x/userguide/calendars) makes
  calendar-bearing values and same-day calendar conversion explicit. That model
  prevents loss of the chosen representation but changes value identity and carries
  more state. We retain Date's ABI and require explicit projection instead.
- The fixed Hebrew rules are the molad/postponement algorithm described in
  [Dershowitz and Reingold, Calendrical Calculations](https://www.cs.tau.ac.il/~nachum/calendar-book/papers/calendar.ps),
  also documented by [Noda Time's Hebrew calculator](https://github.com/nodatime/nodatime/blob/3.2.x/src/NodaTime/Calendars/HebrewScripturalCalculator.cs).
  Our implementation uses civil numbering, no cache, a narrower validated range,
  and an epoch offset to Date.DayNumber. No Noda runtime dependency is introduced.
- [Java 21 Chronology](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/time/chrono/Chronology.html)
  distinguishes the chronology factory from calendar-specific dates. Its extensible
  provider model is useful precedent but unnecessary for two closed policies.
- [CLDR 48 Swedish](https://github.com/unicode-org/cldr/blob/release-48/common/main/sv.xml)
  and [Hebrew](https://github.com/unicode-org/cldr/blob/release-48/common/main/he.xml)
  are comparison data, not a bundled locale database. Our explicitly documented
  numeric subset and padding/precision choices avoid promising complete CLDR behavior.

Validation is focused on calendar/culture consumers, source/IL visibility, independent
.NET fixtures, and matching reference/library snapshots. See
[the comparison probe](experiments/calendar-dotnet/README.md) and
`tests/calendar_globalization.rs`. Full localization, additional calendars, parse
errors, calendar-bearing civil values and anniversary policies remain open.

The [.NET Hebrew numeral implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Globalization/HebrewNumber.cs)
provides comparison for traditional numeral rules. This profile uses Unicode punctuation
and includes the millennium explicitly. [sys-locale](https://docs.rs/sys-locale/0.3.2/sys_locale/)
provides OS preferred-language discovery rather than a bundled culture database.

The focused Rust fixture checks all 656 year starts and every month length; month
endpoints and projected components cover 5770–5789 and both range-edge years.
The executable Raven consumers cover formatting, arithmetic, discovery/fallback,
Hebrew numerals and immutable policy visibility. Run `verify_globalization.py` with
`--bridge`, `--runtime` and `--runner` (the measure_async diagnostic host); its larger
contract fixture needs a one-million-instruction limit, not the CLI's default limit.
