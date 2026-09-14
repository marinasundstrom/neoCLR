# neoCLR Preview 6 — collections and target stabilization

Version: **0.1.0-preview.6** · Tag: **v0.1.0-preview.6** · Date: **2026-09-14**.

This preview builds on familiar CLR value/reference semantics with neoCLR's own
runtime library and an experimental Raven target. It adds consistent array and
collection contracts, Map/HashMap, direct ArrayList filtering, Option/Result-based
query terminals and a clearer compiler-to-importer boundary.

## Install and try

Use these matching assets from the GitHub prerelease:

- `neoclr-0.1.0-preview.6-osx-arm64.tar.gz`
- `raven-sdk-0.1.12-neoclr.13-osx-arm64.tar.gz`
- `raven-vscode-0.1.12-neoclr.13.vsix`
- `raven-toolchain-notices.tar.gz`

Source, validation evidence, `release-manifest.json` and `SHA256SUMS` accompany them.
Prebuilt tools support **macOS arm64**. Raven requires .NET SDK
**11.0.100-rc.1.26425.128**, Python 3.9+ and VS Code for editing. The runtime itself
does not require .NET. Source CI covers the runtime on Linux, macOS and Windows.

Extract the runtime bundle and SDK into separate folders. Install the matching VSIX
using **Extensions: Install from VSIX**, then run from the runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code demo
```

Open `demo/Main.rvn` and select **Terminal → Run Task → neoCLR: Run saved project**.
Reload existing VS Code windows after replacing the extension. The default demo
prints `42`, `Saved`, `Completed`, `Overflow`, `Value found`, `42`, `Absent`, each on
its own line. The normal Raven run/debug toolbar is not this target's pipeline.

Copy `tools/samples/application-order-collections.rvn` over `demo/Main.rvn` for the
collection workflow; `application-orders.rvn`, `library-maps.rvn`,
`library-query-terminals.rvn` and `library-patterns.rvn` provide focused alternatives.
See the [workflow guide](raven-order-workflow.md) for exact sample names and outcomes.

Direct IL demonstrations need only the runtime:

```sh
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

## Changes since Preview 5

- Arrays expose the generic `System.Array<T>` shape and iteration contracts while
  retaining ordinary CLI array instructions. Mutable arrays are invariant; a
  read-only interface is a separate capability, not permission to mutate a wider array.
- Collection capability interfaces separate observation from mutation. Map/HashMap
  add a bounded dictionary-like API; ArrayList filters work directly on the collection.
- First/Last return Option; Single returns Result with explicit cardinality errors.
  Predicate overloads are included. Recompile callers using older terminal or
  FindIndex contracts; FindIndex now returns Option rather than a sentinel index.
- Reflection-array iteration, array extension queries and reference/string signatures
  are stabilized. Runtime library API design remains provisional and welcomes feedback.
- Raven compilation and neoCLR import are explicit stages, preserving assembly and
  signature identities. Explicit supplied libraries support bounded nongeneric types,
  constructors, properties, interface/virtual dispatch and namespace functions.
- The experimental Raven build includes reviewed general compiler fixes and the
  attribute-emission regression correction found by the broader Raven sample audit.
  General fixes live on Raven main; neoCLR-specific policies remain experimental.

## Limits and validation

This is a preview subset, not binary compatibility with the full .NET platform.
See [acceptance scope](raven-preview-acceptance.md), [library import limits](raven-library-import.md),
[query contracts](raven-query-api.md), [Map contracts](map-contracts.md) and
[filter contracts](arraylist-filtering.md). Cleanup guarantees cover documented normal
outcomes; fault unwinding, runtime async and a full debugger are not promised.

The runtime library remains authored in neoIL. Migration to Raven, new runtime
nullability semantics and alternative compiler backends are deferred. Old Neo language
material is historical and is not the primary demonstration.

Publication requires exact-source CI and extracted-package checks. The attached
manifest/evidence identifies tested commits, artifacts and limitations. Raven's main
stability audit found and corrected a NanoFramework attribute regression; its unrelated
MacCatalyst host sample needs newer Xcode and is not claimed as a passing build.
This distribution is not a normal Raven release or a hardware NanoFramework certification.
