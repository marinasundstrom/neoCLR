# Preview 15 qualification

Preview 15 integrates execution, garbage collection and scheduler/suspension into
one runtime, with native AOT as a publishing target. The packages target macOS
ARM64 and Windows x64. SDK version: `0.1.15-neoclr.20261014.2`; VS Code extension:
`0.1.15-neoclr.20261014.3`.

## Evidence

- [Source, host and minimum-Rust checks](https://github.com/marinasundstrom/neoCLR/actions/runs/38545373946): passed, including canonical extracted-source validation.
- [Windows SDK/package](https://github.com/marinasundstrom/neoCLR/actions/runs/38542979837): passed default-JIT package acceptance, three-mode JSON, HTTP, runtime await, GC, editor and inlay checks.
- [TLS and cryptography on both hosts](https://github.com/marinasundstrom/neoCLR/actions/runs/38545373900): passed.
- [Retained Windows HTTP host](https://github.com/marinasundstrom/neoCLR/actions/runs/38545373921): passed.
- [Final JSON and editor delta](https://github.com/marinasundstrom/neoCLR/actions/runs/38547814472): checks changed libraries in interpreter, JIT and AOT and the packaged configuration watcher, member hover and inlays.
- [JSON formatting evidence](json-formatting.json) records macOS DOM and stream checks across all three modes.
- [Final gates](final-gates.json) records component revisions and report hashes. Earlier candidate failures remain in this directory as development evidence; they are not the shipped package's qualification.

The final delta changes JSON APIs and the editor client. It reuses the qualified
compiler/server and runtime binaries; unchanged GC, scheduler and HTTP checks
are not repeated. The SDK compiler is Raven `793219d855b17f01fb805e215968b78871eb3804`;
the editor client includes `507f35571ea40076df49193433d0b242a462cea7` with the same
packaged language server as the SDK.

## Documentation

The API snapshot and full website build pass, including source-link guards for
JSON node conversion, serializer options, Tasks, HTTP and PostgreSQL declarations.
Publication uses the main-branch source URLs after the release changes reach main.

## Package scope

Both bundles include the four base native libraries. The macOS bundle additionally
includes PostgreSQL for local loopback/trust development. Runtime await is the
default on both platforms. See the [release notes](../../preview-15-release-notes.md)
for migration guidance and remaining platform limits.
