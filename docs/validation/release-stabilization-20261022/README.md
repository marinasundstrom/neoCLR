# Development release qualification — 2026-10-22

This record tracks the development candidate after Preview 16. It is not a
published release or a declaration that every gate has passed. Historical
[Preview 16 qualification](../release-20261018/README.md) remains unchanged.
Targets are macOS ARM64 and Windows x64; Raven main and `codex/neoclr` remain
separate. The current package compiler is Raven `301e668a5`.

| Gate | Current evidence | Remaining work |
| --- | --- | --- |
| Source | [Archived suite](full-source-run.json): 2,233 passes, four ignored; its one failure is the subsequently repaired [Character sample](character-sample-repair.json). [25 smoke programs](source-smoke.json) pass. | Preserve the failed archive result and focused repair separately; final release inputs must be identified. |
| macOS package/editor | [Complete package](complete-macos-package.json): extracted samples and HTTP exchange, 26 installed-editor checks, inlays, two ten-cycle configuration recovery runs. | Final publication archive and provenance verification. |
| macOS native build kit | [Extracted kit](extracted-native-kit.json): console/HTTP, C/C++ native interop, Cargo-free consumers and TLS dependency tamper rejection pass. | Match final package inputs before publication; explicit HTTPS-server profile remains source-only. |
| macOS HTTPS | [Matching-package project](macos-https-project.json): 15 AOT security/recovery/cleartext cases and native TLS transport tests pass. | Windows HTTPS needs matching candidate evidence; no HTTPS throughput claim. |
| Windows | [Qualification run 39841298887](https://github.com/marinasundstrom/neoCLR/actions/runs/39841298887) reuses package run 39835531991 with repaired GC fixture and UTF-8 client transport. | Runtime/editor qualification passes; [raw reports and lineage](windows-qualified/summary.json) retain 24 await, three JSON, 132 header and 26 editor checks plus both recovery runs. Windows HTTPS run 39843930421 is in progress. |
| Performance | [Complete-package macOS measurements](../../../benchmarks/release-20261022/macos-complete/README.md) pass all five aggregate stages; the earlier development series is retained separately. [Windows run 39841353425](https://github.com/marinasundstrom/neoCLR/actions/runs/39841353425) has passed its package prerequisite and is running. | Windows measurements remain pending. Recheck final publication artifact identities against measured inputs. |
| Publication | No new release published. | Complete remaining gates, select and verify final artifacts, then publish; website publication is separate. |

The platform/operating-system discovery and default line-ending API request is
[planned work](../../platform-roadmap.md), not supplied by the test harness's
UTF-8 transport fix. Sorted collection proposals remain on hold. Do not broaden
release qualification into those API projects implicitly.
