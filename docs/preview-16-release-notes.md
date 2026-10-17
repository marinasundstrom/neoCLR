# Preview 16: native metadata from compiler to runtime

Preview 16 · 18 October 2026 · experimental prerelease

Matched runtime, Raven SDK and VS Code extension for **macOS ARM64** and **Windows x64**.
SDK/extension version: `0.1.16-neoclr.20261017.1`.

## Native library construction

Raven builds System.Runtime and dependent libraries from native neoCLR metadata
and explicit runtime-service contracts. Production compilation no longer needs
the NeoCLR.CoreProbe/CLI semantic bootstrap. The editor imports those same native
references, so completion describes the selected neoCLR libraries.

The Raven compiler and language server still run on .NET. That tooling requirement
is separate from the metadata and runtime used by compiled neoCLR programs.

## Types and runtime contracts

Native metadata represents structural unions, intersections, tuples and function
types as types rather than ad-hoc nominal carriers. Supported shapes retain runtime
introspection. This remains bounded support: consult the type guides for generic,
conversion and backend restrictions. Further structural expansion is parked.

Nominal interfaces support reference-only variance. Interface conformance remains
explicit; intersections do not introduce implicit duck-typed interface matching.
Function types support typed patterns and provisional reflection invocation.

Checked operators and contexts, consistent exclusive/inclusive range endpoints,
and overflow-safe signed integer iteration extend the language/runtime contracts.
Readonly fields and type initializers retain native metadata and execution rules.
Introspection describes fields, initializers and generic variance; reflection checks
supported operations at runtime.

## SDK and editor

The matched SDK and extension use native references for completion, hover,
navigation and type inference. Callback inference preserves results such as
`Task.Run(() => 41)` as `Task<int>`. Project configuration and native reference
changes invalidate diagnostics and recover after a failed project reload.

Runtime-owned await is the default. Explicitly clearing the runtime-async helper
setting selects existing heap state-machine lowering, including when the setting
contains only whitespace. Ordinary Raven/.NET async behavior is unchanged.

## Updating and scope

Install the matching platform bundle and its VS Code extension together. Rebuild
applications and native hosts; do not combine an older SDK/library bundle with the
new backend. Source syntax using the former range endpoint conventions also needs
review: `..` is exclusive and `..=` is inclusive in this compiler.

neoCLR remains experimental, with evolving APIs and metadata. Qualified platforms
are macOS ARM64 and Windows x64. JIT is the development default; interpreter execution
and bounded AOT publishing remain available. Source debugging is not available.
HTTP supports HTTP/1.1, without HTTP/2/3 or connection pooling. The optional
PostgreSQL provider targets local loopback/trust development, not general production
connectivity.

The macOS bundle includes PostgreSQL; the Windows bundle contains the four base
libraries. Compiler and editor prerequisites include .NET 11 SDK
`11.0.100-rc.1.26425.128` and .NET 10. Python 3 supports the bundled build/run tools.

## Qualification and measurements

Both extracted bundles pass sample and HTTP acceptance. Both installed editors pass
native completion, hover/navigation, exact `Task<int>` inlays and twenty repeated
project-recovery cycles. Runtime-owned await and heap state-machine configurations
pass all 24 interpreter/JIT/AOT cases per platform. The canonical source full suite,
25 smoke programs, formatting/Clippy, host checks and minimum-Rust checks pass.

Security-provider checks cover both platforms and all three execution modes.
Matching-package source-built HTTPS server qualification passes 15 macOS AOT and
27 Windows JIT/AOT cases, including trust/hostname/ALPN rejection, cancellation,
recovery and cleartext isolation. This source-host workflow is separate from the
bundled HTTP examples.

Fresh measurements on each platform validate 55 CPU/JSON processes and 1,152
plain/typed-JSON HTTP responses. Reports retain startup separately, five-round
ranges, correctness and input hashes. Interactive macOS and hosted Windows results
do not establish production capacity or a cross-platform speed ranking.

- [Qualification and exact component identities](https://github.com/marinasundstrom/neoCLR/tree/v0.1.0-preview.16/docs/validation/release-20261018)
- [macOS measurements](https://github.com/marinasundstrom/neoCLR/tree/v0.1.0-preview.16/benchmarks/release-20261018/macos)
- [Windows measurements](https://github.com/marinasundstrom/neoCLR/tree/v0.1.0-preview.16/benchmarks/release-20261018/windows)

## Downloads

Use the complete platform bundle for the runtime, SDK, extension and samples:

- [macOS ARM64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.16/neoclr-preview16-osx-arm64.tar.gz)
- [Windows x64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.16/neoclr-preview16-win-x64.tar.gz)
- [SHA-256 checksums](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.16/SHA256SUMS)

Standalone SDK archives, platform VSIX files, tool notices and machine-readable
qualification are also attached. Install `editor/raven-vscode.vsix` from the bundle
and open a folder under `samples/`. See the [installation guide](https://marinasundstrom.github.io/neoCLR/try/).

The qualified-source archive is the exact full-suite-tested `efa1687c8` snapshot.
The release tag additionally contains documentation, qualification evidence and
benchmark/tooling fixes; runtime and class-library source are unchanged across
that delta. Package manifests/provenance and the validation asset identify the
platform build revisions and hashes. Publication repacking only corrects the README
bootstrap notice and its manifest entry; executable/library bytes are unchanged.
