# Integral Object contracts — 2026-10-20

SByte, Byte, Int16, UInt16, UInt32, UInt64, IntPtr and UIntPtr now participate in
Object equality, hashing and invariant decimal display in interpreter/JIT/AOT.
They reuse existing scalar boxes, exact nominal tags, typed payload readers and
signed/unsigned integer formatting services. No new primitive layout, public
interface, compiler mapping or ABI is introduced.

[Validation](validation-20261020.json) records passing execution and exact artifacts.
The [Raven consumer](Main.rvn) covers signed minima/maxima, unsigned high bits,
UInt64.MaxValue, native-width zero/negative/high bits, separate boxes, wrong types,
null, dictionary keys, composite formatting, direct inherited calls and explicit GC.
Native width is qualified on macOS ARM64; no Windows or 32-bit run is claimed.

An initial native run found that narrow payloads must use the shared typed reader:
raw 32-bit loads do not sign-extend stored SByte/Int16 values. The final adapter uses
the same reader as ordinary field reads/unboxing, including zero-extension for
unsigned narrow types. Equality preserves the full value; hashes can collide across
nominal types without making the values equal.

Comparison: [.NET Int16](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Int16.cs)
and [SByte](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/SByte.cs)
use exact primitive types, value hashes and decimal formatting. neoCLR retains its
culture-independent default numeric display; broader format/provider support remains
separate. Wide hashes fold the upper and lower halves; numeric hash values are not
persistent identifiers. Current native-width behavior is 64-bit.

Equality/hashing do not create intermediate guest objects inside these adapters.
Object conversion still boxes; decimal display allocates the final UTF-8 string
through existing services. No throughput or measured allocation reduction is claimed.
Existing typed EquatableTo declarations remain explicit and unchanged.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/integer-object/IntegerObject.rvnproj \
  --bundle target/integer-object-fixed-bundle/bundle \
  --aot target/integer-object-fixed-bundle/bundle/bin/neoclr-codegen \
  --output target/integer-object-fixed-app
python3 docs/experiments/integer-object/check.py \
  target/integer-object-fixed-bundle/bundle target/integer-object-fixed-app \
  --output docs/experiments/integer-object/validation-20261020.json
```
