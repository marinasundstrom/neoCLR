# Native positional record value semantics

Follow-up: [closed tagged-union components](../record-unions/README.md) now qualify
ConsoleKeyInfo equality/hash. The artifacts and limitations below describe the
earlier checkpoint; the follow-up also reruns this consumer against its new bundle.

Development qualification, 2026-10-20. This fixes the original canonical-text
record's generated equality rejection and establishes a bounded value-semantics
foundation. It does not close every record release gap.

`Program.rvn` covers canonical String/Char equality and matching hashes, inequality
on either component, typed and boxed value equality/hash, null and different boxed
types, nested records, Boolean/integer/Int64/enum components, and nullable reference
records. `Unsupported.rvn` verifies a clear NEOMETA001 for floating-point components
rather than silently emitting identity comparisons. `check.py` runs the same built
consumer through interpreter/JIT/AOT and compiles that negative case.

## Design and comparison

Ordinary Raven/.NET record generation uses EqualityComparer<T>.Default and generic
HashCode accumulation. Native generation prepares equivalent memberwise operations
for the supported types using the selected neoCLR core. String/Char equality remains
canonical with matching hashes; original UTF-8 storage is unchanged. Integer,
Boolean and enum comparisons and hash inputs use direct values. Int64 hashes fold
the two 32-bit halves. Declared text/nested-record hash methods are called directly.
Reference components handle null before Object dispatch. Hash values are not stable
serialization identifiers. No throughput or allocation benchmark is claimed here.

The backend now admits verified value Object.Equals/GetHashCode overrides and uses
the existing borrowed receiver projection for boxed calls. The original slot,
receiver and signature checks remain required. The interpreter's record execution
needed no change. This avoids introducing CLR-only comparer APIs into the library,
but the compiler-side admission policy is intentionally provisional and bounded.
See [bridge details](../../raven-cli-bridge.md#native-positional-record-equality-and-hashing--2026-10-20).

## Remaining gaps

- Generic records, unions and other unsupported value components. ConsoleKeyInfo
  still has an unsupported Optional<int> component; KeyValuePair remains generic.
- Imported value-record discovery, authored record members and record inheritance.
- Floating-point component equality/hash. An initial interpreter experiment handled
  NaNs, but native boxed floating-point dispatch rejects; that is not three-engine
  support and the compiler now rejects those value-method calls explicitly.
- Native primitive boxed Object equality/hash adapters remain incomplete. An initial
  Boolean-component boxing path faulted in AOT; direct component operations avoid
  that path, without qualifying ordinary boxed Boolean APIs.
- Generated formatting and wider comparer/EquatableTo policies. Unsupported records
  retain storage, accessors and deconstruction rather than incorrect value semantics.

No new public library type or authored member is introduced in this slice. The
website's Raven integration page describes the development capability and limits;
existing library API snapshots remain applicable. A website build is not required
for this prose-only update, and publication is separate.

## Evidence and reproduction

[Validation](validation.json) retains exact commands, results and binary hashes for
the matched development cohort. Compiler sources are on Raven `codex/neoclr`; the
change is target-only and does not require a Raven main carry. Raven commit `054c1c8d9` contains the tested compiler sources. The compiler binary
was built immediately before that commit from the same sources; hashes identify the
exact tested artifacts. This is not a release artifact. Final local artifacts:

- `target/record-compiler-v6`
- `target/record-libraries-v6` (all five source libraries, including PostgreSQL)
- `target/record-bundle-v6/bundle`
- `target/record-values-final-v6`

Raven's source-core metadata probe verifies generated member inventory and that
unsupported/generic records remain storage-only. The existing backend
`object_value_dispatch_preserves_identity_overrides_and_null_faults` regression also
passes. Focused Rust formatting and staged whitespace checks pass.

With a newly built matching development bundle and native backend:

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/record-values/RecordValues.rvnproj \
  --bundle target/record-bundle-v6/bundle \
  --aot target/release/neoclr-codegen --output target/record-values-rebuilt
python3 docs/experiments/record-values/check.py \
  target/record-bundle-v6/bundle target/record-values-rebuilt \
  --output target/record-values-rebuilt/validation.json
```

Use the supported platform toolchain environment for native linking. Final extracted
package, Windows and release performance qualification remain separate gates.
