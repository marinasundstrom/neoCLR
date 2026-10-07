# Native class-library project bundle — 2026-10-07

Development gate, not an installable SDK or full bootstrap completion.
Raven `8fbacaa9f` adds explicit prebuilt project-reference compilation. The tested
compiler binaries were built from that source before committing; exact hashes and
commands are in the [evidence](native-class-library-bundle-2026-10-07.json).

## Build and ownership

The checked-in project graph builds Runtime (175 sources), Data (6), Networking (9,
including its two current native service adapters) and Web (9). Data and Networking
reference Runtime; Web references Data and Networking and imports Runtime transitively.
Runtime has no dependency on those higher-level projects. Platform extraction is open.

```sh
python3 scripts/build-native-class-library.py \
  --compiler /tmp/native-bundle-compiler1007/rvnc.dll \
  --compiler-revision 8fbacaa9f \
  --core /tmp/failure1006b/Core.dll \
  --bootstrap-directory /tmp/array-runtime1007/runtime-owned \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Debug/net10.0/NeoCLR.Metadata.Translate.dll \
  --output /tmp/native-class-library-fresh
```

The directory must be fresh. Runtime builds against the compile-time retained seed;
finalization binds the retained runtime model to that exact artifact. Data, Networking
and Web then build once with `--no-build-references`. Each still resolves the full native
project graph and validates prebuilt identities. Outputs are copied to the staged bundle;
`bundle.json` records their hashes and is written last. The bundle contains four native
libraries, primitive Core, finalized seed and the explicit ownership manifest.

The initial implementation rebuilt Runtime through Web's graph after finalization.
The Runtime hash changed, so the guard rejected publication. The final workflow keeps
the exact compiled snapshot through dependent builds. It does not claim deterministic
assembly output or weaken digest checks. Missing bootstrap input also rejects without
publishing a bundle manifest. Build directories left by failures are incomplete.

## Executed acceptance

Against the staged artifacts, `scripts/verify-separate-web.py` separately compiles,
verifies and executes all five unchanged consumers, with expected stdout and exit 0:

- Network deadline checks.
- HTTP header lookup.
- HTTP base address and overloads.
- HTTP JSON client operations.
- Route parsing.

`verify-native-library-project.py` also compiles/runs the unchanged HTTP headers source
through `rvnc neoclr --project --run`, and checks invalid Object selection preserves its
last successful output. Consumer builds contain no library sources.

The extended `verify-native-project-graph.py` passes both ordinary graph builds and
prebuilt mode. Broken dependency sources do not affect explicit prebuilt consumption;
dependency bytes are unchanged. Removing a required artifact rejects without replacing
the consumer. Existing cycle and dependency binding-failure checks still pass.
No shared .NET symbol/emission changes were needed; ordinary project-system regression
evidence from the preceding slice is reused.

## Remaining release gates

The staged artifacts are not a complete runtime/compiler/editor distribution. Core and
retained bootstrap model remain explicit dependencies. Platform projects, matching API
documentation artifacts, source-link/bundle rendering and live editor acceptance of the
split remain open. Public API signatures are unchanged in this slice; the Raven feature
page records development status. A website build/deployment was not required or performed.
