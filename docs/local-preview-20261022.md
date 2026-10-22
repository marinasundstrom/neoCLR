# Local Preview 17 candidate — 2026-10-22

This macOS ARM64 installation is a release candidate, not a published release.
It pairs Raven `301e668a5` with the readiness-repaired neoCLR runtime `32184c938`.

- Bundle: `~/.local/share/neoclr/preview17/neoclr`
- Side-by-side SDK: `~/.raven/sdk/0.1.17-neoclr.20261022.1`
- Default VS Code extension: `raven.raven-vscode@0.1.17-neoclr.20261022.1`

Open a bundled sample folder in VS Code, for example:

```sh
code ~/.local/share/neoclr/preview17/neoclr/samples/collections
```

The sample settings select the bundled SDK. General Raven command-line wrappers
retain their existing SDK selection; installing this candidate does not redirect
ordinary .NET projects. Reload an already-open VS Code window to activate the
upgraded extension.

JIT and interpreter package checks, HTTP exchanges, 26 editor acceptance checks,
inlay checks and two ten-cycle configuration recovery runs pass. See the
[installation evidence](validation/release-stabilization-20261022/local-installation/summary.json).
The installed SDK's 469 files match the bundle; the installed extension's 87 server
files and client entry point match its VSIX.

The previous SDK and Preview 16 bundle remain available. To restore the previous
extension:

```sh
code --install-extension ~/.raven/backups/preview17-20261022/raven-preview16.vsix --force
```

The Raven primary checkout remains on `codex/neoclr` at `301e668a5`; main is in
`/Users/robert/Projects/Raven-checked-conversions`. No branch switch was found.
The candidate SDK was built from an isolated archive of the pinned compiler
revision. A preceding shared-checkout packaging attempt exited without a diagnostic;
its cause remains unconfirmed.

The [live website](https://marinasundstrom.github.io/neoCLR/try/) describes the
candidate while retaining Preview 16 download links. Windows qualification and
final release publication remain separate gates.
