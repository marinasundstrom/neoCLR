# Experimental Raven toolchain attribution

These notices accompany the Raven compiler bridge, SDK and language server and the
VS Code extension JavaScript shipped with neoCLR. They supplement Raven's own MIT
license and upstream THIRD-PARTY-NOTICES.txt; they do not replace those files.

The inventory was collected from the actual `.6` SDK/server/bridge `.deps.json`
files, and from esbuild's production extension input graph, using Raven revision
9b269f9d0d71c4302c9008ff6bf6d0c6b1d21c9c. It contains 26 NuGet and five npm packages.
Framework-provided .NET assemblies are external prerequisites, not bundled here.

`manifest.json` records declared licenses, package versions, preserved text hashes
and their sources. Texts come from the restored package where supplied; otherwise
from the upstream repository commit recorded in its NuGet specification. Mono.Cecil
uses its upstream 0.11.6 tag because its package omits the repository commit.

Ship this directory with the runtime bundle and as a companion attribution archive
when distributing the separate SDK/VSIX. Their existing package layouts omit some of
these notices. Retain neoCLR's root notices and third-party tree for the Rust runtime.
A changed dependency graph must be reviewed and this inventory updated before release.
