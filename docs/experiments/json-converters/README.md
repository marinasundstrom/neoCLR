# JSON converter and union prototype — 2026-10-11

Status: incomplete experiment, not an enabled runtime API. The author requested
.NET-like JSON ergonomics and union serialization. .NET 11 C# unions write the
active payload directly, without polymorphic discriminators. See the
[official contract](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/union-types)
and [decision history](../../development-timeline.md#2026-10-11--net-11-union-json-contract-clarified).

`prototype.patch` preserves all proposed runtime changes and their draft CLI/XML
API declarations. Apply with `git apply docs/experiments/json-converters/prototype.patch`
in an isolated checkout to resume; it changes the runtime library and requires
rebuilding the native libraries and refreshing both API snapshots. It is deliberately
not applied to main. The normal JSON serializer retains its existing contract.

The prototype provides DOM-based `JsonConverter<T>` callbacks, an ordered exact-type
registration collection, per-operation snapshots, serializer options overloads and
an explicitly registered `JsonUnionConverter<T>`. Union payloads use normal Raven
construction and patterns. Ambiguous token kinds fail before payload conversion.
Read callbacks are prepared once per occurrence before ordinary mapper construction;
user callback effects are not transactional. These are proposed contracts, not
qualified guarantees. Generic registration uses static extensions because imported
instance generic methods were rejected by the tested compiler bridge.

`Main.rvn` and `Native.rvnproj` retain the intended end-to-end consumer, including
union arrays/lists/maps, converter ordering, explicit null, callback counts, borrowed
streams and output validation. **This consumer does not pass yet.** The copied Raven
compiler built the prototype libraries and consumer, but the tested native backend
rejected reflected union arrays. The interpreter reached the earlier root union
assertions and then failed at reflected union-array construction. Native value boxing
is also restricted; removing the array test would not establish native union support.

`dotnet/Comparison.csproj` is a .NET 11 preview-language comparison consumer. A local
.NET SDK 11.0.100-rc.1.26425.128 run produced `42`, `"hello"`, `null`, `ambiguous`, and
`unknown-shape`. Re-run with `dotnet run --project docs/experiments/json-converters/dotnet/Comparison.csproj`.
This checks the reference behavior only; it does not qualify neoCLR.

The compiler copy used during exploration came from Raven revision
`8b917fbcb4182f639deb830f32c84f810e727898` with unrelated working changes; this is
provenance, not a reproducible clean revision claim. No compiler source was changed.
The temporary compiled bundles are not a release artifact.

Remaining work: qualify native boxing and reflected union collections, test model
properties and mutation snapshots, and decide metadata-driven union discovery,
structural/custom classification and nullable payload contracts. Raven disallows
passing a known inactive default union; the prototype uses an explicit `Empty` case
for JSON null. That is a compatibility gap from .NET 11's default-union semantics.
Public documentation/snapshots must be regenerated only after the runtime surface is
ready. HTTP options integration remains a follow-up.
