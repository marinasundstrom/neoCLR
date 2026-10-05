# Ordinary .NET field-return repair — 2026-10-05

Raven main `08f34891b` repairs a reference-field assignment whose RHS block returns
from its enclosing method. On parent `e33591945`, both the early-return and fallthrough
paths throw `InvalidProgramException`; the three existing reference-owner mutation
controls pass. This was a real ordinary .NET compiler defect, discovered during native
HTTP integration, not an unsupported NeoCLR API.

The emitter evaluates the receiver once, saves it, evaluates the RHS on an empty stack,
and reloads the saved object for the store. Debug and Release tests replace the receiver
variable during RHS evaluation and check that the original object is still selected.
Early return skips the store; fallthrough stores the value. Receiver evaluation precedes
RHS effects. Existing value-owner address behavior remains unchanged.

Ten focused .NET tests pass on main. The same fix is integrated as Raven `9a4f74884`;
14 focused field/address and portable pattern-body tests pass there. The local main
integration is complete and the temporary `codex/fix-field-return-stack` branch is deleted.
No remote push, runtime change, Runtime Contract change or new metadata encoding is
part of this slice. Native emission's existing receiver spill remains separate.

Reproduce in Raven with .NET 11:

```sh
dotnet test test/Raven.CodeAnalysis.Tests/Raven.CodeAnalysis.Tests.csproj \
  --filter 'FullyQualifiedName~ReferenceOwnerFieldTests|FullyQualifiedName~RefFieldCodeGenTests|FullyQualifiedName~FieldInitializationTests|FullyQualifiedName~ValueTypeReceiverCodeGenTests' \
  -p:WarningLevel=0
```

The integration check also selects `PortablePatternBodyTests` and supplies
`NeoClrMetadataProject` pointing to the matching metadata project from NeoCLR
`3020b237`. The new regression is
`ReferenceFieldAssignmentWithReturnPreservesReceiverOrder`; assertions observe execution,
mutation and identity, rather than instruction layout.

The native sample count remains nine of ten. The next sample, `application-inheritance`,
requires coordinated general class virtual/abstract authoring, reader materialization,
validation, Raven declaration admission and runtime dispatch evidence. Existing Object
slots and interface-abstract method flags are insufficient; merely relaxing Raven's guard
would not implement the contract. Native LSP/VS Code acceptance and release website,
download and setup review remain required after the sample gate.
