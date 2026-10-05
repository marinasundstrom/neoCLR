# Raven source Object emission — 2026-10-05

Raven `246e8a5db` now emits a selected source System.Object through the independent
metadata library (`9d880af0`). Explicit ObjectRoot/ObjectRootSlot capabilities admit its
baseless abstract declaration and three concrete virtual slots. Source-root signatures,
base construction and overrides use output-owned definitions; the emitter does not
reopen loader objects. Bootstrap validation uses the registered assembly identity,
independently of which assembly owns Object.

The probe compiles a Raven source root and Item with overrides plus a Display method.
The PE is loaded into NeoCLR with explicit host root selection; the emitted Display
constructs Item and returns "source root override". This advances source-to-runtime
execution beyond manually authored metadata. The source uses fixture bodies rather than
the full production Object services. No runtime implementation changes were necessary.

Validation: pre-change focused compiler baseline 31 cases; final 47 cases pass, including
.NET inheritance/virtual/constructor behavior and explicit capability denial. The old
shared-plan test incorrectly expected already-supported reference overrides to be
unrecognized; `a16955e8c` separately corrects it to test capability rejection. No classifier
behavior was changed by that correction. The native probe also checks no output publication
for extra virtual slots, generic reference owners and unregistered bootstrap references.
All 15 runtime root-identity tests pass, including the new Raven artifact case.

```sh
# In Raven, using the existing explicit primitive declaration bootstrap:
dotnet build tools/NeoClrMetadataProbe/NeoClrMetadataProbe.csproj \
  -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:WarningLevel=0
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --source-object-root /path/to/IntrospectionCoreParams.dll /tmp/source-root.pe
# In neoCLR, the checked-in artifact makes runtime validation self-contained:
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk \
  cargo test --release --test object_root_identity
```

The [evidence manifest](source-object-emission-2026-10-05.json) records bootstrap and
artifact hashes. Compiler regeneration requires that explicit bootstrap; runtime fixture
execution uses an empty seed and does not require the bootstrap or compiler installation.

Limits: ordinary driver/ownership configuration and imported-root consumer selection are
not wired yet. Generic reference owners under the selected source root reject explicitly
until constructed local-base authoring is supported; extra virtual slots also reject.
The .NET adapter retains its source-root guard. The complete production System and native
VS Code/Tasks-await gates remain open. No independent .NET compiler fix needs backporting.

The author now requests a metadata disassembler before resuming broader integration.
This is the next bounded task; runtime suspension and green threads remain deferred.
