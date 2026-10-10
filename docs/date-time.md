# Date and Time core values

The core was implemented 2026-09-08; the 2026-10-09 development migration names
the separate values `System.Time.Date` and `System.Time.TimeOfDay`,
with validated factories, readonly components and value equality/ordering. This
core established representation and construction. Subsequent development adds
[formatting and calendars](calendar-globalization.md), civil arithmetic,
[timezone mapping](time-zones.md), and the bounded parsing described below. The
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
| TimeOfDay.Create(Int32 hour, Int32 minute, Int32 second) | Result<TimeOfDay,InvalidTimeError>; whole seconds |
| TimeOfDay.Create(Int32 hour, Int32 minute, Int32 second, Int32 fractionTicks) | Same Result; fractionTicks is 0–9999999 within the second |
| TimeOfDay.FromTicks(Int64 ticks) | Same Result; validates the time-of-day range |
| TimeOfDay.Hour / Minute / Second / Millisecond / FractionTicks | Readonly Int32 components; Millisecond truncates smaller fractions |
| TimeOfDay.Ticks | Readonly Int64 total ticks since midnight |
| Equals(T other) / CompareTo(T other) | EquatableTo<T>/ComparableTo<T>, readonly managed receiver and owned input |

Hours are 0–23, minutes and seconds 0–59. Leap-second input 60 and 24:00 are rejected.
These APIs are culture-independent. Errors currently distinguish invalid date from
invalid time, not each bad component; their ToString returns InvalidDate/InvalidTime.
All factory failures are ordinary Result values, not runtime faults. There are no
public component constructors, setters or references to internal storage.

`default(System.Time.Date)` and initobj yield 0001-01-01. `default(System.Time.TimeOfDay)` yields
midnight. These are real values, not null or absence. Use Option<Date>/Option<TimeOfDay>
for optionality. Copying and default construction use existing runtime mechanisms.

## .NET baseline and decisions

Primary sources consulted 2026-09-08:
[DateOnly](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly?view=net-10.0),
[DateOnly.DayNumber](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly.daynumber?view=net-10.0)
and [TimeOnly.Ticks](https://learn.microsoft.com/en-us/dotnet/api/system.timeonly.ticks?view=net-10.0),
plus the [date/time design comparison](date-time-design.md).
Reuse the familiar ranges, Gregorian model and tick precision. Date and TimeOfDay retain the separate concepts. TimeOfDay makes the meaning
explicit inside System.Time; DateOnly/TimeOnly remain the .NET comparison. The
namespace/type rename requires updating imports and rebuilding consumers. Values do not require Object or ValueType ancestry.

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

## Development parsing (2026-10-10)

Date, TimeOfDay and LocalDateTime expose `Parse(value: string)` and
`Parse(value: string, culture: Culture)`, returning `Result<T, DateTimeParseError>`.
The first overload snapshots Culture.Current; explicit Culture.Invariant is the
portable choice for stored data. Neither overload changes host culture or infers a
zone/offset. The default calendar is Gregorian for all currently supported cultures.

| Culture | Date | Time | Local date/time separator |
| --- | --- | --- | --- |
| Invariant | yyyy-MM-dd | HH:mm:ss[.fffffff] | T |
| SwedishSweden | yyyy-MM-dd | HH:mm:ss[,fffffff] | one space |
| HebrewIsrael | dd.MM.yyyy | HH:mm:ss[.fffffff] | one space |

Brackets indicate an optional fraction, with **one through seven** ASCII digits.
A single fractional digit means tenths of a second; seven preserve 100 ns ticks.
The parser accepts complete ASCII numeric text only, without trimming. It rejects
extra precision rather than rounding, Unicode digits, omitted seconds, date/time
suffixes, leap seconds and 24:00. Factories remain the authority for Gregorian leap
years, years 1–9999 and clock ranges. InvalidFormat identifies grammar failure;
OutOfRange identifies invalid numeric fields. LocalDateTime validates the date
component before the clock component and reports its first failure. Default errors have no active union
case; use the returned case, not default construction.

The UTF-8 implementation bounds input to 10/16/27 bytes before encoding a byte snapshot,
then checks ASCII digits and fixed positions. This avoids culture-dependent host parsing
and keeps interpreted/native rules in the Raven library. It allocates a small byte
snapshot, so this is not an allocation-free parsing claim.

### .NET comparison and provisional scope

Reviewed 2026-10-10: [.NET date/time parsing](https://learn.microsoft.com/en-us/dotnet/standard/base-types/parsing-datetime),
[DateOnly.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.dateonly.parse?view=net-10.0)
and [DateTime.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.datetime.parse?view=net-10.0).
Adopt the familiar Parse overload structure and current/explicit culture distinction.
Reuse neoCLR's established typed Result convention instead of .NET's throwing Parse
or Boolean/out TryParse. The benefit is recoverable, composable errors; the cost is
a different caller signature and a separate error taxonomy. This does not add a
second TryParse spelling for the same Result operation.

.NET accepts a much broader language and offers ParseExact/TryParseExact and style
options. This first implementation mirrors neoCLR's existing numeric formatting
profiles; it is an incomplete parsing implementation, not a claim that strict numeric
forms replace .NET's general contract. Adding general patterns/providers, whitespace
styles or calendar parsing remains future work. Delegating to host parsers would
introduce platform-dependent accepted forms; implementing the bounded grammar in
Raven provides the same explicit-culture behavior on all targets. These preview
choices may evolve before a stable release.

ZonedDateTime parsing remains a separate gap. Timezone conversion remains an explicit
operation after civil parsing; Instant parsing below requires an explicit offset.


### Duration and offset parsing (development, 2026-10-10)

`Duration.Parse(value)` returns `Result<Duration, DateTimeParseError>` for invariant
`[-][d.]hh:mm:ss[.fraction]`. Days have 1–8 ASCII digits (leading zeros accepted),
clock fields exactly two digits, and optional fractions 1–7 digits. Hours are 0–23,
minutes/seconds 0–59. The complete signed Int64 tick range is supported, including
`-10675199.02:48:05.4775808`; a tick beyond either boundary returns OutOfRange.
Input is bounded to 26 UTF-8 bytes before materialization. Negative accumulation
avoids trying to represent the positive magnitude of Int64.MinValue.

`TimeOffset.Parse(value)` returns `Result<TimeOffset, DateTimeParseError>` for `Z`
or `±HH:mm[:ss]`. Minutes/seconds are 0–59 and the existing inclusive ±18-hour
range applies. `-00:00` means zero; the model has no unknown-offset sentinel.
The parser rejects fractions, named zones, incomplete fields and whitespace.
Both parsers use ASCII digits, consume complete input, and report InvalidFormat
for unsupported grammar and OutOfRange for numeric ranges. No host parser or
ambient culture participates. Grammar is bounded; excess day/fraction width is
InvalidFormat even when a shorter equivalent could represent a valid value.

Reviewed against [.NET TimeSpan constant formatting and parsing](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings)
and [DateTimeOffset constructor constraints](https://learn.microsoft.com/en-us/dotnet/api/system.datetimeoffset.-ctor?view=net-10.0).
Duration adopts the constant-format shape and 100 ns resolution; general culture
patterns, whitespace styles and ParseExact remain gaps, not a replacement contract
for .NET's broader Parse surface. Result errors retain neoCLR's existing adaptation.
TimeOffset is a standalone displacement, unlike DateTimeOffset: its pre-existing
second precision and ±18-hour range exceed DateTimeOffset's whole-minute ±14-hour
constraint. Preserving that domain supports existing zone data, at the cost of
requiring range/precision checks when exchanging values with DateTimeOffset.

The next Instant slice implements an explicit numeric offset or Z, as described below. A future
ZonedDateTime parser must additionally define zone identifiers, offset consistency,
gaps and repeated-local-time resolution. These are review directions, not implemented
APIs. A parser must not lose a zone identifier by treating it as a fixed offset.


### Explicit-offset Instant parsing (development, 2026-10-10)

`Instant.Parse(value)` returns `Result<Instant, DateTimeParseError>` for
`yyyy-MM-ddTHH:mm:ss[.fraction]` followed by `Z` or `±HH:mm[:ss]`. It composes the
invariant local parser with TimeOffset.Parse, preserving all 1–7 fractional digits
and the existing offset range/precision. It requires uppercase T/Z, seconds and
complete ASCII input; no culture or host-zone default is read. Local fields are
validated before offset fields and the first failure is returned. Length is bounded
to 20–36 UTF-8 bytes before snapshots/slices are created. This is not allocation-free.

The local year must be 1–9999. Conversion uses existing TimeOffset.ToInstant;
that bounded civil domain plus ±18 hours provably fits signed Unix ticks. An offset
can place UTC just outside civil years 1–9999. This preserves Instant's existing
wider timeline, rather than clamping it to the civil projection domain. Projecting
such an instant at a different offset may return InvalidDateError. The original
text/offset is not retained in Instant, which stores only UnixTimeTicks.

Compare [.NET round-trip timestamp formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-date-and-time-format-strings)
and [DateTimeOffset.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.datetimeoffset.parse?view=net-10.0):
the numeric timestamp shape and 100 ns resolution are familiar. This bounded
implementation requires an offset to avoid environment-dependent interpretation;
.NET's general culture/style surface remains an implementation gap. Existing
whole-second/±18-hour offsets and Instant's wider UTC range deliberately preserve
neoCLR's established type domains, unlike DateTimeOffset's minute/±14-hour and
civil UTC bounds. Interop must validate those constraints. Wider-year textual forms,
general patterns and named-zone parsing remain unimplemented.
