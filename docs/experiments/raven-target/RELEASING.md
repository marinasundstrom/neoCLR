# Releasing the Raven/neoCLR experiment

This is a neoCLR experimental distribution containing a separately packaged Raven
build. It does **not** follow Raven's full product release cycle or imply a normal
Raven release. Keep compiler changes on the Raven experimental feature branch; do
not merge or tag Raven main just to ship this bundle.

## Scope and evidence

The [release walkthrough](../../runtime-raven-preview.md) defines two primary paths:
the runtime with `samples/neoil`, and Raven programs with the matching experimental
SDK/VSIX. Keep old Neo language material historical and outside the main demonstration.
Run `tools/verify_neoil.py --samples samples/neoil --system lib/System.neoil --runtime bin/neoclr`
from the extracted bundle in addition to the Raven checks below. A complete release
must carry both paths and their documentation, not just compiler integration assets.

Use [the neoCLR preview acceptance criteria](../../raven-preview-acceptance.md) for
what the demonstration must expose. All existing public runtime APIs are in scope;
full CLR/compiler feature parity is not. Maintain the supported/unsupported matrix
and list outstanding defects plainly. Packaging is not evidence of API support.

Before packaging, record the exact clean neoCLR and Raven commit IDs, Raven branch,
build tool versions, target OS/architecture, and an explicit experimental package
version. Choose a fresh version for each build; do not overwrite a previously
published artifact. The installed `0.1.12-neoclr.6` is a validated local build, not
a prescribed next release version.

Run focused compiler regression tests for changed Raven code and neoCLR tests for
changed runtime code. Run the end-to-end Raven integration checks and all release
demo programs. This targeted evidence replaces a full Raven release gate; it does
not replace testing the experiment itself. No unrelated Raven release workflows,
marketplace publication, or default-SDK changes are required.

## Package Raven separately

From the selected Raven feature-branch checkout, set `EXPERIMENT_VERSION` to the
chosen fresh version and `EXPERIMENT_RID` to the supported runtime identifier (for
example `osx-arm64`). Set `EXPERIMENT_OUTPUT` to a fresh absolute artifact directory.
The following are existing Raven packaging scripts; run them sequentially:

```sh
RAVEN_PACKAGE_OUTPUT="$EXPERIMENT_OUTPUT" scripts/package-sdk.sh "$EXPERIMENT_RID" "$EXPERIMENT_VERSION"
RAVEN_PACKAGE_OUTPUT="$EXPERIMENT_OUTPUT" scripts/package-vscode.sh "$EXPERIMENT_VERSION"
```

The output includes the versioned SDK archive and `raven-vscode.vsix`. Retain both
as distinct release assets, together with checksums and provenance. Check `git
status` after packaging because the scripts regenerate sources and stage extension
content. Review any source changes before assigning final provenance; do not label
a dirty build as an exact committed revision.

Build neoCLR from its recorded revision and bundle the matching runtime library,
metadata declarations, bridge, demo projects and instructions. Package these as
neoCLR assets rather than inserting them into an ordinary Raven release. The bridge
must use the same compiler build used for validation.

## Build the checkout-independent bundle

After committing the source changes and building the separate SDK, use a fresh output
directory. The packaging command requires clean neoCLR and Raven checkouts:

```sh
python3 docs/experiments/raven-target/package_bundle.py --raven /path/to/Raven --sdk /path/to/raven-sdk-VERSION-osx-arm64 --runtime /path/to/neoclr --output /fresh/path/to/bundle --version runtime-api-poc-BUILD
```

The script publishes the compiler bridge with its dependencies, generates matching
declaration metadata, flattens the target runtime library, copies the language server,
samples and validation scripts, and records revision/version/file-hash provenance.
It currently supports the tested macOS arm64 host. It does not publish or install.
The resulting README explains extraction, configuration and the dedicated project tasks.
The builder also copies neoCLR's LICENSE, notice inventory and preserved license
texts, and Raven's LICENSE/THIRD-PARTY-NOTICES.txt; the file manifest covers them.
Before publishing binary assets, review the actual bridge, server, SDK and VSIX
dependency inventories against their notices. Carrying upstream source notices alone
does not establish complete attribution for every bundled dependency. The validated
local .6 artifacts predate this packaging correction and must not be presented as
containing it; prepare a fresh candidate rather than overwriting those artifacts.

`run_project.py`, the saved-program suite and focused checks accept either `--raven`
for source development or `--bridge /path/to/Probe.dll --system /path/to/System.neoil`
for a published build. Each build attempt owns fresh output, and failed compilation
cannot execute stale output. The supplied library is copied alongside that output.

## Primary MSBuild workflow checks

The primary Raven entry point is `msbuild-demo/Demo.rvnproj`, configured with the
matching SDK. Build with standalone MSBuild and run the verified artifact separately.
Also build and run `project-reference-demo/App/Demo.rvnproj`; its expected output is
`42` followed by `Library call`. This exercises one library dependency, including a
namespace function, without changing Raven or using Microsoft.NET.Sdk.

Run `tools/verify_msbuild.py --bundle /extracted/bundle --sdk /extracted/sdk` and record
its result. Run the full editor suite against the primary project and against the
library application's folder with `--project-references`, before the first library
build. The latter checks source-project resolution without a prebuilt DLL. Include
changed-library rebuilds, incompatible reference packs, dependency compiler failures
and stale-output rejection. Do not claim multi-level dependency or restore support.

## Validate the actual packaged build

Extract/install into an isolated directory or VS Code profile; preserve the user's
normal Raven SDK selection. Follow [local installation instructions](VSCODE.md) with
the new artifact paths. Record actual paths and versions in the build manifest.

1. Check the packaged compiler and language server start with the documented .NET
   prerequisite. Record compiler version and the matching source revision.
2. Open a bundled Raven project targeting neoCLR. Verify completion and hover resolve
   the supplied runtime declarations and do not silently fall back to host .NET APIs.
3. Edit, save, build and run a demo through the neoCLR task. Verify value/reference
   behavior, Result/Option propagation and Void completion, plus the runtime-library
   demonstrations required by the current coverage inventory. Capture guest output.
4. Exercise expected failures: incompatible source and unsupported IL must fail
   without executing stale output; recoverable API failures must return their Result
   cases. File tests must use temporary directories and verify round-trip bytes.
5. Repeat from the packaged layout outside the development checkouts. Remove any
   dependence on developer-specific paths, or document and validate the explicit
   source-build prerequisite. Do not claim a standalone bundle unless this passes.

The stdio server checks, saved-project checks, match matrix and file probes in this
folder are repeatable development evidence. Run them with the packaged server and
compiler/bridge where the distribution supports it. Include `verify_application.py`
and `verify_orders.py` from the bundle: these verify application type semantics,
interface/virtual dispatch, escaping/shared captures, GC and order persistence using
the packaged bridge/runtime. Their temporary projects preserve the installed demo.
Source-checkout checks alone do
not establish that the packages work.

## Publish and repeat

Create a manifest containing both source revisions, package versions, platform and
prerequisites, asset SHA-256 hashes, validation commands/results, coverage matrix and
known limitations. Use `local-toolchain.json` as an example of provenance, not as a
manifest to overwrite with unverified results.

Update `CHANGELOG.md` under Unreleased for the build/release work and follow
[neoCLR's changelog workflow](../../changelog.md) when publishing. Keep prior release
notes unchanged. Publish the neoCLR build and matching experimental Raven SDK/VSIX
together with installation instructions and the manifest. A new tag/version/date
is chosen when publishing; this procedure does not announce one.

For the next experimental release, repeat revision selection, focused validation,
separate packaging and package-level smoke tests. Reuse the procedure and extend the
coverage evidence as the bridge grows; a full Raven release remains independent.

## Experimental tool dependency notices

The [tool notice inventory](../../../third-party/raven-tools/README.md) supplements
both projects' upstream notices. Check the actual SDK and runtime bundle:

```sh
python3 docs/experiments/raven-target/verify_tool_notices.py --tools /path/to/artifacts
```

For the VSIX, also compare its production JavaScript dependency graph when rebuilding
Raven; the .deps.json check only covers .NET packages. Preserve the eight reviewed npm
package notices and refresh the inventory if the esbuild inputs change. Distribute
`third-party/raven-tools` together with Raven's LICENSE and THIRD-PARTY-NOTICES.txt
as a companion attribution archive for the separate SDK and VSIX. The runtime bundle
contains the same texts. Do not describe the separate upstream assets as containing
notices that are supplied only in the companion archive.


For builds containing the prototype query API, also run `tools/verify_queries.py`
against the extracted demo project with its packaged bridge, runtime and System
library. Add `--queries` and `--extensions` to the editor checks. These are new source
checks, not retroactive claims about earlier archived validation reports. Document
remaining query and cleanup boundaries from the query API doc.

For builds containing target-aware array diagnostics, keep
`<RavenAllowArrayCovariance>false</RavenAllowArrayCovariance>` in the demo project
and add `--array-invariance` to the editor check. It verifies implicit/explicit
array conversion diagnostics and correction after an edit. Older SDKs ignore this
new property; do not report source-built checks as validation of an older bundle.


For builds containing generic array unification, also pass `--array-shape` to the
editor checks. This verifies Array<int> member/query completion and int[] alias
hover. Keep the unified-array sample in the saved-project suite, and select
`RavenIterationArrayShapeType` in new demo projects. Both compiler spellings emit
ordinary CLI arrays; no separate generic-array allocation format is required.


For the collection capability prototype, run `verify_collection_capabilities.py`
with the same project/bridge/library/runtime arguments as the other saved-source
checks and add `--collection-capabilities` to the editor checks. Rebuild declarations
and the bridge together: inherited Count/indexer ownership changed, and older SDKs
need the inherited-indexer compiler correction. Do not treat the existing .11
installation as containing these later source changes.


## Map prototype source checks

The Map slice adds `runtime/raven/Map.neoil`, generated core Map/MutableMap/HashMap
signatures and `samples/library-maps.rvn`. Rebuild the bridge, emit fresh core
metadata with `--interfaces`, and regenerate System with `collection_library.py`.
Do not combine this new core metadata with a stale target System library. The
installed .11 bundle remains unchanged; package a fresh build to distribute Maps.

`verify_project.py --collections` includes the Map sample. Run
`verify_collection_capabilities.py` for the read/mutation and invariant-generic
negative cases, the `--signatures` probe for metadata validation, and append `--maps`
to the full `verify_editor.py` invocation for completion through Map, MutableMap
and HashMap. Run `cargo test --test raven_collections` for direct IL/GC tests.
The [Map contract](../../map-contracts.md) lists current comparer and iteration
limits that must accompany a build. These checks do not imply a full Dictionary API.


## Query terminal source checks

The terminal slice adds First/Last Option results and Single's Result/SingleError
contract. Rebuild the metadata core and System library together using the same
fresh-source procedure as the Map slice. `verify_project.py --collections` and
`verify_queries.py` include `library-query-terminals.rvn`; `verify_editor.py --queries`
now expects all three new operators. Run `cargo test --test query_terminals` and
`--signatures` as well. Publish the [query contract](../../raven-query-api.md), including
its normal-completion cleanup boundary and absence of fault-unwinding guarantees.
Do not claim that the current installed .11 tools already contain these additions.

## ArrayList filtering source checks

Regenerate core metadata and System together. FindIndex now returns Option<Int32>
instead of Int32/-1; recompile callers and migrate their absence handling. The
[filtering contract](../../arraylist-filtering.md) documents all seven operations,
callback behavior and limits. Run `verify_project.py --collections`,
`verify_application.py`, the `--signatures` probe and `verify_editor.py --collections`
with the same target configuration used for the full library checks. Run
`cargo test --test list_filters --test query_terminals --test raven_collections`
and retain `--test predicate_search` for the historical Neo profile regression.
The .NET comparison is `dotnet run --project docs/experiments/list-filters/dotnet`.
These source changes do not refresh the installed .11 SDK or VSIX.

## Combined collection application checks

The [order collection scenario](../../raven-order-workflow.md#collection-integration-scenario-2026-09-13-source-slice)
uses application-defined class payloads through HashMap, ArrayList filtering and
array/interface queries with Option/Result propagation. Run `verify_application.py`
against the actual candidate: it includes GC-pressure and hash-collision variants.
The full editor invocation with `--maps` also checks member discovery on application
payloads after lookup and filtering. The sample and expected output are included
by the existing sample-directory packaging step. Keep this source capability distinct
from already published or installed builds lacking the newer library metadata.

## Predicate terminal overload source checks

First, Last and Single now also accept Func<T, Boolean>. Refresh core metadata and
System together; .12 binaries predate these overloads. The project/query suites use
the updated terminal sample; application checks use Single(predicate) with an Order
payload. Run the signature probe and `cargo test --test query_terminals` for outcome,
callback-order, disposal, fault and allocation checks. `verify_editor.py --queries`
checks predicate signature help as well as extension discovery. Preserve the
[documented forward Last scan](../../raven-query-api.md#predicate-terminal-overloads-2026-09-13)
and normal-outcome cleanup limits in release notes.

### Source target-contract checkpoint (2026-09-14)

The next bundle's project must select `RavenTargetCoreAssemblyName` as well as the
metadata core. Use the matching Raven target-contract branch; old .12 tools are not
updated by source changes. Rebuild compiler, bridge and editor tools together when
preparing a new SDK. The normal compiler path and remaining adapters are described in
[normal target compilation](../../raven-target-compilation.md).

Run `verify_compiler_target.py` with the rebuilt rvnc, packaged bridge/System/runtime
and a generated project, in addition to existing saved-project checks. It separately
compiles and imports PE artifacts and tests rejection without host fallback. Packaging
now carries the verifier; this checkpoint does not publish or install a new bundle.

For development builds containing the Option/Result operators, also run
`tools/verify_outcome_operators.py` against the bundled saved project, importer,
System library and runtime. Require the operator completion checks in
`verify_editor.py --unions`. Package the query verifier's `query_basic_cases.py`
helper together with the verifier. These requirements apply to future bundles;
published Preview 8 artifacts are unchanged.
