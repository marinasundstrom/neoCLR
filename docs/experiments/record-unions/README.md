# Native record tagged-union components

Development qualification, 2026-10-20. Extends the earlier
[record value checkpoint](../record-values/README.md) to closed declared tagged-union
components, including the public ConsoleKeyInfo.TextScalar: Optional<int> contract.
This is a bounded release-gap fix, not full record or release-package qualification.

## Behavior and evidence

Generated ==, !=, typed/Object Equals and GetHashCode inspect the active case and
recursively compare/hash its payloads. Typed operations use public case extraction
and getters without payload boxing. Default carriers with no active case remain
distinct from active empty cases; Optional.None, Some(0) and default Optional<int> are
three different values. Canonical String/Char equality and matching hashes apply
inside payloads, while original UTF-8 storage remains unchanged.

- [Baseline](baseline.json): the previous compiler rejects a minimal Optional<int>
  record comparison with the targeted unsupported-component diagnostic.
- [Validation](validation.json): interpreter, JIT and AOT pass the public consumer;
  Optional<double> still receives the targeted unsupported-component diagnostic.
  Covers ConsoleKeyInfo, nested Optional, Result, multi-payload source cases,
  canonical/nullable text, Char, default carriers, null and boxed Object dispatch.
- [Regression](regression.json): the earlier record-values consumer and its negative
  test pass against the same new bundle, preserving primitive, enum, nested-record
  and reference-record behavior.
- [Allocation measurement](allocation-results.json): 128 typed ConsoleKeyInfo
  equality/hash iterations allocate two managed objects, identical to baseline,
  and reclaim both. Collection checks rise from 8 to 9,864. This is evidence about
  managed allocations only, not CPU cost, throughput, host allocation or explicit
  boxing. The eager collection activity warrants separate performance investigation.
- Raven's source-core metadata probe passes supported/unsupported union component
  inventory checks; final output is target/record-union-source-core-final.

Compared with .NET record comparer machinery, this native policy avoids generic
comparer dependencies and payload boxes. It generates repeated public case tests,
which can increase code size and work for larger unions. Hashes are not persistent
identifiers. Generic record owners, unsupported payloads, imported arbitrary value
record discovery and generated formatting remain gaps. No new runtime instruction,
Runtime Contract configuration or ordinary .NET behavior change is introduced.
See [bridge ownership and limitations](../../raven-cli-bridge.md#native-record-tagged-union-components--2026-10-20).

## Reproduction and artifact identity

The tested compiler was built from Raven 054c1c8d9 plus this native union-component
change, now committed as Raven 0199ba140 on codex/neoclr. Library provenance intentionally says 054c1c8d9-dirty-record-unions-v2;
these are development artifacts, not released packages. Validation JSON records
compiler/backend/runtime/core/application SHA-256 values. Local artifacts:

- target/record-union-compiler-v2
- target/record-union-libraries-v2
- target/record-union-bundle-v2/bundle
- target/record-union-app-v2
- target/record-union-allocation-v2
- target/record-union-regression-v2

Build Program.rvn and Allocation.rvn using their corresponding rvnproj files with
scripts/build-native-project.py, console profile and the matched bundle. Native
host linking on this machine uses the Xcode clang toolchain and MacOSX26.2 SDK.

```sh
python3 docs/experiments/record-unions/check.py \
  target/record-union-bundle-v2/bundle target/record-union-app-v2 \
  --output target/record-union-validation.json
python3 docs/experiments/record-unions/measure.py \
  target/record-union-allocation-v2 --output target/record-union-allocations.json
python3 docs/experiments/record-values/check.py \
  target/record-union-bundle-v2/bundle target/record-union-regression-v2 \
  --output target/record-union-regression.json
```

The allocation driver reuses the existing counter host from console-read-key and
relinks the exact generated application object and host recipe. It does not read
terminal keys; the workload constructs key values and exercises their value methods.

## API reference

The matching native core and CLI signature scaffold include all five generated key
members. The native reference uses the existing pinned RavenDoc 03bcc23d6 renderer.
Its operator URL collision overwrites == with !=, so both signatures are explicitly
covered in the linked [manual console reference](../../../api-docs/console.md#key-value-equality)
and tracked in api-docs/manual-members.json. Fixing the general renderer route is a
Raven-main follow-up; it is not hidden by describing missing documentation as a
native platform limitation. Website publication remains a separate manual action.

Documentation validation: both reference snapshots pass their checks, all four
native snapshot tests pass, and the complete website build/link validation passes.
Existing missing-summary notices remain tracked; the website was not published.
