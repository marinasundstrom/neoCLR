# Ordinary value display overrides — 2026-10-20

Index, Range, InvalidRangeError, OverflowError, EnvironmentError, InvalidUtf8Error,
InvalidDateError, InvalidTimeError and InvalidPathError now override Object.ToString.
Their existing bodies and strings are unchanged. The reference bridge declares the
same overrides. No special runtime type, generated equality interface or box layout
is introduced.

The [baseline](baseline-20261020.json) builds successfully but fails the boxed-display
assertion in interpreter, JIT and AOT. The repaired [consumer](Main.rvn) passes all
three engines; [validation](validation-20261020.json) records commands and hashes.
It covers direct, boxed and composite formatting, Index/Range boundaries and typed
interfaces, equal hashes, null/wrong types, distinct empty struct types and GC.

Compared with .NET, an authored value ToString override participates in ordinary
Object virtual dispatch. Method hiding is insufficient. This repair does not remove
Object boxes or the component strings created by Index/Range formatting. Returning
an error label reuses its existing literal. Buffer-based composite formatting is a
separate performance opportunity, not a measured gain from this correctness fix.

Both reference snapshots were refreshed from the matching library/bridge. The
optional PostgreSQL provider is included in the documentation bundle so existing
public routes remain present. Native metadata rendering and snapshot validation pass.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/value-display/ValueDisplay.rvnproj \
  --bundle target/value-display-bundle/bundle \
  --aot target/value-display-bundle/bundle/bin/neoclr-codegen \
  --output target/value-display-app
python3 docs/experiments/value-display/check.py \
  target/value-display-bundle/bundle target/value-display-app \
  --output docs/experiments/value-display/validation-20261020.json
```
