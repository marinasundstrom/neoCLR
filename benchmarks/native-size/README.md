# Native output and development installation size

Measure the application assembly, final AOT executable and build-tool installation
as different quantities. An application assembly does not contain the matching
class libraries or execution engine. The AOT executable contains selected program
code and linked runtime services, but uses operating-system libraries. SDK size is
a developer installation cost, not an AOT application's deployment requirement.

The first measurement uses the existing Unicode hello sample in
`tools/native/samples/hello/Main.rvn` and the JSON object-mapping consumer in
`docs/experiments/json-object-mapping/{Mapping,Main}.rvn`. Both programs are executed
and checked before their sizes are recorded. These are two concrete examples, not
an average application size or a comparison with .NET Native AOT.

## Reproduce

Use a matching macOS ARM64 development bundle and a release `neoclr-codegen` build.
Create separate projects for the source sets above, importing
`$(NeoClrBundleRoot)/lib/NeoCLR.ClassLibrary.props`, with `TargetFramework=net10.0`,
`OutputType=Exe` and `AssemblyName=App`. The native build driver supplies the bundle
property. Keep source and output folders separate.

```sh
python3 scripts/build-native-project.py --project work/hello/App.rvnproj \
  --bundle /path/to/bundle --aot target/release/neoclr-codegen --output work/hello-native
python3 scripts/build-native-project.py --project work/json/App.rvnproj \
  --bundle /path/to/bundle --aot target/release/neoclr-codegen --output work/json-native \
  --reflection-type StationReport
python3 benchmarks/native-size/measure.py --bundle /path/to/bundle \
  --backend-revision <backend-commit> --compiler-revision <raven-commit> \
  --sdk-archive /path/to/raven-sdk.tar.gz --bundle-archive /path/to/neoclr-bundle.tar.gz \
  --case hello=work/hello-native --case json=work/json-native --output work/sizes.json
```

The measurement validates the bundle catalogue and native output hashes. Installed
sizes sum logical file lengths, excluding generated caches and filesystem block
rounding. The full bundle includes its SDK, runtime/backend, libraries, sample
sources, editor extension and documentation; these rows overlap and must not be
added together. Archive rows measure the actual compressed input files, not an
estimated compression ratio. External .NET SDKs, Apple tools, Cargo, NuGet caches
and system libraries are excluded. Those are build prerequisites, not contents of
this bundle. No extra strip step is applied to executable outputs.

Retain the JSON report with compiler input hashes, native link dependencies and
build configuration alongside the website table. Size changes require new builds
and measurements; never update the table based on an expected optimization.

## 2026-10-21 development cohort

The [retained report](macos-arm64-20261021.json) uses Raven `328349bf7` (SDK
`0.1.0-dev.20261021`) and backend source `5c493aa10`, on macOS ARM64 with the
macOS 26.2 SDK. This is a development size study, not a published release or HTTP
qualification. The backend uses the unchanged 2,048-function selection budget.
Both executable dependency lists contain only macOS libSystem. The JSON executable
contains approximately 4.16 MB of text code and 5.05 MB of read-only constants;
its size should not be attributed just to symbols or reflection without further
investigation. Mach-O virtual zero-fill regions are not counted as file bytes.
