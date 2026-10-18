# Preview 16 record and Object-display reproduction

Author-directed compiler investigation, 2026-10-18. The installed Preview 16
bundle and user sample are never edited: verify.py copies the project and writes
isolated fixtures, while using the installed compiler and libraries.

```sh
python3 docs/experiments/preview16-record-display/verify.py \
  --bundle /path/to/preview16/neoclr-native-poc \
  --backend /path/to/candidate/neoclr-codegen \
  --output /new/output/directory
```

The report retains exact commands, stdout/stderr, exit codes and SHA-256 identities
of the compiler, both backends, runtime and manifest. Candidate execution uses the
installed native host. No installation, publication or Windows qualification is
implied. The candidate is a debug macOS ARM64 build; these are correctness checks,
not performance measurements.

The [retained interpreter/JIT report](interpreter-jit-results.json) passes all five
fixtures. AOT consumer verification is being added in the next validation slice;
the low-level record/primitive GC tests already compile and link native objects.

## Findings

- The exact `record struct Person(val Name: string, val Age: int)` compiles with
  Preview 16. Construction and Name access execute as `Bob`. The earlier reported
  NEOMETA001 emitter error is **not reproduced** for this exact declaration; its
  original source/compiler conditions remain unknown.
- Passing Person to JSON reaches a different, reproducible backend rejection:
  nonempty record boxing was unsupported. The candidate copies the value payload
  with typed reference roots and now reaches the same UnsupportedMapping result as
  the interpreter. JSON mapping of record structs is still a library gap.
- The class sample printing either result or result.Value reproduces Preview 16's
  broad Object-display error. Candidate output is compared exactly with the
  interpreter. A pattern-unwrapped Ok value is a separate control. The class fixture returns
  Result.Error(JSON reflection operation failed) in both modes, and the Ok-only
  control therefore prints nothing. This confirms display parity, not successful
  class JSON mapping.
- Class mapping requires explicit reflection retention. The class fixture uses
  ordinary storage fields, not public properties; successful execution must not be
  interpreted as proof that Name/Age properties were serialized.

## Regression coverage and bounds

The native `rooted_boxed_records_copy_trace_and_dispatch_payloads` test checks
copy-in/copy-out isolation, a heap string reachable through the box, byref override
calls, empty-box compatibility, null/type-mismatch fault kinds and source sites,
heap canaries and final complete reclamation. Three rooted-boxing and three
Object-display checks pass. Ten structural checks pass; five nominal cast-fault
checks expose a separate existing RuntimeError/InvalidCast mismatch, assigned to
its own follow-up slice. All reference-containing structural GC checks pass.

No public signatures, Runtime Contract options or temporary CLI encodings change.
The native backend owns the repair. Existing layout/type/function bounds remain;
enum and whole structural-union boxing are not newly admitted. Private report keys
emptyRecordBoxes/emptyRecordBoxSites retain their legacy names for compatibility.
