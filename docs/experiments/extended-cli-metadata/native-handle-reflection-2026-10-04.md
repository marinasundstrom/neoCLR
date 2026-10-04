# Native handle and construction services — 2026-10-04

The ordinary driver now compiles an expanded handle provider and consumes only its
native artifact. The consumer verifies and exits 42 with empty stdout after checking:

- equality and inequality of constructed type identities;
- generic argument count and opaque argument-handle return;
- shape, display-name and metadata-token queries;
- object-type lookup against the corresponding typeof identity;
- real reflection construction of a public nongeneric class, including the constructor's
  initialized field value (42) and the constructed object's type identity;
- missing-constructor and unsupported generic-construction statuses.

A separate invalid-argument consumer faults with `generic argument index out of range`.
Catalog checks reject missing/duplicate services and unbound descriptor-returning
TypeInfo/TypeFields selections. No production descriptor stub replaces source behavior.
[Commands, binaries and hashes](native-handle-reflection-2026-10-04.json).

## Changes and boundary

The selected core Object is now admitted in native symbol-authored dependency
signatures (Raven `80ebdc0a7`). It remains the exact nominal core reference, not an
erased Value or an opaque primitive. The shared generator and .NET implementation
are unchanged; this adapter-only fix is not an independent compiler-main candidate.

The native service catalog admits the exact System.Object and RuntimeTypeHandle
signatures for the exercised services. It still rejects arbitrary nominal results.
The seed generator reuses the existing ObjectTypeHandle/TypeDisplayName native bindings
rather than defining duplicates. The acceptance driver copies selected wrappers from
the checked catalog into an explicit retained seed, preserving all source ownership.

These services already existed in the runtime; no new runtime instruction, metadata
version, allocation shortcut or guest API was introduced. Construction still goes
through ordinary newobj frames and public-constructor checks. Its existing bounded
profile admits concrete nongeneric reference classes; wider construction is not claimed.
This restores the compiler/runtime bridge described in the existing
[reflection direction](../../reflection.md) and
[type-token comparison with .NET](introspection-native-next-2026-10-04.md).
There is no performance claim. The benefit is observable execution through native
assemblies; the cost remains explicit bootstrap/catalog and artifact ownership.

## Reproduction and remaining work

Use the commands in [the typeof gate](native-typeof-2026-10-04.md) with a freshly
generated comparer-storage core and a fresh output directory. The same driver now
includes the expanded cases and negative execution. The reference CLI core remains
semantic bootstrap only; actual service execution uses the retained native seed.
Website/guest API snapshot claims are unchanged because the work is target integration,
not a new public runtime API. Existing metadata/.NET evidence remains applicable;
the focused validation here is the native compiler build, executable gate and catalog checks.

The [refreshed source audit](introspection-handle-services-audit-2026-10-04.json) adds
unchanged HashCode and the handle-service catalog to the previously recorded full JSON /
introspection candidate. It reports 95 errors, including 42 distinct missing services,
and publishes no output. Most missing services return source-owned descriptors or
vectors of descriptors. ParameterSnapshot and Object.GetType are also absent; the
production typeof contract remains unselected. These are dependency-closure failures,
not evidence of 95 independent compiler bugs.

Next close the descriptor/service ownership boundary: supply exact source-owned
TypeInfo/member/assembly/module identities to the service contracts without creating
competing bootstrap definitions; preserve the runtime's snapshot/factory layouts.
Then bind the unchanged descriptors, identify any remaining inherited/override emission
gaps, and run public JSON object mapping. The test provider proves the lower execution
boundary and must not be mistaken for completion of those production sources.
