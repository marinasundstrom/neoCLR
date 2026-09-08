# Date and time API direction

Direction recorded 2026-09-08. The [Date/Time core](date-time.md) is now implemented;
the [local system clock](local-clock.md) is also implemented. Parsing, formatting
and globalization are postponed beyond this preview scope.
Start with separate date-only and time-of-day values. Do not require a dummy date
for a time or a dummy midnight/timezone for a date. The purpose is a coherent
foundation, not immediate calendar, timezone and formatting completeness.

## Familiar baseline and proposed structure

Primary .NET sources consulted 2026-09-08:
[DateOnly and TimeOnly](https://learn.microsoft.com/en-us/dotnet/standard/datetime/how-to-use-dateonly-timeonly)
are shipped .NET types, introduced in .NET 6. They already provide the separation
we want. DateOnly represents Gregorian dates in years 1–9999; TimeOnly represents
time within a day. Adopt that separation, comparing ranges and precision before
settling storage. neoCLR's opportunity is to make these the initial library model
and apply its value and Result conventions consistently, rather than claiming the
separation is absent from .NET.

| Concept | Intended meaning | Baseline / provisional naming |
| --- | --- | --- |
| Date | Calendar date without time or timezone | System.Date, compare DateOnly |
| Time | Time of day without date or timezone | System.Time, compare TimeOnly |
| Duration | Signed elapsed amount; may exceed a day | Compare TimeSpan; name unsettled |
| Instant | Position on a timeline | Compare UTC DateTime / DateTimeOffset; name unsettled |
| Local date + time | Date and time without a timezone mapping | Explicit composition; later API |
| Offset / timezone | Fixed displacement / named rule set | Separate concepts; later API |

`Date` and `Time` are now the chosen preview names in the core slice. Retaining
DateOnly/TimeOnly would increase direct API familiarity; shorter names express the
primary concepts without referring to a combined type. Review migration, imports
and readability with samples before choosing. In neoCLR these are ordinary
value-default types; no nominal runtime value/reference category is needed.

[.NET's type-selection guidance](https://learn.microsoft.com/en-us/dotnet/standard/datetime/choosing-between-datetime)
distinguishes dates, offsets, elapsed intervals and timezone rules. A fixed offset
is not a timezone identity. A local date/time can map to zero or two instants during
clock transitions. Keep those outcomes explicit in any later conversion API.

## Responsibility and unresolved decisions

Calendar validation, arithmetic, comparison, parsing and formatting belong in the
library. Neo uses ordinary calls, properties and Option/Result matching. Reading a
clock uses a narrow runtime/host service; a public injectable source remains later work;
reading the current time must not be implicit in value construction. Monotonic
elapsed measurement and civil time are separate services.

The core slice settles names, Gregorian ranges, 100 ns precision, zero defaults
and validated factories. Continue evaluating the remaining questions:

- Gregorian range, day numbering and time resolution. .NET's 100 ns tick precision
  is the implemented choice; revisit nanoseconds only with evidence and migration costs.
- Valid default representations and protection of invariants. Existing initobj,
  copying, direct IL stores and future reflection must not silently manufacture an
  invalid date. Review private fields/validated construction before claiming the
  runtime enforces the full calendar invariant. Immutable bindings alone do not.
- Validated factories returning Result for invalid input, with concrete error
  distinctions. .NET constructors commonly throw; Result is more explicit here but
  changes signatures and caller handling.
- Date arithmetic at month ends and leap years, overflow, and time arithmetic
  across midnight. Do not silently discard a day carry when the caller needs it;
  compare TimeOnly's wrapping and wrapped-day overloads before choosing.
- Parsing and formatting are deferred; decide an exact invariant grammar when resumed. Culture-sensitive parsing, alternate
  calendars, leap-second policy, timezone database distribution and DST resolution
  remain separate work with their own research and tests.

## Projected slices

1. Decide Date/Time names, representations, defaults and construction invariants;
   add small .NET probes for boundaries and arithmetic behavior.
2. Implement separate Date and Time values with validated factories, readonly
   components and Equatable/Comparable (implemented). Demonstrate
   birth dates and daily schedules in Neo; test IL/artifact bypasses and errors.
3. Read the host local date, time and offset in a single snapshot (implemented);
   see [local clock](local-clock.md). This is the preview date/time milestone.
4. Later, add signed durations and deliberate arithmetic/carry contracts, clock
   injection and instant/timezone APIs when a concrete application needs them.
   Parsing, formatting and globalization remain deferred.

Value copying should be small and require no retained heap reference. Parsing
creates text/result values under existing rules; clock and timezone resources need
separate ownership and determinism analysis. No JIT, allocation or performance
improvement is asserted without measurements.
