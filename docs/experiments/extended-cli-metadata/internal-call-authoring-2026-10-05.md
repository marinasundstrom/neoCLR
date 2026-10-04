# Bodyless runtime-service authoring — 2026-10-05

Production descriptor services cannot depend on duplicate bootstrap declarations of
source-owned TypeInfo/member types. The metadata API now supports bodyless nongeneric
assembly-function declarations with signatures owned by the output graph. This permits
future source service declarations to live beside those types. It is not yet Raven
syntax admission or completion of JSON object mapping.

## Contract and comparison

The API reuses .NET's [MethodImplAttributes.InternalCall value](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodimplattributes?view=net-10.0)
(0x1000, primary documentation checked 2026-10-05), and NeoCLR's existing impl_flags
runtime contract. This is an implementation-provided call, not a portable promise that
an arbitrary desktop CLR assembly can supply runtime implementations. CLI metadata
shape is preserved; there is no new opcode or payload version.

SetInternalCall on a definition or builder selects a bodyless assembly function.
Native names use the exact namespace-qualified service name required by the runtime;
ordinary generated function names are unchanged. Native read/materialization, reference
projection and metadata API imports preserve the flag and identity. Signatures can use
local nominal types without reopening an importer or adding bootstrap duplicates.

The alternative of expanding the CLI bootstrap with descriptor definitions would
reintroduce competing ownership; inventing a separate service opcode would duplicate
existing runtime binding. The selected path keeps output identity explicit at the cost
of additional authoring validation and upcoming compiler declaration admission.
No performance improvement is claimed. Runtime binding still rejects unknown names or
unsupported signatures; authoring does not install or load native code.

## Validation

148 C# metadata groups pass, including definition/builder parity, CLI/native flags,
reference projections, source-owned nominal result signatures, malformed implementation
bits, illegal bodies, generic/type-owned authoring and internal-call entry-point rejection.
The executable gate is:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests \
  -- --internal-call-runtime target/debug/neoclr /tmp/internal-call-gate
```

It writes a PE containing real TypeArgumentCount/TypeEquals declarations and a local
type token. The runtime verifies and runs it with exit 42. It also writes a library,
imports its internal-call contract through native metadata, and verifies/runs a separate
consumer with exit 42. An unknown runtime service rejects in both verify and run.
An explicit empty test System seed avoids duplicate builtins in the default bundle;
it is a test dependency, not a production bootstrap ownership claim.

The host API manual covers the new members. Guest API snapshots and website production
capability claims are unchanged. Existing .NET compiler behavior is untouched; CLI
flags are checked through metadata reading rather than attempting to execute a custom
runtime service in the desktop CLR.

[Executable evidence and artifact hashes](internal-call-authoring-evidence-2026-10-05.json)
record the tested base revisions and explicit test dependency.

## Next boundary

Raven needs explicit runtime-service declaration admission, distinct from arbitrary
extern/PInvoke methods. Then service declarations can use source descriptor identities
in one output assembly. ParameterSnapshot/vector ABI, runtime factory layouts and
production Object.GetType/RuntimeContext ownership still need to be integrated before
unchanged descriptor and JSON mapper sources can compile and execute together.
