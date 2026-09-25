# Directional interface consumer

Development sample using EquatableTo<int>, ComparableTo<int> and
ConvertibleInto<int>. Compile with a matching neoCLR target compiler/reference,
import through the Raven bridge, and run against the matching Raven System library.
Expected output is in [expected.txt](expected.txt).

The explicit implementation converts to 42 and compares against 42; the primitive
ordering contract compares equal. No implicit conversion is involved.

The current local compiler fails generated-record interface assignment with both
old and new names. Record configuration is renamed, but this fixture uses explicit
implementations so it does not claim to validate that separate compiler feature.

Validation (2026-09-25): compile/import/run produced exactly `42`, `0`, `True`.
The 47 selected Rust tests, bridge signature checks and library/API snapshot checks
pass. Full tests and website build were skipped by author direction. The older
source-coverage audit still stops at its unrelated Int64ToString caller check; its
existing inventory is updated only for these interface contracts.
