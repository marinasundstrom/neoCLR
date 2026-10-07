# Relocatable bundle configuration — 2026-10-07

Development evidence for headless workspace import and ordinary compiler execution.
This does not replace installed VS Code acceptance of the split class-library layout.

## Implemented

The build script emits `NeoCLR.ClassLibrary.props` and includes its hash in the bundle
manifest. The configuration supplies four explicit native references, primitive Core,
retained seed, ownership manifest and Runtime Object/async selection. Paths use
MSBuildThisFileDirectory; consumers do not inherit absolute build-machine paths or
library sources. Source-root and bootstrap-intrinsic authoring are disabled.

Raven `e93fcfdc1` includes evaluated .props/.targets imports in native metadata inputs.
This is the project metadata layer used by both the compiler and native-enabled
language server. Ordinary .NET project behavior and metadata formats are unchanged.
The recorded library build used Raven `8fbacaa9f`; workspace/probe/consumer tools include
the changes committed as `e93fcfdc1`. Exact binaries and commands are retained in the
[evidence](native-bundle-configuration-2026-10-07.json).

## Validation

- Built all four class-library projects and staged the matching seed/configuration.
- Copied the bundle into `SDK with spaces` beneath a fresh consumer directory.
- Loaded the unchanged HTTP headers source through a project importing only the props
  file, without bootstrap environment variables or library sources.
- Verified native symbol owners for Runtime Object, Data JsonValue, Networking IPAddress
  and Web HttpClient, and exactly four native references plus one primitive bootstrap.
- Confirmed imported configuration, seed and assembly paths are metadata inputs.
- Rejected an invalid Object owner and a missing Web artifact without publishing a
  partial workspace. Restored inputs match their bundle manifest hashes.
- Compiled/executed the same project through the ordinary driver: exact HTTP header
  output and exit 0. Existing native project/catalog contracts also pass.

Reproduce after building a fresh bundle with `build-native-class-library.py`:

```sh
python3 scripts/verify-native-bundle-project.py \
  --bundle /tmp/native-config-bundle1007 \
  --probe /absolute/path/NeoClrMetadataProbe.dll \
  --compiler /tmp/native-config-compiler1007/rvnc.dll \
  --compiler-revision e93fcfdc1 \
  --runtime target/release/neoclr \
  --output /tmp/native-config-acceptance-fresh
```

The probe must be built with the NeoCLR metadata adapter. The output directory must
be fresh. The harness verifies manifest hashes; the project loader itself uses the
explicit native reference catalog, not a bundle-manifest parser.

## Remaining work

Networking Platform extraction still requires a deliberate service contract: current
raw handle/Value adapters remain internal. No visibility widening was performed.
See the [boundary inspection](package-boundaries-2026-10-07.md#networking-extraction-inspection-2026-10-07).
API-documentation packaging, installed VS Code validation and a complete distribution
remain open. The website development note is updated; public API snapshots are unchanged
and no website build or publication was required.
