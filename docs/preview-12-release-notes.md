# neoCLR Preview 12 — native Raven compilation

Preview 12 is a bounded end-to-end native metadata POC for **macOS arm64**. Raven
compiles selected class-library sources into NeoCLR assemblies, imports those native
libraries into separate applications, and emits assemblies that the runtime executes.
The included SDK and VS Code extension are a matched development build.

## Install and verify

Download `neoclr-preview12-osx-arm64.tar.gz`, extract it, and enter `neoclr-native-poc`.
Install .NET 11 (the compiler host; the tested build used RC1), .NET 10 SDK/reference
packs, and Python 3. VS Code is required for editor use.

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
code --install-extension editor/raven-vscode.vsix
```

Open an individual directory under `samples/` in VS Code. The `neoCLR: Run` task
builds and runs it using the bundled compiler/runtime. API help uses adjacent XML or
Raven Markdown documentation. No global Raven SDK or development-server override is
required. The HTTP examples are paired server/client programs; the verification tool
starts and checks both automatically.

## Qualified behavior

- Artifact-only native library import, collections/query execution and shared identity.
- Tasks/await, JSON reflection/object mapping and live HTTP/JSON client/server checks.
- Installed editor completion, hover/API help, metadata navigation, reference refresh,
  diagnostics/recovery and native build/run (19 checks).
- A diagnostic metadata disassembler: `bin/neoclr disassemble <assembly>`.
- Website API reference through RavenDoc, including function signatures and union docs.

## Explicit limits

The primitive CLI reference and retained native runtime seed are temporary bootstrap
inputs. Their source generation and hashes are documented in the repository's
`docs/native-poc-bundle.md`; application and rebuilt-library references use native
metadata. This is not full System source bootstrap, a stable metadata ABI, or support
for every Raven feature. Unsupported generic async/entry shapes remain explicit.
The runtime still uses UTF-8 storage with grapheme-based Char semantics.

Only macOS arm64 is qualified for this native toolchain release. Preview 11 downloads
remain available for the older Windows runtime/bridge workflow. The legacy bridge
snapshot audit remains failing and is not evidence for this native distribution.
The .NET backend remains Reflection/Reflection.Emit; replacing it is future work.
No .NET performance comparison or performance improvement is claimed.

See `docs/experiments/extended-cli-metadata/native-bundle-2026-10-05.md` for the
bounded execution evidence and `docs/preview-12-validation.json` for release artifacts.
