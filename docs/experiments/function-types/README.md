# Function migration evidence

Development branch work, 2026-09-28. The [design](../../function-types.md) and
[tracker](../../tracking/runtime-language.md#function-types-and-objects--2026-09-28)
own scope and completion. Named function types remain possible future work.

Native `tests/function_types.rs` exercises structural binding and signature
identity without a nominal callable declaration, generic substitution and higher-order
storage, artifact round trips, retained receivers, frame-receiver rejection,
no-result invocation and nominal classification. The no-result case covers both
call and callvirt; the latter exposed and corrected a verifier stack-result bug.

`Main.rvn` checks the public descriptor split: nominal declarations implement
NominalTypeInfo and MemberInfo, arrays implement only TypeInfo, common display and
type queries remain available, equality works across providers and object discovery
and collection preserve descriptors. A constructed Option<int[]> keeps a nominal
outer descriptor and a structural generic-argument descriptor. `verify.py` also rejects declaration-only
members on a plain TypeInfo parameter.

Run against a freshly built bridge, generated library and matching reference:

```sh
python3 docs/experiments/function-types/verify.py \
  --runtime target/debug/neoclr \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --system target/function-types/System.neoil \
  --reference target/function-types/NeoCLR.CoreProbe.dll
```

The descriptor consumer has compiled, passed native verification, and executed with
`Nominal and structural descriptors passed`. Full Function replacement is not
complete: the Raven callable importer and library still admit delegate transport
and nominal callable declarations. The native shape tests do not prove that every
Raven function expression yet has structural runtime identity.

## Deferred general compiler candidate

With Raven `5dde32fbf` on its isolated `neoclr` branch, placing the new provider's
expression-bodied Module getter before its Handle field emitted a null-returning
body despite the source calling RuntimeServices.TypeModule. The importer rejected
that body as `expected System.Introspection.ModuleInfo, found FaultNull`.
Placing declaration getters after the storage and other methods emitted the expected
field read and service call, and the same consumer then passed. This is a source-order
observation, not a diagnosed root cause or a compiler fix. The current library uses
that validated ordering. Reduce and validate this candidate on ordinary CLI metadata
in a main-based Raven feature branch before integration; do not copy target policy
or fixtures into Raven main.

Consumer migration also compiles the assembly/nested-type/introspection/array
examples and the attribute, union-construction and route-mapper fixtures against
the matching reference. The older introspection-object and introspection-flags fixtures still hit the
existing nonpublic multi-argument conversion admission limit. An exploratory
public-helper compile of the flags fixture additionally hit unsupported System.Type[]
attribute metadata. These broader fixtures are not counted as passing evidence;
the focused descriptor consumer uses a public assertion helper and passes.

Validation for the descriptor checkpoint: 76 native Function/callback/reflection
tests pass across function_types, delegates, reflection, reflection_arrays,
reflection_construction, reflection_members, reflection_properties and
reflection_hierarchy. Full library bootstrap regeneration and source/output hash
validation pass, as does the RavenDoc reference snapshot check. The migrated
attribute-introspection consumer executes successfully, including nominal type
attributes and union-case discovery. No website build or publication was performed.

The migrated reflection, flags and type-preview consumers execute, and the old
System.Reflection descriptor namespace is rejected by the compiler. Raven's
matching integration note is committed on its isolated neoclr branch as `0218f75af`.
