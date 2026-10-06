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

## Union completion fix

Raven `d29179810` uses the existing lazy source method-signature declaration path
before synthesizing the Object.ToString override. It resolves the actual source
method rather than a bootstrap substitute. Four reduced cases initially crashed;
all now pass and preserve override identity with either file order and either
same-file declaration order. The 206 existing source-root/union semantic/generic
tests still pass; the final focused source-root run passes 23 tests. No metadata
encoding or emitter boundary changes were needed.

Both all-source audit variants now terminate normally with diagnostics and publish
no assembly. They still do **not** compile:

| Layout | Inputs | Errors | Result |
| --- | --- | --- | --- |
| Source RuntimeTypeHandle | 178 | 82 | Exit 1, no output |
| Retained bootstrap RuntimeTypeHandle | 177 | 74 | Exit 1, no output |

Retaining the bootstrap handle removes the typeof contract configuration error and
seven conversion diagnostics. This is a diagnostic control, not permission to claim
source ownership of that handle. Both layouts have 45 missing RuntimeServices member
diagnostics covering console, environment, math, GC, pointer conversions and time-zone
services. Remaining scope, conversion, let-else and constructor diagnostics need reduced
cases after ownership and service inputs are coherent; some may be cascades. Full-source
compilation finally provides a diagnostic inventory rather than an initialization crash.
The existing released library controls are reused from the preceding loader slice.

## Reusing the implemented handle ownership contract

The follow-up found that source-handle ownership was already implemented and executed
in the [October 5 handle gate](source-handle-ownership-2026-10-05.md). The new audit had
reused the smaller release manifest without its explicit native primitive selection.
This was an audit input omission, not a new compiler defect.

`full-owned-handle` now assigns the declaration to Numbers, selects the existing native
primitive contract, and assembles a retained seed with exactly the empty competing
handle declaration removed. It includes all 178 inputs and reaches the same 74 binding
errors as the retained-handle control, without RAVT003. No compiler change was needed.
The seed transformation, runtime executable, manifest, source hashes and assembler
command are recorded. This case still publishes no assembly and is a diagnostic layout,
not a new complete ownership manifest; source Object/seed overlap and other ownership
checks must still be addressed before successful emission.

Run with `--case full-owned-handle --runtime target/release/neoclr` and the other audit
arguments. The checked-in POC seed source is used for this case; other cases retain the
supplied `--seed`. Without `--runtime`, the existing default case selection is unchanged.

## Priority order from this evidence

1. **Complete the runtime-service declaration surface.** Group the 45 missing-member
   diagnostics by existing runtime implementation; provide real native adapters and
   executable family tests. Do not add stubs or treat missing audit inputs as new
   runtime deficiencies. Reuse the existing Storage adapters.
2. **Reduce residual binding diagnostics with coherent inputs.** Scope, let-else,
   generic conversion and constructor diagnostics must be separated from service and
   ownership cascades before assigning compiler fixes.
3. **Finish the full-source ownership manifest and seed.** Carry forward the existing
   explicit source-handle contract and remove competing Object/other selected owners
   when binding succeeds; do not claim the diagnostic manifest is ready for linking.
4. **Replace the mandatory CLI primitive reference with native core metadata.** Keep
   that work in the target importer/host contracts; the emitter consumes symbols.
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
