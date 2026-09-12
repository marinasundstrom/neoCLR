# Raven minimal target contract: emission evidence

Recorded 2026-09-12. This completes the investigation/probe part of slice 2 in the
[Raven experiment](raven-target-experiment.md). The [reproducible probe](experiments/raven-target/README.md)
uses the compiler revision pinned in the [backend map](raven-backend-integration-map.md).
A standalone target and executable binary path remain planned.

## What the first program requires

| Concern | Observed compiler output / .NET baseline | neoCLR requirement and placement |
| --- | --- | --- |
| Library lookup | PE declarations and framework discovery; target fixture Console call binds correctly | Compiler target must resolve an explicit declaration set. Missing Console currently falls back; enforce target isolation before claiming success. |
| Identity | Core retargeting changes assembly scopes, but leaves mscorlib from the fixture | Choose a canonical System identity and validate every reference against a closed declaration set. Name rewriting alone is insufficient. |
| Console | Static `System.Console.WriteLine(System.String)` returning CLI void | Bind to the actual System Console method and its existing runtime intrinsic, not the placeholder fixture body. Preserve ordinary static-call resolution. |
| Main body | `nop`, `ldstr`, `call`, `ret` | Reuse compatible operations; specify the no-result boundary below before translation. |
| Generated definitions | Unit, nullable attributes, entry-point helper and their methods | Decide explicitly which metadata is supported or omitted by a target emitter. Unsupported definitions must not disappear accidentally. |
| Core types | Object, ValueType and attribute definitions absent from the fixture | Provide usable compiler declarations without implying a decision to require ValueType ancestry in neoCLR's object model. |
| Errors | This input emits no exception regions | Reject unsupported exception regions; future Result projections must call target APIs rather than introduce guest catch wrappers. |

The report inventories assembly/type references, method signatures, bodies and entry point.
It is not an exhaustive metadata validator: custom-attribute arguments, recursive signature
resolution, calling conventions and feature closure need validation in the artifact slice.
The host framework still supplies most binding types. This is an emission probe, not an
independent core library. No new allocation or reference-default semantics are selected.

## No-result calls are a real boundary

The CLI permits a method to return no value; `ret` then leaves no result on the caller's
stack. Calls identify their target/signature through a metadata token. These are runtime
instruction contracts, distinct from Raven's language Unit projection. See Microsoft's
[`ret`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ret?view=net-10.0)
and [`call`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.call?view=net-10.0)
documentation (consulted 2026-09-12), and the ECMA-335 baseline in
[design research](design-research.md).

neoCLR's current [Console](../runtime/System/Console.neoil) returns inhabited `Void`.
Reading Raven's standard call/return bytes directly with that convention would leave an
unexpected value on the evaluation stack. There are three candidate adaptations:

1. Add explicit no-result method support to the runtime and preserve CLI stack behavior.
   This serves other compilers, but adds a distinction alongside inhabited Void throughout
   signature validation, invocation and reflection.
2. Translate a bounded CLI input into neoCLR's existing representation, inserting the
   required result disposal and Void return construction. This preserves today's runtime
   and writer, but requires verified signature-driven translation, debugging offsets and
   a clear boundary between input IL and internal instructions.
3. Change Raven's emitter to target inhabited Void directly. This makes the difference
   explicit in generated artifacts, but increases compiler coupling and complicates reuse
   by C# and other frontends.

**Provisional preference:** evaluate option 2 alongside a CLI container in slice 3, using
this tiny static-call subset. It has the smallest initial compiler change. Do not silently
change the meaning of standard CLI bytes or claim general compatibility. The choice is
not implemented; confirm it with empty methods, nested void calls, non-void returns and
invalid stack states before loader work. A broader runtime contract may justify option 1.

## Next bounded work

Compare a CLI PE/metadata subset with a versioned neoCLR format plus reference facade.
First make a closed core declaration fixture and expose/reject host fallback in the target
resolver. Inventory generated helper requirements and choose their treatment. Then define
assembly identity, supported tables/signatures/opcodes, size limits, feature rejection and
the call boundary. Loading and executing the real System library remains the milestone;
serializing this report or executing its empty fixture is not equivalent.

Validation must include missing assemblies/members, wrong signatures, invalid tokens,
unsupported exception regions and inconsistent stack behavior, as well as HelloWorld.
The existing neoCLR source/JSON path remains the execution control. Performance, JIT
benefits and general Raven/C# compatibility have not been measured or demonstrated.
