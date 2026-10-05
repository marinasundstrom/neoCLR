# Native POC release gate — 2026-10-05

**Publication remains blocked.** The installed compiler/editor gate and website API
rendering pass on macOS arm64. A reproducible runtime/bootstrap/library distribution
is still required. No release, tag or website deployment was performed.
[Machine-readable evidence and artifact hashes](native-release-gate-2026-10-05.json).

The author's subsequent direction supersedes the legacy-bridge repair priority below:
[the native source release path](native-source-release-2026-10-05.md) now rebuilds the
selected libraries and executes consumers without invoking that bridge. The legacy
source/archive audit still fails; native bootstrap dependency packaging remains open.

## Qualified tooling and documentation

- Raven integration `5f6298c1237ecc34d8f00d0ee3c681e9316b9a5b` builds a native-enabled
  SDK and VSIX using the explicit `RAVEN_NEOCLR_METADATA_PROJECT` packaging option.
  Ordinary .NET packaging remains the default. The local label
  `0.1.13-neoclr.dev.20261005` is a test artifact label, not a published release.
- VS Code 1.140.0 passes **19 extension-host checks** using the extracted SDK and
  the installed VSIX directory in an isolated profile. The test harness loads that
  directory through `--extensionDevelopmentPath`; the client log confirms the bundled
  server, with no checkout/server-path override. This is not an unattended UI-install test.
- Checks cover XML/Markdown hover and completion help, sidecar-only refresh,
  member fallback, artifact replacement, missing-reference recovery, native navigation,
  unchanged collections and Tasks/await execution, failed-output preservation and
  an ordinary .NET editor control. Option/Result hovers display authored API prose.
- A separate ordinary .NET project compiled with the same extracted SDK executes
  with the expected exit status 42 and no stdout.
- Shared API XML accompanies native libraries where documentation IDs match. CLI
  carrier implementation notes now live in the website reference-support page rather
  than native IDE prose. This does not claim complete runtime API documentation.
- RavenDoc publisher `137431e44109c6050af23075401cc2090784729c` restores authored
  union-case summaries through logical-ID lookup followed by physical carrier-ID
  fallback. The regression failed before the fix; nine focused .NET documentation
  tests pass. The fix was integrated and pushed to Raven main independently, then
  cherry-picked into the integration branch; the integrated fix branch was removed.
- The API reference was genuinely regenerated with the matching compiler bridge;
  its assembly bytes remained identical. The XML source fingerprint was refreshed.
  Snapshot validation, all **18 website tests** and the **1,803-page website build**
  pass. Desktop and 390×844 mobile browser inspection confirmed navigation/layout
  and authored Some/None case summaries. Pages identify themselves as development
  snapshots while preserving published Preview 11 links and release notes.

## Reproduction

Build the SDK and VSIX from the recorded Raven revision with the metadata project
from the recorded NeoCLR source revision:

```sh
RAVEN_NEOCLR_METADATA_PROJECT="$NEO/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj" \
  bash scripts/package-sdk.sh osx-arm64 0.1.13-neoclr.dev.20261005
RAVEN_NEOCLR_METADATA_PROJECT="$NEO/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj" \
  bash scripts/package-vscode.sh 0.1.13-neoclr.dev.20261005
```

Use the packaging arguments documented in Raven's `docs/compiler/distribution.md`.
Extract the SDK, install the VSIX into an isolated extension directory, and prepare
fixtures with `scripts/prepare-native-editor.py --sdk "$SDK"` instead of `--raven`,
plus the core, seed, ownership, runtime and library inputs documented in the
[original editor gate](native-vscode-acceptance-2026-10-05.md). Adjacent XML and
Markdown sidecars are copied automatically. This run supplied the shared authored
API XML beside the recorded Numbers and Http references.

Set `systemDocumentation: true` in the generated `acceptance.json` to require
Option/Result prose. Remove `raven.languageServerPath` from both generated settings
files (workspace and native folder) to select the installed bundled server. Run
VS Code with an isolated user-data directory, the installed extension directory as
`--extensionDevelopmentPath`, and `scripts/native-vscode-acceptance.cjs` as
`--extensionTestsPath`. The report must say `passed: true`; CLI launch status alone
is insufficient. The JSON evidence records exact paths and hashes for this run.

## Remaining release blockers

1. Canonical extracted-source validation at NeoCLR `954fb2d7` rejects stale legacy
   bootstrap fingerprints for Tasks, Workers, String and Instant. Full regeneration
   with the current compiler/rebuilt bridge stops at generated
   `System.DateTime::.ctor(System.LocalDateTime)`: `initobj Byte` on a field address
   has neither local nor argument provenance in the temporary CLI importer. The
   importer currently admits local/value-constructor receiver initialization only.
   This is a bootstrap bridge limitation, not evidence that native union execution
   failed. Repair and test field-address initialization, then regenerate the complete
   snapshot through the normal tool. No fingerprint was edited to pretend success,
   and failed regeneration published no fragments.
2. The runtime, primitive bootstrap, retained seed, ownership catalog and source-built
   library artifacts used in acceptance have recorded hashes, but have not yet been
   assembled and reproduced as one distributable installation. Finish that gate
   before publishing matching downloads or claiming a clean-source installation.
3. Re-run the canonical source validator after snapshot repair and qualify the final
   bundle. Full-library bootstrap, all-platform qualification and complete API prose
   coverage are not implied by this bounded POC.

The source notice audit now validates **79 registry dependencies and 156 notice
files**. Fifteen missing dependency records were repaired in `954fb2d7`.
The first fresh Rust build selected an incompatible CommandLineTools SDK. A
command-local `DEVELOPER_DIR` and `SDKROOT` selecting Xcode's MacOSX26.2 SDK allows
libffi to build; no global developer setting changed. The initial debug test run
was deliberately stopped as incomplete and restarted with optimized all-target tests.

The optimized all-target run completed across 236 targets: **1,782 passed, two
failed, none ignored**, exit 101. Both failures were stale assertions, not runtime
execution defects: the existing reflection setter admits no-result service calls,
and the fault-service rejection diagnostic had changed. Commit `76e970a0` corrects
those expectations without changing runtime behavior, retains rejection of invalid
signatures and adds a no-result check-service rejection control. The two corrected
files were copied into the extracted source; the focused rerun passes all **10 tests**.
Unaffected results were reused rather than rerunning the full suite. This does not
turn the failed canonical snapshot audit into a passing source-release report.
