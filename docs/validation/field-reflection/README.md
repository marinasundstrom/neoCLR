# Field and type-initializer reflection qualification

Development validation, 2026-10-16, macOS ARM64. Structural work remains parked.

| Consumer | Interpreter | Managed JIT | Sanitized AOT |
| --- | --- | --- | --- |
| `Main.rvn` | 42 | 42 | 42 |
| `Construction.rvn` | 42 | 42 | 42 |
| `Overflow.rvn` | ArithmeticOverflow | ArithmeticOverflow | ArithmeticOverflow |

The field consumer verifies static/instance filtering, readonly/literal facts,
boxed constant values, module-scoped tokens, private static initializer identity
and parameter metadata, public constructor filtering, strict/relaxed timing,
and absence of nominal initializers on function and array types. Inspection,
literal reads, denied private access and denied readonly/literal writes leave
initialization untouched. The first reflected static read initializes once;
mutable writes validate exact boxed values and static operations ignore receivers.
Instance readonly writes are denied and invalid instance receivers are rejected.

The construction consumer uses `TypeInfo.CreateInstance()` as the first strict
initialization trigger and verifies that repeat construction does not rerun it.
The overflow consumer reads a static field whose initializer overflows. All modes
propagate the terminal fault; native hosts retain an untouched result slot.
These checks qualify ordinary runtime-owned initialization, not direct `.cctor`
invocation or broader method-reflection backend support.

## Reproduction

Build the native library with `scripts/build-native-class-library.py` and use the
matching Raven native compiler (`codex/neoclr`). Then:

```sh
python3 docs/validation/field-reflection/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output /absolute/path/to/fresh-validation-directory
```

Use `--source docs/validation/field-reflection/Construction.rvn` for construction,
or `--source docs/validation/field-reflection/Overflow.rvn --overflow` for the fault.
The runner resolves the explicit State retention root from emitted metadata rather
than guessing its TypeDef index from source order. Schema 7 retains field metadata,
getters/setters and constructor descriptions separately. `construct` retains the
existing public parameterless-construction bridge. Rooting never grants private access.

## Revisions and evidence

- Raven main `289019055`, deliberately carried as `97f5252c9`, fixes qualified static
  compound mutation; all 29 CheckedMutationTests pass on .NET 11. The native driver
  targets .NET 10 and compiles these consumers. Ordinary .NET policy is unchanged.
- Native SDK token preservation: `5d3f6d6cf`; combined origin-table limits:
  `40d40c854`. Static/literal roundtrips and a 257-field mixed declaration pass.
- `fields-results.json` records the fresh
  `target/field-reflection-libraries-20261016g` bundle and final function/array guard.
- `construction-results.json` and `initializer-fault-results.json` retain the
  unaffected earlier `...20261016f` bundle checks. Construction uses the corrected
  shape-selector dispatch; the fault case does not query shape metadata.
- Every result file records exact compiler, metadata SDK, runtime/codegen, source,
  host, bundle and emitted-image hashes and command output. Local full selection
  manifests are retained under the recorded output directories.
- Focused tests also cover descriptive snapshots and constructor/method separation,
  schema-7 permission validation, native shape-query bounds and existing reflection
  behavior. API reference fingerprints are checked separately.

The bounded reflection execution contract remains nongeneric nominal owners,
reference-class instance fields and exact built-in scalar/reference payloads.
Generic field signatures are preserved in introspection; general closed-generic
reflection execution, arbitrary boxed value payloads and private execution are not
claimed. Literal metadata currently carries Int32. Use matching runtime and
System.Runtime provider layouts; the legacy System.Type descriptor profile retains
its original layout. This is development qualification, not publication or a release.
