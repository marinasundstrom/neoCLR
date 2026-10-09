# neoCLR Preview 13 — modules and native compilation POC

Candidate qualification is in progress for **Windows x64** and **macOS ARM64**.
Publication is pending the final source, package and editor checks.

This preview brings module-oriented metadata and tooling, a source-built
Runtime/Data/Networking/Web library split, improved API documentation and an
experimental ARM64 ahead-of-time compiler. It remains a proof of concept with
explicit capability limits, not a stable platform or a drop-in .NET replacement.

## Install the matching toolchain

Install .NET 11 and .NET 10 SDKs, Python 3 and VS Code. The Raven compiler and
language server require .NET; the neoCLR interpreter does not.

Extract the package for your OS and architecture and keep its folders together.
From the extracted `neoclr-native-poc` directory, verify the installation:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

On Windows, use `python` in place of `python3`. Install `editor/raven-vscode.vsix`
through VS Code's **Install from VSIX** command. Open `samples/collections` and run
**Terminal → Run Task → neoCLR: Run**. The `tasks` and `json` directories are other
standalone examples. The verifier runs the paired HTTP examples automatically.

Keep the compiler, extension, runtime and libraries together when updating. Native
metadata and module identities have evolved since Preview 12: rebuild applications
against the new bundle. Time types now live in `System.Time`, including `TimeOfDay`.

## Native compilation teaser

Raven emits neoCLR metadata and instructions; the experimental AOT backend lowers
a supported subset into ARM64 object code. The checked source experiments include
console input/output, unions and patterns, control flow, faults, GC, native String
services and bounded HTTP workloads. Interpreted/native comparisons and recorded
benchmarks are available on the [native compilation page](https://marinasundstrom.github.io/neoCLR/features/native-compilation/).

The bundled `neoCLR: Run` task executes through the interpreter. AOT is a separate
source-build experiment, qualified on macOS ARM64. Windows x64 toolchain support
does not imply an x64 AOT backend. See the repository's `tools/aot-poc` and native
experiment guides for explicit build inputs and capabilities. No general-purpose
native compiler, stable ABI or native hot reload is promised by this preview.

## Known limits

- The distribution retains an explicit primitive CLI core and native runtime seed.
  Native-only production String compilation is demonstrated with a bounded core
  fixture; full source-owned native core bootstrap remains unfinished.
- HTTP remains a bounded cleartext POC. Unsupported compiler/runtime shapes fail
  explicitly; not every sample is supported in every execution mode.
- The website preserves module members, constants, XML help and Type extensions
  with declaring-assembly information. Some older API routes remain explicitly
  marked migration gaps; source debugging is not available.
- AOT String support uses immutable UTF-8 text, grapheme-based characters and a
  bounded intern pool. Wider Object display, reflection, trimming and GC evolution
  remain future work. Operating-system dependencies remain.
- Recorded benchmarks are development measurements on named configurations,
  not performance guarantees for either release platform.

Final artifact hashes and exact qualification revisions will be recorded with the release.
