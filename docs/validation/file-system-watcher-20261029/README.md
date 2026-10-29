# Standalone physical file watching — 2026-10-29

Windows and macOS are the author-directed release priorities. This receipt records
local macOS arm64 execution; Windows and Linux remain unqualified. Backend
availability is not platform validation.

The native and interpreter consumer exercises defaults, typed setup failures,
recursive filename filtering, removed subscriptions, real creation, atomic file
replacement, stop/restart and disposal. A native host harness checks cross-invocation
handle rejection and teardown without explicit Close. The interpreter ownership
unit test checks the equivalent resource-registry boundary.

Six provider tests cover real OS delivery, handle release, Unicode/wildcard matching,
paired renames, ambiguous rename loss, overflow/recovery, and preserving known
changes alongside rescan errors. One codegen binding test checks the exact private
transport signature. No broad suite or performance claim is included.

The configuration sample printed `before`, rejected malformed JSON while retaining
previous values, then printed `after` following atomic replacement and exited 0.
It uses an earlier provider binary from this same slice; the final queue fix was
subsequently exercised by the lifecycle consumer.

API validation regenerated the aggregate reference, rendered the current native
libraries with RavenDoc, checked real declaration links (including watcher members),
and refreshed the pinned native snapshot. Compiler/renderer binary hashes and
artifact hashes are retained in the JSON receipt; source revision labels denote
base revisions plus working changes, not pristine compiler-source provenance.

Reproduce with the matching compiler/runtime bundle:

```sh
cargo test -p neoclr-filewatch-provider
cargo test -p neoclr watcher_handles_are_invocation_owned_and_close_on_drop
python3 scripts/validate-filewatch.py --bundle /path/to/bundle --output target/watch-check
python3 scripts/build-api-docs.py --check
```

On macOS the local run selected Xcode 26.2's SDK/toolchain through DEVELOPER_DIR
and SDKROOT. Windows needs its MSVC/Rust toolchain and matching bundle; the test
driver selects the Windows native host automatically. Configuration file-input
qualification is separate from standalone watcher qualification.
