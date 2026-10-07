# Source-built Console gate — 2026-10-06

Unchanged Console/ConsoleReadError sources compile to Console.dll; a separate consumer
imports that artifact natively and executes with exit 42, exact stdout and stderr.
It checks string/int/bool/ulong/nint/nuint overloads, UTF-8 text, line input, EOF,
closed-wrapper errors, and a fresh output wrapper sharing the still-open channel.
The tested source is [console-consumer.rvn](bootstrap/console-consumer.rvn).

Use `verify_source_console.py --help` for required paths. Build the Raven target Probe
with `-p:RavenRoot=<compiler-checkout>` and supply the source-native-integer gate's
NativeIntegers.dll and ownership.json. Supply matching Numbers.dll, compiler, and runtime.
The script generates a primitive comparer-storage core without Console and a seed with
only the exact legacy Console type/WriteLine service removed. It records input, source,
compiler component, runtime and output hashes in validation.json. The checked-in
[evidence](source-console-2026-10-06.json) records this run, including every command.
Raven a6ee91610 was used; runtime changes are based on fda3d2f5 and identified by hashes.

No public Console signatures changed. The CLI primitive core remains an explicit
bootstrap dependency. This is a reduced bootstrap profile, not a compiler name-resolution
workaround: source Console is the only selected owner in the consumer. Its dependencies
are the existing native Numbers and NativeIntegers libraries; no source stubs or CLI
projections substitute for those libraries.

The source WriteLine InternalCall has CLI-style no-result execution. Blocking host-call
completion previously pushed Value::Void unconditionally, leaving an invalid stack at
ret. It now respects the callee convention. Legacy unit-returning WriteLine still pushes
its inhabited value; focused runtime coverage exercises both. This preserves the current
CLI distinction between void execution and NeoCLR's explicitly inhabited unit.

The [full-owned-handle compilation audit](native-bootstrap-console-2026-10-06.json)
now reports 14 diagnostics across 190 source inputs (previously 30/188). It publishes no
assembly. Remaining errors are RuntimeFailure/NativeAllocation imports and names,
let-else termination, and HTTP Task return conversion. That diagnostic audit retains its
existing seed reduction and is not an execution gate for a complete System assembly.

Validation: 12 Console tests, three console-stream tests, the no-result service
signature test and the source consumer pass. The API snapshot was regenerated from
the matching Probe; its assembly is unchanged (only the generator input fingerprint
changed). No website build or publication was performed. Raven documentation is
synchronized in a6460f74e; no compiler code changed in this slice.
