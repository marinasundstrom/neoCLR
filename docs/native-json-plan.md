# Native JSON through introspection and reflection — 2026-10-09

The author directs native JSON serialization as the next HTTP showcase priority:
"We need to get JSON serialization working. That requires introspection and reflection
support." This takes precedence over further general hosting/reload work. Keep the
current serializer and its interpreter contract as the semantic reference.

## Consumer and scope

`docs/experiments/native-json` is a typed round-trip probe using the public
`JsonSerializer.Deserialize<Report>` and `Serialize(Object)` APIs. Its string, Int32
and Boolean properties exercise UTF-8 text, type identity, property metadata,
construction, boxing and getter/setter invocation. The interpreter prints
`{"Name":"Café","Count":3,"Active":true}`. Native execution is **not yet admitted**.
The HTTP follow-up must use the same mapper through JsonContent/client JSON APIs,
not a separate serializer or hand-written JSON strings.

The existing mapper first validates the complete input tree, then constructs and
assigns the model. Preserve its checks for public instance properties, constructor
availability, receiver/value compatibility, unsupported shapes, depth/item limits,
null/polymorphism restrictions and terminal user-code faults. Test real setter effects
and ensure invalid input does not run constructors/setters. Existing nested-class and
typed-array support must remain a visible coverage obligation after the flat-model gate.

## .NET comparison and provisional implementation choice

Microsoft's [reflection/source-generation comparison](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/reflection-vs-source-generation)
and [source-generation guide](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/source-generation)
(reviewed 2026-10-09) describe reflection-based metadata discovery and generated
metadata/serialization paths. System.Text.Json's Native AOT use requires source
generation because its reflection path needs APIs unavailable in that environment.
This does not mean Native AOT has no reflection of any kind.

For neoCLR, first retain a bounded closed-world metadata set and generate checked
invocation dispatch for the existing introspection/reflection facades. This is a
provisional native backend design, not a new public reflection API. It preserves
one mapper and enables other metadata consumers, but costs retained metadata,
boxed values and dispatch machinery. Source-generated per-model codecs remain an
alternative with potentially smaller metadata and less dispatch; they would add a
second serialization path and do not supply the requested general introspection
foundation. No performance advantage is claimed without measurements.

## Bounded slices and admission

1. **Implemented foundation:** preserve opaque RuntimeTypeHandle storage through
   specialization; close generic `ldtoken` operands, retain token-only type shapes
   and emit image-local identity. Handles pass through locals, parameters/results
   and fields; equality distinguishes closed types. They are not GC roots or native
   pointers. Numeric conversion remains rejected. This is not TypeInfo discovery.
2. **Next:** make the primitive object representation needed by the mapper complete
   and retain source type/property/constructor/accessor metadata before private AOT
   projection removes it. The current specialization path clears property records;
   simply accepting `ldtoken` cannot restore that metadata later.
3. Bind the exact type-identity/shape/property-list services used by TypeInfo.
   Keep metadata roots separate from ordinary call reachability. Missing metadata
   must produce an explicit admission diagnostic, not silently empty property lists.
4. Supply checked property get/set and parameterless construction dispatch over
   selected bodies. Preserve access checks, boxing/type checks, fault sites and GC
   ownership. No runtime-generated code or arbitrary dynamic loading is required.
5. Pass the flat JSON round trip and negative cases, then the typed HTTP client/server
   consumers on macOS ARM64 and Windows x64; extend to nested/array mappings.

Explicit metadata-root configuration, metadata size budgets and representation of
revisions/image ownership still need validation. Runtime handles from separate images
must not be compared, persisted or used across reload. Keep native Windows ARM64
qualification separate. Ordinary .NET/Raven behavior and Runtime Contract settings
remain unchanged; this work is in neoCLR's native selection/backend layers.

## Evidence

The original typed probe rejects native specialization with
`specialization requires closed reference-free local value types: RuntimeTypeHandle`.
The type-token slice removes that rejection; subsequent native admission still rejects
Boolean boxing in `ObjectMapper.ReadValue` at instruction 110. Further metadata
service and reflection dispatch gaps remain.

`type-tokens.neoil` checks primitive identity, distinct nominal types, distinct closed
generic shapes and a handle stored in a generic class. The Rust regression compares
native/interpreter execution with the same supplied System module and checks that
handles are absent from GC trace slots. Invalid numeric casts fail before an object
is emitted. `scripts/validate-native-type-tokens.py` runs the same C consumer on
macOS and Windows. See [validation evidence](native-type-token-validation.json).
