# Source JSON codec execution — 2026-10-04

The [reproducible driver](bootstrap/verify_source_json.py) builds the five unchanged
JSON document/syntax/value/error and ReflectionError sources, compiles separate
consumers against artifacts only, verifies the linked assemblies and executes both
consumers with exit 42 and empty stdout. [Commands, source/artifact hashes and revisions](source-json-codec-2026-10-04.json).

The production-shaped `Json` library and its public DOM consumer remain separate from
`JsonContract`, which includes a [test-only entry point](bootstrap/json-document-checks.rvn)
for the internal DocumentReader/DocumentWriter. The test calls their actual bodies;
it does not replace JsonSerializer, change visibility or alter any production source.
The two copies are never linked together. Each consumer gets a manifest assigning the
JSON declarations to its exact selected library; dependencies remain explicit.

The codec test covers nested object/array round trips, Unicode text, surrogate-pair
decoding, shared-object mutation, duplicate keys, invalid trailing commas, trailing
content, unpaired surrogate rejection and cycle rejection. It exercises the recently
fixed local-assignment and conditional propagation in real source library methods.
The public DOM consumer additionally retains number parsing and duplicate-field checks.

Compiler revision: `3a99915c8`; general compiler fixes are already in main. No compiler,
runtime or public metadata API changed in this test slice. Focused .NET evidence from
the previous slice is reused; this native gate does not establish .NET source-library
parity or public JsonSerializer/object-mapping completion.

Next: [native introspection prerequisites](introspection-native-next-2026-10-04.md).
