# Enum helpers in the Raven target

The sample uses both `System.Enum.GetNames<TEnum>()` / `GetValues<TEnum>()` and
`GetNames(TypeInfo)` / `GetValues(TypeInfo)`. Generic values remain typed. Discovery
returns boxed values with the enum's identity, rather than underlying integers.
It also checks named/unnamed/flags formatting and retained snapshots during repeated
queries under GC pressure.

```sh
python3 docs/experiments/enum-helpers/verify.py \
  --bundle /path/to/matching/development-sdk \
  --runner target/release/examples/measure_async
cargo test --lib enums::tests
cargo test --test raven_reflection
```

The verifier rejects non-enum generic arguments at compilation and non-enum TypeInfo
at execution. Native tests cover unsigned ordering, aliases, nominal unboxing and
heap-budget enforcement. Signature probes reject weakened enum constraints and
non-enum substitutions. Current bridge admission covers EntryKind, TaskState and
BindingFlags; this does not add arbitrary application enum import.

These are development APIs. [Design and .NET comparison](../../enums.md#raven-enum-helpers-development)
describe snapshot allocation costs, ordering and formatting limits. Internal native
metadata queries are not additional public APIs.

Latest SDK validation: 501 allocations, 10 collections, peak 64 objects, zero live
objects after completion. Both invalid generic argument forms and non-enum TypeInfo
are rejected at their documented boundary. The historical Neo manifest uses explicit
legacy snapshots; this sample targets the Raven profile.

Regression validation: all 25 reflection checks pass across the full run and the
focused rerun after repairing the historical profile. The 16 bootstrap arithmetic,
enum, IO and library checks also pass after updating the stale native-service count
for the existing string construction/indexing/interning services. The combined site
build validates 862 API pages.
