# Native introspection dependency boundary — 2026-10-04

This follows capability batch 3 in the full-System strategy. The document/value JSON
layer now compiles and executes, so introspection can be developed as its own shared
capability batch rather than by patching individual JSON APIs.

## Evidence and existing implementations

Adding the unchanged introspection, runtime reflection and RuntimeContext sources to
the serializer candidate yields 191 diagnostics, including 51 distinct missing
RuntimeServices names. [Exact candidate, diagnostics and hashes](json-introspection-expansion-2026-10-04.json).
These are binding-stage results; they do not establish downstream emission support.
The candidate also lacks HashCode, the ParameterSnapshot bootstrap view, and a selected
RuntimeTypeHandle declaration. Bootstrap Object has no GetType member and the ownership
manifest has no RuntimeTypeOfContract. This is not a complete dependency closure.

The legacy bridge already inventories these services in
`docs/experiments/raven-target/RuntimeServiceBindings.cs`. The native subset in
`NativeServiceCatalog.cs` deliberately rejects opaque-handle and arbitrary nominal
signatures. Existing runtime implementations must be reused where present, not inferred
missing from compiler diagnostics. The runtime already has `LoadTypeToken(Type)`, its
typed-stack verifier result RuntimeTypeHandle, and type-inspection service gating.
ParameterSnapshot is an existing bootstrap-only view of a parameter-info vector, with
Length/Get mapped to vector operations; it must not become a second mutable array ABI.

## Ordered next slices (planned, not implemented)

1. **Type-handle vertical gate.** Author/read an opaque RuntimeTypeHandle signature and
   a type-token operand through the C# metadata API. Cover local/external/constructed
   identity and generic substitution, dependency mismatch and round trips. Use CLI
   type references and `ldtoken` where supported; no host reflection object crosses
   the shared emitter boundary. Prove generated native PE loads and returns the same
   canonical type identity. Do not treat the empty source handle declaration as an
   ordinary data struct.
2. **Compiler and bootstrap contract.** Map the existing RuntimeTypeOfContract to the
   selected source descriptor/context identity. Supply semantic type operands from
   Raven symbols and explicit artifact identities. Keep ordinary .NET defaults and
   its Reflection/Emit backend unchanged. Bind only a small proven handle-service
   subset initially; add a native `typeof` consumer and preserve .NET controls.
3. **Descriptor ownership and factories.** Decide exact seed/source ownership for
   Object.GetType, RuntimeTypeInfo, nominal/function descriptor implementations and
   ParameterSnapshot. Add HashCode to the explicit source set. Preserve immutable
   snapshot semantics and factory layouts; resolve required inheritance/override
   gaps through existing target capabilities. The C# metadata-only introspection
   facade remains separate from execution-bound Raven runtime descriptor instances.
4. **Member services then mapping.** Add property enumeration, accessors and creation
   contracts with actual runtime bindings; then arrays, construction and invocation
   signatures required by ObjectMapper. Recompile unchanged sources and run the
   existing public serializer/stream samples. Treat fresh residual failures at their
   owning layer, not as confirmation that all current cascades are compiler bugs.

## .NET comparison and provisional decisions

The .NET baseline uses [ldtoken](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldtoken?view=net-10.0)
to produce runtime handles and [Type.GetTypeFromHandle](https://learn.microsoft.com/en-us/dotnet/api/system.type.gettypefromhandle?view=net-10.0)
to obtain a type object (primary documentation checked 2026-10-04). NeoCLR already
separates its opaque handle from TypeInfo via RuntimeContext. Reusing the instruction
and explicit identity model avoids a new opcode and preserves the current metadata
architecture; the cost is coordinated signature, builder, reader, symbol and runtime
validation. It does not imply recreating the .NET reflection API or changing the
metadata-only C# facade into a runtime loader. Scope of supported token categories,
default-handle behavior and cross-context identity require explicit tests before
publication; these notes do not declare them implemented.

## Type-token authoring checkpoint

The metadata API now authors opaque handle signatures and standard type tokens.
147 C# metadata groups pass. The focused executable gate is:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --type-handle-runtime target/debug/neoclr /tmp/type-handle-runtime
```

This verifies the generated native PE and executes it with exit 42. CLI execution
also checks canonical local/constructed identities and distinct method/owner generic
substitution. No runtime opcode or format-version change was necessary. This is a
partial first slice: external dependency identity, runtime handle-service comparisons,
Raven typeof, descriptor factories and JSON object mapping are still open. The
metadata-only C# facade remains independent of runtime descriptors. No website
capability claim or guest API snapshot change is appropriate for this host-only API.


## Compiler boundary checkpoint

Raven `1c3e14bc7` now passes the [native typeof gate](native-typeof-2026-10-04.md),
including an instantiated method parameter and an external nominal token. Its
artifact-only consumer executes the real TypeName binding through a test provider.
The remaining work in slices 2–4 is production ownership, descriptor factories,
ParameterSnapshot/vector contracts, declaration/override admission and the member
services required by the unchanged mapper. This gate does not replace those sources.


## Runtime handle-service checkpoint

The [expanded gate](native-handle-reflection-2026-10-04.md) now exercises real runtime
identity, generic argument, shape/display/token and object-type services, plus
parameterless reflection construction. The compiler's native Object signature gap is
fixed in Raven `80ebdc0a7`. Reassessing the unchanged production source closure with
HashCode and the current catalog still rejects before publication: descriptor-returning
services, ParameterSnapshot, Object.GetType and the production typeof contract need
coherent ownership. Do not interpret cascading conversion/overload errors as separate
compiler regressions until that dependency closure binds.
