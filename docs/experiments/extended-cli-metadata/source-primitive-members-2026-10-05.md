# Source primitive members and cumulative System build — 2026-10-05

Fixed the isolated source/import String member lookup difference. The host previously
removed current-output primitive providers from imported selection without recording
the source member owner. Primitive receivers therefore saw only the bootstrap surface.
Raven `de5645e96` now passes an explicit SourcePrimitiveTypes set in MetadataImportOptions.
Scalar expression/signature identities stay with the bootstrap; exact named member
lookup and ordinary member-signature completion use the source declaration. Source
names alone do not opt in, imported/source ownership cannot overlap, and source lookup
does not fall back to bootstrap-only members. Only the NeoCLR target admits this option.
There is no importer object reuse during emission or .NET backend change.

This is the temporary primitive-core bootstrap split, not a permanent metadata alias.
The full source-owned core can later replace it. Constructors/indexers and other
intrinsic handling are not redesigned here. The .NET ergonomic expectation remains
that explicitly selected source and imported declarations offer the same named members;
ordinary .NET lookup must not start redirecting same-named source declarations.

## Validation

The previous 19 provider/core tests pass before the change; those plus six new source
provider tests pass afterward (25). They check methods/properties/static and literal
receivers, source order, canonical scalar identity, explicit opt-in, missing members,
configuration copying/conflicts and .NET rejection. This is target bootstrap support,
not a demonstrated general .NET defect to backport independently.

[Updated compilation audit](system-source-primitive-audit-2026-10-05.json):

- Source and imported minimal String probes both emit.
- 75-file baseline, 82-file encoding and 87-file text-stream combinations emit.
- 108-file JSON/introspection combination and 109-file combination with ResultOperators
  emit. That is 105 production System files plus four native adapters in one library.
- Tasks/Concurrency now pass binding in the 115-file combination, then reach the
  metadata writer's 256-type cap. The separate six-file task library still emits.
- Storage/networking remain at missing service contracts; the complete library still
  fails binding. No claim that all code generation is complete follows.

[Executable evidence](source-primitive-json-evidence-2026-10-05.json) compiles both
unchanged JSON consumers against the single cumulative Numbers.dll, with library
sources absent. Native verify succeeds; the nested/array/mutation consumer returns 42
with exact expected stdout, and the earlier mapping sample returns 0 with its expected
success line. The retained seed and 100-million-instruction budget remain explicit.
The metadata/runtime source revision is  f36aa9d7; binary/source hashes identify the
actual compiler build before its commit. No runtime or format change was required.

Reproduce the compilation with audit_post_json_compilation.py as documented in the
[assessment](system-post-json-assessment-2026-10-05.md). Use its cumulative-json/Numbers.dll
and ownership.json with verify_cumulative_json.py, supplying --compiler, --library,
--ownership, --seed, --core, --runtime and a fresh --output directory. It compiles only
the consumer sources and records commands, output and artifact hashes.

Next remove the inconsistent writer/native-reader type budget exposed by the larger
library, retaining the existing CLI reader's bounded row contract. Then address storage
and DNS/socket service families and remaining source/bootstrap ownership.
