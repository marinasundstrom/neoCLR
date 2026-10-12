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
The [explicit interpreter control](acceptance-interpreter.json) passes the same
extracted bundle, including [independent HTTP requests and the client/server pair](acceptance-interpreter-http.json).
This isolates the remaining object-size rejection to native/JIT lowering.
The type-budget change alone is not release qualification. No SDK or extension
installation was performed. Windows validation remains outstanding.

The [archive identity](archive.json) identifies the local candidate; temporary
extraction paths in evidence are provenance, not portable run instructions.


## Separate heap-object and inline-value bounds — 2026-10-12

The 204-lane HTTP async frame is valid managed metadata. The native profile
incorrectly coupled its heap payload budget to the 128-lane inline-value budget.
A class still crosses the native call ABI as one pointer regardless of its payload;
inline records expand into ABI lanes. Optimizing Raven's hoisted fields can reduce
allocations later, but should not be required to admit this valid heap object.

`OBJECT_LANES`/`NEOCLR_NATIVE_OBJECT_LANES` now permit 256 payload lanes (2056 bytes
including the header). Inline records and record-array elements stay at 128 lanes.
The allocator checks the same object maximum before reserving storage. Heap capacity,
array limits, root descriptors and type limits are unchanged. Rebuild both backend
and native host: an older helper intentionally rejects a newly admitted wide object.
There is no metadata, pointer ABI or managed API change.

Compared with .NET, this remains a bounded experimental native profile rather than
unrestricted class layouts. Separating heap and inline budgets avoids expanding
value-call signatures just to hold async state. The cost is admitting larger object
allocations and GC scans; the existing total heap budget still applies. This does
not claim an async performance improvement or fix compiler field liveness.

Focused macOS ARM64 checks pass:

- 12 native reference tests, including admission at 204/256 lanes and rejection at
  257 before object emission; maximum-size execution, zero initialization, heap
  canaries, insufficient capacity and allocator rejection of an oversized object.
- Two backend/host limit-agreement tests.
- The unchanged cyclic/unknown/oversized inline-value rejection test.
- Native GC under undefined-behavior/bounds sanitizers, including a child referenced
  only from the final lane of a maximum-size object and subsequent full reclamation.

Reproduce with `cargo test --manifest-path tools/aot-poc/Cargo.toml --test console
reference_`, the `limits::` binary tests and the `values` test
`cyclic_unknown_and_oversized_inline_layouts_never_emit`. Build `native-gc-test.c`,
`native-gc.c` and `root-probe.c` with Clang `-std=c11 -Wall -Wextra -Werror
-fsanitize=undefined,bounds` and run the resulting program. Use the matching platform
SDK/toolchain. Full extracted HTTP acceptance is recorded separately below.
