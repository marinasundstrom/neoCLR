# Explicit generated record equality contracts — 2026-10-20

This slice checks the ordinary interface path before adding automatic native record
interface synthesis. Source `record struct Key(Value: int) : EquatableTo<Key>`
already uses its generated typed Equals implementation for interface dispatch.
The record remains an ordinary value type with a normal interface implementation;
this is qualification of existing behavior, not a newly introduced record ABI.

A record with unsupported component equality/hash was previously rejected during
metadata encoding with NEOMETA003 (`missing public interface implementation`).
The [baseline](baseline.txt) retains that result. Native emission now detects the
promised selected-core EquatableTo<Self> interface when its generated equality body
cannot be supplied and reports a targeted NEOMETA001 component-contract error.
It does not add Object-identity fallback or publish an interface without a body.

Raven fix: `0c7ee885c` on `codex/neoclr`.
The source-core compiler probe tests supported and unsupported explicit contracts,
alongside the existing record/imported-component checks. The retained consumer
checks positive and negative equality through EquatableTo<Key> across interpreter,
JIT and AOT. [validation.json](validation.json) records commands and hashes;
[compiler-probe.txt](compiler-probe.txt) records the fresh compiler regression result.
Positive execution reuses the preceding matched union-formatting bundle because
this change affects only rejection of unsupported contracts. No new class-library
API, layout or calling convention is introduced; no API snapshot refresh is needed.

## Historical automatic synthesis gap

[Automatic native record interfaces](../record-auto-interfaces/README.md) now close
the nongeneric synthesis gap. This section records the preceding investigation,
not current guidance to add interfaces manually.

Shared Raven binding currently looks up System.IEquatable<T>, while neoCLR's core
exposes EquatableTo<T>. Historical RavenRecordAssemblyName/RavenRecordEquatableType
project properties are not an implemented record runtime contract. .NET's ordinary
IEquatable<T> synthesis must remain the default. A native mapping needs explicit
assembly/type identity and tests for configured, absent and invalid contracts.

Do not simply attach the interface to every native record yet: generic owners and
unsupported components still lack emitted equality bodies. Interface synthesis,
body admission, imported metadata and semantic conversions must agree. Otherwise
we would advertise semantics the compiler cannot provide. ConsoleKeyInfo's implicit
interface and imported-component use remain unresolved; this slice deliberately
claims neither an automatic mapping nor complete record semantics.

The next implementation should establish that coordinated contract, keeping
records as ordinary classes/structs with compiler-generated members/interfaces.
This follows the author's [.NET runtime/ABI default](../../design-research.md).
Runtime Contract configuration and ordinary .NET behavior are unchanged here.

```sh
python3 docs/experiments/record-interface-contracts/check.py target/record-union-format-bundle-v2/bundle target/record-interface-app --output docs/experiments/record-interface-contracts/validation.json
```
