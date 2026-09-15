# Initial instant and clock API

Implemented for the Raven target on 2026-09-15. This is an initial layer, not a
complete calendar or scheduling API.

```raven
import System.*

let clock: Clock = SystemClock()
let instant = clock.Now
let local = instant.ToLocalDateTime()
let duration = Duration.FromTicks(ticks: 10000000L)
```

`Clock` is an interface with a read-only `Now: Instant` property. Applications can
receive a Clock as a dependency and tests can implement it to return a fixed instant.
`SystemClock` is a stateless class with a public parameterless constructor. It reads
the host wall clock. It does not promise monotonic readings: system adjustments can
move the clock backwards. There is no default provider attached to the interface.

`Instant` and `Duration` are Raven-authored value types with private storage, equality
and comparison. An Instant stores signed 100-nanosecond ticks since the Unix epoch
(1970-01-01T00:00:00Z); a Duration stores a signed count of the same ticks without an
epoch. All Int64 tick values are valid. The factories are `Instant.FromUnixTimeTicks`
and `Duration.FromTicks`; properties are `UnixTimeTicks` and `Ticks`. Zero-initialized
values mean the Unix epoch and zero duration respectively. Tick precision is a
representation choice, not a guarantee of clock resolution. Sub-tick clock readings
are rounded down, including before the epoch. Leap seconds are not modeled.

`Instant.ToLocalDateTime()` converts that specific instant using the host system's
local time-zone rules and the existing Gregorian Date/Time representation. It returns
Date and Time together, without retaining an offset or zone. It does not read the current clock again.
The existing calendar supports years 1–9999; conversion outside that range faults.
A future calendar API should make such range failures explicit recoverable results.
Formatting, selectable calendars and zones, duration arithmetic, timers, and monotonic
elapsed-time measurement are deferred. This limited system-zone conversion is for
the demo; its result is intentionally environment-dependent.

The former `Clock.GetLocalNow()` convenience and `LocalDateTime.UtcOffsetSeconds`
are removed. Use the injectable clock and explicit conversion. LocalDateTime
is a value carrying only local date and time. There is no attempt
to infer a unique instant back from an ambiguous local date and time.

## Comparison and implementation boundary

.NET's [TimeProvider](https://learn.microsoft.com/en-us/dotnet/standard/datetime/timeprovider-overview)
combines wall-clock access, local-zone information, monotonic timestamps and timers.
Its abstraction already supports deterministic tests. neoCLR starts with a narrower
interface rather than reproducing that entire surface. The split into Instant,
Duration and a replaceable clock follows the distinction demonstrated by
[Noda Time's IClock](https://www.nodatime.org/3.3.x/api/NodaTime.IClock.html) and
[core concepts](https://www.nodatime.org/3.3.x/userguide/concepts).
Sources reviewed 2026-09-15; these are existing APIs, not proposals.

The benefit is a small injectable dependency and an explicit distinction between a
point on the timeline and its local calendar rendering. Costs include new API names,
a deliberately limited tick/range policy and an environment-dependent conversion.
This is not a claim of better precision or performance than .NET or Noda Time.

No new instruction or CLI metadata form is needed. The experimental reference
catalog exposes ordinary structs, an interface property and a class implementation.
The importer maps those contracts to existing calls and virtual dispatch. Two typed
runtime services read Unix ticks and render supplied ticks into local components;
Instant and Duration logic is Raven-authored, while SystemClock and the existing
LocalDateTime construction boundary remain small NeoIL implementations. Raven's
compiler and .NET runtime contract are unchanged.

Validation: `tests/instant_clock.rs` checks host-clock bounds, interface dispatch,
system-zone rendering and calendar range faults; `src/clock.rs` checks epoch and
sub-tick boundaries. `library-instants.rvn` exercises a Raven-defined fixed Clock,
the system implementation, Duration and named arguments. It is part of the saved
project integration suite. The website's clock excerpt comes from that same sample.
