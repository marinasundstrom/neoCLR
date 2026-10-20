# Foundational culture formatting — 2026-10-20

Author direction: make globalization and formatting one foundation, add equivalents
of DateTimeFormatInfo/NumberFormatInfo and enumerate optional calendars. Cover the
existing invariant, Swedish and Hebrew-Israel profiles, and add UK/US English.

The existing immutable `DateTimeFormat` is the date/time information and formatting
policy; `NumberFormat` is its numeric counterpart. `Culture.DateTimeFormat` and
`Culture.NumberFormat` select them explicitly. This avoids a second date/time policy
layer. No ambient mutable culture, new runtime service, ABI or compiler feature is
introduced. The documentation bridge mirrors ordinary library members.

| Culture property | Name | Gregorian date | Decimal | Grouping |
| --- | --- | --- | --- | --- |
| Invariant | empty | yyyy-MM-dd | . | comma |
| SwedishSweden | sv-SE | yyyy-MM-dd | , | U+00A0 |
| HebrewIsrael | he-IL | dd.MM.yyyy | . | comma |
| EnglishUnitedKingdom | en-GB | dd/MM/yyyy | . | comma |
| EnglishUnitedStates | en-US | M/d/yyyy | . | comma |

All profiles report grouping `[3]` and default numeric precision 2. Signs remain
ASCII plus and hyphen-minus; localized signs/bidi markers,
currency, percent, NaN/infinity data and primitive numeric specifiers remain gaps.
Numeric metadata does not imply the existing invariant primitive parsers accept
localized text. Duration g/G and default time fractions consume the shared decimal
policy; invariant duration c and date/time O stay invariant.

DateTimeFormat exposes date/time/fraction separators, the date-time separator and
the supported short/long time patterns: US `h:mm tt` / `h:mm:ss tt`, otherwise
`HH:mm` / `HH:mm:ss`. US exposes AM/PM designators; 24-hour profiles return empty strings. These describe `t` and
`T`; they are not accepted as custom format strings. US uses twelve-hour time, including midnight/noon handling. Other profiles use
24-hour time. Default time output retains seven fractional digits. Complete localized
pattern expansion remains a gap, not a claim of full .NET/CLDR equivalence.

`OptionalCalendars` returns Gregorian then Hebrew for every profile, matching the
existing independent culture/calendar policy. Gregorian is the default. Hebrew
projection uses numeric fields except he-IL, which retains Hebrew numerals/month
names. Enumeration is not an OS locale calendar inventory or a recommendation of
regional usage. Returned calendar/group-size arrays are caller-owned snapshots;
mutation cannot change a policy. Scalar strings reuse literals. Policy creation
allocates, so callers can retain policies. Duration's internal separator lookup does
not construct a NumberFormat merely to retrieve one string.

The preference resolver accepts en-GB/en-US case-insensitively, including underscore
and encoding/modifier forms. Unsupported/neutral English retains invariant fallback;
it does not silently select the US. Existing sv/he/iw resolution is unchanged.

## Comparison and alternatives

[.NET DateTimeFormatInfo](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.datetimeformatinfo?view=net-10.0)
and [NumberFormatInfo](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.numberformatinfo?view=net-10.0)
organize formatting data under a culture. neoCLR follows that structure with immutable
bounded policies, retaining its established DateTimeFormat name. Unlike .NET it has
no writable clone/user overrides or general IFormatProvider yet. Immutability avoids
shared policy mutation, at the cost of no custom conventions until a validated
construction API exists. Unsupported functionality remains tracked as gaps.

[.NET OptionalCalendars](https://learn.microsoft.com/en-us/dotnet/api/system.globalization.cultureinfo.optionalcalendars?view=net-10.0)
reports selectable calendars. neoCLR reports its already supported explicit choices
rather than importing an OS-dependent locale restriction. This preserves independent
calendar selection but differs from platform-specific .NET calendar inventories.

## Validation

The consumer checks all five profiles, actual date and duration formatting, invariant
roundtrip forms, language identity, preference resolution/fallback, calendar projection,
and mutation isolation for returned arrays. The JSON evidence records execution in
interpreter/JIT/AOT with binary hashes. API snapshots are generated from the matching
library/reference builds. This is a bounded foundation, not full locale conformance.

The author supplied a screenshot emphasizing UK/US clock conventions. The implementation
therefore includes US twelve-hour time rather than leaving it as a follow-up. The
[.NET standard time formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-date-and-time-format-strings)
provide the t/T and AM/PM comparison; invariant O and s remain 24-hour. Culture
selection never selects a time zone.

Qualification completed: the new consumer and the existing Formattable/composite
formatting consumer both pass in interpreter, JIT and AOT (six executions). Four
native snapshot tests pass, and all 261 public declarations map to native API pages.
No compiler changes or measured throughput claim are involved.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/culture-formats/CultureFormats.rvnproj \
  --bundle target/culture-formats-clock-bundle/bundle \
  --aot target/culture-formats-clock-bundle/bundle/bin/neoclr-codegen \
  --output target/culture-formats-clock-app
python3 docs/experiments/culture-formats/check.py \
  target/culture-formats-clock-bundle/bundle target/culture-formats-clock-app \
  --output docs/experiments/culture-formats/validation-20261020.json
```
