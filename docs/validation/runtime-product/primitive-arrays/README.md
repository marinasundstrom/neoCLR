# Rooted primitive-array qualification

Native-GC code generation selects rooted byte allocation, byte reservation and
scalar-array construction. Each helper validates its inputs, retries its sole
allocation at the active caller boundary, then initializes without reentry before
publishing. The scalar payload stays atomic to the collector. Existing native
entry points, non-GC emission and borrowed-input UTF-8 encoding stay noncollecting.
Reference-bearing arrays and general collection pacing are outside this slice.

The sanitized local native gate passes 23 executions. The new pressure fixture
covers all four forms (ordinary/reserved bytes and ordinary/reserved scalars),
missing/foreign boundaries, validation precedence, live exhaustion, unchanged
failure output, retained live bytes, empty arrays and final reclamation. It verifies
payload zeroing and initialization maps, then stores an allocation address as scalar
bits and proves that those bits do not retain the object. The relocated package
check also passes in GC and non-GC configurations.

Reproduce with the selected platform toolchain:

```
python3 scripts/validate-native-gc.py --output NEW_DIRECTORY
python3 -m unittest discover -s scripts -p test_native_gc_layout.py
cargo test --locked -p neoclr-aot-poc --test console byte_arrays_bounds_borrows_and_faults_match_interpreter
cargo test --locked -p neoclr-aot-poc --test console native_scalar_arrays_preserve_types_defaults_roots_and_faults
```

All three compiled tests pass: 30 byte-array and 77 scalar-array interpreter
comparisons, plus byte resource-limit checks. The compiled consumers are macOS ARM64 tests. Byte tests retain ordinary borrowed
access and use indexed initialization for reserved arrays, whose element borrows
remain unsupported by the existing native compiler. Checks assert rooted symbol
imports for GC builds and compare results/faults with the interpreter. No Raven
bridge metadata, Runtime Contract option or public managed API changes.
Windows qualification is separate; a native C gate is not a fresh managed Windows
or HTTP run. This is admission correctness work, not a benchmark or pacing claim.

[Hosted run 38444176481](https://github.com/marinasundstrom/neoCLR/actions/runs/38444176481)
at `e91f5f384` passes 23 GC/root executions, socket contracts and relocated-package
checks on both macOS ARM64 and Windows x64. macOS profiler attachment also passes.
The retained reports include exact source hashes and commands; this focused native
gate does not claim a fresh managed Windows, JIT, HTTP or ARM64-Windows matrix.
