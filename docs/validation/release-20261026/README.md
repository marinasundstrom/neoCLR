# Preview 18 qualification — 2026-10-26

All required source, macOS ARM64 and Windows x64 package, editor, HTTPS and
measurement gates pass. Matching SDK/editor: `0.1.18-neoclr.20261025.1`, pinned
Raven `ed1d08def`. Synchronous GC remains the default. macOS preserves the optional
PostgreSQL provider; Windows contains the four base libraries.

[Final asset identities](release-assets.json) distinguish original binary inputs,
qualified source and later validation-only changes. The qualified source archive
matches canonical CI exactly. Failed preparation attempts remain in the
[preparation record](../release-preparation-20261025/README.md).

| Gate | macOS ARM64 | Windows x64 |
| --- | --- | --- |
| Extracted package and HTTP | JIT and interpreter pass | Default JIT and HTTP pass |
| Async / JSON / headers | 24 / 3 / 132 pass | 24 / 3 / 132 pass |
| Editor | 26 checks, inlays, two ten-cycle recovery runs | Same acceptance passes |
| HTTPS | 15 security/recovery/cleartext cases | 27 JIT/AOT cases |
| PostgreSQL | Live interpreter/JIT/AOT, contracts, ten protocol-peer cases | Not included |
| Native kit | Extracted console/HTTP and C/C++ interop pass | Separate source-build support |
| Release measurements | Five stages, 55 CPU/JSON processes, 1,152 HTTP responses | Same counts pass |

- [Final macOS package evidence](../release-preparation-20261025/postgresql/README.md)
- [Windows package evidence](../release-preparation-20261025/windows-final/)
- [Final macOS measurements](../../../benchmarks/release-20261026/macos-preview18-final/README.md)
- [Windows measurements](../../../benchmarks/release-20261026/windows-preview18/README.md)

Results are bounded qualification, not arbitrary application compatibility or peak
concurrent throughput. Windows measurements are lower than the prior hosted run;
the CPU model differs, so a runtime-only explanation is not established. Earlier
base-only macOS candidate measurements do not identify the final archive.

The release website builds and checks all 6,133 pages ([build log](website-build.log.gz)).
Website deployment is a separate manual workflow after artifact publication.

## Published — 2026-10-26

[Preview 18](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.18)
is published at `280b52e728d8671578b1945fa57a1a3ef1c67f7d`. All 12 uploaded
asset sizes and GitHub SHA-256 digests match the qualified local files; the
public checksum download also matches. [Website deployment](https://github.com/marinasundstrom/neoCLR/actions/runs/40307428309)
passes at the same commit. Live homepage, installation and HTTP/execution benchmark
pages contain the release links and final measurements.
[Publication evidence](publication.json) records the checks and asset URLs.
