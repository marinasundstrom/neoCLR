# Native System bootstrap frontier — 2026-10-06

Preview 12 compiles 128 of 166 production System files plus six native adapters.
The 38 omitted production files are a build selection, not 38 demonstrated compiler
failures. This fresh inventory uses the released compiler/bootstrap/library artifacts
and preserves their hashes. [Complete evidence](native-bootstrap-frontier-2026-10-06.json).

| Case | Result | Interpretation |
| --- | --- | --- |
| Released baseline (115 inputs) | Emits | Compilation control |
| Released network/HTTP (19 inputs) | Emits | Compilation control |
| Storage (11 production files) without adapters | Rejects | Missing inputs; not a new runtime deficiency |
| Storage with existing three adapters | Emits | Previous storage work is reusable |
| Six small omitted contract/operator files | Emits | Compilation only; no new execution claim |
| Five calendar/time-zone files | Rejects | Five missing service names plus cascades |
| Nine service-facing files | Rejects | Console, Environment, Math, GC/native memory declarations and group dependencies |
| Seven core files without adapters | Rejects | Diagnostic baseline, not a valid production ownership layout |
| All production sources with existing adapters and source Object root | Crashes | PE-only reflection-loader assumption |

`--case` permits focused repeats. Cases deliberately retain explicit bootstrap inputs;
the combined Numbers owner is diagnostic. Controls must emit or the default audit
fails. An audit process completing is not a successful full-library build. Reports
include abnormal termination, diagnostics and output-publication state.

## First core fix

The full-source compiler crashed casting SourceAssemblySymbol to PEAssemblySymbol.
Removing source RuntimeTypeHandle did not change this failure. Raven fix `6b31997a9`
uses assembly-symbol metadata lookup and restricts reflection interning to actual PE
assemblies. Imported Object bases/signatures also use the explicitly selected source
root; the initial cast-only fix exposed a second Object identity in the regression.
No importer objects are passed to emission, and no new metadata encoding is introduced.

Sixteen pre-change source-root tests passed. The added regression reproduced the crash
only with the source-owned root. All 41 final focused tests pass, including ordinary
.NET and PE-owned-root controls. The fixed compiler still emits both released library
controls. This native source-root contract is not on Raven main yet; the fix is isolated
on `codex/source-object-metadata-resolution`, rather than merging the native backend as
a prerequisite. It changes no published Preview 12 binaries.

The full-source build now passes that crash but fails in GetObjectToStringMethod:
union structural ToString synthesis requests the source Object method before its
member declaration is available. Both full-source variants publish no assembly.
This is not yet a successful System build or a complete set of remaining diagnostics.

## Priority order from this evidence

1. **Complete source Object signatures before dependent union synthesis.** Add an
   order-independent minimal union/root regression. Do not fall back to the old PE
   root or disable synthesis to hide incomplete source declarations.
2. **Reassess canonical core identities.** Once declaration binding can finish, separate
   Object, RuntimeTypeHandle, Value and Void ownership issues from cascades. Preserve
   the explicit primitive contracts and normal .NET behavior.
3. **Replace the mandatory CLI primitive reference with native core metadata.** Keep
   that work in the target importer/host contracts; the emitter consumes symbols.
4. **Complete service families and source coverage.** Reuse the existing Storage
   adapters; cover missing Console/Environment/Math/GC/memory and time-zone signatures
   with real runtime consumers. Group splits above are diagnostic, not final assemblies.
5. **Close the bootstrap loop.** Rebuild against emitted native core/library artifacts,
   then run artifact-only applications and editor acceptance without the old seed/core.

This is the same semantic expectation as .NET: a type's base and override identities
must not depend on whether definitions come from source or metadata. NeoCLR's explicit
source-core mode exposes an ordering dependency that an already-built CLR core hides.
No runtime rewrite, new language semantics, or performance improvement is implied.

Reproduce with `scripts/audit-native-bootstrap.py --compiler <rvnc.dll>
--compiler-revision <declared-revision> --core <Core.dll> --seed <System.neox>
--libraries <directory-containing-Numbers.dll-and-Http.dll> --output <fresh-directory>`.
Use `--case full-source` for the next reduced iteration; unaffected controls may reuse
this evidence, and filtered runs explicitly record which controls were included.
