---
title: Experimental runtime hosting
---
# Experimental runtime hosting

Development API (2026-10-05), on `codex/extended-cli-metadata`. These Rust APIs are
host-only; they are not guest System members or a stable native hosting ABI.

## Explicit Object-root loading

```rust
impl LoadedProgram {
    pub fn with_modules_and_object_root(
        module: &Module,
        library: &Module,
        dependencies: &[Module],
        object_root: &metadata::TypeDefId,
    ) -> Result<Self, Fault>;
}

pub fn assembler::read_modules_with_object_root(
    inputs: &[ModuleInput<'_>],
    library: &Module,
    object_root: &metadata::TypeDefId,
) -> Result<Vec<Module>, Fault>;
```

- `module`, or the first reader input, is the application. Additional inputs are
  dependencies. `library` is the retained `System` module without an entry point.
- `object_root` selects a library definition by exact module name, optional revision
  and zero-based type row. It cannot select an application definition. The host must
  obtain this identity from its validated dependency catalog; a row is not stable
  across rebuilds and a revision label does not itself authenticate artifact contents.
- The selected definition must be the only `System.Object`, public, abstract,
  fieldless, non-sealed, nongeneric, top-level, reference Record, with no base.
- Required slots are public concrete virtual instance `String ToString()`,
  `Boolean Equals(System.Object)` and `Int32 GetHashCode()`, with ordinary receivers
  and parameters and no generic/interface implementation contracts.
- Missing/duplicate roots, wrong identities, incompatible shapes/slots, malformed
  modules, missing references and inaccessible uses return `Fault` during loading.
  Existing visibility, native-service and instruction validation still applies.
- With explicit selection, `System` may refer to supplied dependencies. Its reference
  list and uses are validated against the complete load set. Default loading retains
  its self-contained System requirement.

The reader returns the original separate artifacts, preserving scopes and identities.
It validates a temporary linked copy. `LoadedProgram` keeps an immutable linked
snapshot. Neither operation mutates input artifacts or executes guest code. Call
`verify()` explicitly before execution; structural validation is not typed verification.

Selection is load-context state, never serialized in JSON, NEOX or PE/#Neo. It must be
supplied again after reading artifacts. Existing `read_modules`, `with_library`,
`with_modules`, `new` and free execution helpers do not select an external root.
The CLI and Raven driver do not yet expose the option. Source Object compilation and
metadata-writer root authoring remain pending.

The Rust `Module` now includes private transient context. Hosts previously using
struct literals must construct artifacts through assembler/metadata readers or JSON
deserialization; existing public metadata fields remain accessible. This is a Rust
source-compatibility change, not an artifact-format change.

## Example

Given application/System/Core artifacts whose identity catalog identifies the Object
row in Core, select it explicitly:

```rust
let root = neoclr::metadata::TypeDefId {
    module: "Core".into(),
    revision: Some("r1".into()),
    index: 0,
};
let program = neoclr::LoadedProgram::with_modules_and_object_root(
    &application, &system, &[core], &root,
)?;
program.verify()?;
let execution = program.run(neoclr::Limits::default())?;
```

The matching revision and artifact-loading cases execute in
`tests/object_root_identity.rs`. See the repository's `docs/loaded-program.md` for
snapshot/execution lifetime and resource limits. This API does not authorize native
P/Invoke; existing safe/unsafe execution boundaries remain unchanged.
