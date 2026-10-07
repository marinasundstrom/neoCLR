# Native erased Value import — 2026-10-07

Raven **801f131ba**, neoCLR **dd184f95**. [Exact input/compiler hashes and outcomes](value-import-2026-10-07.json).

NativeNamedTypeSymbol formerly converted every native primitive designation to a
Raven SpecialType by enum name. NeoCLR Value has no corresponding CLR special type,
so merely importing the full source-owned library threw System_Value Enum.Parse.
It now remains a nominal value type with SpecialType.None and its original assembly
identity. Return/parameter symbols refer to that same declaration. Existing target
ownership contracts retain responsibility for its native meaning; no new special enum,
.NET representation or runtime behavior is introduced.

The focused C# regression can be run from Raven:

```sh
dotnet run --project tools/NeoClrMetadataProbe \
  -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:UseRavenCoreReference=false /property:WarningLevel=0 \
  -- --native-value-symbols /path/to/Core.dll
```

It authors Value and Int32 providers and an Echo signature, then verifies semantic
category, exact owner/signature identity and unchanged numeric classification. The
baseline failed with the original Enum.Parse; the fixed check passes. Both existing
ErasedValueOwnershipTests pass on .NET. Compiler build and whitespace checks pass.
No shared compiler fix requires backport: the changed classification is in the native
adapter, while the .NET reflection loader is untouched.

The ordinary compiler command now binds unchanged application-order-collections against
the emitted library, without library sources, then rejects emission with NEOMETA001
naming System.Collections.ArrayList`1. The diagnostic now includes the unsupported type
identity. No output artifact is published. The emitted ArrayList<T> metadata names the
source System.Object as its base; the consumer's selected semantic root still comes
from primitive bootstrap. Reconcile that root ownership/capability relationship next;
this observation is not authorization to bypass arbitrary inheritance validation.

Earlier full-artifact runtime verification/control execution remains the admission
baseline. No broad application execution or full bootstrap completion is claimed here.
