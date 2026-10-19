# Unicode whitespace trimming — 2026-10-19

The Raven implementation scans immutable UTF-8 through ReadOnlySlice<byte> and
materializes with the existing checked slice service. No new runtime intrinsic or
native/FFI dependency is introduced. Static private helpers keep the implementation
within the current native String wrapper contract.

Main.rvn checks all 25 Unicode 17 White_Space scalars with all three APIs, mixed
edges, interior spaces, combining marks, emoji and format characters that are not
whitespace. Comparisons are ordinal so canonical equality cannot hide byte changes.
Unchanged and empty inputs check reference identity. Trimming classifies scalars,
not graphemes: a leading space before a combining mark is removed independently.

```sh
python3 docs/experiments/string-trimming/check.py \
  target/string-trim-bundle-v4/bundle target/string-trim-public-v4 \
  --output target/string-trim-validation.json
```

Build the library with build-native-class-library.py, prepare a matched bundle with
prepare-jit-bundle.py, then build Main.rvnproj using build-native-project.py's console
profile. Compiler code is Raven 2d22055dc with metadata adapter e4709c451, as in the
canonical text qualification. The interpreter is unchanged; the backend additionally repairs access to private
static String helpers after owner projection. Original external private calls still
reject before projection. Validation uses the pinned Xcode SDK on macOS ARM64.

Allocation.rvn runs 128 iterations with identical output in baseline, unchanged
and changed modes. Re-link with the existing string-building allocation diagnostic
host to measure managed allocations and final reclamation. Counts include the
process, not merely the method, and do not measure CPU, host allocations or HTTP
throughput. The scan creates no temporary byte/Char arrays. Changed nonempty output
copies retained bytes and retains the existing runtime Value transport overhead.

Policy research and tradeoffs are recorded in the
[String review](../../design/string-api-review-20261019.md#whitespace-trimming-2026-10-19).
These are development APIs; release-package qualification remains separate.


## Results

All three interpreter/JIT/AOT executions pass; see [validation.json](validation.json).
The focused native projection test also passes, including legal helper execution,
null `callvirt` receivers and rejected external private calls. The backend repair is
`6e7bf1d9a`; source String changes use libraries v2 and bundle v4. The separately
recorded hand-authored direct-call null discrepancy is outside this API fixture.
API fingerprints, four native snapshot tests and the 4,550-page website build pass.

For 128 iterations, the [allocation diagnostic](allocations.json) reports:

| Mode | Managed allocations | Reclaimed | Collections |
| --- | ---: | ---: | ---: |
| Baseline | 2 | 2 | 1286 |
| Unchanged trim | 2 | 2 | 4614 |
| Changed trim | 130 | 130 | 7174 |

Unchanged trimming adds zero managed allocations; changed nonempty trimming adds
one per result in this native workload. All tracked allocations are reclaimed.
Eager boundary collection counts increase with method calls, so this result must
not be presented as a throughput gain. Paced/concurrent-GC comparison and CPU
measurement remain separate work. Reproduce the counter check with:

```sh
python3 docs/experiments/string-trimming/measure.py \
  target/string-trim-allocation-v1 --output target/string-trim-allocation.json
```
