# Development release qualification — 2026-10-22

This record tracks the development candidate after Preview 16. It is not a
published release or a declaration that every gate has passed. Historical
[Preview 16 qualification](../release-20261018/README.md) remains unchanged.
Targets are macOS ARM64 and Windows x64; Raven main and `codex/neoclr` remain
separate. The current package compiler is Raven `301e668a5`. The replacement
[versioned candidate](candidate.json), `0.1.17-neoclr.20261022.1`, is installed locally on macOS and both platform package gates pass; earlier development archive passes do not qualify these new bytes.

| Gate | Current evidence | Remaining work |
| --- | --- | --- |
| Source | [Canonical full-source/25-smoke and Windows host checks pass](source-qualified/README.md), as do all three minimum-Rust jobs and 19 local type-identity/terminal tests. | The Unix readiness repair passes both host jobs, all minimum-Rust jobs, 37 local socket tests and 512 refusal attempts. Canonical rerun 39853093086 passes full-source tests and 25 smoke programs. The earlier failing run remains failed. |
| Versioned local installation | [Installed candidate](local-installation/summary.json): JIT/interpreter samples and HTTP, 26 editor checks, inlays and two ten-cycle recovery runs pass. SDK and default extension are installed with rollback. | [Usage](../../local-preview-20261022.md). Rebuilt native modules/JIT host differ from development artifacts; remaining release gates must retain exact provenance. |
| macOS package/editor | [Complete package](complete-macos-package.json): extracted samples and HTTP exchange, 26 installed-editor checks, inlays, two ten-cycle configuration recovery runs. | [Readiness-repaired archive](socket-repair-package/summary.json) passes extracted JIT/interpreter samples and HTTP; only runtime/JIT-host bytes change. Final publication identity remains open. |
| Versioned Windows | [Exact candidate reports](versioned-windows/summary.json): extracted samples/HTTP, GC, 24 await, three JSON, 132 headers, 26 editor checks, inlays and both recovery runs pass. HTTPS run 39859994782 also passes. | Windows benchmark run 39859668223 and final asset/publication review remain open. |
| Versioned macOS runtime/API | [Exact candidate reports](versioned-macos/summary.json): 24 await executions, three JSON executions, 132 header cases, 15 HTTPS cases and extracted native-kit HTTP/C/C++ checks pass. | Preserve the initial TLS harness timeout separately. [Final macOS measurements](../../../benchmarks/release-20261022/macos-preview17/README.md) pass all five stages. Complete Windows measurements and final review before publication. |
| macOS native build kit | [Matching repaired-SDK kit](socket-repair-native-kit/summary.json): console/HTTP, C/C++ native interop, Cargo-free consumers and TLS dependency tamper rejection pass. | SDK manifest matches the qualified package. Recheck final publication identities; explicit HTTPS-server profile remains source-only. |
| macOS HTTPS | [Matching-package project](macos-https-project.json): 15 AOT security/recovery/cleartext cases and native TLS transport tests pass. | [Windows HTTPS](windows-https/summary.json) passes 27 JIT/AOT cases against the qualified package; no HTTPS throughput claim. |
| Windows | [Qualification run 39841298887](https://github.com/marinasundstrom/neoCLR/actions/runs/39841298887) reuses package run 39835531991 with repaired GC fixture and UTF-8 client transport. | Runtime/editor qualification passes; [raw reports and lineage](windows-qualified/summary.json) retain 24 await, three JSON, 132 header and 26 editor checks plus both recovery runs. [Windows HTTPS run 39843930421](windows-https/summary.json) also passes. |
| Performance | [Complete-package macOS measurements](../../../benchmarks/release-20261022/macos-complete/README.md) pass all five aggregate stages; the earlier development series is retained separately. [Windows measurements](../../../benchmarks/release-20261022/windows/README.md) also pass all stages against the qualified archive; their dirty-checkout limitation and source hash audit are retained. | Both platform measurements pass. Recheck final publication artifact identities against measured inputs. |
| Archive identity | [Both development archives](archive-identity.json): all 2,283 macOS and 2,278 Windows manifest files verify; each SDK and VSIX share 87 identical language-server files and version `0.1.0-dev.20261021`. | Eleven PDB/XML files per SDK are intentionally excluded from VSIX by Raven packaging rules. These are development identities, not final publication assets. |
| Publication | No new release published. Versioned candidate builds are started: [Windows run](https://github.com/marinasundstrom/neoCLR/actions/runs/39855375522), local macOS SDK. | Complete versioned-package gates, verify final artifacts, then publish; website publication is separate. |

The platform/operating-system discovery and default line-ending API request is
[planned work](../../platform-roadmap.md), not supplied by the test harness's
UTF-8 transport fix. Sorted collection proposals remain on hold. Do not broaden
release qualification into those API projects implicitly.
