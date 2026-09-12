# Minimal core reference artifact for Raven

Recorded 2026-09-12. This slice supplies the compiler declarations needed by the small
static-call corpus in the [Raven binary experiment](raven-binary-profile.md). It does
not supply executable implementations of a core library or replace neoCLR's System.

## Implemented artifact

The [probe](experiments/raven-target/README.md) builds `NeoCLR.CoreProbe.dll` using
Roslyn 4.12.0 with an empty reference set and metadata-only emission. The artifact
carries `ReferenceAssemblyAttribute`. A raw metadata check requires zero AssemblyRef
rows, and the explicit dependency audit checks its type/member references.

Raven then receives **only this reference artifact**, with
`MetadataImportOptions("NeoCLR.CoreProbe")` and the corresponding emit target identity.
No .NET reference pack is supplied to these compilations. The compiler and artifact
builder themselves still run on .NET; this is a target metadata boundary, not a new
compiler host or bootstrap milestone. The name/version are experiment identities.

The declarations are in [CoreDeclarations.cs](experiments/raven-target/CoreDeclarations.cs).
They include:

| Declaration group | Purpose and limit |
| --- | --- |
| Object, ValueType, Enum and primitive wrappers | Supply familiar CLI compiler metadata for primitive signatures and generated Unit helpers. This does not impose ValueType ancestry or class/value defaults on neoCLR's runtime. |
| String, Array and Type | Minimal type identities; no full text, array or reflection API is provided by this artifact. |
| Attribute, AttributeTargets, AttributeUsageAttribute and compiler attributes | Resolve compiler-generated metadata and attribute constructors. This is not an implemented runtime attribute system. |
| Console.WriteLine(string) returning CLI void | Target declaration for the first program; execution must eventually bind it to neoCLR's real System Console implementation. |

Source bodies exist only to generate reference metadata. They are not API implementations
and must never be used as the runtime behavior of Object equality, text or Console.
Published API identity/versioning and generation from the actual runtime library remain
open; the fixture is deliberately smaller than System.

## Evidence and scope

The corpus binds and emits a Console call, an empty entry point, nested no-result calls,
and an Int32-returning function. Each emitted image passes dependency resolution against
only the core artifact. The Console image has exactly one declared AssemblyRef, pointing
to that artifact. Omitting Console produces the expected missing-name diagnostic; changing
WriteLine to take Int32 produces the expected string-to-int diagnostic. Reversing the
application/core order yields the same audit result.

The [report](experiments/raven-target/results.json) captures core types, assembly references,
method bodies and diagnostics. These are emission/metadata results. No guest code ran on
neoCLR, and no CLI-to-runtime stack translation or verifier is implemented by this slice.
Generated Unit and attribute helper methods still need an explicit reader/admission policy.

## Correction to the earlier inventory

Cecil may synthesize a conventional mscorlib AssemblyRef in memory while decoding or
resolving core types. The earlier report read AssemblyRefs after methods, so its application
inventory could include that synthetic reference. The report now snapshots references
before method inspection, and the audit snapshots all input reference tables before any
cross-assembly resolution. A raw metadata check independently confirms the new core has
no assembly dependencies.

The earlier incomplete Console fixture **does** have a real mscorlib dependency and still
fails its audit; its missing core declarations were also real. The corrected application
inventory no longer reports Cecil's synthetic reference as an on-disk dependency. Historical
reports and discussions should be read with this distinction.

## .NET comparison and next step

This reuses the separation between compiler reference assemblies and runtime implementations
described in [Microsoft's reference-assembly documentation](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies)
(consulted 2026-09-12). The artifact uses Roslyn metadata-only emission with private members
excluded, as documented there.
It also keeps CLI primitive signatures and core metadata conventions at the frontend
boundary, instead of adding a new Raven symbol importer. The benefit is demonstrated
compiler reuse; the cost is maintaining an accurate projection of neoCLR APIs and specifying
how imported identities and methods bind to runtime definitions. It is not evidence of
binary compatibility with the full .NET library.

The metadata baseline remains ECMA-335 Partitions II/III and the primary sources in the
[binary profile](raven-binary-profile.md). Runtime Object/ValueType policy and inhabited Void
semantics remain separate decisions. Next, implement the bounded binary reader and validate
the static-call translation using this corpus, including invalid stack/token/feature cases.
The real Console mapping and generated-helper handling must be explicit before execution.
