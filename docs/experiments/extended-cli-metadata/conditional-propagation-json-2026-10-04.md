# Conditional propagation and native JSON library — 2026-10-04

The five unchanged sources `JsonDocument`, `JsonSyntax`, `JsonValue`, `JsonError`
and `ReflectionError` now compile as an independent native library. The runtime loads
and verifies the linked library. A [separate public consumer](bootstrap/json-library-consumer.rvn)
imports only artifacts and returns 42, checking number parsing, duplicate-field
rejection, collection mutation and object identity through an alias. No JSON sources
are passed to the consumer invocation. [Commands, dependency and source hashes](conditional-propagation-json-2026-10-04.json).

A [focused conditional consumer](bootstrap/conditional-propagation.rvn) separately
checks both branches and both propagation outcomes, including prefix side effects
and early error returns; it verifies and returns 42 with empty stdout.

## Compiler change and .NET control

The reduced lowering test failed before the fix: both conditional branches retained
`BoundPropagateExpression`. Shared lowering now stores branch success into a common
temporary and keeps residual returns at statement boundaries. Eager operands retain
left-to-right evaluation. Disposal/using blocks and arbitrary argument/receiver
spilling remain excluded. There is no public metadata API, encoding, runtime or
bootstrap configuration change.

Raven integration `3a99915c8` passes 21 focused .NET tests. The independent fix is on
main at `e1df355a2`, with 29 propagation, Runtime Contract and async tests passing;
its temporary fix branch was removed. This is local integration, not a remote push.
Ordinary .NET keeps its existing emitter. The .NET tests prove language behavior;
they do not establish a paired .NET source-JSON library gate.

## Next dependency boundary

The [unchanged serializer/object-mapper expansion](json-serializer-dependencies-2026-10-04.json)
fails before publication with this same dependency catalog: `TypeInfo`, `PropertyInfo`,
`Object.GetType` and reflection-array runtime services are absent. Establish the
introspection source/dependency ownership and runtime-service contract before treating
cascading symbol/conversion diagnostics as separate compiler defects. The public codec
and object mapping are not included in this milestone; internal DocumentReader/Writer
are compiled and verified, but their full document round-trip execution is not yet proven.
