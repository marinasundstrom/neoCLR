# Stabilization for the next demonstrable preview

Recorded 2026-09-14. This is the active work order, not a release certification or a
new version assignment. Earlier preview validation records remain historical.

1. Finish reviewing the remaining general Raven metadata fixes: mixed application/
   metadata generic constructions and interface implementation references, then
   classify mixed union-pattern and array-factory changes. Integrate independently
   proven CLI-compatible fixes into Raven main. Keep neoCLR-specific code, mappings
   and tests on the experiment branch.
2. Synchronize reviewed main fixes into the experimental Raven branch. Verify the
   combined compiler, metadata projection and importer against the existing
   [preview acceptance criteria](raven-preview-acceptance.md), particularly arrays,
   reflection iteration, query callbacks, union destructuring and propagation.
   Report limits rather than adding new API surface during stabilization.
3. Build the runtime and special Raven SDK/VSIX bundle from pinned commits. Validate
   archive contents and a fresh extracted project, target completion and run tasks.
   Use the repository release procedures and record exact artifact hashes/toolchains.
4. Run the applicable source CI and package gates on the selected candidate commits.
   Update release notes and demonstration instructions with verified behavior and
   known limits, then tag/publish the preview. A previous green run or local focused
   test is not certification of a new candidate.

The Runtime library remains authored in neoIL for this release. Migration to Raven,
new runtime nullability semantics and possible compiler backend redesign are deferred.
General compiler fixes may benefit .NET Framework/NanoFramework, but validation must
name the actual targets exercised; modern .NET checks do not certify those runtimes.

See the [integration assessment](raven-target-evaluation.md) for completed fixes and
remaining candidates, and [source release](source-release.md) for release mechanics.
