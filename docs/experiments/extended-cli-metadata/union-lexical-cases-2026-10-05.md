# Union lexical cases and release reference regeneration — 2026-10-05

The release reference producer now succeeds on Raven's native integration line with
unchanged Option source. A union method could find its own case symbol but reject it as
unimported when the bootstrap's legacy companion occupied its wildcard import scope.
Earlier expression-body diagnostics exposed this latent lookup defect. Main also
returned an error type for a cold semantic query, even where later emission succeeded.

Raven now includes the cases of lexically enclosing unions during case lookup. Locals,
parameters and ordinary members retain precedence; unrelated unions still require their
normal import or qualification. This is compiler binding, not a metadata representation
change or a new bootstrap mapping. No Runtime Contract settings, source rewrites,
metadata encoding changes or runtime changes are required.

## Validation

- Raven main `edff20273`: 190 focused union, bootstrap replacement and imported empty-case
  tests pass. The fix was developed separately from native target work, integrated into
  local main, and the temporary fix branch deleted.
- Raven integration `2b683df67`: 196 focused tests pass, including expression-return
  diagnostics. Tests cover cold semantic queries, emission, execution and source/legacy
  declaration overlap, with and without a wildcard case import.
- The broad suite found a stale constructor bound-statement-count test, reproduced on
  unchanged main. Its replacement verifies empty/payload case selection and payload
  extraction through execution. No new constructor capability is claimed.
- The current CLI bridge built against the repaired integration compiler regenerates
  `NeoCLR.CoreProbe.dll` with SHA-256
  `300ec80dedfe40ca416fdea2bf195d26e4bd628e8876c701628448ad35eda7d4`, exactly matching the
  current checked-in website snapshot. No artifact refresh is necessary.
- [Dual-target driver evidence](union-lexical-cases-2026-10-05.json): plain and generic
  union factories return their own empty case without an import. Owned applications
  and separately compiled library/consumer pairs execute on CLR and NeoCLR, return 42,
  and preserve expected output, payload copies and native union attributes. Invalid
  case metadata still rejects without publishing output.

Reproduce the driver probe from Raven:

```sh
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --union-declaration-driver /path/to/rvnc.dll /path/to/neoclr \
  /path/to/NeoCLR.CoreProbe.dll /path/to/full-System.neox /tmp/fresh-union-check
```

Use the full union-probe seed recorded in the evidence (`/tmp/override-System.neox`),
not the source-owned JSON sample's trimmed seed. The latter deliberately lacks String
and correctly rejects synthesized display operations without its source libraries.
The recorded source/artifact hashes and commands identify the tested configuration.
The tested compiler binaries include this slice atop `3b6d6089d`; runtime sources are
unchanged from the existing sample runtime. The metadata repository base is `996b0da5`.

The native original-sample gate remains nine of ten: `application-inheritance` is next.
Native editor/LSP qualification, matching release artifacts and website publication remain
open. The website's current development status remains accurate; only the reference
producer's recorded compatibility limitation is now resolved.
