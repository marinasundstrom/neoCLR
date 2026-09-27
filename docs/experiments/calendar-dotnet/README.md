# Calendar comparison

Pinned .NET 10.0.100 probe, run 2026-09-27 on macOS arm64. Reproduce from this folder:

```sh
dotnet run --project calendar.csproj -- ../../../tests/fixtures/hebrew-calendar.json
```

The fixture contains all 656 complete Hebrew years 5344–5999. Each row is
`[year, startDayNumber, daysInMonth1, ...]`; numbers are independently computed
by .NET HebrewCalendar. neoCLR's Rust test compares every year start and month length, with both month endpoints and
component projection for years 5770–5789 and the two range-edge years. The probe also checks month/year clamping and prints the
host .NET cultures' Gregorian default and numeric conventions.

The bounded Hebrew implementation excludes .NET's partial first Hebrew year 5343.
This is calendar comparison evidence, not timezone, sunset, localization or speed evidence.
