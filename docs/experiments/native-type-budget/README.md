# Bounded native type inventory — 2026-10-12

The extracted development SDK 0.1.13-neoclr.20261012.2 builds all five packaged
projects with Raven native 6b5e052ce. Collections, Task.Run and JSON execute through
the installed-default JIT contract. HTTP then exceeded the 512 specialized-type
budget with the current managed networking/TLS libraries.

The specialization/selection type budget is now 1024. The native collector admits
that same upper type index for typed reference arrays; the tag representation and
object layout do not change. Input inventories remain 1024 types/8192 functions;
selected functions remain 2048 and object/value layout remains 128 lanes. Pair the
rebuilt backend with its rebuilt native host. This is an experimental implementation
bound, not a CLI metadata or managed language limit. Unlike the CLR's general type
loader, this backend intentionally rejects closed worlds outside explicit budgets.

Validation on macOS ARM64:

- 50 backend unit tests, including the compiler/header limit agreement, pass.
- Selection accepts 1024 types and rejects a 1025-type input before output.
- Generic expansion accepts 1024 shapes and rejects 1026 from an input below 1024.
- The native GC test passes with undefined-behavior/bounds sanitizers. It traces
  children of reference arrays at the first and last valid type indices, rejects
  the next index and reclaims unreachable objects.
- Rebuilt, hash-checked archive extracted outside the checkout into a path with
  spaces passes collections, Tasks and JSON, but **does not pass HTTP**.

[The retained HTTP failure](acceptance-default-http.json) is the next gate:
`ReportServer`'s generated async state machine has 204 flattened lanes, beyond the
unchanged 1032-byte/128-lane object bound. Investigate compiler field lifetime/layout
and the existing runtime-await project contract before increasing layout budgets.
The type-budget change alone is not release qualification. No SDK or extension
installation was performed. Windows validation remains outstanding.

The [archive identity](archive.json) identifies the local candidate; temporary
extraction paths in evidence are provenance, not portable run instructions.
