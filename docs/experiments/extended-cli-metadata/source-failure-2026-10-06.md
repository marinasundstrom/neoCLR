# Source System.Fail — 2026-10-06

The unchanged System/Functions.rvn compiles with two internal RuntimeFailure adapters.
A separate consumer imports Failure.dll, calls System.Fail, and receives the exact
`source failure` UserFault, empty stdout and exit 1. Its following return 42 is never
executed. This is an expected terminal-failure gate, not a successful application exit.

`verify_source_failure.py --help` lists required paths: matching compiler, Probe,
Numbers library, ownership manifest, seed and runtime. It generates the comparer-storage
primitive profile without Fail, builds the native library and consumer, verifies the
linked program and checks execution. [Recorded evidence](source-failure-2026-10-06.json)
contains commands and all input/output hashes. Raven a6ee91610 and runtime code based
on 896728a9 were tested. The source adapter/type is internal; public Fail is unchanged.

The new exact no-result runtime service neoCLR.Runtime.Fail matches source void
execution. The legacy neoCLR.Runtime.Fault service remains inhabited-unit for old seed
and Numbers bodies. A separate identity avoids admitting conflicting return signatures
under one InternalCall name. Both use the existing UserFault implementation; no metadata
format change, new instruction or cleanup semantics are introduced. Five fault-code
regressions and native signature controls pass, including wrong argument/result rejection.

This follows the existing comparison with .NET in [terminal failures](../../system-fault.md):
Fail terminates a guest invocation while the embedding host survives. It does not add
.NET exception handling or promise finally/disposal execution.

The [full-owned-handle audit](native-bootstrap-failure-2026-10-06.json) drops from 14 to
12 diagnostics across 192 source inputs. It publishes no assembly. RuntimeFailure import
and name errors are gone. NativeAllocation still accounts for one missing import and
three names; six let-else termination and two HTTP Task return conversion errors remain.
Raven currently recognizes terminal Fail only by the legacy core identity. The next
compiler slice must supply source/native terminal-call ownership through an explicit
contract while keeping unrelated .NET methods unchanged. This gate does not claim to
have solved that semantic boundary, native allocation, or full System bootstrap.

Raven integration documentation is synchronized in 9b73a7333. The API snapshot was
regenerated from the matching Probe and checks pass; the reference assembly itself
is unchanged. Existing website fault semantics remain accurate; no website build or
publication was needed for this internal bootstrap adapter.
