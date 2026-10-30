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
| Windows | Prior scalar run fails at compiler build; stale pins updated. Fresh host/package execution remains required. The workflow now includes real-file watcher and configuration reload checks against extracted package binaries, retaining their reports even on failure. |
| Libraries / consumers | Reuse [optional-argument source evidence](../runtime-defaults-20261030/README.md) and other linked feature receipts as focused evidence only. Fresh package consumers remain required. |
| Editor / HTTPS | macOS editor acceptance (26 checks), inlays, two ten-cycle configuration recovery runs and all 15 HTTPS/cleartext cases pass. Windows acceptance remains open. |
| Performance | Fresh package release measurements required; existing development experiments are not release measurements. |
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
