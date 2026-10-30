# Preview 19 preparation — 2026-10-30

Status: **candidate preparation; not approved or published**. The author requested
“Pursue a new release”. Preview 18 remains the current published release.
[Draft notes](../../next-preview-release-notes.md) define the proposed bounded scope.
This record owns the current release gates; older release/tracker checkpoints are
historical evidence only.

## Candidate inputs

- Initial committed neoCLR source: `3c54970c36791c43af54f2af362f74f2ee57c5de`.
- Matching Raven `codex/neoclr`: `6ceeb1be8790e105a1394cb99fded41fafad5b4c`.
  Its separately validated main fixes were deliberately carried; no full merge.
- Candidate SDK/editor version: `0.1.19-neoclr.20261030.1`.
- Targets: macOS ARM64 and Windows x64. macOS PostgreSQL remains subject to its
  matching package gate. Neo.Signals remains a separate source experiment.
- Local checkout initially had only untracked
  `docs/experiments/runtime-handoff/dotnet/obj/`; source archives exclude it.
- At preparation start, neoCLR main was 11 commits ahead of origin/main and the
  remote Raven branch was `34f14c8ce`. At the author’s request, Raven main `83e700ab6` and codex/neoclr `6ceeb1be8`
  were pushed separately. neoCLR `d590e9f7a` is committed and synced.
  [Canonical/host CI](https://github.com/marinasundstrom/neoCLR/actions/runs/40986852173)
  and [Windows toolchain qualification](https://github.com/marinasundstrom/neoCLR/actions/runs/40986852134)
  are running; old green runs do not qualify the candidate.

## Gates

| Gate | Current evidence / remaining work |
| --- | --- |
| Source archive | Canonical source CI passes on c1b817ce6 and bb631c0e8 after the repairs below; initial local failures remain preserved. |
| Format / strict Clippy | Formatting repaired; needless test borrows repaired after Clippy diagnostics. Strict Clippy and formatting now pass; seven lookup tests pass. |
| API reference | `build-api-docs.py --check` and four native snapshot tests pass on the candidate inputs. |
| Validator controls | Four Windows scalar controls, seven HTTP readiness/diagnostic controls, bundle diagnostic control and async diagnostic control pass. |
| macOS SDK | Matching SDK/VSIX and all 14 libraries built. Extracted default-JIT/HTTP package, watcher/reload, async (24), headers (132), PostgreSQL and standalone build-kit checks pass. |
| Windows | Scalar, HTTPS, security and watcher checks pass on the revisions documented below. Configuration reload exposes a missing native file-input binding; a bounded implementation now awaits a freshly rebuilt package and target-host checks. Editor acceptance remains open. |
| Libraries / consumers | Reuse [optional-argument source evidence](../runtime-defaults-20261030/README.md) and other linked feature receipts as focused evidence only. Fresh package consumers remain required. |
| Editor / HTTPS | macOS editor acceptance (26 checks), inlays, two ten-cycle configuration recovery runs and all 15 HTTPS/cleartext cases pass. Windows acceptance remains open. |
| Performance | Five-round macOS stages pass as loaded-host qualification only (see stage index below). Windows measurements await a passing package. No controlled performance gain is claimed. |
| Website / notices | 25 of 26 website tests pass; stale Serve sidecar expectation repaired and its focused rerun passes. Two package archive tests pass. The repaired full website build checks 6,714 pages. NuGet and npm notice audits pass. Candidate archive/VSIX hashes are recorded; published asset identities remain unset. |
| Publication | No tag, GitHub release, upload or website deployment. Publication date unset. |

## Preserved failures and repairs

The [first source report](source-sdk27-failure.json) records archive identity,
notice audit and the failed optimized Cargo command. Installed clang/ld could not
read macOS 27 SDK `.tbd` architecture `arm64e.x1`; no tests ran. The independent
retry explicitly selects `/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`.
This is a host toolchain selection, not a runtime change or qualification waiver.

[Prior Windows scalar run](https://github.com/marinasundstrom/neoCLR/actions/runs/40953830066)
used neoCLR `bc3b3df53` and Raven `71cafd353`. Its
[retained build log](prior-windows-raven-build.log.gz) reports CS0266 in
`NativeFieldSymbol.cs`: old compiler code expects `int?` but the metadata API now
returns `object`. The current matching compiler already handles that API. Update
scalar/toolchain and HTTP/HTTPS rebuild pins to the selected revision; this fixes
input selection but does not establish passing Windows execution.

Formatting repairs touch layout/module ordering only. Clippy repairs remove
redundant references in existing lookup/retention tests without changing behavior.
The full source run identifies the initial committed snapshot, not subsequent
preparation edits; final evidence must identify their delta explicitly.

## Reproduction

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-release.py \
  --revision 3c54970c36791c43af54f2af362f74f2ee57c5de \
  --distribution native --toolchain stable --release --output /fresh/source-output
```

Follow [source validation](../../next-preview-validation.md) and the
[package procedure](../../experiments/raven-target/RELEASING.md). Use exact fresh
artifact identities and retain failures separately. Do not promote development
claims or close gates solely because source builds or packaging complete.

## Subsequent gate repairs

The full local source run exposes seven archived Raven reflection failures with
`reflection snapshot field count mismatch`. The test-only adapter now includes
RuntimeMethodInfo's function-signature field; generated historical snapshots and
production runtime behavior are unchanged. [All 26 reflection tests pass](reflection-fixed.log.gz)
with the repair. Preserve the original full-run failures; the final committed-source
CI must still pass.

Hosted canonical Clippy rejects the newly deprecated `AtomicI64::fetch_update`.
A statement-local allowance retains Rust 1.85 support, whose standard library lacks
its replacement. [Provider Clippy passes](watcher-clippy.log.gz); no allocation,
ordering or overflow behavior changes. All three hosted minimum-Rust jobs pass on
`1f22baee9`; hosted macOS host checks also pass.

Website repairs align the callback excerpt with backtick names, deduplicate shared
manual pages, keep Signals in reference navigation, add watcher guide navigation,
remove a duplicate function-reflection anchor and escape generic signatures in
Signals headings/link labels. [The final full build](website-build-clean.log.gz)
checks 6,714 pages. Existing description-coverage warnings remain visible; a website
build is not publication.

[Fresh macOS watcher evidence](macos-filewatch.json.gz) uses the packaged compiler
and rebuilt libraries. Both native and interpreter lifecycle consumers and the
native ownership harness pass. This is a focused base-package check; default-JIT
archive/extract, editor, HTTPS and release measurements remain separate gates.

## Extracted package and source outcomes

The [initial optimized full source run](source-full-initial.log.gz) completes 273
test targets with 2,293 passing tests and seven archived reflection failures. All
seven belong to the 26-test suite passing after the adapter repair. Do not describe
the original source run as passing or smoke-qualified: validation stops before
smokes when the full tests fail. Fresh canonical CI runs on synced `ef334fa1d`.

The [fresh macOS archive](macos-package.json.gz) passes default-JIT sample execution
and both HTTP request/client-server cohorts; all 2,590 manifest files verify.
The initial acceptance attempts rejected documented retention warnings despite
correct exit status/output. Updated validators retain those diagnostics in reports:
ordinary JIT samples admit only the exact DAM2004 warning; HTTP admits bounded
DAM2001 requirements 512/513 only with supplied explicit reflection roots. Unknown
warnings, runtime faults and wrong stdout/status still fail. Eight focused validator
checks pass. The archive's binaries/libraries/SDK/VSIX are unchanged by this tooling
repair. Final distribution hashes and remaining package gates are still required.

[Configuration reload](macos-reload.json.gz) passes native/interpreter real-file
changes, malformed settings, replacement, deletion, optional sources and disposal.
The SDK notice audit covers 27 NuGet dependencies and verifies all 35 package notice
sets. This does not substitute for the separate VSIX JavaScript dependency review.

[Windows package run 40987072316](https://github.com/marinasundstrom/neoCLR/actions/runs/40987072316)
builds the SDK, libraries and default-JIT archive, then fails the old collections
validator on the same DAM2004 warning (exit zero, exact expected stdout). Its
[failed receipt](windows-package-warning-failure.log.gz) remains separate from
future qualification; watcher/reload/editor stages did not execute in that run.


## Continued qualification

Canonical source CI passes on [c1b817ce6](https://github.com/marinasundstrom/neoCLR/actions/runs/40989617699)
and the subsequently synced [bb631c0e8](https://github.com/marinasundstrom/neoCLR/actions/runs/41034240950).
These supersede the earlier pending CI status above; the failed local baseline
remains preserved. The website workflow on ef334fa1d also passes.

[macOS installed editor acceptance](macos-editor.json.gz) passes all 26 checks,
including reference replacement/recovery, unsaved edits, task execution and await.
[HTTPS acceptance](macos-https.json.gz) passes with Homebrew Python/OpenSSL,
including TLS 1.2/1.3, malformed connections, cancellation and credential rejection.
The system Python lacks TLS 1.3 and cannot run this gate. Separate editor inlay,
repeated configuration recovery and cleartext isolation remain to be checked.

[Windows run 40989683340](https://github.com/marinasundstrom/neoCLR/actions/runs/40989683340)
passes extracted package acceptance and GC, then stops at async cleanup JIT:
[the receipt](windows-await-warning-failure.json.gz) records exit zero and expected
PASS output with only DAM2004. The async validator now accepts that exact warning
only for JIT, retaining raw stderr; its focused test rejects other diagnostics and
rejects the warning for AOT/interpreter. Remaining Windows stages require a rerun
against the same packaged artifacts with this validator repair.


The [inlay acceptance](macos-inlays.json.gz) passes. The complete
[HTTPS/cleartext receipt](macos-https-cleartext.json.gz) passes all 15 cases,
including isolation of ordinary cleartext hosts. The
[production npm audit](npm-notices.json.gz) finds the same eight package versions
covered by the preserved notices; rebuilding the production input graph produces
exactly the SHA-256 of the installed VSIX JavaScript. Both dependency notice audits
are now complete for these artifacts.

Windows requalification [41113298319](https://github.com/marinasundstrom/neoCLR/actions/runs/41113298319)
uses source 7a81940d1 and reuses the archive from 40989683340. This is a validator
repair against unchanged package inputs; no successful outcome is inferred while
that run is pending.


The [first repeated editor recovery run](macos-editor-configuration-1.json.gz)
passes ten invalidation/recovery cycles for both native projects. The
[initial header-contract run](macos-header-warning-failure.log.gz) passes all 44
cases under AOT/interpreter and all JIT responses, but rejects DAM2004 at shutdown.
The shared subprocess checker now accepts an explicitly supplied exact stderr set,
with empty stderr still the default, and returns raw diagnostics for receipts.
Only header and async JIT consumers opt into the exact warning. Focused subprocess
controls verify unexpected diagnostics and nonzero exits still fail.


[All 132 header contracts](macos-header-contracts.json.gz) now pass across
AOT/interpreter/JIT. The [second editor recovery run](macos-editor-configuration-2.json.gz)
also passes ten cycles. [PostgreSQL qualification](macos-postgresql.json.gz) passes
contracts, live queries in all three modes and all ten protocol cases; the disposable
loopback-only database was removed afterward.

The [initial extracted kit failure](macos-kit-missing-include.json.gz) identifies a
missing `heap-adoption.inc` during hello compilation. Package support selection and
build fingerprints now include `.inc` files alongside C sources/headers, covering
all three current native fragments. A rebuilt kit requires fresh extracted acceptance.
One concurrent [async attempt](macos-async-source-drift.json.gz) stopped because
this builder edit changed its inputs during compilation, after nine passing cases;
it is not a passing complete run. A fresh run starts after the source edit settles.

Fresh fetches verify zero ahead/behind commits for neoCLR main, Raven main and
Raven codex/neoclr. Windows run 41113298319 was superseded by
[41113778505](https://github.com/marinasundstrom/neoCLR/actions/runs/41113778505)
with the streamed diagnostic fix; package binaries remain unchanged.


The [repaired extracted kit](macos-build-kit.json.gz) passes console/HTTP execution,
HTTP fault propagation and all four tamper controls, with Cargo absent from PATH,
no checkout inputs and standalone execution in an empty environment. The fix changes
packaging/fingerprinting only; native runtime fragments themselves are unchanged.


The [settled-source async run](macos-async.json.gz) passes all 24 cases across
AOT/interpreter/JIT. [Extracted native interop](macos-kit-interop.json.gz) passes
native executable/library consumers and tamper rejection. Candidate archive/VSIX
identities are retained in [the artifact inventory](candidate-artifact-hashes.json);
these are qualification candidates, not published release assets. Five-round macOS
release measurements are now running after local qualification builds finish.


The initial release measurement [report](macos-benchmark-warning-failure.json.gz)
and [log](macos-benchmark-warning-failure.log.gz) stop at the first JIT worker:
exit zero and exact measurement protocol, with DAM2004. Execution and HTTP benchmark
callers now explicitly admit only that exact warning in JIT mode and retain stderr
in every sample. Shared subprocess controls still reject nonzero exits and unknown
messages; timing/workload/output checks are unchanged. A fresh five-round measurement
run uses this harness repair; the failed attempt is not performance evidence.


Windows run 41113778505 passes package acceptance, GC, [all 24 async cases](windows-runtime-await.json.gz),
JSON and all 132 headers. Watcher compilation/linking succeeds, but
[dependency admission](windows-watcher-dependencies.log.gz) rejects the Rust provider's
Windows system DLLs. The builder now allows `bcryptprimitives.dll`,
`api-ms-win-core-synch-l1-2-0.dll` and `ntdll.dll` when file watching is linked, as it
already does for security providers; unrelated DLLs remain rejected. Runtime source
and provider binaries are unchanged. Watcher/reload/editor execution remains open.

Windows HTTPS [41115276438](https://github.com/marinasundstrom/neoCLR/actions/runs/41115276438)
uses the matching candidate archive. Its [JIT diagnostic failure](windows-https-warning-failure.log.gz)
has a successful TLS 1.3 protocol exchange, exit zero and exact output. The HTTPS
validator now admits only DAM2004 in JIT mode, with focused negative controls.
The earlier automatic workflow used an older default archive without `System.Panic`;
that mismatched run does not qualify this candidate.

MacOS measurement stages execution, longer native CPU and plain HTTP pass five
rounds. JSON HTTP stops on [documented DAM2001/2004 diagnostics](macos-http-json-warning-failure.log.gz).
Its harness now reuses the existing bounded HTTP diagnostic policy only when explicit
reflection roots are supplied, hashes that policy source, and preserves per-sample
stderr. Only the JSON HTTP stage is repeated; passing measurement stages are retained.


All macOS measurement stages now pass five rounds; the
[stage index](macos-benchmark-stages.json) links the unchanged execution/CPU/plain
receipts and the separately rerun JSON HTTP receipt. The original aggregate failure
is preserved. These are shared-development-host measurements: unrelated work stayed
active and a scalar-test compile overlapped JSON HTTP. Do not treat their timing
ranges as controlled regression evidence or claim a throughput improvement.

The older [Windows scalar log](windows-scalar-initial.log.gz) exposes stale tests:
fresh Raven outputs NEOX envelopes, while the helper decodes only PE; optimized
scalar bodies need not contain relocations. The relocation assertion also
[fails locally](macos-scalar-baseline.log.gz). Tests now decode both transports and
require relocation only for the external console adapter, preserving executable
output/fault checks. The [Windows debug security run](windows-security-initial.log.gz)
reports compiler exit 3221225725 (stack overflow). The debug-only Windows CLI entry
now uses an 8 MiB compiler worker, matching the existing lazy compiler worker budget;
optimized entry behavior is unchanged. Fresh target-host checks remain required.


[Windows HTTPS project qualification](windows-https-project.json.gz) and its
[protocol receipt](windows-https.json.gz) pass on
[41116132713](https://github.com/marinasundstrom/neoCLR/actions/runs/41116132713),
using the same candidate archive with corrected JIT diagnostic acceptance.


[Local scalar rerun](macos-scalar-fixed.log.gz) passes all three cross-host COFF
checks; four Windows execution cases remain explicitly ignored on macOS and must
pass in the target-host workflow. Formatting passes. These checks do not establish
the Windows debug worker outcome; its security workflow is the executable gate.


[Windows scalar qualification](windows-scalar-fixed.json.gz) passes on
[41117251029](https://github.com/marinasundstrom/neoCLR/actions/runs/41117251029),
including all fresh Raven/native execution cases. The debug security compiler now
gets past stack exhaustion; [its next failure](windows-security-missing-exports.json.gz)
is an outdated fixture DLL export list. Both crypto and TLS fixtures now export
all four existing GC state-handle entry points; no native ABI is added.

Windows package run 41116128327 again passes runtime contracts and now builds the
watcher executable with admitted system DLLs. Its [ownership harness link](windows-watcher-harness-failure.log.gz)
reveals that the validator replaced both the HTTP entry and Windows console adapter.
Only the HTTP entry is now substituted; the console adapter stays linked. Two
focused command-construction tests pass, including missing/duplicate entry rejection.
The workflow moves watcher/reload/editor gates before repeated runtime contracts;
all gates are still required, with unchanged package binaries.


## Windows file-input release gap

Package [41118644109](https://github.com/marinasundstrom/neoCLR/actions/runs/41118644109)
passes [watcher native/interpreter/ownership checks](windows-watcher-fixed.json.gz),
then fails configuration reload because Windows does not bind Runtime.ReadAllText.
This is a real native capability gap, not a diagnostic-policy failure.
The reader now uses strict UTF-8 path conversion, shared Win32 file handles and
the existing bounded regular-file/error/decode contract. File output remains
unsupported. The default JIT selects file input on both platforms and the existing
file-watcher binding; matching runtime, compiler and native hosts must be rebuilt
together. Watcher/reload validators now require native, interpreter and JIT execution.

[Local C contract checks](macos-file-input-c.log.gz) pass for rooted/unrooted
reads, Unicode paths, BOM/NUL preservation, byte limits, error precedence, allocation
failure and cleanup. [Windows-target codegen checks](macos-windows-file-input-codegen.log.gz)
pass ten cross-host cases including opt-in enforcement and the expected COFF import;
four Windows execution tests are correctly ignored on macOS. The C fixture also
contains Windows device, sharing-denial and handle-count checks, which remain
pending target-host execution. Old package receipts do not qualify these new binaries.

The Windows job in [security run 41118644286](https://github.com/marinasundstrom/neoCLR/actions/runs/41118644286)
passes [cryptography](windows-security-crypto-fixed.json.gz) and
[TLS bindings](windows-security-tls-fixed.json.gz) after the fixture export repair.
The macOS job was cancelled; this is explicitly Windows evidence, not an all-platform
workflow pass.

[Six interpreter file-input cases and the default-JIT binding test](macos-file-input-root.log.gz)
pass, as do [both native/interpreter parity and exact opt-in tests](macos-file-input-parity.log.gz).
The API snapshot and Rust formatting checks pass.


## Refreshed macOS candidate and Windows rebuild

Source fix `6edc02db00a386201a61f1bc95f7ebb9e3892029` is committed and pushed.
The rebuilt runtime/backend/helper passes [watcher ownership and native/interpreter/JIT
lifecycles](macos-filewatch-three-modes.json.gz), plus [configuration reload in all
three modes](macos-reload-three-modes.json.gz). The matching
[extracted standalone kit](macos-build-kit-file-input.json.gz) passes console/HTTP
and negative controls. Updated archive hashes are in the candidate inventory.
SDK, libraries and VSIX inputs are unchanged; earlier editor, PostgreSQL, TLS and
measurement receipts identify their original binaries and are not relabeled as
measurements of the refreshed runtime.

Fresh Windows [package run 41121692409](https://github.com/marinasundstrom/neoCLR/actions/runs/41121692409)
builds all components from this source; it does not reuse the older archive.
Target-host file-input, three-mode watcher/reload, installed editor and package gates
remain pending. Final version promotion, matching Windows measurements and asset
identities remain open before publication.

[Archive/extract default-JIT acceptance](macos-package-file-input.json.gz) passes
2,590 manifest files, 12 commands and the HTTP/JSON exchange on the refreshed macOS
candidate. No tag, release upload or website deployment has been performed.


## Target-host reader and validation harness follow-up

[Windows GC/native-input contracts](windows-file-input-gc.json.gz) pass in
[41121685095](https://github.com/marinasundstrom/neoCLR/actions/runs/41121685095),
including the new Unicode, strict decoding, limits, sharing-denial, device rejection
and repeated handle-cleanup fixture. The workflow nevertheless fails its separate
[relocation test](native-network-qualification-failure.log.gz): copied collector
sources need the canonical GC include directory. Diagnostic instrumentation also
still names pre-consolidation GC fields. The helper now supplies that include path,
uses the current stack boundary and reads allocation counts through the existing
statistics accessor. [Local relocated/instrumented builds pass](macos-gc-layout-fixed.log.gz).

The macOS network receipt fails a synthetic runnable with order zero. Real queue
entries have nonzero submission order; the fixture now uses order one, retaining
the assertion that ready work must not wait. [Focused sanitized execution passes](macos-readiness-fixed.log.gz).
Runtime-product qualification stopped at an outdated workspace inventory; its
explicit expected set and input hashes now include the watcher provider. These
repairs change harnesses, not candidate runtime behavior.

The independent Windows console-project workflow still downloads Preview 13 and
fails native metadata admission when paired with the current backend. It does not
qualify the Preview 19 candidate; a matching-package rerun remains required.


The [old Windows console-project failure](windows-console-old-package-failure.log.gz)
is preserved. That validator now requires an explicit extracted bundle, verifies
its manifest and uses its catalogued backend instead of downloading Preview 13
and mixing in a checkout build. Console-project and HTTPS workflows select
package run 41121692409, wait for successful native-toolchain qualification, and
retain the selected run metadata. A failed package stops dependent checks.
Workflow YAML parses and the validator's CLI/bytecode checks pass; actual Windows
project/HTTPS execution remains pending the package gate.
