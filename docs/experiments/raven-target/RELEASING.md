# Releasing the Raven/neoCLR experiment

This is a neoCLR experimental distribution containing a separately packaged Raven
build. It does **not** follow Raven's full product release cycle or imply a normal
Raven release. Keep compiler changes on the Raven experimental feature branch; do
not merge or tag Raven main just to ship this bundle.

## Scope and evidence

Use [the neoCLR preview acceptance criteria](../../raven-preview-acceptance.md) for
what the demonstration must expose. All existing public runtime APIs are in scope;
full CLR/compiler feature parity is not. Maintain the supported/unsupported matrix
and list outstanding defects plainly. Packaging is not evidence of API support.

Before packaging, record the exact clean neoCLR and Raven commit IDs, Raven branch,
build tool versions, target OS/architecture, and an explicit experimental package
version. Choose a fresh version for each build; do not overwrite a previously
published artifact. The installed `0.1.12-neoclr.4` is a historical local build, not
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

**Current distribution gap:** the development runner compiles its source bridge
against a Raven checkout. A public bundle must either package a runnable bridge
with its dependencies or explicitly include and test the required source/tooling
setup. It must not ship local checkout paths as if it were standalone. The installed
SDK and VSIX alone do not close this gap.

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
compiler/bridge where the distribution supports it; source-checkout checks alone do
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
