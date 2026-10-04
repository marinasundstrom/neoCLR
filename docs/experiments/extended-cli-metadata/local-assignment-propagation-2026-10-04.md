# Local assignment propagation — 2026-10-04

The reduced [native consumer](bootstrap/local-assignment-propagation.rvn) compiles,
verifies and executes with return 42 using the existing source-built libraries.
It checks successful assignment, single operand evaluation, failure residual and
that failure returns before the following mutation. [Commands and artifact hashes](local-assignment-propagation-2026-10-04.json).

Raven shared lowering previously left propagation in local assignment right-hand
sides. The .NET backend already supported this source shape; normalizing direct and
eager-binary right-hand sides lets the native emitter use the same semantics.
Two new focused lowering tests failed before the fix. Sixteen focused integration
.NET tests pass; 24 propagation/Runtime Contract/async tests pass independently on
main at `9faabb1a2`. The fix branch was deleted after integration. Native integration
is `eb83baa4d`; the runtime remains `68e0f74f`. Nothing was pushed remotely.

No runtime, public metadata API, CLI bridge encoding, ownership or dependency
selection changes. Property, field, array and parameter assignment spilling remains
outside this bounded fix. Native support for constant payload subpatterns and static
field storage is not established by this consumer: it uses instance storage and
extracts case payloads before comparing them.

The [unchanged JSON retry](json-after-assignment-2026-10-04.json) still rejects another
`BoundPropagateExpression` and publishes no output. Nested conditional/argument
propagation needs reduction next; JSON compilation/execution is not yet complete.
