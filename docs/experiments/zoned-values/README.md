# Named zone and zoned value contracts — 2026-10-20

TimeZone and ZonedDateTime now declare EquatableTo for their own types and provide
matching Object equality/hash overrides. Separately constructed equal values can
therefore serve as the same dictionary key; callers needing owner identity can use
Object.ReferenceEquals. This deliberately changes the earlier provisional identity
behavior, without changing class representation or construction/lifetime rules.

- TimeZone compares the original case-sensitive IANA identifier. All instances use
  the same immutable bundled rule database; no per-instance custom rules, refresh or
  mixed database snapshots are supported. Aliases remain different named identities.
  Hashing uses the matching ordinal text hash, and ToString returns the stored ID.
- ZonedDateTime compares its Instant and named Zone. The validated constructor makes
  local fields and offset deterministic under those fixed rules. Overlap candidates
  with identical civil fields but different instants remain unequal; the same instant
  projected into different zones also remains unequal. Compare Instant explicitly
  when only timeline identity matters. Hashing combines the same two components.
- ZonedDateTime retains ordinary diagnostic type-name ToString; this change does not
  introduce a culture-dependent serialization or round-trip format. Calendar and
  provider/service classes are not automatically converted to value objects.

## Comparison and costs

[.NET TimeZoneInfo](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/TimeZoneInfo.cs)
uses ID and rule equality, hashing the ID. neoCLR uses case-sensitive ordinal ID
comparison to match its existing IANA lookup grammar; it has no custom rules or rule
refresh to compare separately. This avoids rule materialization on each comparison,
but adding mutable/multiple rule databases requires extending identity first. It does
not merge aliases just because their offsets happen to agree today.

.NET DateTimeOffset does not retain a named zone. The richer
[Noda Time ZonedDateTime contract](https://github.com/nodatime/nodatime/blob/main/src/NodaTime/ZonedDateTime.cs)
provides the useful comparison: preserve zone identity rather than equating every
projection of an instant. neoCLR's current LocalDateTime does not retain a calendar
identity; calendars are presentation policies. No operators, ordering contract or
runtime-special zoned type is synthesized.

The equality/hash bodies use existing fields and ordinal/integer helpers, with no
explicit guest wrappers, rule arrays or formatted strings. Objects remain reference
values and their construction still allocates. No measured throughput or allocation
reduction is claimed. ToString on TimeZone reuses the stored string.

## Validation

The [consumer](Main.rvn) covers typed, Object and EquatableTo calls; independent zone
instances; alias distinction; Stockholm's 2024 autumn overlap; same-instant/different-zone
values; null and wrong types; duplicate and distinct dictionary keys; display consistency;
and retained values across GC. [Validation](validation-20261020.json) records interpreter,
JIT and AOT execution with artifact hashes. The compiler reference, XML help and native
API snapshot include the new members. The inventory maps all 260 public declarations
to native API type pages; this is not an exhaustive test of all type members.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/zoned-values/ZonedValues.rvnproj \
  --bundle target/zoned-value-bundle/bundle \
  --aot target/zoned-value-bundle/bundle/bin/neoclr-codegen \
  --output target/zoned-value-app
python3 docs/experiments/zoned-values/check.py \
  target/zoned-value-bundle/bundle target/zoned-value-app \
  --output docs/experiments/zoned-values/validation-20261020.json
```
