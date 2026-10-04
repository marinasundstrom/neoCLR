# External grapheme metadata — 2026-10-04

The metadata API now preserves owned Unicode grapheme Char across native assembly
references, through snapshot import or references authored solely from identity and
signature facts. No Raven importer objects are required by emission. The existing
native module/type binding tables retain exact dependency scope; no format revision
or runtime instruction changes were needed.

Compared with CLI Char, the native value preserves a complete grapheme, including
combining marks and ZWJ sequences. Ordinary CLI UTF-16 code-unit behavior is unchanged.
This follows the existing text-model decision rather than adding a numeric Char category.
The cost is an explicit native storage designation and native-only executable output.
See [public API contracts](../../../api-docs/experimental-metadata.md#owned-native-grapheme-declaration-development-2026-10-04).

## Reproduction

From the NeoCLR feature checkout, with a built `target/debug/neoclr`:

```sh
NEOCLR_GRAPHEME_ARTIFACT=/tmp/grapheme-external.neox dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_metadata_grapheme.py --runtime target/debug/neoclr --artifact /tmp/grapheme-external.neox --output /tmp/grapheme-external-fresh
```

Use a fresh output directory. The C# suite passed 143/143 groups. Each consumer library
forwards an external Char argument through the producer's managed-receiver Echo method
and returns the same grapheme. A separate neoIL root checks combining-mark and ZWJ emoji
text equality; both roots verify and exit 42 with empty stdout. The empty System seed
contains no competing Char. Tests also check canonical introspection identity, missing
catalog dependency, conflicting owner, foreign reference, invalid identity, late ordinary
method reinterpretation and CLI executable rejection.

[Captured commands, output and artifact hashes](grapheme-external-2026-10-04.json).
The runtime binary was built from the existing String-dispatch implementation; its hash
is recorded. No runtime or Raven source changed in this slice. Raven remains at
6c5b43ff6; NeoCLR parent revision is 4a73c54c. This proves the metadata reference boundary,
not the source-built Char class-library gate. Next wire Raven's Char provider separately
from numeric primitive mapping, then execute the unchanged source methods and interfaces.

The host C# APIs use the manual metadata reference linked above. Guest API snapshots are
unchanged; the previously recorded stale full-bridge snapshot remains a separate task.
No website build or broad runtime suite was needed for this metadata-only change.
