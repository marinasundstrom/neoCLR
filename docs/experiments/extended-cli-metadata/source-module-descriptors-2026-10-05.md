# Source-owned module descriptor factories — 2026-10-05

The next blocker after source-declared internal calls was reproducible with a generated
native assembly: TypeModule returning an output-owned ModuleInfo interface failed
runtime verification with `reflection binding signature mismatch`. Existing services
expected fixed native descriptor names, while the metadata writer preserves nominal
assembly identities in generated names. No compiler or format change fixes that mismatch
by itself.

## Implemented boundary

The runtime now binds TypeModule against the actual returned ModuleInfo interface. Its
metadata origin must identify System.Introspection.ModuleInfo; the internal
RuntimeModuleInfo provider is resolved in that exact source assembly **and module**.
There is no global source-name fallback, dependency loading, or renamed nominal type.
The provider must implement the selected interface and retain the production source
layout: StoredIdentity and StoredName, both String, in that order. Missing, ambiguous,
abstract, generic, nested or incompatible providers reject.

The runtime materializes the trusted module metadata snapshot into that provider and
returns a reference viewed through the selected interface. It validates the snapshot
fields and heap budget. It does not invoke a user constructor or load reflected user
objects. This follows the existing trusted metadata factory model; source origin selects
the known layout contract while actual nominal identities remain unchanged.

Like .NET metadata, source-qualified names are not sufficient to identify types across
assemblies. This uses the assembly/module identity already carried in metadata instead
of treating matching names as interchangeable. The benefit is reuse of source-built
runtime descriptors without duplicate bootstrap definitions; the cost is explicit
factory-layout validation and additional linking context during service classification.
No performance improvement is claimed and no encoding/version change is introduced.

## Evidence and limits

The C# API gate creates a ModuleInfo contract, a provider with the production two-string
layout, and a bodyless TypeModule declaration. The generated native PE verifies and runs,
dispatches through the interface getter and returns 42 after comparing the reflected
module name. A wrong provider layout rejects during both verification and execution.
This is a focused factory-contract fixture, not compilation of the full ModuleInfo.rvn
source and its dependencies.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --source-module-runtime target/debug/neoclr /tmp/source-module-gate
cargo test --lib source_module_tests
cargo test --test runtime_services
cargo test --test reflection
```

149 C# metadata groups pass. Two focused Rust tests cover assembly/module isolation,
ambiguous providers, exact snapshot identity/content, heap limits and runtime-service
classification. Twelve existing service tests and eleven legacy reflection tests pass.
[Revisions, commands and artifact hashes](source-module-descriptors-evidence-2026-10-05.json).
Existing canonical descriptor bindings retain their original path. There is no new
public API; guest API snapshots and production website claims are unchanged.

TypeInfo/member factories, Option payloads, immutable parameter vectors and managed
array results still need coordinated source-identity support. The remaining production
class-library declaration and override gaps also remain open. This does not complete
reflection or JSON object mapping. Broader provider registration/caching should follow
actual requirements rather than a new reflection layer.
