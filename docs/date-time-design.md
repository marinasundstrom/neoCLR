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

## Injectable clock follow-up (2026-09-13)

The author reaffirmed the existing date/time foundation and wants its environmental
API to be instantiable and mockable. Preserve Date/Time as values. Review a small
clock interface with system and fixed/test implementations, versus a broader provider
that also supplies monotonic timestamps and timers. Names and construction contracts
remain open; the current static clock is not already injectable.

A date-dependent order-workflow test should accept a clock and produce the same result
without reading real wall time. Distinguish wall-clock acquisition, timezone conversion
and elapsed-time measurement. Add timers only when async consumers need them, rather
than requiring the entire provider surface for basic date/time tests.

[.NET TimeProvider](https://learn.microsoft.com/en-us/dotnet/standard/datetime/timeprovider-overview),
consulted 2026-09-13, is the modern comparison baseline. Evaluate the narrower API's
simplicity against future adapter and compatibility costs. This records direction;
it does not implement a clock abstraction or expand globalization scope.

### Clock contract and default implementation proposal — 2026-09-15

The author proposed `Clock` as an interface and `SystemClock` as its implementation,
so application code can accept an abstract time provider and tests can inject a
controlled clock. The author also considered `Clock.Instance`, explicitly questioned
its design, and did not select it for implementation.

The assistant recommends placing a possible `Instance` convenience on `SystemClock`
and selecting it at application setup, while time-dependent functions accept `Clock`.
A static default accessor does not itself prevent dependency injection; the problem
is application logic repeatedly selecting the system clock instead of using its
supplied dependency. Do not introduce a mutable global replacement mechanism.

This resembles Noda Time's narrow IClock/SystemClock approach. Its
[IClock guidance](https://nodatime.org/2.4.x/api/NodaTime.IClock.html) recommends passing
an instance to time-dependent code. Modern .NET provides the broader
[TimeProvider abstraction and System default](https://learn.microsoft.com/en-us/dotnet/standard/datetime/timeprovider-overview),
including timestamps and timers. Both sources were consulted 2026-09-15. A narrow
Clock reduces the initial API and test-double burden; its cost is later composition
with elapsed time and timer APIs. UTC/instant versus local snapshot semantics remain
open and must be resolved before choosing methods. This is a library design proposal;
`Clock.GetLocalNow()` remains the implemented static API.

## NeoCLR Time API v1 proposal — 2026-09-15

The author proposed a more complete, deliberately layered time model. `Instant` is
the fundamental point on a timeline and has no timezone, calendar or locale.
`Duration` is an exact elapsed amount. Civil and presentation concepts are separate:

| Type | Meaning | Primary responsibility |
| --- | --- | --- |
| `Date` | Calendar date | Calendar fields and arithmetic |
| `Time` | Time of day | Clock fields and subsecond precision |
| `LocalDateTime` | Date plus time without a zone | Civil scheduling input |
| `Offset` | Fixed UTC displacement | Unambiguous fixed-offset representation |
| `OffsetDateTime` | Local date/time plus offset | Local representation with an offset |
| `TimeZone` | Named timezone rules | Offset, DST and historical transitions |
| `ZonedDateTime` | Instant plus local projection and zone | Resolved zoned representation |
| `Calendar` | Calendar system | Calendar fields and calendar arithmetic |
| `Period` | Calendar-based amount | Civil arithmetic such as months and days |

The intended relationship is `Instant -> TimeZone -> local projection`, with
`Calendar` applied independently to that projection. A single instant can therefore
be represented in different zones and calendars without conflating timezone rules
with calendar rules. The proposal explicitly rejects introducing a new combined
`DateTime` type merely to reproduce .NET's historical combined model.

`Clock` remains a narrow provider of `Instant`:

```text
interface Clock { Now: Instant }
SystemClock, FixedClock, ManualClock
```

Application code should receive a `Clock`; choosing a system default belongs at the
composition boundary. `FixedClock` and `ManualClock` make deterministic tests possible
without coupling the API to a dependency-injection container. A broader provider for
monotonic time, timers and local snapshots remains a separate decision, consistent
with the existing .NET `TimeProvider` comparison and the narrower Noda Time `IClock`
precedent.

Mapping a `LocalDateTime` through a `TimeZone` must represent daylight-saving gaps and
overlaps explicitly. The proposed result is a domain union such as:

```text
LocalTimeMapping
  = Unique(ZonedDateTime)
  | Ambiguous(ZonedDateTime, ZonedDateTime)
  | Skipped
```

This keeps ordinary timezone transitions out of exception-based control flow. The
exact disambiguation and skipped-time policy remain open. Likewise, `Duration` and
`Period` are intentionally different: adding 24 elapsed hours to an `Instant` is not
assumed to produce the same result as adding one calendar day to a `Date`.

Parsing and timezone lookup should use the existing `Result` convention, for example
`Date.Parse`, `Instant.Parse` and `TimeZone.Find`; `Option<T>` represents absence such
as an optional event end. The proposed public surface keeps recognizable .NET naming
and member ergonomics (`date.Year`, `date.AddDays(3)`, `Duration.FromMinutes(30)`),
while separating concepts that .NET commonly exposes through overlapping types and
policies.

This is a proposal, not a settled API or a claim that the Raven-shaped examples are
currently valid Raven source. It extends the implemented `Date`/`Time` foundation but
does not implement `Instant`, timezone data, alternate calendars, `Duration`,
`Period`, parsing or formatting. Before adopting it, specify exact value ranges,
precision, default and invariant behavior, arithmetic overflow/carry, leap-second
policy, timezone database distribution and DST resolution. Validate metadata and
reflection identity, invalid construction and bypass attempts, deterministic clocks,
calendar boundaries, DST gaps/overlaps, offset conversions and .NET interop. Compare
the layered model with .NET `DateOnly`/`TimeOnly`/`DateTime`/`DateTimeOffset`, Noda
Time's `Instant`/`LocalDateTime`/`ZonedDateTime`, and the current neoCLR Date/Time
contracts before treating the architecture as locked.

## Minimal implementation — 2026-09-15

The first implemented subset is documented in [Instant and Clock](instant-clock.md):
Clock.Now, SystemClock, Raven-authored Instant and Duration, and system-local
Instant.ToLocalDateTime. LocalDateTime now contains Date and Time only. The author
explicitly requested keeping this change minimal and using Raven, not the archived
Neo language. All remaining Time API v1 types above are planned, not implemented.
The broader model remains design direction, not a frozen set of signatures.
