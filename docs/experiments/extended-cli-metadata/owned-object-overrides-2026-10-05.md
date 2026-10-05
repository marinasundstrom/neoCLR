# Owned Object override execution — 2026-10-05

From parent `c260c2b0` plus this slice, the metadata API authors a derived class against
a local root, with protected base construction and three concrete overrides. Equals
uses the exact authored Object identity. Both manual definitions and builder convenience
methods share validation. The writer checks signature selection again before encoding;
adding a root after a bootstrap-based Equals cannot publish mixed identities.

As in CLI, root declarations use Virtual/NewSlot while overrides use Virtual with the
inherited slot. Native encoding uses existing flags, type signatures and dispatch.
Without a local root, explicit legacy System binding behavior is unchanged.

Validation: 158/158 C# metadata contract groups and 14/14 Rust root-identity tests pass.
The new Rust case loads API-produced PE and invokes three API-produced wrapper functions:
ToString returns "derived override", GetHashCode returns 93, Equals returns true.
Constructors and calls are API-produced, not consumer stubs. The constants are fixture
method bodies, not implementations of production Object services. Ordinary CLR override
execution controls still pass; wrong/bootstrap root signatures and root self-overrides
are rejected. Introspection checks virtual flags and canonical Equals parameter identity.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --owned-object-overrides-image tests/fixtures/metadata-container/owned-object-overrides.pe
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
  cargo test --release --test object_root_identity
```

Fixture SHA-256: `e7ddfc3c661fe74a78d1c226e42e288cb0d78436280fa2b161f2cae1fe7e2da3`.
The existing root/boxing fixture is unchanged. Native runtime code is unchanged;
explicit host admission still uses `4e9e4045`. Regeneration changes projection MVIDs.

The native-reader-to-CLI-projection API still rejects inherited classes explicitly;
the compiler's original graph-to-PE path passes. No new rewriting support is claimed.
The manual metadata API reference is updated; the generated guest snapshot check retains
its previously reported stale-input failure. No unrelated website build.

Matching Raven documentation is committed at `9c28e4ca2`.
Raven code reviewed at `926af0db3`: do not remove the source-root guard yet. SourceTypePlan
rejects the baseless abstract declaration; SourceCallablePlan lacks a root virtual-slot
capability; special Object mapping must select the source owner instead of importing it.
Those shared contracts and target adapter changes must be tested with ordinary .NET
controls and the source-root executable probe. Driver/consumer root selection, production
System, VS Code/LSP and native Tasks/await release samples remain open.
