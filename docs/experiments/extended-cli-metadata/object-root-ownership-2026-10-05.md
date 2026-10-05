# Source Object ownership investigation — 2026-10-05

Source Object remains blocked. This slice establishes executable root-identity
controls; it does not replace the retained seed or enable source Object compilation.
Compiler baseline: Raven `485b364fc` (implementation `1ac78fdfa`). Runtime baseline:
neoCLR `7d5fcd70`. No compiler implementation or assembly format changes are included.

## Finding

The runtime currently admits intrinsic Object slots from the retained `System`
module. Two isolated attempts with the same root in a separate `Core` module fail:
a rootless override reports “override requires an inherited virtual method”, and
boxed Int32 display reports “unknown base type”.

An experimental change compared the method identity with the loaded root definition
instead of requiring `System`. Both reproductions then executed, but the existing
`boxed_intrinsic_dispatch_does_not_claim_other_object_definitions` regression failed:
an application-defined type named System.Object acquired intrinsic hashing behavior.
That experiment was reverted. A matching name plus a self-consistent declaring
identity does not establish that the host selected this definition as the runtime root.

The current boundary is locked by `tests/object_root_identity.rs`: seed-root display,
equality, hashing, rootless override dispatch, binary seed/application loading,
duplicate/forged ownership rejection, wrong scope rejection, and rejection of implicit
external-root selection. The last case is a supported rejection, not successful
source-library execution. Existing artifact-only Object service evidence remains valid.

## Next implementation boundary

Implement root ownership as one explicit contract across these layers, in this order:

1. Specify the selected root's assembly identity in bootstrap ownership/configuration.
   Reject competing seed/source definitions and inconsistent dependency identities.
2. Resolve keyword `object`, named System.Object, implicit class bases and source
   override signatures to that selected semantic identity. Source Object itself must
   not acquire the bootstrap Object as an accidental base. Preserve ordinary .NET.
3. Give metadata definitions and builders equivalent root authoring support. Use CLI
   Object signatures where appropriate; derive boxing and Object slots from the
   selected identity instead of reopening importer objects. The root is not a scalar
   primitive. Keep unsupported root configurations diagnostic before publication.
4. Carry that explicit selection to runtime linking. Only then replace the current
   System-module slot admission rule. Retain the application lookalike rejection and
   validate exact type/member identities, revisions, complete slot contracts and
   single ownership. Do not add a name-only fallback.
5. Replace the seed root and run unchanged source Object plus the cumulative library,
   then the separate JSON/Tasks/Object consumers. A compiler-only success is insufficient.

This is a refinement of the existing Object-slot contract, not a new text/value model
or a proposed general Reflection layer. The [.NET/CLI comparison](reference-object-overrides-2026-10-05.md#contract-and-comparison)
remains applicable: exact inherited slots and dedicated Object signatures are preserved.
Explicit selection costs coordinated host/compiler/writer/runtime work, but avoids
silently granting an arbitrary declaration core semantics. A new serialized ownership
field versus host-supplied configuration remains an implementation decision; neither
has been introduced here.

## Validation

Run the bounded controls:

```sh
cargo test --release --test object_root_identity
cargo test --release --test object_equality boxed_intrinsic_dispatch_does_not_claim_other_object_definitions
```

Result: seven new controls and the existing lookalike regression pass after reverting
the experiment. Matching Raven integration notes are committed as `e86c436eb`.

On this machine, set SDKROOT to the installed MacOSX26.2.sdk. No performance claim,
public API change, website feature change or full-System completion is made.
