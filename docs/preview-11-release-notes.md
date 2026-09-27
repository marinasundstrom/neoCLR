# neoCLR Preview 11 — async, text and Web API foundations

Version **0.1.0-preview.11** · tag **v0.1.0-preview.11**.

**Qualified 2026-09-27.** The source archive is `5731cb90`. The macOS runtime
bundle records `3b08108b`, and Windows records `8e5ae745`; their runtime inputs are
identical. Later changes repair test/validator expectations and packaging provenance.
The final tag adds release documentation and evidence.

The experimental Raven SDK was built at `0d5aa83b9` on `neoclr`. Bundle provenance
records `cdc4ca48b`, which adds only RavenDoc changes; compiler, language-server,
extension and packaging inputs are identical.

This preview extends the released HTTP POC with working generic async consumers,
shared-context Task.Run, text encoding foundations and richer bounded Web API
samples. This remains an experimental platform with changing APIs.

## Packages

- **macOS arm64:** matching neoCLR runtime/toolchain bundle, experimental Raven SDK
  and VS Code extension. Building Raven applications requires Python 3.9+ and
  .NET SDK 11.0.100-rc.1.26425.128. Running compiled applications requires neither .NET
  nor the compiler.
- **Windows x64:** native runtime ZIP with System library, four direct-runtime
  samples and dependency notices. Run with PowerShell; no .NET installation is
  needed. The executable imports VCRUNTIME140.dll and requires the
  [Microsoft Visual C++ x64 Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist).
  This release does not qualify the Raven SDK/bridge/editor workflow on Windows.
- Source and validation evidence accompany the binaries. Separate SDK/VSIX assets
  use the companion Raven tool notices archive.

From the extracted macOS runtime bundle, select the separately extracted SDK:

```sh
python3 configure.py --sdk /absolute/path/to/raven-sdk
python3 samples/http/http-json/verify.py --toolchain-root "$PWD" \
  --sdk /absolute/path/to/raven-sdk --runner tools/http-runner --routed
python3 samples/http/runtime-route-mapper/verify.py --toolchain-root "$PWD" \
  --sdk /absolute/path/to/raven-sdk --runner tools/http-runner
```

The route mapper is reusable sample source, not a new installed hosting framework.
On Windows, from the extracted native ZIP:

```powershell
./bin/neoclr.exe run samples/neoil/type-categories.neoil --system lib/System.neoil
```

Expected output is 42, 7 and 9 on separate lines.

## Included changes

- Task.Run shares captured objects and supports completion/typed callbacks,
  unwrapping and cancellation. Generic async methods on nongeneric owners and
  generic instance async receivers have focused suspension/capture coverage.
  Unit-valued delegate results now follow the compiler's caller-pop contract.
- Numeric/object interpolation uses virtual ToString, with null contributing empty
  text. Raven diagnoses a missing usable String.Concat overload instead of silently
  dropping interpolation. General compiler fixes are integrated independently on Raven main.
- Equality and ordering comparers, explicit ordinal/case-insensitive String comparison,
  Unicode casing, numeric Number contracts and concrete Result-based parsers.
- Stateful Encoder/Decoder foundations with UTF-8 defaults and strict ASCII selection
  in stream text adapters. String retains its UTF-8/grapheme model; a general
  StringBuilder and wider text API remain deferred.
- Nested typed JSON and typed arrays within the 1,024-byte payload budget, route
  patterns with typed parameters, and a runtime attribute-driven mapper sample
  which validates and caches its mappings at startup.
- Reflection and interface support used by these consumers, plus calendar/time-zone
  foundations. See the feature and API references for each bounded contract.
- HTTP malformed-request recovery and measured metadata/dispatch improvements from
  the sample-server CPU investigation. These are not throughput or latency guarantees.

Rebuild applications with the matching compiler, reference metadata, importer and
runtime library. Published older generated artifacts are not assumed compatible.

## Deliberate limits

HTTP remains cleartext HTTP/1.1 with one connection per exchange. Response bodies
and server request bodies are buffered; known-length client uploads are bounded
at 65,536 bytes. TLS, pooling, HTTP/2/3, response streaming and unknown-length uploads
remain future work. Option/null, enum and Uuid JSON mapping are not included.

The importer supports a bounded CLI subset, not arbitrary .NET programs. Generic
methods on generic owners and wider generic/value-type interface implementations
remain outside the admitted surface. Static call-graph analysis conservatively rejects
Object dispatch graphs involving open generic value overrides. Explicit interface accessors and private
instance helpers retain documented limitations. Task.Run uses runtime-owned native
worker execution; a green-thread implementation is a possible future direction.

## Validation

- Linux: all **1,659 tests passed** at `8e5ae745`; format and strict Clippy passed.
  The canonical job subsequently failed because its sample validator expected zero
  instead of counter's intentional exit 42. The validator-only correction at
  `5731cb90` passes all 25 extracted source/artifact samples and native integration.
  Runtime and test code are unchanged; the full suite was not repeated for this correction.
- macOS/Windows: focused host and ABI checks pass; Rust 1.85 compiles all targets
  on Linux, macOS and Windows. Windows ZIP extraction verifies 194 payload files
  and executes four samples on Windows Server 2025 x64.
- Extracted macOS tools: 22 MSBuild checks, routed HTTP peers/client/server,
  runtime route mapping and schema rejection, 11 upload cases, four direct-runtime
  samples, four selected Task.Run consumers, two interpolation cases and editor
  completion pass. Runtime/SDK archive files and uploaded asset digests are verified.
- Compiler/API checks: 639 metadata assertions, eight Raven interpolation tests,
  all 14 focused Task.Run consumers and matching library/API snapshots pass.
  Dependency notices cover 64 locked Rust packages, 27 NuGet and eight npm packages.
- Documentation: 18 focused renderer/reference checks pass. No local website build
  or website publication is included in this release operation.

See [the validation record](preview-11-validation.json), the accompanying evidence
archive, release manifest and SHA256SUMS. The raw canonical report retains its failed
sample-stage status; it is not presented as a wholly green CI job.
