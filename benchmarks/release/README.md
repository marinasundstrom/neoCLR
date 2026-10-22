# Release benchmark refresh

Run the same suite against the matching extracted candidate on macOS ARM64 and
Windows x64. Results qualify only the recorded platform and artifacts; a passing
macOS run does not qualify Windows. These measurements cover interpreter, JIT and
AOT CPU/JSON workers, longer JIT/AOT CPU batches, and plain/JSON HTTP workloads.

```sh
python3 benchmarks/release/run.py --bundle /path/to/extracted/bundle --output /path/to/new/results --rounds 5
```

On Windows run from an x64 MSVC developer shell, using `python`. On macOS select
a matching Xcode installation with `DEVELOPER_DIR`. Both require the compiler's
.NET SDKs and native build prerequisites. The source checkout supplies the benchmark
fixtures and native build support; this is not a standalone SDK qualification.
Stop concurrent builds and full test suites before measuring. Compilation is outside
timed worker batches; aggregate step durations include preparation and are not
runtime performance measurements.

`release-benchmarks.json` records host/CPU, source revision and dirty state, runner
and bundle manifest hashes, command order, exits and hashes of child reports. Child
reports preserve sample distributions and compiler/library/input hashes. A nonzero
exit, nonpassing child report or changed manifest fails the refresh. Keep failed
reports, logs and the candidate archive's hash with the release evidence.

New reports retain `sourceStatusBefore` and `sourceStatusAfter` as Git porcelain-v1
records, including individual untracked files, status columns and rename pairs.
`dirty` describes the initial checkout; generated build outputs do not retroactively
change it. Final Git inspection errors are recorded without discarding measurement
results. A dirty checkout is disclosed rather than automatically rejected. Older
reports without these fields retain their original provenance limitations; this
change cannot recover the missing Windows dirty-path list.

The Windows `Native release benchmarks` workflow requires an explicit successful
`Native toolchain qualification` run from the same neoCLR commit by default. For a benchmark-only repair, explicitly
supply `candidate_revision` as the package's full commit hash to reuse the unchanged
candidate. Both candidate and benchmark-source revisions are retained; do not use
this override to qualify runtime/library changes absent from the package. It no longer
silently benchmarks a historical package. Run it on the candidate ref; its artifact
retains both the package run and archive hash. macOS uses the same runner locally
against its separately qualified candidate.

Publish separate tables for each host and retain historical results rather than
overwriting them. Compare only compatible workloads/configurations and disclose
hardware, collector and build changes. This suite does not establish concurrent-GC
parity, peak HTTP concurrency, installed SDK size or native import/export support;
those remain separate release checks. Do not describe refreshed measurements as
complete until both platform reports pass.
