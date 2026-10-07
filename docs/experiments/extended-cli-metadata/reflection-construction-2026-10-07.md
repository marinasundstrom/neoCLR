# Native construction with source owners — 2026-10-07

The no-argument TypeInfo.CreateInstance implementation now uses the internal
NativeReflection facade. It calls the existing ReflectionConstructionCheck and
ReflectionConstruct runtime services, declared as source-owned InternalCall functions.
Runtime behavior and public Reflection API signatures are unchanged. Source Object
and RuntimeTypeHandle no longer pass through the bootstrap construction signature.

Like CLR reflection, construction still runs the actual constructor and checks
accessibility; this change does not bypass initialization or grant additional access.
The distinct facade avoids competing with the bootstrap RuntimeServices static
member. The temporary CLI bridge exposes the same two names and translates them to
the same existing services, with exact bootstrap-owner and signature validation.
This bridge support is compatibility for older builds, not a fallback for native
references. The native path uses definitions emitted from Raven sources.

## Validation

- [Native evidence](reflection-construction-2026-10-07.json): ordinary Raven driver
  compiles the source-Object fixture, source RuntimeTypeHandle, the new facade and
  native declarations. A metadata-API-authored consumer references that library.
  Verification succeeds; execution returns 42 with no output. It proves a distinct
  object is allocated and its constructor writes 42, rather than copying the original
  object's mutated value 7. Private construction returns status 3; a missing
  parameterless constructor returns status 4.
- This focused gate does not run the complete public Result-returning reflection
  extension or ordinary Raven imported-root consumer. Full System is still blocked.
- C# `ConstructionBindingChecks` passes for both bridge facade methods and rejects
  the wrong result signature, a forged bootstrap owner and an unlisted operation.
  Reproduce with Probe `--reference-library-core` followed by
  `--construction-binding-checks`. Probe builds with zero warnings/errors.
- The RavenDoc assembly was regenerated with Probe `--reference-core`; snapshot
  refresh/check passes. NativeReflection is internal in the real class library;
  its public bootstrap scaffold is not a public platform API or API-reference item.
- [Full audit](native-bootstrap-construction-2026-10-07.json): **197 inputs**, no
  binding errors, no output published. The full-source list excludes seed-only
  ObjectIntrospection because source Object already owns GetType. Earlier POC
  ownership and the extension itself are unchanged.
- New frontier: Environment.GetCurrentDirectory fails generic-call stack validation
  because imported System.Value and source System.Value do not share ownership.
  Resolve the explicit intrinsic/signature contract rather than weakening validation.

Compiler binary corresponds to Raven `793220f33`; neoCLR base `653c4c35` plus this
slice. Native/compiler/dependency/source hashes and commands are recorded in the
JSON evidence. No compiler or runtime implementation change was necessary.
