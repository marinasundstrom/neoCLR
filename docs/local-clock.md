# Local system clock

Implemented 2026-09-08. `System.Clock.GetLocalNow()` reads the host clock and returns
an owned `System.LocalDateTime` snapshot with readonly properties:

| Property | Type | Meaning |
| --- | --- | --- |
| Date | System.Date | Local Gregorian calendar date |
| Time | System.Time | Local time of day, in 100 ns ticks |
| UtcOffsetSeconds | Int32 | Local time minus UTC, in seconds at capture |

```swift
let now = System.Clock.GetLocalNow()
let date = now.Date
let time = now.Time
System.Console.WriteLine(date.Year)
System.Console.WriteLine(time.Hour)
```

Date, time and offset come from one sampled instant, so a midnight boundary cannot
mix two readings. Date and Time retain their separate value semantics. The snapshot
has private storage and no public constructor. Its zero default is the minimum date,
midnight and zero offset; only GetLocalNow reads the clock.

This is wall-clock time and can jump when the system clock changes. Tick storage
precision does not promise clock accuracy or resolution; sub-100 ns fractions are
truncated. The captured offset is not a timezone identifier or a rule for future
appointments. Parsing, formatting, globalization, elapsed-time measurement, clock
injection and general timezone conversion are deferred.

## .NET comparison and implementation boundary

.NET's shipped [TimeProvider.GetLocalNow](https://learn.microsoft.com/en-us/dotnet/api/system.timeprovider.getlocalnow?view=net-10.0)
returns DateTimeOffset using the provider's UTC reading and local timezone offset.
NeoCLR preserves the single local reading with its offset, but uses a static Clock
and a composition of Date and Time. This keeps the preview small and exposes the
components directly; the cost is no interchangeable clock provider, duration type,
or DateTimeOffset arithmetic yet. The naming and snapshot contract are provisional.

The runtime's `LocalClock` InternalCall supplies eight Int32 components in an owned
array. Ordinary library IL validates and constructs Date, Time and LocalDateTime.
Reachability reports LocalClock and ManagedArrays for the helper; no opcode or
metadata extension is needed. The host uses [Chrono 0.4.45 Local::now](https://docs.rs/chrono/0.4.45/chrono/struct.Local.html)
with the clock feature and default features disabled, pinned through Cargo.lock.
This delegates platform clock/offset handling instead of maintaining native bindings
for each operating system. It adds a dependency and its platform support costs.

On Unix, the backend consults TZ/system timezone configuration and caches timezone
information. Its [timezone resolution](https://docs.rs/chrono/0.4.45/src/chrono/offset/local/unix.rs.html)
ultimately falls back to UTC if local configuration cannot be resolved; this preview
cannot separately report that fallback. Immediate detection of configuration changes
is not promised. Other supported hosts use the backend's platform implementation;
platform-specific validation beyond the current macOS run remains necessary.
Unsupported calendar years or leap-second representations fault instead of producing
an invalid Date/Time value.

## Run and validate

```sh
cargo run --locked -- run examples/source/local-clock.neo
cargo test --locked --test local_clock
```

The sample prints labeled numeric date/time components and the UTC offset, without
adding formatting APIs. Tests check a real host reading against its UTC interval,
Neo artifact execution, isolated Unix TZ configurations, and deterministic conversion
across midnight with sub-tick truncation in runtime unit tests.
