# Native POC bundle

## Development project-to-executable workflow (2026-10-09)

### Windows x64 console (source checkout)

Development profile, [qualified on Windows Server 2022 x64](windows-native-project-validation.json).
Use a Windows x64
**Developer PowerShell / x64 Native Tools prompt** with MSVC C tools, Rust/Cargo,
Python 3.12 and the .NET SDK required by the selected Raven bundle. The pinned
Preview 13 Windows bundle uses .NET 11 RC1 `11.0.100-rc.1.26425.128`; project evaluation
also uses .NET 10. Download and extract
[the Windows bundle](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.13/neoclr-preview13-win-x64.tar.gz)
and verify its [published checksum](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.13/SHA256SUMS).
From this source checkout:

```powershell
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python scripts/build-native-project.py --profile windows-console `
  --project samples/native-windows/App.rvnproj `
  --bundle C:/path/to/neoclr-native-poc `
  --aot tools/aot-poc/target/debug/neoclr-aot-poc.exe `
  --output target/windows-native-app
./target/windows-native-app/app.exe
```

The helper sets `NeoClrBundleRoot` for project imports, verifies bundle bytes,
compiles the evaluated Raven project and links a static-CRT EXE with MSVC. The
output directory must be new. `build.json` retains commands, diagnostics, input
hashes and dependencies. Failed compilation/linking never publishes `app.exe`;
an existing destination is preserved. Copy only `app.exe` to deploy: the intended
runtime dependency is Windows' `KERNEL32.dll`, with no .NET, Raven bundle or helper
DLL needed beside it. [Run 37958647006](https://github.com/marinasundstrom/neoCLR/actions/runs/37958647006)
at `a0b50a96db0d1d43fa81c860fcbbaca4dbde9736` passes five fresh project builds and
standalone executions: managed arrays/text, interpolation, binary input/output and
a divide-by-zero fault. All match interpreter stdout/stderr and exit status exactly.
The EXEs range from 167,424 to 381,952 bytes in this recorded static-CRT build
(the managed sample is 216,064 bytes); this is artifact size, not a performance claim.
Existing-output preservation and failed-rebuild/stale-output rejection also pass.
All 126 retained artifact hashes, 25 source inputs, 32 pinned-bundle inputs and five
project inputs are verified; the backend hash is consistent across all five build
reports. See the [retained evidence](windows-native-project-validation.json).

The synchronous profile includes the existing bounded value/reference lowering,
native GC, stack guards, console input/output, UTF-8 text and Int32 formatting.
The sample exercises array allocation/churn, string aliases and UTF-8/NUL output.
Interpolation is separately qualified; default Object display for arrays remains
an existing backend restriction, so the combined sample formats Int32 explicitly.
HTTP, file/path, task/scheduler, character segmentation and extended integer service
bindings are excluded. Existing backend unsupported-feature diagnostics still apply.
Windows native unwind/SEH, arbitrary reentry and green threads remain unqualified.
This source workflow does not publish a Windows development kit or change Preview 13.

Compared with .NET Native AOT's project publish workflow, this exposes neoCLR's
existing UTF-8/Raven contracts through a source-checkout driver and explicit host
profile. It needs more manual toolchain setup and supports a narrower subset; no
performance superiority is claimed. Reuse the
[host and stack research](windows-native-host.md) and the existing native-execution
investigation. The macOS workflows below retain their separate contracts.

### Relocatable development kit

The native console and opt-in HTTP workflows can be staged as a separate development kit with
the compiler/library bundle, a freshly built AOT tool, native adapters and a sample
project. From a source checkout, using a qualified native POC bundle:

```sh
python3 scripts/package-native-build-kit.py \
  --bundle /path/to/neoclr-native-poc \
  --output /path/to/new-kit-candidate
python3 scripts/verify-native-build-kit.py \
  --archive /path/to/new-kit-candidate/neoclr-native-build-kit-osx-arm64.tar.gz \
  --output /path/to/new-kit-qualification
```

Packaging requires Cargo and its locked dependencies available locally. It builds
the AOT executable, checks its architecture/OS dependencies, verifies the source
bundle's catalog, and archives only catalogued bundle files plus selected build
support. The kit includes dependency license texts/inventory, the AOT lockfile,
source/input hashes and an archive checksum. Candidate creation does not publish
a release or change Preview 13 assets. A dirty source state is recorded explicitly.

Extract the archive and run from its `neoclr-native-build-kit` directory:

```sh
python3 scripts/build-native-project.py \
  --project samples/hello/App.rvnproj --output ../hello-native
../hello-native/app
```

Application builds need Python 3.9+, the bundled compiler's .NET SDKs and Apple's
macOS build tools; neither Cargo nor the source checkout is needed. The helper
selects the kit-owned compiler/library bundle and backend automatically, verifies
kit file hashes before invoking tools, and rejects overrides to a different pair.
The shipped sample is catalogued: use a separate project importing the kit's
`bundle/lib/NeoCLR.ClassLibrary.props` when editing your own application. Hashes
establish consistency, not an authenticated signature. The standalone executable
does not require the kit, .NET or a managed runtime installation.

The extraction verifier builds from a temporary path with spaces outside the
checkout, with Cargo removed from PATH, runs the executable alone with an empty
environment, and checks that modified adapters, backend and compiler configuration
are rejected before tool invocation. The synchronous console profile below is
unchanged; Windows now has the separate source-checkout profile above.
The opt-in HTTP profile described below has its own qualification; the default
console profile does not acquire HTTP services or a task pump.
The [2026-10-09 extracted-kit evidence](experiments/native-build-kit-validation.json)
records the tested archive hash, source inputs, bundled-toolchain manifest and results.

### Source-checkout workflow

From a neoCLR source checkout on macOS ARM64, build the current AOT tool and use
an ordinary executable `.rvnproj` importing the selected split bundle's
`lib/NeoCLR.ClassLibrary.props`:

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 scripts/build-native-project.py \
  --project /path/to/App.rvnproj \
  --bundle /path/to/neoclr-native-poc \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /path/to/new-native-output
/path/to/new-native-output/app
```

The output directory must be new. The command uses the bundle's compiler to
evaluate/build the project, compiles the resulting neoCLR metadata/IL, then links
the console host and selected runtime adapters. Apple `xcrun` selects Clang and
the SDK explicitly. The compiler still needs its documented .NET SDKs; the
resulting executable needs only macOS libSystem. This source-checkout helper is
development work, not a new capability in the published Preview 13 bundle.

The explicit `macos-arm64-console-v1` profile supports synchronous console entry,
selected UTF-8 text operations and Int32 formatting, using the existing private
ABI v4, native root reporting and a bounded 1 MiB nonmoving managed heap. It
renders guest faults, checks that guest root frames have unwound and collects at
quiescent shutdown. Development Main(string[]) roots now receive
[managed command-line arguments](native-entry-arguments.md) through a matching
private process-entry ABI. It supplies no task pump, socket
services, file services, guarded recursion or general native reflection. Unsupported
reachable services/instructions fail through the backend's existing diagnostics.
This is a bounded deployment policy, not a permanent platform restriction or a
stable public hosting ABI.

`build.json` records commands, diagnostics, selected compiler/backend/library and
adapter hashes, SDK/compiler identification, emitted artifact hashes and dynamic
dependencies. Library catalog hashes are checked before compilation. The project
must reference the same bundle; arbitrary extra project-reference assemblies are
not yet added to the AOT dependency catalog. The helper currently consumes Raven's
`Native build output:` line (last output after reference builds); a structured
compiler output contract is future work. Recorded hashes establish input consistency,
not a signed provenance chain or a complete reproducible MSBuild input inventory.

An unsuccessful build retains its report and intermediate files but publishes no
`app`. Existing output directories are rejected without replacing their contents.
The command does not run the application. The focused acceptance harness does:

```sh
python3 scripts/verify-native-project-build.py \
  --bundle /path/to/neoclr-native-poc \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /path/to/new-qualification-directory
```

It checks projects/output paths with spaces, UTF-8 and interpolation, a guest
division fault against the interpreter, executable-only deployment with an empty
environment, unsupported-recursion rejection and preservation of existing output.
The [2026-10-09 acceptance record](experiments/native-project-build-validation.json)
passes those cases and rejects a failed rebuild despite its previously emitted DLL.
The relocatable kit above packages this workflow with its backend/adapters. The
HTTP profile has separate service and host contracts; Windows HTTP remains open.
This reuses the [.NET deployment comparison](native-execution-investigation.md):
project-level native build ergonomics are familiar, while neoCLR's currently
bounded runtime-service profile and explicit bootstrap dependencies remain costs.

### Opt-in HTTP project profile (development)

An extracted kit also contains `samples/http`, using the existing checked-in Raven
HTTP server source unchanged. Build it with an explicit service profile:

```sh
python3 scripts/build-native-project.py --profile http \
  --project samples/http/App.rvnproj --output ../http-native
../http-native/app
```

The sample prints an ephemeral loopback port, serves one `GET /greeting` with the
UTF-8 body `Café 🌍`, then closes. The helper compiles the evaluated project with the
same native bundle and records `macos-arm64-http-v1` in the build report. Console
remains the default; native HTTP support is not inferred from an application's
references. Use `--http` with `scripts/verify-native-build-kit.py` to include the
HTTP consumers in extracted-kit acceptance.

This profile links the existing [HTTP correctness host](../benchmarks/native-web/http-host.c),
with a 1 MiB native GC heap, scoped sockets/tasks, private default-queue draining,
retained callback dispatch and the macOS ARM64 stack guard. It enables the existing
listener, accept, transfer/deadline, TaskQueue and integer-text bindings. The host
checks guest-frame balance, scope cleanup and collection at shutdown, and propagates
guest faults. Its 15-second completion timeout is checked cooperatively between
guest calls; it is not preemption or a bound on a guest call that never returns.
The profile expects an application whose selected code emits the private queue and
callback exports used by this host; it is not a universal replacement for console.

This is the existing bounded server POC made accessible through projects and a kit,
not a new public hosting ABI or a production server deployment profile. It adds no
green threads, general runtime suspension, scheduler fairness, host-I/O async-entry
support, Windows backend or performance claim. Runtime lifecycle contracts should
co-evolve as described in [scheduling design](runtime-scheduling-design.md#co-evolution-with-native-foundations--2026-10-09):
replace host queue pumping with scheduler admission/readiness, move root/stack
ownership to activations when suspension exists, and retain code generations across
callbacks/suspended work for future reload. Keep these replacements distinct from
the library's observable Task/HTTP behavior.

[Extracted HTTP project evidence](experiments/native-http-project-validation.json)
records greeting and fragmented requests, duplicate-length and handler-error
rejection, and a guest fault raised in the callback, all matching interpreter
response bytes, output, faults and exit status. The host's existing scope/root/GC
cleanup checks pass. Native runs use an executable-only directory and empty
environment; no new throughput, concurrency or sanitizer claim is made. Existing
[native HTTP sanitizer evidence](../benchmarks/native-web/http-validation.json)
remains the supporting evidence for the unchanged host/adapters.

## Preview 13 qualification (2026-10-09)

[Preview 13](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.13) is published
for Windows x64 and macOS ARM64. Public bundle links return HTTP 200; all five
uploaded asset digests match the qualified local artifacts.

The author has requested Windows x64 and macOS ARM64 packages for the next POC.
Both matching compiler/runtime/library/editor bundles pass extracted consumers
and 26 installed-editor checks per host. Canonical source validation and focused
Windows/macOS host checks pass. [Exact evidence](preview-13-validation.json) records
package hashes and the separate runtime, compiler, library and source-test revisions.
ARM64 AOT experiments do not imply Windows x64 native-code generation support.

The current retained seed adds the ordinal String replacement service after
Preview 12. Its freshly assembled SHA256 is
`0d44005b8fb48a4655fef1cbb33b5e37987d9a06a8dc8ecbbeb3b1622f0137fd`.
The release validator checks this current pin; Preview 12's historical pin below
is unchanged. The seed remains an explicit bootstrap input, and full native-only
core bootstrap remains outside this bounded release qualification.

This is the development native release path. It does not invoke the CLI translation
bridge. Raven libraries are built into native metadata first, then distributed with
the matching native-enabled SDK, VSIX and runtime. The primitive CLI core and retained
native runtime seed remain explicit bootstrap inputs; full System source ownership
is still separate work. The bundle does not claim qualification on other hosts.

## Build and stage

First use `scripts/build-native-poc-libraries.py` and the checked-in ownership profile
as described in the [native source gate](experiments/extended-cli-metadata/native-source-release-2026-10-05.md).
Use the same SDK compiler, primitive core and retained seed when staging:

```sh
python3 scripts/package-native-poc.py \
  --sdk "$SDK" --vsix "$VSIX" --libraries "$LIBRARIES" \
  --core "$CORE" --seed "$SEED" --runtime "$RUNTIME" \
  --runtime-revision "$RUNTIME_REVISION" --output "$CANDIDATE"
```

The output directory must be new. The command checks library bytes against the build
report, compiler/core/seed identity by SHA256, current selected sources against their
build hashes, and canonical ownership before staging. It records supplied runtime and
compiler revision declarations separately from file hashes; a declared revision alone
is not proof of binary origin. Runtime compilation remains a separate explicit step.
The manifest records whether the packaging checkout was dirty.

The resulting `neoclr-native-poc.tar.gz` is a local candidate with no inferred release
version. No tag, release upload or website deployment occurs. It includes tooling,
source-built libraries, bootstrap files, authored API XML, preserved notices, consumer
samples, verification tools, and relative VS Code projects/tasks. Consumer projects do
not contain the library sources. Shared XML supplies descriptions for matching IDs;
it does not imply complete native API documentation coverage.

## Extract and qualify

Extract into a new location, including a path with spaces when checking portability.
From the extracted `neoclr-native-poc` directory:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

This requires Python 3 and the .NET SDK/runtime required by the packaged compiler
(current candidate: net11 host and net10 project reference packs). No repository
checkout or global Raven installation is used by those commands. The verifier checks
all manifest file hashes before invoking the bundled compiler/runtime, compiles five
separate projects, executes collections/identity, Tasks/await and JSON mapping, then
runs actual HTTP/JSON client/server checks with exact result assertions. A changed
file fails before compilation. It writes a failure report and exits nonzero on failure;
a launch or compilation success alone is not the gate.

For IDE use, install `editor/raven-vscode.vsix`, open one individual `samples` folder,
and run the `neoCLR: Run` task. The bundled server comes from the installed extension;
project reference and task paths are relative to the extracted installation. Do not
set a development-checkout language-server override for installation acceptance.

Hash verification establishes consistency with the manifest, not a signed trust chain.
This tooling supplements the still-failing legacy source/archive snapshot audit; it
does not rewrite that audit's result or assert that legacy bootstrap generation works.
The provenance procedure below closes the retained-input reproduction gap.
Publication remains separate from local packaging and does not expand the translation bridge.

## Preview 12 bootstrap provenance

The retained seed is now pinned at `runtime/raven/native/poc-seed.neoil`. It is the
bounded primitive/service selection previously assembled by the source JSON gate,
not a replacement implementation of application or source-library APIs. Build it:

```sh
neoclr assemble runtime/raven/native/poc-seed.neoil System.neox --format neox
```

The primitive CLI reference is reproduced by the existing declaration generator:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot="$RAVEN_SOURCE"
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-comparer-storage-core Core.dll
```

On 2026-10-05, with Raven integration revision `2554322c4` and NeoCLR `ef9d53ce`,
these exactly reproduce the previously accepted SHA256 artifacts:

- Core.dll: `64aae0e5ef83fe113a38d21c74cabe1bfabbd9de3526d941ea2236b630686bbc`
- System.neox: `2175d583a36e6de17086d25adc31376d9c3aea4a0c287f81043d3649dfc5c1dc`

The generator reuses the existing bootstrap declarations; no application/native
library is translated into CLI metadata. Full native primitive bootstrap is later work.
The author selected `v0.1.0-preview.12`, macOS arm64, and integration into NeoCLR main.

### Windows HTTP project profile (development)

From a Windows x64 MSVC developer shell, with the same pinned bundle and backend
prerequisites as the console profile:

```powershell
python scripts/build-native-project.py --profile windows-http --project docs/experiments/http-server/Native.rvnproj --bundle path/to/neoclr-native-poc --aot path/to/neoclr-aot-poc.exe --output path/to/new-output
path/to/new-output/app.exe
```

This opt-in profile supports the bounded HttpServer showcase with the shared
HTTP host, Winsock and guarded Windows collector/stack owner. Its only admitted
dynamic dependencies are KERNEL32.dll and WS2_32.dll. Validation and remaining
client/ARM64 boundaries are tracked in [HTTP parity](native-http-parity.md).
Windows run [37961490213](https://github.com/marinasundstrom/neoCLR/actions/runs/37961490213)
passes greeting, fragmented requests, duplicate-length rejection, handler errors
and callback faults with interpreter parity. The same cases pass on macOS ARM64.
This is the bounded server showcase, not general HttpClient support.

The sample prints an ephemeral loopback port and serves one request. In a second
terminal run `curl.exe http://127.0.0.1:<printed-port>/greeting` on Windows (or
`curl` on macOS). The body is `Café 🌍`; the server then closes. Select
`--profile http` for the same sample on macOS. The checked-in project matches the
validated project template and imports the bundle selected by the builder.

## Rebuild development libraries for current HTTP samples

Current source samples use `Task.CompletedTask` and `Task.FromResult`, which are not
in the published Preview 13 libraries. From this checkout, prepare a separate
matching development bundle before building those samples:

```sh
python3 scripts/prepare-native-development-bundle.py \
  --bundle /path/to/neoclr-native-poc \
  --compiler-revision 71cafd353900394a4f670a8ad8691597f6115091 \
  --output target/http-development
```

Use `target/http-development/bundle` as `--bundle` for the HTTP project build or
validation command. The command verifies the input manifest, rebuilds the metadata
translator and all four Raven libraries, and records source/tool hashes and logs.
It reuses the selected bundle's explicit primitive-only Core.dll and compiler;
it does not claim to remove the primitive bootstrap. The output must be new and
is a development artifact, not a reissued Preview 13 release. The Windows HTTP
Actions perform this preparation automatically using the pinned archive.
