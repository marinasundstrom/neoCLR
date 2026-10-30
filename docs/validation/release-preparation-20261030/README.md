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
| Source archive | Full canonical validation, strict Clippy, host boundaries and all minimum-Rust jobs pass on 8595dea1f (run 41123751866). Later fixture-only deltas have focused checks below; initial failures remain preserved. |
| Format / strict Clippy | Formatting repaired; needless test borrows repaired after Clippy diagnostics. Strict Clippy and formatting now pass; seven lookup tests pass. |
| API reference | `build-api-docs.py --check` and four native snapshot tests pass on the candidate inputs. |
| Validator controls | Four Windows scalar controls, seven HTTP readiness/diagnostic controls, bundle diagnostic control and async diagnostic control pass. |
| macOS SDK | Matching SDK/VSIX and all 14 libraries built. Extracted default-JIT/HTTP package, watcher/reload, async (24), headers (132), PostgreSQL and standalone build-kit checks pass. |
| Windows | Fresh package run 41121692409 passes all extracted-package, editor, watcher/reload and SDK contracts on 6edc02db0. Matching HTTPS passes 27 cases. Standalone console passes stream/record consumers; compact fault parity required the final runtime correction below. Final package run 41130221734 and its dependents are pending. |
| Libraries / consumers | Reuse [optional-argument source evidence](../runtime-defaults-20261030/README.md) and other linked feature receipts as focused evidence only. Fresh package consumers remain required. |
| Editor / HTTPS | macOS editor acceptance (26 checks), inlays, two ten-cycle configuration recovery runs and all 15 HTTPS/cleartext cases pass. Windows editor acceptance, inlays and two recovery runs pass; matching HTTPS passes 27 cases. |
| Performance | Five-round macOS stages pass as loaded-host qualification only (see stage index below). Windows five-round measurements also pass on package 41121692409. Exact final-version binary refreshes are queued/running; prior receipts are not relabeled. No controlled performance gain is claimed. |
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


[Native network/GC run 41123011320](https://github.com/marinasundstrom/neoCLR/actions/runs/41123011320)
passes both target hosts on committed harness revision `40fbef8166529ed9cd4204490fc6b8b178f31253`:
[macOS network](macos-native-network-validation-fixed.json.gz),
[macOS GC](macos-native-gc-validation-fixed.json.gz),
[Windows network](windows-native-network-validation-fixed.json.gz) and
[Windows GC](windows-native-gc-validation-fixed.json.gz). Relocation/instrumented
source checks also pass in both jobs.

The [local runtime-product run](macos-runtime-product-fixed.json.gz) passes workspace
build, CLI, hosting, inline values, scalar execution, compiled roots, native GC,
security, relocation, notices and HTTPS build controls. It started from `052f2a36e`
with the harness repairs then committed as `40fbef816`; it is local delta evidence,
not a clean-source run of that original revision. The receipt records validator
and workspace manifest hashes. Hosted runtime-product run 41123011225 remains
the clean-source counterpart.

Fresh fetches confirm neoCLR main, Raven codex/neoclr and separate Raven main are
all 0 ahead/0 behind their upstreams. No Raven branch merge was performed.


## Refreshed measurements and record-test repair

The [five-round aggregate](macos-refreshed-release-benchmarks.json.gz) passes against
the refreshed macOS archive (candidate3), with separate
[execution](macos-refreshed-execution.json.gz),
[longer native CPU](macos-refreshed-cpu-native.json.gz),
[plain HTTP](macos-refreshed-http-plain.json.gz) and
[JSON HTTP](macos-refreshed-http-json.json.gz) receipts. All bundle hashes remain
unchanged throughout measurement. Unrelated Aspire, browser and VM activity remained
active; host load was substantial. These establish workload qualification only,
not an isolated regression comparison or throughput improvement. Further local
compiler/test builds were deferred until this run completed. Source-status records
include unrelated fixture/evidence edits; no benchmark or candidate binary was changed.

[Windows scalar run 41121685230](https://github.com/marinasundstrom/neoCLR/actions/runs/41121685230)
passes the [updated suite](windows-scalar-file-input.json.gz), including the explicit
file-input binding control. Hosted runtime-product qualification passes both
[macOS](macos-runtime-product-hosted.json.gz) and
[Windows](windows-runtime-product-hosted.json.gz) on `40fbef816`.

The collections workflow [fails an obsolete rejection](windows-collections-stale-rejection.log.gz):
generated record equality, reference records and mutable positional components
were already integrated. Replace those exclusions with an executable consumer
covering equal/unequal values, reference-record equality and mutable-value copying.
[Native/interpreter/JIT execution and the remaining read-only-property rejection](macos-record-contracts.json.gz)
pass with the matching candidate; [build provenance](macos-record-contracts-build.json.gz)
identifies source and binaries. Windows console-package qualification includes the
same consumer; its target-host result remains required. Historical evidence is unchanged.

The standalone console validator still requires explicit matching package input.
Its old bootstrap constants remain available solely for existing development
validators that rebuild compiler/libraries from source; preserving those imports
avoids breaking their separate bootstrap path.


The refreshed PostgreSQL package check passes mapper contracts and live JIT/interpreter
queries in its [initial receipt](macos-postgresql-refreshed-initial.json.gz). Its AOT
link selects the incompatible installed macOS 27 SDK and fails before execution.
A [focused SDK 26.5 continuation](macos-postgresql-refreshed-aot.json.gz) verifies all
unchanged input hashes, relinks the already emitted object and passes the live AOT
consumer. The original failure remains unchanged. Unchanged hostile-wire cases reuse
the earlier recorded coverage. The disposable PostgreSQL container was stopped/removed.

Windows [package run 41121692409](https://github.com/marinasundstrom/neoCLR/actions/runs/41121692409)
now passes all required package, three-mode watcher/reload, installed-editor and
SDK runtime gates. Its exact source remains `6edc02db0`; later changes are validation
and evidence updates. Dependent HTTPS, standalone console and benchmark workflows
consume that successful archive.


Canonical [source CI 41123751866](https://github.com/marinasundstrom/neoCLR/actions/runs/41123751866)
passes all jobs on `8595dea1f46da4c3cd95974d418d7e1ad2e697a1`, including strict
Clippy, full source validation, both host boundaries and all three Rust 1.85 checks.

The refreshed Windows archive verifies [2,585 files and extracted samples](windows-refreshed-package.json.gz).
Retained package receipts pass [GC](windows-refreshed-gc.json.gz),
[watcher](windows-refreshed-watcher.json.gz), [reload](windows-refreshed-reload.json.gz),
[24 async cases](windows-refreshed-await.json.gz), [JSON](windows-refreshed-json.json.gz),
[132 headers](windows-refreshed-headers.json.gz), [26 editor checks](windows-refreshed-editor.json.gz),
[inlays](windows-refreshed-inlays.json.gz) and two configuration-recovery runs
([first](windows-refreshed-config-1.json.gz), [second](windows-refreshed-config-2.json.gz)).
Matching [HTTPS project qualification](windows-refreshed-https-project.json.gz) and
[27 protocol cases](windows-refreshed-https-protocol.json.gz) pass on
[41123260046](https://github.com/marinasundstrom/neoCLR/actions/runs/41123260046).

The independent HTTP-client [failure](windows-http-client-mode-failure.log.gz)
selected default JIT while requesting interpreter limits. Its shared interpreter
arguments now explicitly select that mode; the retained Windows-built task-factory
consumer [passes the focused interpreter check](macos-http-client-mode-fixed.json.gz).

The matching standalone console [failure](windows-console-stream-fixture-failure.log.gz)
is an obsolete source fixture calling removed synchronous offset/count overloads.
Input/output fixtures now use borrowed slices and unit-valued full writes. Empty
operations, partial input, NUL output, flush and closure failures remain covered;
invalid view bounds retain their existing memory-view test coverage.
[All six native/interpreter/JIT cases pass locally](macos-console-streams-current.json.gz),
with [input](macos-console-input-build.json.gz) and [output](macos-console-output-build.json.gz)
build provenance. Windows target execution of these repaired fixtures remains required.


## Final runtime diagnostic correction and versioned candidate

The repaired Windows console run passes record, managed, interpolation and both
stream consumers, then [fails strict fault-text parity](windows-fault-display-mismatch.log.gz).
Native output names `Divide` and `Main`; compact interpreter diagnostics expose
reserved encoded metadata helpers. `FaultDiagnostic` now uses the same existing
diagnostic-name decoder as guest stack capture. Captured identities and legacy
detailed formatting remain unchanged; malformed/unrecognized names remain literal.
This is a runtime presentation correction, not permission to normalize mismatched
errors in the validator.

[28 focused CLI/fault/stack tests](macos-fault-display-tests.json.log.gz) pass. The
[retained Windows-built fault consumer](macos-fault-display-native-parity.json.gz)
now matches the recorded Windows native diagnostic byte for byte when interpreted
by the rebuilt macOS runtime. A fresh Windows package must verify the same fix.
Compiler behavior, Runtime Contract configuration and native ABI are unchanged.

The runtime candidate is now assigned `0.1.0-preview.19`; SDK/editor version remains
`0.1.19-neoclr.20261030.1` with Raven `6ceeb1be8`. Publication date and release assets
remain unset. Earlier package/measurement receipts remain valid for their explicitly
recorded inputs, but do not masquerade as receipts for this final CLI rebuild.

The final versioned workspace release build passes with SDK 26.5. The CLI has no
`--version` command; its package version is verified through the locked Cargo
manifest and build receipt rather than treating the usage response as a version.

[Windows five-round benchmarks](windows-refreshed-benchmarks-release-benchmarks.json.gz)
pass on [run 41122439416](https://github.com/marinasundstrom/neoCLR/actions/runs/41122439416),
with retained [archive hash](windows-refreshed-benchmarks-archive-hash.json.gz),
[package selection](windows-refreshed-benchmarks-package-run.json.gz) and
[source revisions](windows-refreshed-benchmarks-source-revisions.json.gz). These
measure the preceding qualified package; the compact fault formatter and version
bump do not change the measured execution or HTTP paths.

Two independent development workflows still need investigation: HTTP-client run
41127754155 passes greeting, numeric, fragmented, malformed, refused and timeout
cases before a pre-cancellation parity mismatch; collections run 41127754028
passes collection consumers but fails to build the runtime test method-state
fixture. Neither failure is being counted as a passing release check.

The HTTP-client pre-cancellation mismatch is also encoded versus source-friendly
fault names ([original report](windows-client-cancellation-failure.json.gz)).
The final versioned runtime reproduces the Windows native diagnostic exactly
using the retained Windows-built consumer
([focused receipt](macos-cancellation-display-fixed.json.gz)).

The method-state build [fails before code generation](windows-method-helper-missing.log.gz)
because its older bootstrap bundle lacks `tools/run-native-sample.py`. Development
bundle preparation now includes and fingerprints the source-matched reflection
resolver; six existing helper contract tests pass. This does not migrate fixtures
or alter runtime behavior.

Final Windows package rebuild [41130221734](https://github.com/marinasundstrom/neoCLR/actions/runs/41130221734)
uses runtime source `a079cffdffea9d9c69f7eb8c55a7b24981281c72`. Console/HTTPS
workflow defaults now select that run and wait for its success before checking it.

Final macOS [method-state tests](macos-final-method-state.json.gz) pass native and
interpreter execution with both discovered tests and their selection controls.
The first extracted final-package check reaches HTTP startup but times out while
local build work is concurrent; retained [HTTP diagnostics](macos-final-http-start-timeout.json.gz)
and [outer failure log](macos-final-package-timeout.log.gz) remain unchanged. No
passing status is inferred from that attempt. A retry of the same archive after
local builds finish is required.

Subsequent Windows development checks identify the second bootstrap mismatch:
its Preview 13 runtime cannot read `before_field_init` in current metadata
([failure](windows-development-old-runtime.log.gz)); reflection/JSON fails identically.
Preparation now builds the source runtime and includes it in the verified catalog,
using it for library preparation and reflection. Two workspace/staging contract
tests pass, including keeping the original bootstrap unchanged. The JIT crypto
[report](windows-crypto-warning-report.json.gz) has all three tests passing, exit zero
and only the exact documented DAM2004 warning. The runner now accepts that exact
warning in JIT mode only; four diagnostic controls reject unexpected/mixed errors
and retain strict native/interpreter diagnostics. These are harness repairs, not
post-release behavior-fixture migration.

The macOS startup [diagnostic comparison](macos-final-startup-comparison.json.gz)
finds final startup at 149 seconds and previous-package startup at 75 seconds on
the loaded host. Both exceed the unchanged 60-second gate; this sequential pair
is not a controlled regression measurement. A [focused HTTP retry](macos-final-http-focused-retry.json.gz)
passes the server-request exchange, then times out starting the paired server.
The final macOS package gate remains unresolved; timeouts have not been relaxed.

Windows HTTP-client run 41130576934 now passes pre-cancellation and cancellation
parity but exposes a continuation-fault parity mismatch, under investigation.

The continuation-fault [retained report](windows-client-callback-frames.json.gz)
shows identical five inner frames and two additional native active entry callers.
This matches the existing active-frame contract and documented causal async-stack
stitching gap. The validator now permits only that exact suffix for callback faults;
ordinary/cancellation cases remain byte-equal. The retained report and five negative
comparison controls pass; target execution of the remaining recovery cases is pending.

The [final macOS standalone build kit](macos-final-build-kit.json.gz) passes console
and HTTP qualification, including the validator's negative controls. Its archive
hash is in the inventory. Packaging records source `4ca2b990b` with documentation
edits in progress and explicit source hashes; it contains the same final runtime
bytes as the extracted macOS candidate. Five-round measurements of that final
macOS archive are running; Windows benchmark run 41130708329 waits for final
package 41130221734 to succeed.

Windows HTTP-client run 41133279887 passes all initial cases, including the
callback-fault comparison, then stops on a stale `http.start_process` import in
its paired-server path. Calls now select the shared process helper explicitly.
Seven existing readiness/process-tree/diagnostic controls pass with Python 3.14;
an earlier Python 3.9 control invocation hit a duplicate-cleanup PermissionError.
The package validation path now optionally selects verified, catalogued runtime
and backend roles (`--packaged-tools`) and records the manifest hash. The Windows
standalone-project workflow also checks HTTP clients against its same final archive,
avoiding an unnecessary compiler/bootstrap rebuild. macOS execution is in progress.

The standalone cryptography project used obsolete array InternalCalls from before
the public facade. Its updated public API consumer passes [all three macOS modes
and provider-omission control](macos-final-public-crypto-project.json.gz).
The expanded JSON corpus requires socket cleanup bindings in its current retained
stream surface. Its validator now explicitly selects the HTTP profile, preserving
all assertions; macOS [build](macos-final-json-corpus-build.json.gz) and
[native/interpreter execution](macos-final-json-corpus.json.gz) pass. Minimal-host
qualification remains a documented gap; Windows reruns remain pending.
