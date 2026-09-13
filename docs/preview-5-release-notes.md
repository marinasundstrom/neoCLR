# neoCLR Preview 5 — applications, patterns and queries

Version: **0.1.0-preview.5** · Tag: **v0.1.0-preview.5** · Date: **2026-09-13**.

This preview demonstrates familiar CLR type semantics with neoCLR's own runtime
library and an experimental Raven target. It adds application classes, inheritance,
interfaces, capturing lambdas, union destructuring and a small deferred query API.
The order-workflow example combines these with Result/Option and file persistence.
The API remains provisional; feedback on its contracts is welcome.

## Install and try

Download the runtime bundle, matching Raven SDK and VSIX from this prerelease:

- `neoclr-0.1.0-preview.5-osx-arm64.tar.gz`
- `raven-sdk-0.1.12-neoclr.8-osx-arm64.tar.gz`
- `raven-vscode-0.1.12-neoclr.8.vsix`
- `raven-toolchain-notices.tar.gz` (companion attribution for the SDK and VSIX)

Source, validation evidence, `release-manifest.json` and `SHA256SUMS` accompany them.
Prebuilt tools support **macOS arm64**. Raven requires .NET SDK
**11.0.100-rc.1.26425.128**, Python 3.9+ and VS Code for the editor experience.
The runtime itself does not require .NET. Other hosts can build the runtime source.

Extract the runtime archive and SDK into separate folders. Install the VSIX using
**Extensions: Install from VSIX**, then run from the runtime bundle folder:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code demo
```

Open `demo/Main.rvn` and use **Terminal → Run Task → neoCLR: Run saved project**.
The default propagation demo prints `42`, `Saved`, `Completed`, `Overflow`,
`Value found`, `42`, `Absent`, each on its own line. Completion and hover use the
bundled target declarations. Reload an existing VS Code window after installation.
The normal Raven run/debug toolbar is not the neoCLR execution pipeline.

Copy `tools/samples/application-orders.rvn` over `demo/Main.rvn` to try the combined
application. It writes `neoclr-orders.txt` in the working directory and ends with
`Pending orders` followed by `Tea`. See the [workflow guide](raven-order-workflow.md).
Try `library-patterns.rvn` and `library-queries.rvn` for focused examples.
Configuration is workspace-local and does not replace the default Raven SDK.

Direct neoIL examples need only the runtime:

```sh
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

## Changes since Preview 4

- Raven application classes and values, constructors, fields, properties, interfaces,
  abstract/virtual inheritance and constructor chaining now cross the bridge.
- Instance and virtual/interface method groups, shared and escaping lambda captures
  work through delegates, including generic Void completion.
- Union patterns accept imported `Ok(let text)` / `Error(let error)` and target-typed
  cases. Payload destructuring and editor support now work with target metadata.
- Application extension methods and runtime `Where`, `Select`, `ToList` demonstrate
  deferred iteration with custom Raven Iterable/Iterator implementations. Queries use
  ordinary methods, classes and delegates; no query opcode was introduced.
- Static-member completion inside constructors is fixed in the experimental Raven build.

The existing primitive/text, Math, collections, file/path, date/time, clock,
reflection-introspection and native-buffer APIs remain available. Result/Option
represent recoverable failures and absence; `System.Void` is usable as a generic
unit type while CLI void calls produce no stack result. Classes and arrays have
reference semantics; values copy. The legacy Neo frontend remains historical.

## Boundaries

This is a proof of concept, not a drop-in .NET implementation. The importer admits
only a bounded CLI subset. Generic application methods/extensions, comprehensive
nullable metadata, array covariance and rectangular arrays remain outside scope.
Reflection is introspection-only. Terminal faults have no guest exception hierarchy.
Automatic iterator cleanup on early exit or faults is not guaranteed. Query argument
validation and mutation behavior differ from .NET; see [query contracts](raven-query-api.md).
No indexed queries, ordering, query provider, async enumeration or array-to-Iterable
conversion is included. A no-result lambda may infer Raven Unit; use explicit
Func with System.Void where required. See the [match matrix](raven-match-matrix.md)
for remaining syntax limitations and [API coverage](raven-runtime-api-coverage.md).

The separately packaged Raven `.8` build remains on `codex/neoclr-target-resolution`
at `000ed511e4b1e124dce34ba4208ac0cda9ce1245`; this is not a normal Raven release or
Marketplace publication. The release manifest records exact sources, checksums and
validation. Published earlier preview notes remain unchanged.
