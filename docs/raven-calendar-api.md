# Raven date, time and local clock APIs

The target exposes all 24 current public methods/accessors on Date, Time,
LocalDateTime and Clock:

| Type | Surface |
| --- | --- |
| Date | Create, FromDayNumber; DayNumber, Year, DayOfYear, Month, Day; Equals, CompareTo |
| Time | Both Create overloads, FromTicks; Ticks, Hour, Minute, Second, Millisecond, FractionTicks; Equals, CompareTo |
| LocalDateTime | Date, Time, UtcOffsetSeconds |
| Clock | GetLocalNow |

Date/Time factories return `Result<Date,InvalidDateError>` or
`Result<Time,InvalidTimeError>`. Typed matches and `?` propagation work with these
value payloads. Clock returns the existing LocalDateTime snapshot directly; the
internal Capture method is not exposed. The [error-value APIs](raven-error-api.md) also expose constructors and ToString.

The existing [date/time design](date-time.md) defines the contracts and their .NET
comparison. This projection preserves the separate calendar-date, time-of-day and
local-clock concepts, with Result-based validation instead of exception-producing
construction. It does not add globalization, parsing, formatting or timezone-rule APIs.
All values retain their runtime value semantics and private storage. CLI default
initialization maps to initobj rather than constructing through private fields.
Concrete instance calls use the existing readonly managed receiver contracts.

## Trying the examples

Copy [the fixed calendar sample](experiments/raven-target/samples/library-calendar.rvn)
into Main.rvn in a fresh [prepared project](experiments/raven-target/README.md).
**neoCLR: Run saved project** demonstrates leap-day validation, day-number conversion,
invalid input, fractional ticks, component access, comparisons and error propagation.
The saved-project suite checks its exact output.

The [live clock sample](experiments/raven-target/samples/library-clock.rvn) prints
year, month, day, hour, minute, second and UTC offset in seconds. Verify it against
the host clock using:

```sh
python3 docs/experiments/raven-target/verify_clock.py /tmp/PROBE/editor/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

The check captures a time window around execution, validates the returned instant,
and compares the local components/offset with the host timezone. It does not assert
that the clock call returns a predetermined date or prove behavior on other hosts.
`verify_editor.py --calendar` checks factory and snapshot completion. Use fresh
metadata; installed SDK/VSIX assets are unchanged.
