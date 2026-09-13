# neoCLR Preview 4 — runtime and Raven

Version: **0.1.0-preview.4** · Tag: **v0.1.0-preview.4** · Date: **2026-09-13**.

Preview 4 presents neoCLR through direct neoIL programs and an experimental target
in Raven, a language built for the CLR. It includes the runtime's own System library,
Raven samples, a matching SDK and VS Code extension, documentation and provenance.
The GitHub prerelease carries the final validation record and asset checksums.

This is a proof of concept, not a production runtime or a drop-in .NET replacement.
The API surface is provisional, and feedback on the contracts is welcome.

## Start here

The prebuilt runtime/Raven bundle and SDK target **macOS arm64**. Other platforms
can build the runtime source; this release does not provide their Raven bundles.
The interpreter itself needs no .NET runtime. Raven tooling requires the .NET 11
preview SDK recorded in `manifest.json` (`11.0.100-rc.1.26425.128` for this build),
Python 3.9 or later for project tasks, and VS Code for the editor experience.

Download these assets together from the release:

- `neoclr-0.1.0-preview.4-osx-arm64.tar.gz`: runtime, adapted System library,
  declaration metadata, compiler bridge, language server, neoIL/Raven samples and docs.
- `raven-sdk-0.1.12-neoclr.6-osx-arm64.tar.gz`: matching experimental Raven SDK.
- `raven-vscode-0.1.12-neoclr.6.vsix`: matching experimental Raven extension.
- `raven-toolchain-notices.tar.gz`: companion attribution for the separate SDK/VSIX.
- `neoclr-0.1.0-preview.4-source.tar.gz`, `validation-evidence.tar.gz`,
  `release-manifest.json` and `SHA256SUMS`: source, validation and provenance.

The SDK and VSIX retain their tested `.6` version; they are the special experimental
Raven distribution, not a normal Raven release or Marketplace publication. Raven
source remains on `codex/neoclr-target-resolution` at
`9b269f9d0d71c4302c9008ff6bf6d0c6b1d21c9c`.

Extract the runtime bundle and, from that folder, try the runtime directly:

```sh
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

The first prints `42`, `7`, `9`, showing shared class identity and independent value
copies. The second prints `Completed`, `Not saved`, showing Result with a Void
completion payload and a recoverable error.

For Raven, extract the matching SDK to its own directory and install the VSIX through
**Extensions: Install from VSIX**. From the runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code demo
```

Reload an already-open VS Code window after replacing the extension. Open `Main.rvn`
and choose **Terminal → Run Task → neoCLR: Run saved project**. Expected output:

```text
42
Saved
Completed
Overflow
Value found
42
Absent
```

Try completion after `System.Date.` or `System.Type.`. Copy another file from
`tools/samples` over `demo/Main.rvn`, save and rerun. The bundle's README and
[runtime/Raven walkthrough](runtime-raven-preview.md) explain files, calendar,
reflection, collections, interfaces and other samples. The file sample creates a
demo file in the working directory. Native buffers require explicit `Free`.

The tasks compile saved files, import and verify the admitted IL, then run neoCLR.
They do not fall back to executing the guest on .NET. The normal Raven toolbar
build/run/debug commands do not implement this pipeline. SDK selection is local to
the demo workspace; configuration does not replace the global SDK default. Rerun
`configure.py` after moving the bundle.

## What this preview demonstrates

- **Familiar CLR type categories.** Ordinary classes and arrays use reference
  semantics; values are copied. GC manages the heap. Managed byrefs address slots;
  native pointers remain a separate interop facility.
- **Result and Option.** Recoverable errors and absence are explicit contracts.
  Raven's bounded propagation and match support work against the target library.
  Terminal faults have no guest exception class hierarchy.
- **Generic Void.** `System.Void` is an inhabited unit type usable in generic
  arguments, including `Result<System.Void, E>`. Ordinary CLI void calls still
  produce no stack result. neoIL accepts lowercase `void` for that return convention.
- **An existing runtime library reached from Raven.** Primitive/text helpers,
  Math, Console and Environment, managed arrays/ArrayList and iteration, File/Path,
  separate Date/Time and local clock, reflection introspection, delegates and native
  buffers. See the [API audit](raven-runtime-api-coverage.md) for member coverage.
- **Value/interface dispatch.** Admitted value-to-interface conversions box a copy;
  direct value operations avoid that allocation. Reference aliases preserve identity.
- **A basic editor experience.** Matching target declarations support completion and
  hover in VS Code, with separate neoCLR project tasks for execution.

## Migration from earlier previews

The current runtime/Raven direction replaces value-by-default and explicit-reference
syntax as the normal object/array model with familiar CLR value/reference categories.
Neo, the earlier concept language, remains in the source tree as historical test
material and is outside this migration. Its previous syntax and published examples
must not be read as the Raven target contract.

Ordinary runs now print only guest stdout. Use `--show-result` to print the runtime
result on stderr. Existing scripts that consumed the former `=> Void`/result suffix
must adapt. String managed storage has a typed null default, distinct from empty
text; methods requiring a real string fault on null. This does not implement a
complete nullable-metadata model.

The adapted ArrayList is a class with a managed backing array, constructors for
initial/default capacity, and indexer access. Union case payloads project through
read/write `Value` properties. BindingFlags uses CLI enum literals and operators.
Internal reserved capacity permits collection elements without inventing default
union cases. Existing `noresult` and uppercase `Void` neoIL behavior remains available
for legacy library code; binary CLI VOID signatures retain their original convention.

## Deliberate limits

The bridge consumes a bounded subset of CLI metadata/IL. Declaration metadata and
the importer are still experimental, not arbitrary PE execution. General application
class hierarchies and interface implementations, capturing closures, nullable metadata,
array covariance/rectangular arrays and unrestricted native layouts are not admitted.
Delegate samples use static targets; reflection is introspection-only. Cleanup during
terminal faults is not guaranteed. Clonable/Closable contracts exist without concrete
implementations in the current library. See the [coverage matrix](raven-runtime-api-coverage.md)
and [match matrix](raven-match-matrix.md) for precise scope.

Published Preview 1–3 notes remain unchanged. This release contains compatibility
changes, provisional API choices and an independently packaged Raven experiment;
it does not promise .NET binary compatibility or complete class-library parity.
