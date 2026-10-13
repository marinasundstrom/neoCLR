# Raven native friend consumer

These native manifests were emitted by Raven's `NeoClrMetadataProbe --native-friends`
at `a3be45460` on `codex/source-object-metadata-resolution`, using neoCLR metadata
revision `a8b15607`. JSON whitespace is normalized for review; content is unchanged.
The probe uses a freshly generated, unstripped primitive Core reference.

The provider grants FriendConsumer access to an internal Hidden class, constructor,
Value field and Answer method. The consumer constructs Hidden, stores 7, reads it,
and adds Answer() (35), returning 42. The assembly attribute constructor deliberately
faults; inspection and compilation must not execute or root it. The marker is authored
in the provider for this isolated contract test; the runtime library does not yet
supply InternalsVisibleToAttribute.

Regenerate from the Raven checkout with matching neoCLR metadata binaries:

```sh
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --native-friends /path/to/unstripped-Core.dll /path/to/output
```

Copy the two resulting JSON manifests here and normalize whitespace. The AOT test
encodes them as NEOX, compiles with the reference-arena profile and, on macOS ARM64,
links and executes a C host checking 42. Denial cases mutate grants or visibility and
require an access diagnostic and no object file. Run from the neoCLR root:

```sh
SDKROOT=$(xcrun --sdk macosx --show-sdk-path) cargo test \
  --manifest-path runtime/codegen/Cargo.toml --test friend_assemblies
```

The SDK selection avoids a host CommandLineTools linker search-path mismatch. The
executable qualification is macOS ARM64 only; this does not establish Windows parity,
signed friend identities, assembly reflection retention, or a new public API.
