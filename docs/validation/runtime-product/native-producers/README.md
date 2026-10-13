# Native producer allocation qualification

The POSIX file-input adapter selects rooted final decoding for generated GC calls.
The descriptor is already closed; malloc-owned bytes remain alive until decoding
finishes. Reads are never replayed. Original native/non-GC calls stay noncollecting.
Windows file-input binding remains explicitly excluded by the compiler.

The shared native-byte decode fixture runs on both platforms and checks native
buffer stability, invalid UTF-8 without collection, live exhaustion, missing/foreign
contexts, unchanged failure outputs and complete reclamation. The macOS-only file
part counts read/close calls under raw exhaustion, rooted exhaustion and successful
retry, deliberately omitting the consumed managed path from caller roots.

All 29 local sanitized GC/root executions and relocated-package checks pass. Two
compiled file-input tests pass, comparing interpreter UTF-8, size/error precedence
and descriptor cleanup. [Hosted run 38451103096](https://github.com/marinasundstrom/neoCLR/actions/runs/38451103096) passes at `d09db30736c9ce140129b6cf8e264eb5a1069e77`: all 29 GC/root executions, sockets and relocated-package checks on macOS ARM64 and Windows x64, plus macOS profiler attachment. Windows runs the shared native-buffer test; only macOS runs the POSIX file portion. No Windows file support or fresh managed Windows/JIT/HTTP matrix is claimed.

The accompanying socket/TLS/crypto/PostgreSQL inventory is source-level review of
unchanged paths, not fresh TLS/SQL workload qualification. Those adapters contain no
managed result allocation requiring migration. General pacing remains disabled;
remaining native allocation families are explicitly recorded in the helper audit.
