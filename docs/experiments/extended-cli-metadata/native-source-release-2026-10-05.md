# Native source-library release path — 2026-10-05

The author clarified that removing the CLI translation bridge is the goal: NeoCLR
consumes native metadata sources. Release work therefore proceeds by rebuilding the
selected Raven libraries directly into NeoCLR assemblies, not by extending legacy
CLI-to-neoIL translation. The attempted legacy DateTime importer change was discarded.
No legacy snapshot fingerprints or generated fragments were committed.

## Executable evidence

[Commands, source/artifact hashes and execution results](native-source-release-2026-10-05.json)
record the extracted native-enabled Raven SDK from `5f6298c1237ecc34d8f00d0ee3c681e9316b9a5b`:

- 115 selected source/adaptor files compile directly into Numbers.dll; 19 HTTP/network
  files compile into Http.dll using native import of that newly emitted Numbers.dll.
- All 15 selected consumer programs compile without library sources. All 13 non-network
  cases execute with expected stdout, exit status and fault behavior where applicable.
  This includes collections/identity, interfaces/inheritance, Tasks/await/cancellation
  and JSON object mapping. HTTP client/server are counted as compile-only in that report.
- Separate live HTTP/JSON checks execute the emitted server and client: Unicode JSON,
  POST mapping, invalid-body and missing-route responses, then the client/server pair.
  Both rounds pass with the rebuilt native libraries. These are execution results,
  not reference-only metadata checks.

`runtime/raven/native/poc-ownership.json` pins the selected cumulative source group and
its canonical semantic identities. `Numbers` remains the existing provisional assembly
name; this is not a new public package-name commitment. The HTTP group references that
assembly and compiles its own unchanged production sources plus native runtime adapters.

## Reproduce

```sh
python3 scripts/build-native-poc-libraries.py \
  --compiler "$SDK/tools/rvnc/rvnc.dll" --compiler-revision "$RAVEN_REVISION" \
  --core "$CORE" --seed "$SEED" --output "$LIBRARIES"
python3 scripts/check-native-poc-samples.py \
  --compiler "$SDK/tools/rvnc/rvnc.dll" --compiler-revision "$RAVEN_REVISION" \
  --core "$CORE" --seed "$SEED" --ownership "$LIBRARIES/ownership.json" \
  --reference "$LIBRARIES/Numbers.dll" --reference "$LIBRARIES/Http.dll" \
  --runtime "$RUNTIME" --async-library Numbers --output "$SAMPLES"
python3 scripts/verify-native-http-json.py \
  --runtime "$RUNTIME" --server "$SAMPLES/http-json-server.dll" \
  --client "$SAMPLES/http-json-client.dll" --seed "$SEED" \
  --module "$LIBRARIES/Numbers.dll" --module "$LIBRARIES/Http.dll" \
  --output "$HTTP_REPORT"
```

Output directories/reports must be new. The sample inventory intentionally returns an
inventory rather than an acceptance exit code: inspect every compiled/execution result.
Compiler revision supplied for an extracted SDK is explicitly labelled declared;
binary hashes record what actually ran. No Git checkout is required beside the SDK.

## Remaining bootstrap and release limits

The primitive CLI core and retained native runtime seed are still explicit bootstrap
inputs. The new build does not generate or translate either. They remain hashed and
visible in evidence; this is not yet a bridge-free full System bootstrap. Next package
these permitted bootstrap dependencies with the source-built native libraries and
matching SDK/runtime, then qualify the extracted installation. Replace remaining
seed-owned declarations incrementally through native source ownership.

The legacy source/archive validator still rejects stale generated CLI-bridge snapshots.
That failure remains true for the legacy distribution; it is not concealed or weakened.
It must not drive new bridge feature work as a prerequisite for this native release
path. Native dependency provenance and packaging remain open gates. No release or
website deployment occurred. Existing .NET compiler/backend behavior is unchanged.
