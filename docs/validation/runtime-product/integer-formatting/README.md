# Rooted integer formatting qualification

Generated native-GC calls use private rooted Int32/Int64/UInt64 formatting
helpers. Existing native arena-only callers and non-GC emission retain the original
noncollecting symbols. Formatting uses native stack scratch, then performs one
managed allocation and publishes without further collection. There are no borrowed
managed inputs. This extends the temporary-root audit; it does not enable pacing.

The local native contract report retains source hashes, commands and 22 passing
executions with undefined-behavior/bounds sanitizers. The new fixture tests signed
limits, unsigned maximum and zero; a full heap with only live objects still faults,
then succeeds after one object becomes unreachable. Missing/foreign boundaries are
rejected; failed calls leave outputs unchanged. Retained live bytes, root-frame
identity and complete final reclamation are checked. The relocated-package check
also passes, including GC and non-GC arena builds.

Reproduce with the selected platform toolchain:

```
python3 scripts/validate-native-gc.py --output NEW_DIRECTORY
python3 -m unittest discover -s scripts -p test_native_gc_layout.py
cargo test --locked -p neoclr-aot-poc --test console rooted_integer_formatting_matches_interpreter
cargo test --locked -p neoclr-aot-poc --test console native_gc_preserves_borrowed_owners_erased_results_and_fault_messages
cargo test --locked -p neoclr-aot-poc --test console wide_integer_text_conversions_records_and_lifetime_match_interpreter
```

All three focused compiler tests pass locally: 15 GC and 15 non-GC wide-integer
comparisons preserve interpreter faults, and the borrowed/erased Int32 consumer
retains its values through repeated collections. GC imports explicitly select
the rooted symbols. Compiler tests are macOS ARM64 consumers. Windows helper qualification is separate;
passing the C contract does not establish a fresh managed Windows or HTTP run.
This is correctness/admission work, not a performance claim. No new managed API or
Raven bridge contract is introduced. See the [audit](../../../experiments/aot-console/native-gc.md#explicit-helper-temporary-values).
