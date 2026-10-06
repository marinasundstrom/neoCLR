# Source/native terminal-flow ownership — 2026-10-07

Raven's explicit RuntimeFailureContract selects an assembly, namespace and function.
Source and native-imported methods expose the same terminal fact to existing binding,
control-flow and lowering. The configured function must be public/static/non-generic,
a namespace member, take one by-value string and return void/unit. Arbitrary .NET
methods remain unchanged. This is a host runtime assertion, not inference from a name.
The existing lowering guard still rejects an impossible return from that function.

The native source-failure acceptance script now generates the `failure` manifest entry.
It checks: direct imported Fail produces the exact UserFault and exits 1; imported
Some/let-else returns 42; imported None/let-else raises the expected fault; compiling the
same flow with Fail sources in the selected source owner also returns 42. A wrong owner
fails without publishing Rejected.dll. No library sources enter imported consumer builds.

[Executable evidence](source-failure-flow-2026-10-07.json) records every command and
artifact hash. The runtime is b933c32b; the compiler snapshot was built from
9b73a7333 plus the terminal contract changes. Its component hashes identify that build.
The verified compiler changes are committed as Raven 296ca0f36. Reproduce using
`verify_source_failure.py --help` and the same explicit bootstrap/native-library inputs.
Eighteen focused C# tests pass: exact source signature/ownership, .NET exclusion,
configuration rejection, option cloning, and existing legacy/.NET terminal-flow controls.

The [full-owned-handle audit](native-bootstrap-failure-flow-2026-10-07.json) selects
Numbers as the source terminal owner and drops from 12 to **four errors across 192
inputs**. All six let-else and both HTTP Task-return conversion errors disappear.
Remaining diagnostics are NativeAllocation's missing import and three member uses.
The audit still publishes no assembly: full source encoding, linking and execution are
not yet established. NativeAllocation is the next bounded integration, including its
pointer signatures and native allocation/checked-size operations.

No runtime service or metadata format changes occur here. CLI void remains the signature;
the existing native Fail service enforces termination. The API/fault guides now mark the
previous source-flow limitation resolved. No website build or deployment is needed.
