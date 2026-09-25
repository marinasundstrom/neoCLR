# Runtime reflection consumer — development, 2026-09-25

This application imports `System.Runtime.Reflection` and uses the real library
extensions over runtime-backed `TypeInfo` and `PropertyInfo`. It does not provide
application-local copies of the reflection implementation or a JSON object mapper.

`CreateReport` in [Main.rvn](Main.rvn) demonstrates Result propagation through
construction and assignment. The fixture checks constructor and setter effects,
boxed scalar reads, invalid values/receivers, read-only and private setters,
private constructors, property tokens, accessor accessibility and nullable reference
round trips. The runtime-owned descriptors and reflected receivers remain rooted.

[TerminalFault.rvn](TerminalFault.rvn) covers a non-void method ending at a terminal
Fault call. Its original message must survive import and execution.

```sh
python3 docs/experiments/reflection-execution/verify.py \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

Use the compiler containing the target-metadata nullable-reference fix (`defb93a49`
on Raven main, integrated as `a66df01a1` on neoclr), matching bridge/reference,
regenerated System library and runtime. The earlier frozen compiler cannot emit
this nullable generic extension signature. Source accessibility fields and application
property tokens also require the matching bridge/runtime.

The first passing run allocated 308 objects, peaked at 165, collected four times and
finished with zero live objects under a 512-object heap. These are synchronous
construction/accessor checks, not evidence for asynchronous suspension or performance.

The assertion helper takes one Boolean because the current private-call conversion
path supports one materialized argument. The nullable String property starts with an
empty string; null assignment is exercised through reflection. Direct source null-to-String
field initialization remains outside the importer's supported conversion path. These
limits are not claims about the intended Raven language or final platform contract.
