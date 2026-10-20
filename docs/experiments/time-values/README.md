# Time value equality, hashing and ordering

Development follow-up, 2026-10-20. Date, TimeOfDay, Instant and TimeOffset already
implemented typed equality/ordering but lacked matching Object equality and hash
methods. LocalDateTime lacked the typed/self-interface contracts too. The current
changes complete those contracts without changing layouts, clocks, calendar rules,
parsers, formatting, or host/runtime ABI.

| Type | Equality and ordering key | Hash input |
| --- | --- | --- |
| Date | Gregorian day number | day number |
| TimeOfDay | complete tick of day | both halves of the tick count |
| Instant | signed Unix tick count | both halves of the signed count |
| TimeOffset | whole-second UTC displacement | signed seconds |
| LocalDateTime | date, then complete tick of day | combined date/time hashes |

Object equality requires the same value type; null and other types compare unequal.
Defaults retain their existing meaning. LocalDateTime comparisons are civil and
never infer a timezone or compare absolute instants. Hash collisions are valid and
must not substitute for equality. Hashes are not persistent identifiers.

This follows the typed/Object equality and hashing shape of
[.NET DateOnly](https://learn.microsoft.com/dotnet/api/system.dateonly).
[.NET DateTime equality](https://learn.microsoft.com/en-us/dotnet/api/system.datetime.equals?view=net-6.0)
compares date/time values without treating Kind as an equality discriminator.
neoCLR's existing separate civil/instant/offset types make that distinction explicit:
LocalDateTime compares civil fields and Instant compares absolute elapsed ticks.
There is no new equality convention based on culture, rendering or object identity.

Typed calls use value fields; they do not require reflection descriptors, strings
or normalized copies. Explicit Object/interface conversions in the test intentionally
box. Generic record equality may also box under the existing runtime policy. This
slice fixes correctness; it makes no measured allocation or throughput claim.

## Evidence

`Program.rvn` covers typed/interface/Object dispatch, null and wrong types, defaults,
range ends, negative/maximum Instant ticks, individual ticks, high tick bits, hash
collisions, date-before-time ordering and direct/generic/imported record components.
`baseline.txt` records rejection of LocalDateTime interfaces against the preceding
library. `check.py` records interpreter/JIT/AOT output and binary hashes.

The public Date MissingHash fixture is removed because Date now has a correct hash.
The existing Raven `tools/NeoClrMetadataProbe/NativeSourceCoreChecks.cs` deliberately
incomplete imported MissingHash type retains negative compiler coverage. Historical
negative files are kept as checkpoint evidence, not current expected behavior.

No compiler change is required. The legacy reference assembly now also declares the
existing EquatableTo/ComparableTo contracts of all six time structs, including Duration.
Both API snapshots are refreshed. Ordinary struct default equality/hash, enum and
floating-point component adapters, broader generic record formatting and the rest of
the class-library audit remain open; this is not release qualification.

The consumer passes interpreter, JIT and AOT with the matched source-built library.
Four API snapshot checks pass. This evidence qualifies these contracts, not the full
library or a release package.

The website build checks 4927 pages. Publication remains a separate operation.
