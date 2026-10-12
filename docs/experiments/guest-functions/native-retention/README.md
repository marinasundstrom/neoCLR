# Native module-function retention — 2026-10-12

[Validation](validation.json) qualifies the unchanged Raven discovery consumer in
interpreter, macOS ARM64 JIT and linked AOT execution. It reads the real TestAttribute's
fixed/named descriptions, method/parameter types and exact module ownership, without
executing test bodies. Empty modules are empty; type methods and child namespaces
are excluded. The native console host performs its ordinary post-entry GC cleanup;
the interpreter additionally reports zero live objects.

## Explicit retention contract

The development backend's `--reflection-roots` schema 5 separates module names,
function descriptors and method-level attributes:

```json
{
  "schemaVersion": 5,
  "types": [],
  "moduleCatalogs": ["[\"GuestFunctions\",\"1.0.0.0\",\"\",\"\",\"0\"]"],
  "moduleFunctions": [
    {
      "assembly": "[\"GuestFunctions\",\"1.0.0.0\",\"\",\"\",\"0\"]",
      "name": "Example.Tests",
      "customAttributes": true
    },
    {
      "assembly": "[\"GuestFunctions\",\"1.0.0.0\",\"\",\"\",\"0\"]",
      "name": "Example.Empty",
      "customAttributes": false
    }
  ]
}
```

Use the exact full assembly identity from the matching image, not its simple name.
There are at most 64 function scopes; each requires exactly `assembly`, `name`, and
a Boolean `customAttributes`. Duplicate scopes, undeclared modules, unknown assemblies,
malformed policy and unsupported signatures fail compilation. All directly owned
function declarations are retained in source metadata order, across visibility levels.
Function retention may be used without retaining the assembly's module-name catalog.
Schemas 1–4 retain their prior meaning; schema-5 type roots use schema-4 fields.

Catalog retention alone does not retain function metadata. Function retention does
not retain method-level attributes unless requested. Neither grants callable entry
points, executes annotation constructors, loads assemblies or registers tests.
Unretained scope/token queries fault rather than fabricate empty metadata. The
qualification runs separate negative tests for missing catalog, function and
attribute retention. Fourteen focused backend reflection tests pass, including
scope identity, malformed contracts and an explicitly retained empty module.

The binder reuses interpreter-authored MethodInfo/ParameterInfo/attribute recipes.
An ownerless MethodInfo has no declaring type; its runtime-owned StoredModule field
must therefore be materialized explicitly, along with descriptor identity. The
private factory grant remains confined to descriptor materialization. Source image
metadata is unchanged. This is backend retention configuration, not a new metadata
format or Raven language/compiler contract.

## Reproduce and limits

```sh
python3 docs/experiments/guest-functions/verify_native.py \
  --bundle target/generic-cleanup-gc-bundle/bundle \
  --compiler target/generic-cleanup-project-compiler/rvnc.dll \
  --backend tools/aot-poc/target/release/neoclr-aot-poc \
  --runner /path/to/current/neoclr \
  --output target/guest-functions-native-check
```

The harness compiles a fresh consumer and reuses the pinned macOS shared-GC host
recipe, including absolute SDK/library paths. It records commands and input hashes;
large compiler inventories are represented by hashes. The original validation above
used the backend directly. The [CLI follow-up](cli-validation.json) now runs JIT and
all three negative retention cases through `neoclr run --reflection-roots` with
explicit compiler/host overrides. `--runner` selects an updated runtime, defaulting
to the bundle runtime when omitted. This harness still uses a platform-specific
AOT host recipe; it is not a portable SDK command or Windows qualification.

The descriptor structure and annotation semantics follow the existing .NET-inspired
API; logical modules scope exact assembly namespaces rather than physical .NET
modules. Open generic signature snapshots, ownerless parameter/return annotations,
general invocation and automatic in-process TestAttribute registration remain open
in [issue #27](https://github.com/marinasundstrom/neoCLR/issues/27). Generated typed
adapters and manual registration remain the runner's supported execution paths.
