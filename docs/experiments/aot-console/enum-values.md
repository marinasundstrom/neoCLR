# Native enum values — work in progress

The HTTP dependency chain needs HttpStatusCode inside HttpError and Result. The
bounded AOT value profile now admits verified Int32 enums as nominal one-field
records. Calls, generic specialization, locals and checked reserved arrays preserve
the enum type; only explicit conv.i4 extracts its storage. Int32 and/or/xor/not
lower directly to integer bit operations. Other integer widths and Boolean bitwise
operations remain outside this native slice.

This reuses the existing [enum contract and .NET comparison](../../enums.md) and
[Raven flags research](../extended-cli-metadata/flags-enums-2026-10-05.md). Like the
.NET baseline, unnamed values, aliases and all signed underlying bit patterns are
valid; flags metadata does not validate combinations. Unlike a new boxed carrier,
reusing the native record profile needs no extra allocation or GC descriptor. It
retains the existing padded array storage and bounded profile costs. This is backend
coverage, not a new public API, bridge encoding, stable ABI or performance claim.
The interpreter contract is unchanged; no interpreter optimization is needed.

`enum-values.neoil` checks generic round trips, explicit storage conversion, all four
bitwise operations, by-reference member calls and array snapshot independence at
Int32.MinValue, -1, 0, 200, 599 and Int32.MaxValue. Focused native/interpreter tests
also reject malformed storage and cross-enum generic arguments before publication.
[EnumValues.rvn](../../../benchmarks/native-web/EnumValues.rvn) exercises actual
HttpStatusCode casts/equality, BindingFlags masks and ArrayList growth/copies.
[Evidence](../../../benchmarks/native-web/enum-validation.json) records matching
execution with a 64 KiB native heap, sanitized adapters and standalone libSystem-only
linkage. Run verify_callbacks.py --case EnumValues with the documented tool paths.

The full HTTP driver still exceeds the 128-type specialization budget. The narrower
TaskResultList source now includes an unnamed status inside a nested union, but its
initial compiler run failed with NEOMETA003 for System.Void. The explicit-core import
fix now lets it compile and execute interpreted; native admission next rejects
HttpError's 40-lane layout against the original 32-lane cap. The following
[64-lane slice](record-arrays.md#http-nested-results-2026-10-08) now runs that consumer
in both modes. The HTTP server itself remains unqualified. No benchmark was added:
this slice restores enum semantics and does not propose a performance improvement.
