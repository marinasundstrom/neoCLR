# Configuration reload lifecycle test

One named NeoClr.Testing case preserves the combined opt-in JSON reload lifecycle:
live DI/section reads, unchanged non-reloading configuration, malformed input
retaining valid data and exposing an error, atomic replacement clearing that error,
required-file deletion retaining data, optional-file creation/deletion, and disposal
stopping reload and rejecting a later load.

Main explicitly awaits that lifecycle, registers its completed Result through a
reference adapter with TestSuite, writes ConsoleReporter output and returns
report.ExitCode. It replaces the handwritten Panic/PASS reporting. Assertion order,
provider setup, two-second settling intervals and disposal sequencing are unchanged.
Setup and terminal runtime failures remain terminal; automatic async/parameterized
discovery is not added.

Python retains all seven READY-triggered file changes and isolated directories. It
checks the exact framework transcript separately from protocol lines and requires
empty guest stderr. Only the exact existing DAM2004 JIT compiler warning is allowed
and recorded separately. Interpreter, JIT and AOT use matching bundle inputs; JIT's
modules/helper/codegen are explicit.

```sh
python3 scripts/validate-configuration-reload.py --bundle <matching-bundle> \
  --runtime <matching-bundle>/bin/neoclr --aot <matching-bundle>/bin/neoclr-codegen \
  --output target/configuration-reload-tests
```

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#configuration-reload)
records the three-mode run and a negative control after the final disposal phase.
The existing [configuration contract and .NET comparison](../../../../docs/configuration.md)
remain applicable; production provider/DI behavior and platform qualification are
unchanged.
