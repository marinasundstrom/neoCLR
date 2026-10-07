# Clean-source bootstrap reproduction — 2026-10-07

This gate starts from independent Git clones, not copies of development worktree
contents. NeoCLR is pinned to ca34025a2e0af6f167a9063ef1a1e6a981ac6ff9 and Raven to
70aea9a7e9e424159a48b0227869245a95bf2ec2. No bin/obj/target outputs, prepared Core,
retained seed or class-library assemblies are copied from previous qualification.
Normal installed tools and Cargo/NuGet/npm package caches remain host dependencies.

## Host prerequisites and ordering

The tested host is macOS arm64. Its default SDK selection mixes the installed Xcode
linker with a macOS 27 Command Line Tools SDK that it cannot parse. The libffi configure
step fails before runtime compilation. A command-local selection of the installed
Xcode MacOSX26.2 SDK succeeds; no global xcode-select setting is changed:

```sh
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
  cargo build --release
```

That path is a recorded host selection, not a portable hard-coded requirement. Select
an installed SDK compatible with the local compiler/linker. The earlier async-readiness
record documents the same class of mismatch with a different installed SDK version.
The current trial preserves the default-build failure as evidence.

Use canonical absolute paths (pwd -P), especially for macOS /tmp paths. Build the SDK,
then the bootstrap tools, sequentially when they share project outputs. An overlapping
probe/translator build and SDK packaging failed with missing reference assemblies.
A standalone metadata build and serial SDK retry were used to recover; no binary
was copied into a missing ref directory. Concurrency/path aliasing is a suspected
contributor, not an independently established compiler defect.

## Build sequence

After cloning and checking out the revisions above, define NEOCLR and RAVEN as their
canonical paths and use fresh output directories. SDKOUT is outside either checkout.

```sh
cd "$RAVEN"
RAVEN_NEOCLR_METADATA_PROJECT="$NEOCLR/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj" \
RAVEN_PACKAGE_OUTPUT="$SDKOUT" \
  scripts/package-sdk.sh osx-arm64 0.1.13-neoclr.dev.20261007

cd "$NEOCLR"
dotnet build tools/metadata/NeoCLR.Metadata.Translate -c Release
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot="$RAVEN"
python3 scripts/prepare-native-bootstrap.py \
  --probe docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime target/release/neoclr \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Release/net10.0/NeoCLR.Metadata.Translate.dll \
  --output "$BOOTSTRAP"
python3 scripts/build-native-class-library.py \
  --compiler "$SDK/tools/rvnc/rvnc.dll" --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --core "$BOOTSTRAP/Core.dll" --bootstrap-directory "$BOOTSTRAP" \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Release/net10.0/NeoCLR.Metadata.Translate.dll \
  --output "$LIBRARIES"
```

SDK names the generated SDK directory inside SDKOUT. Package its matching extension
and follow the [split distribution commands](native-split-distribution-2026-10-07.md)
with the newly built runtime and libraries. Extract into a fresh directory and run the
bundled verifier. The runtime seed must be finalized against this Runtime output;
never substitute an older matching-looking artifact or alter its identity hashes.

This is source bootstrap on a .NET-hosted Raven toolchain with explicit CLI Core and
retained native services. It does not mean Raven self-hosting on NeoCLR or removal of
all bootstrap declarations. RavenDoc redesign remains deferred by author direction.


## Results and scope

The source-only build chain succeeds after the recorded recoveries. Both checkouts
remain git-clean. A new archive extracted into a path containing spaces verifies all
1,028 files; all nine compile/run commands pass: collections, Tasks/await, JSON mapping,
HTTP server/client compilation and both HTTP request modes. Expected output, successful
exit status and the existing mutation/identity assertions remain intact.

The new VSIX installs in an isolated VS Code profile. All 26 editor acceptance checks
pass using that installed package directory and the extracted SDK/runtime, including
native metadata imports/help/navigation, reference/configuration reload, output
preservation, orders/Tasks execution and ordinary .NET hover. The test harness uses
extensionDevelopmentPath pointing at the installed package, not checkout extension
code. Its bundled server assemblies/configuration match the freshly staged SDK.

[Machine-readable revisions, commands and hashes](clean-bootstrap-reproduction-2026-10-07.json).
The initial failures and recovery are part of this evidence; the first unqualified
command sequence did not pass. No developer binaries were substituted. Package caches
and installed toolchains are permitted dependencies; cache-free/offline installation,
other operating systems and a final versioned release are not established here. The
persistent hover report remains open. No publication or main-branch merge occurred.
