# Date and Time core values

Implemented 2026-09-08: separate System.Date and System.Time value-default types,
with validated factories, readonly components and value equality/ordering. This
core establishes representation and construction; parsing, formatting,
arithmetic, durations and timezone mapping remain planned. The
[local clock](local-clock.md) now provides actual system date/time.

## Representation and API

Date represents a proleptic Gregorian calendar date from 0001-01-01 through
9999-12-31. Its private Int32 day number is days since 0001-01-01, inclusive range
0–3652058. Time represents a time of day with private Int64 ticks since midnight,
0–863999999999, at 100 ns per tick. Neither stores a timezone, offset or clock.

| API | Contract |
| --- | --- |
| Date.Create(Int32 year, Int32 month, Int32 day) | Result<Date,InvalidDateError>; validates all components and leap years |
| Date.FromDayNumber(Int32 dayNumber) | Same Result; validates the representable range |
| Date.Year / Month / Day / DayOfYear / DayNumber | Readonly Int32 components; DayOfYear starts at 1 |
| Time.Create(Int32 hour, Int32 minute, Int32 second) | Result<Time,InvalidTimeError>; whole seconds |
| Time.Create(Int32 hour, Int32 minute, Int32 second, Int32 fractionTicks) | Same Result; fractionTicks is 0–9999999 within the second |
| Time.FromTicks(Int64 ticks) | Same Result; validates the time-of-day range |
| Time.Hour / Minute / Second / Millisecond / FractionTicks | Readonly Int32 components; Millisecond truncates smaller fractions |
| Time.Ticks | Readonly Int64 total ticks since midnight |
| Equals(T other) / CompareTo(T other) | Equatable<T>/Comparable<T>, readonly managed receiver and owned input |

Hours are 0–23, minutes and seconds 0–59. Leap-second input 60 and 24:00 are rejected.
These APIs are culture-independent. Errors currently distinguish invalid date from
invalid time, not each bad component; their ToString returns InvalidDate/InvalidTime.
All factory failures are ordinary Result values, not runtime faults. There are no
public component constructors, setters or references to internal storage.

`default(System.Date)` and initobj yield 0001-01-01. `default(System.Time)` yields
midnight. These are real values, not null or absence. Use Option<Date>/Option<Time>
for optionality. Copying and default construction use existing runtime mechanisms.

## .NET baseline and decisions

Primary sources consulted 2026-09-08:
[DateOnly](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly?view=net-10.0),
[DateOnly.DayNumber](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly.daynumber?view=net-10.0)
and [TimeOnly.Ticks](https://learn.microsoft.com/en-us/dotnet/api/system.timeonly.ticks?view=net-10.0),
plus the [date/time design comparison](date-time-design.md).
Reuse the familiar ranges, Gregorian model and tick precision. Date/Time are the
chosen preview names: shorter primary concepts, with a naming migration cost for
DateOnly/TimeOnly users. Values do not require Object or ValueType ancestry.

Storing separate year/month/day fields would use more components and require more
cross-field invariant checks. A day number gives a valid zero representation and
simple comparison. Tick storage similarly gives a valid zero. Nanoseconds would
also fit in Int64 for a day, but 100 ns reuses .NET's precision and simplifies future
interop; sub-100 ns precision is intentionally unavailable. This choice can be
revisited during preview with explicit data migration, not silent reinterpretation.

Unlike .NET's throwing component constructors, factories return typed Result.
That makes invalid input explicit at the call site, at the cost of unwrapping even
successful construction. New convenience constructors must preserve this policy.
Do not add a DateTime-like combined type merely to implement these separate values.

## Runtime, library and host boundaries

Everything in this slice is ordinary platform IL: no new runtime primitive, service,
opcode or metadata format. Gregorian decomposition uses bounded calendar-cycle
arithmetic; month lookup takes at most 12 iterations. Factory bounds checks occur
before multiplication. No performance superiority is claimed.

Private field access and aggregate construction are already checked by the runtime
and verifier, including indexed field operations. Factories can write the private
representation; external guest code cannot forge it through newobj, stfld or ldflda.
Readonly receiver contracts protect component observation and comparison. Raw native
memory and trusted host imports remain outside these guest invariants: the embedding
API can import a shape-correct Value::Object with invalid private numeric contents.
Hosts accepting untrusted input must use the validated factories. This slice does
not add semantic validation to host object import or promise general reflection writes.

Neo borrows readonly receivers automatically and uses properties (`date.Year`) rather
than get_ calls. Int64 ticks can be passed/returned through typed values, but Neo's
literal/arithmetic subset still focuses on Int32/Double; the component factories
make Time usable without requiring Int64 source literals.

## Run and validate

```sh
cargo run --locked -- run examples/source/date-time.neo
cargo test --locked --test date_time --test field_access
(cd docs/experiments/date-time-dotnet && dotnet run)
```

The sample prints 2024, 23 and InvalidDate, then returns 42. The pinned SDK 10.0.100
probe exercises defaults and endpoints and generates 192 Gregorian month-boundary
cases across leap years and century transitions. To regenerate the fixture:

```sh
(cd docs/experiments/date-time-dotnet && dotnet run -- ../../../tests/fixtures/date-cases.json)
```

Tests cover component roundtrips, last ticks, invalid bounds, value comparison,
source/artifact execution, zero defaults and unchecked attempts to access private
storage. The [local clock](local-clock.md) now supplies actual system date/time.
Parsing, formatting and globalization are deferred; arithmetic and time carry
remain later work as described in the plan.
