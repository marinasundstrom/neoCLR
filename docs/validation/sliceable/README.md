# Nominal slicing qualification — 2026-10-15

Development implementation on neoCLR main, paired with Raven codex/neoclr
827bca9c41a8d2073b8bb1af717fae5bed8930c0. Structural work remains parked.

## Contract

System.Sliceable<TSlice> exposes Slice(Range) returning Result<TSlice,
InvalidRangeError>. Conformance is explicit. Array<T> and ArrayList<T> implement
Sliceable<T[]>; valid slices are independent shallow copies. ArrayList uses Count,
not Capacity. The interface adds neither an indexer nor enumeration.

Declared range indexers call Slice. Intrinsic T[] range syntax keeps
RuntimeHelpers.GetSubArray; its faulting Range.GetOffsetAndLength now delegates to
TryGetOffsetAndLength, preserving existing fault messages. There is one bounds
predicate, and allocation of the copied array follows validation. Index construction
and unrelated allocation/runtime faults are not converted into Result errors.

## Evidence

The fresh five-library bundle is target/sliceable-libraries-20261015-final. The
retained results include exact commands, compiler/image/bundle hashes, log hashes
and status checks. No runtime or compiler behavior change was needed for nominal
interface dispatch. The CLI documentation bridge was rebuilt from matching sources;
production libraries and consumers use native metadata.

- Main.rvn exercises interface dispatch for arrays and lists, ordinary/from-end
  bounds, reversed and out-of-bounds selections, empty slices at either end and an
  empty receiver, Count versus Capacity, and independent copied element slots.
- Faults.rvn tests a list selection into unused capacity, reversed list bounds,
  a from-end boundary beyond the start, and invalid intrinsic array slicing.
- Interpreter, macOS ARM64 AOT and managed JIT pass. AOT hosts use undefined/bounds
  sanitizers. Both native hosts collect after each invocation and verify no leaked
  root frames or retained heap storage. Fault hosts verify UserFault, exact text
  and an untouched result slot. Results returning 42 denote successful checks.
- Initial host linking picked a CommandLineTools SDK incompatible with the active
  Xcode linker. The runner now requests the macosx SDK explicitly; successful
  interpreter results and generated objects were reused when relinking the hosts.
- API snapshot validation and website generation validate public reference coverage.

These checks qualify integer element arrays/lists. They do not establish broader
variance, view lifetimes, concurrency guarantees or allocation-free slicing.
Reference-element copies retain references by the existing CopyRange contract.
No new benchmark is claimed.

## Reproduce

Build the native bundle with scripts/build-native-class-library.py, using the
explicit bootstrap inputs documented in docs/experiments/native-structural-types.
Then from the repository root (use fresh output directories):

```sh
python3 docs/validation/sliceable/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle target/sliceable-libraries-20261015-final \
  --output target/sliceable-check
python3 docs/validation/sliceable/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle target/sliceable-libraries-20261015-final \
  --output target/sliceable-fault-check --faults
```

The runner compiles the retained source, validates interpreter exit/fault text,
selects and links AOT, and checks managed JIT results. It needs built runtime tools
in target/debug, macOS ARM64 and Xcode. Detailed logs stay in the output directory;
compact results are retained here.
