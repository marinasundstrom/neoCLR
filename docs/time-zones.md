# TimeOfDay, offsets and named zones

Prefer specific types: Date, TimeOfDay, LocalDateTime, Instant and ZonedDateTime express
the information an API needs. DateTime is an optional union for contracts that
intentionally accept either a local or a zoned value.

## Contracts

| Type | Meaning and implemented operations |
| --- | --- |
| TimeOfDay | Validated 100 ns ticks within one day; Add(Duration) wraps modulo a day; ToString() uses current culture and ToString(Culture) is explicit |
| LocalDateTime | Civil Date plus TimeOfDay; Add(Duration) carries the date and returns InvalidDateError outside years 1–9999; it does not apply zone rules |
| Instant | Signed Unix 100 ns ticks; Add(Duration) returns OverflowError on Int64 overflow |
| TimeOffset | Fixed whole-second UTC displacement from -18h through +18h; Zero and zero default; FromSeconds returns InvalidTimeError for invalid offsets; Seconds, Equals and CompareTo |
| ZonedDateTime | Immutable reference value retaining Instant, Zone, LocalDateTime and Offset; Create(instant,zone) validates the projection; no custom equality or singleton identity contract |
| DateTime | Nominal union of existing LocalDateTime and ZonedDateTime types; no implicit choice of a zone |
| TimeZone | Immutable named IANA rule selection; Find(id), Utc, GetSystem(), Id, DatabaseVersion, MinYear, MaxYear, GetUtcOffset, AtInstant and MapLocal |

TimeOffset.ToInstant(local) is unambiguous and fits Instant's range for every valid
civil input. TimeOffset.AtInstant returns Result<LocalDateTime,InvalidDateError> for
civil years 1–9999. Historical offsets can include seconds; this deliberately differs
from .NET DateTimeOffset's minute-granularity offsets and ±14h limit. The wider offset
contract costs interop validation; it does not imply support for arbitrary zones.
No OffsetDateTime type is introduced in this slice.

TimeZone.AtInstant and ZonedDateTime.Create return Result<ZonedDateTime,TimeZoneError>.
The projection is Gregorian Date/Time; the separate Calendar and culture formatter
can render Hebrew or another supported representation of that same local day.

MapLocal returns Result<LocalTimeMapping,TimeZoneError>:

- Unique(value): one ZonedDateTime.
- Ambiguous(earlier,later): two ZonedDateTime values in increasing Instant order.
- Skipped: no such local time, including non-hour gaps and skipped civil dates.

Mapping never chooses an overlap candidate or shifts a gap. Applications decide.
OutOfRange is distinct from Skipped. Error cases are UnknownZone, OutOfRange and
SystemZoneUnavailable. Unknown/absent system zones do not silently become UTC.
TimeZone.GetSystem reads the OS IANA name; each returned zone is a snapshot selection.
Culture and region do not determine a timezone.

## Optional local-or-zoned contract

```raven
public union DateTime(LocalDateTime | ZonedDateTime)
```

This supersedes the earlier proposal's decision not to introduce DateTime. It does
not recreate .NET DateTime.Kind. Existing values convert directly into the nominal
union; type patterns extract LocalDateTime or ZonedDateTime without wrapper cases.
The standard union default is inactive (HasValue=false, Value=null); it is not midnight
or an implicitly zoned value. Compiler-generated ToString is diagnostic, not a date
formatter. Match an active alternative and use the culture formatter on its civil value.

## Rules, precision and distribution

The host pins chrono-tz **0.10.4**, containing IANA **2025b**, and iana-time-zone
**0.1.65** for system discovery. DatabaseVersion reports `2025b`; this is a reproducible
snapshot, not a claim to ship the newest political rules. Updating the dependency and
lockfile, rebuilding the runtime and rerunning transition tests updates the data.
There is no hot reload, network fetch, OS TZif parser or Windows-ID translation API.
Find is case-sensitive and accepts the bundled IANA identifiers and aliases; Id
preserves the requested spelling. UTC uses the same named-zone conversion range.

Both UTC and projected local years must be **1900–2099** for named zones. The generated
Stockholm table, for example, ends with 2099 transitions; continuing its last offset
indefinitely would give wrong future summers. The bounded range is deliberate and
narrower than the fixed-offset civil range. It does not certify historical accuracy
for every region before 1970, nor future government decisions. Leap seconds are not
represented. Negative Unix ticks use Euclidean division, retaining the final 100 ns
before the epoch correctly.

The new TimeZoneRules service covers pure bundled-rule queries. Mapping additionally
requires ManagedArrays for its bounded native result array. SystemTimeZoneName uses
ProcessEnvironment. Neither conversion reads the wall clock. The existing
Instant.ToLocalDateTime host-local convenience remains unchanged; it uses host rules
and can differ from an explicit pinned TimeZone. Prefer the explicit API for repeatability.

The host returns validated numeric rule results. Raven owns public factories, typed
errors, mapping unions and resolved objects. No new runtime temporal primitive or
Date/Time ABI change is introduced. Named mapping allocates short arrays and one or
two resolved objects; parenthesized DateTime carries either a civil value or an object
reference. This is not a performance improvement claim or a general union-layout policy.

## Comparisons and choices

Primary sources checked 2026-09-27:

- [.NET 10 TimeZoneInfo.ConvertTimeToUtc](https://learn.microsoft.com/en-us/dotnet/api/system.timezoneinfo.converttimetoutc?view=net-10.0)
  already provides named rule conversion. Its convenience conversion throws for invalid
  local times and selects standard time for an overlap. neoCLR exposes the mapping
  first; callers must write a policy, at the cost of more explicit branching.
- [.NET DateTimeOffset constructor](https://learn.microsoft.com/en-us/dotnet/api/system.datetimeoffset.-ctor?view=net-10.0)
  establishes the narrower offset baseline. Offset and zone identity are different
  concepts; retaining a zone makes future rule-based operations possible.
- [Noda Time 3.2 ZoneLocalMapping](https://nodatime.org/3.2.x/api/NodaTime.TimeZones.ZoneLocalMapping.html)
  models zero, one and two matches. We adopt that distinction through Raven's standard
  union syntax; richer resolvers and zone intervals remain absent.
- [Java 21 ZonedDateTime](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/time/ZonedDateTime.html)
  is a precedent for retaining zone plus instant. Its normal local resolution adjusts
  gaps and uses an overlap policy; this slice requires the caller to choose explicitly.
- [.NET documentation issue 32773](https://github.com/dotnet/docs/issues/32773)
  reports that resolving an overlap using BaseUtcOffset can miss historical base-offset
  changes. This motivates returning actual candidate offsets rather than assuming the
  current base offset; it is a reported guidance problem, not evidence that all .NET
  timezone conversion is wrong.
- [.NET runtime issue 126940](https://github.com/dotnet/runtime/issues/126940)
  reports a .NET 10/11 boundary-conversion difference near DateTime.MinValue and links
  the fix. It motivates explicit boundary tests; neoCLR chooses errors, not saturation.
- [chrono-tz 0.10.4](https://docs.rs/chrono-tz/0.10.4/chrono_tz/) and its pinned generated
  tables supply host rules; the existing Chrono dependency handles civil timestamps.
  Compared with OS data, bundling improves cross-host repeatability but adds binary
  data and an explicit update responsibility. Dynamic/provider-based rule loading is
  a future design decision, not an implemented extension point.
- Raven's [parenthesized union contract](https://github.com/marinasundstrom/raven/blob/main/docs/lang/spec/dotnet-implementation.md)
  uses public variant constructors and typed extraction. The bridge projects compiled
  source, not a handwritten DateTime carrier. The exact selected parenthesized shape
  is validated separately from nested-case unions.

## Validation and open scope

The [Raven example](experiments/raven-target/samples/library-time-zones.rvn) shows the
Stockholm overlap and accepts a DateTime union. The [contract consumer](experiments/raven-target/samples/library-time-contracts.rvn)
checks both variants, array retention, inactive default, fixed-offset round trips,
100 ns precision, arithmetic overflow/carry, Israeli projection, gaps, overlaps,
explicit errors and system discovery. The larger consumer uses the diagnostic
runner with a one-million-instruction limit and a small GC budget.

Native tests cover Stockholm/New York overlaps, Lord Howe's half-hour change,
Apia's skipped date, Kolkata's half-hour offset, Paris's historical seconds and
range extremes. [The .NET 10 probe](experiments/time-zones-dotnet/README.md) compares
selected modern mappings. Signature/source/guest-IL checks reject invalid signatures,
private construction and private storage writes. macOS execution does not validate
Windows/Linux system-zone discovery. API/library snapshots must match the bridge.

Period arithmetic, duration convenience factories, parsing/serialization, zone-rule
providers, custom zones, broader ranges, gap/overlap resolver helpers and scheduling
remain open. Resource localization remains independent; no date/time-specific resource
interface or loader is introduced.

A focused RavenDoc render covers the six new types. The DateTime heading omits its
parenthesized variants; the API reference links an exact declaration supplement.
LocalTimeMapping payload pages use the existing manual-member mechanism. Website
sample extraction was checked without running the full site build.

## Native source bootstrap (development, 2026-10-06)

The calendar group now compiles into a separate native library, and an artifact-only
consumer executes named-zone offsets, gaps/overlaps and the optional local-or-zoned
union. The internal mapping service returns managed Int64 arrays. See the
[gate and remaining full-System blockers](experiments/extended-cli-metadata/source-calendar-2026-10-06.md).
