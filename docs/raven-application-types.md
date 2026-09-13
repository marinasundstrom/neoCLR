# Raven application types

## First slice (2026-09-13)

The source import bridge now admits non-generic application classes and value types,
instance fields, constructors, property accessor bodies and ordinary instance methods.
Classes retain identity through calls and `ArrayList<T>` storage. Values copy through
assignment and collection insertion. The Raven spelling for the latter is `struct`;
this does not change the legacy Neo frontend.

Run [application-types.rvn](experiments/raven-target/samples/application-types.rvn)
with the source bridge and the experimental Raven branch. Its output is
`42`, `99`, `7`, `42`, `7`, on separate lines. Preview 4's published bridge/SDK do not
contain this slice; their assets remain unchanged.

```
python3 docs/experiments/raven-target/verify_application.py /path/to/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

The project and declaration assembly can come from the Preview 4 demo. Build the
Raven compiler and then the bridge as described in the [integration guide](experiments/raven-target/README.md).
The check also verifies fresh saved-source compilation, emitted source-map lines and
rejection of type initializers and readonly fields before execution.

## CLR comparison and implementation layers

This follows the existing [type-category decision](roadmap.md#type-model-migration-directive-2026-09-12) and
[interface contract research](raven-interface-contract.md#netcli-comparison-and-placement):
ECMA-335 Partition I §8 defines value/object behavior, Partition II defines TypeDef,
Field and MethodDef metadata, and Partition III specifies `newobj`, `call`, `callvirt`
and field access. The compiler still emits these ordinary CLI constructs. This slice
fills a neoCLR importer gap, rather than claiming an improvement over CLR behavior.

The bridge derives application declarations from supplied metadata instead of adding
per-application entries to the library catalog. It uses token-derived internal type
and field names, with method-token/IL-offset mappings preserved after body relocation.
Property accessors execute; application PropertyInfo metadata projection is not yet
included. No new metadata encoding or instruction is required.

Nominal class constructors use the existing runtime class allocation path. Value
constructors currently lower to a helper that initializes a value local, calls the
constructor body through its managed address, and returns a copy. This follows CLI
value initialization while keeping the older runtime constructor-capability protocol
separate. Value instance no-result methods adapt to the existing unit-return receiver
convention internally; the input CLI still has a no-stack-result void return.

Unsupported metadata fails explicitly: generic application definitions, static
initialization/storage, enums, custom/explicit layout, nested definitions, readonly
fields were outside the first slice. Inheritance/interface support is described below. Reachable bodies
retain the existing bounded opcode and exception-handler restrictions. This is not a
general-purpose PE loader or a full metadata verifier. No performance claim is made.

## Raven findings and target policy

Two compiler issues surfaced: constructing target-library generics with application
TypeBuilder arguments crossed MetadataLoadContext boundaries, and rewritten generic
member signatures incorrectly created an external self-assembly reference. The
experimental Raven branch (`cbd87efa8`) now preserves generic signatures without host type loading
and keeps application type references scoped to their defining module. Focused metadata
tests cover both class and value arguments. The compiler uses
[Type.MakeGenericSignatureType](https://learn.microsoft.com/en-us/dotnet/api/system.type.makegenericsignaturetype?view=net-10.0)
(documentation consulted 2026-09-13) for the mixed metadata/source signature.
Microsoft documents this as a reflection signature facility; compatibility with our
persisted emission path is demonstrated by the tests, not assumed from that description; the neoCLR execution check supplies the
end-to-end behavior evidence. Validation passed 12 focused Raven metadata tests,
all 49 existing saved-project cases, and the four application checks.

The author emphasized that this work should also stabilize Raven as a versatile
compiler, with nanoFramework targeting as relevant prior experience. Distinguish:

- Framework-neutral compiler bugs, candidates for separate review/upstreaming.
- Target-configurable contracts, such as Iterable/Iterator versus IEnumerable/IEnumerator.
- Deliberate neoCLR semantics, particularly generic Void and Result-based errors.

Making neoCLR work with Raven remains the primary objective. All Raven edits stay on
its experimental feature branch; ordinary .NET support must retain its behavior.
`Task<Void>` and its no-payload completion require a future async design, including
language return handling, emitted signatures, awaiter contracts and runtime suspension.
This records a requirement, not an async implementation or a chosen task API.

## Interface and inheritance slice (2026-09-13)

The source bridge now also admits application interfaces with public abstract
instance contracts, class implementations, interface inheritance, and ordinary
non-generic class inheritance with virtual/abstract methods and overrides. Same-name
public MethodImpl records emitted by Raven for implicit implementations are validated
against their contract; renamed explicit implementations and default interface bodies
remain outside this slice. Value-type interface implementations are not yet admitted.

The runtime now dispatches nominal class `callvirt` through the concrete allocation,
including an interface implemented by a base class. Direct `call` preserves its exact
selected method. Override validation preserves the no-result convention as well as
parameter and return types. Class constructor calls can chain to this type or the
direct base, preserving the same allocation. Runtime checks reject a different
receiver, repeated chaining, cyclic delegation, and a derived constructor returning
without chaining. Class fields retain their existing allocation-default rules.

This extends the earlier [inheritance research](class-dispatch.md) and follows CLI
virtual dispatch and constructor-call patterns. It does not introduce value slicing,
a new virtual-call opcode, or an exception hierarchy. The bridge eagerly admits
instance bodies of the used application types so dispatch targets cannot remain
unvalidated merely because source calls mention only an interface. The existing
body/type limits still apply. Application names remain token-derived in runtime
introspection; emitting the full application reflection surface is later work.

Raven also incorrectly forced final/new-slot flags for declared abstract or virtual
methods that implicitly implement an interface. The experimental compiler correction (`62105de24`)
preserves declaration intent; tests exercise ordinary and target metadata emission.
This is a compiler bug fix, distinct from configurable Iterable/Iterator contracts
and the intentional generic-Void difference.

The interface sample prints `42`, `99`; the inheritance sample prints `7`, `42`.
Both are included in `verify_application.py`. Runtime tests in
`tests/nominal_inheritance.rs` cover dispatch, no-result overrides and rejected
constructor behavior, including execution without opting into typed verification.

Validation: the six application checks pass, as do the focused runtime inheritance/
dispatch suites and strict Clippy. Ten focused Raven tests pass, including dispatch
flags through ordinary and experimental metadata emission.
