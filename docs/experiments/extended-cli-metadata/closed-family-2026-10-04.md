# Closed families and protected constructors — 2026-10-04

The ordinary Raven driver now compiles the same closed root/protected constructor
case for .NET and NeoCLR. Both direct programs return 42 after initializing base and
derived fields and mutating through a base-typed alias. Each target also builds a
library, then builds and runs the consumer with only that artifact referenced. An
external direct child rejects before output (RAV0306).

[Commands, exact dependencies and artifact hashes](closed-family-driver-2026-10-04.json).
Reproduce with `bootstrap/verify_class_bases.py --closed-family --help`; its compiler,
runtime, primitive core, retained seed, source library and ownership manifest inputs
remain explicit. Generated library/consumer sources are exact partitions of the
checked-in sample; no library declarations enter the consumer invocation.

## Representation and boundaries

The metadata library's AddClosedClass and five-argument TypeDefinition constructor
share validation. Native closed roots are nongeneric top-level abstract, nonsealed
classes. Protected constructors use standard CLI Family and native protected/Family
access. No new instruction or native schema version is introduced. Root constructors
cannot be invoked via allocation; derived constructors call them on their receiver.

The existing native is_closed_hierarchy flag is authoritative. The nonexecutable CLI
reference projection records Abstract but currently has no closed-family custom
attribute. Executable CLI Write rejects these native declarations explicitly; the
ordinary Raven .NET backend retains its existing closed-family metadata emission.
CLI-only readers must not infer closure from Abstract or confuse it with Sealed.
Richer closed-family/permits metadata encoding is an author-requested later design pass.

Native definitions and introspection preserve the classification and canonical direct
children. Raven symbols consume these views, including actual local base identities;
source-file and explicit-permits policy stays in Raven. Emission consumes symbol facts
and explicit artifact identities. DeclareClassBase supplies a bounded output-owned
reference conversion contract for nongeneric classes within one dependency assembly,
without reading importer objects. The runtime checks actual linked dependency bases.

This does not author a source class deriving from another assembly, generic/nested
closed roots, arbitrary abstract/virtual class methods, protected ordinary members,
closed-interface enforcement or general sealed-leaf runtime enforcement. Those remain
separate gates. No primitive-core/seed/source-ownership change or projection fallback
was introduced.

## Validation and next failure

- 146 C# metadata groups pass, including definition/builder parity, native round trips,
  facade accessibility/direct children, invalid flags, abstract allocation rejection,
  and cyclic/conflicting/foreign/interface reference-base contracts.
- Generated closed-family PE verifies and executes with return 42 on NeoCLR.
- 35 focused Raven .NET 11 capability/constructor regressions pass.
- The runtime prerequisite passed 55 affected Rust tests.

The unchanged five-source JSON group now passes the closed/protected declaration
boundary and stops at a lowered BoundPropagateExpression. No output is published.
[JSON retry](json-after-closed-family-2026-10-04.json). Next reduce that failure and fix
shared lowering rather than inventing target-only propagation semantics. JSON
compilation/execution is not yet claimed complete.
