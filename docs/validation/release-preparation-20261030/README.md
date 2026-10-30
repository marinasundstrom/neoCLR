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
| Source archive | First attempt fails before tests at SDK/linker setup; see below. Separate SDK 26.5 optimized full-suite run started. |
| Format / strict Clippy | Formatting repaired; needless test borrows repaired after Clippy diagnostics. Strict Clippy and formatting now pass; seven lookup tests pass. |
| API reference | `build-api-docs.py --check` and four native snapshot tests pass on the candidate inputs. |
| Validator controls | Four Windows scalar evidence tests and six HTTP readiness/process cleanup tests pass. |
| macOS SDK | Fresh matching SDK packaging started; no extracted package qualification yet. |
| Windows | Prior scalar run fails at compiler build; stale pins updated. Fresh host/package execution remains required. The workflow now includes real-file watcher and configuration reload checks against extracted package binaries, retaining their reports even on failure. |
| Libraries / consumers | Reuse [optional-argument source evidence](../runtime-defaults-20261030/README.md) and other linked feature receipts as focused evidence only. Fresh package consumers remain required. |
| Editor / HTTPS | Matching extracted SDK/VSIX completion, hover, recovery and HTTPS acceptance required on both targets. |
| Performance | Fresh package release measurements required; existing development experiments are not release measurements. |
| Website / notices | 25 of 26 website tests pass; stale Serve sidecar expectation repaired and its focused rerun passes. Two package archive tests pass; release website review/build, complete bundled dependency notices and final artifact hashes remain required. |
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
