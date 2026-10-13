# SDK runtime-await integration — 2026-10-13

This qualifies rebuilt development SDKs, not published or already installed bundles.
The SDK supplies private await/generator providers and selects Raven's existing
native Runtime Contract. State-machine lowering remains an explicit project option.
See [scope and migration](../../../runtime-product-integration.md#sdk-default-runtime-await-integration--2026-10-13).

## Native carrier and stack ownership

- [macOS/Windows carrier run](https://github.com/marinasundstrom/neoCLR/actions/runs/38462503102)
  at `aaab0f9cfd4f31e400b46977075b0cb6afadb19b` passes both GC/root and
  socket/package gates. Reports retain revision, input hashes and command outcomes.
  Owned-body/idle logs retain survival, abandonment and cleanup checks, including
  a pre-existing Windows host fiber. Report/log line endings are normalized.
- [Windows host run](https://github.com/marinasundstrom/neoCLR/actions/runs/38464364208)
  at `fcc077d7f` passes guarded allocation, collector/root checks, small-stack
  rejection, generated-frame limits, cleanup/reuse and host-owned fiber acceptance.
  Its predecessor `38463958274` failed because the harness still expected fibers
  to be rejected; the underlying revised C test had passed.

Windows fibers reserve 1 MiB per activation and are deleted when complete or
abandoned. They remain thread-affine and are not cached. This preserves the existing
root ownership contract without making a new public embedding ABI. No collector
algorithm, pacing default or throughput claim is part of this integration.

## Managed SDK gate

The gate uses the pinned Raven compiler `6b5e052ce3648f9247ac76aba275587a3a5a1bb4`,
rebuilt libraries and matching runtime/backend/helper binaries. It builds ordinary
SDK projects without manually including a provider, inspects lowering, then runs
language behavior, cleanup, terminal faults, HTTP server/client, an explicit
state-machine fallback and generator lifetime/GC cases in interpreter, JIT and AOT.

```sh
python3 scripts/validate-runtime-await-sdk.py \
  --bundle PATH_TO_REBUILT_BUNDLE --runtime PATH_TO_RUNTIME \
  --aot PATH_TO_BACKEND --output target/sdk-await-check
```

`--cases client generator` limits a focused rerun. Windows uses the MSVC x64
command environment; the HTTP workflow's `runtime_await` input runs the same gate.
All 21 local macOS cases pass at the `5b5354c5e` implementation, using the
source hashes in `macos-sdk-20261013.json`. The gate checks those hashes again
after execution. `macos-sdk-build-provenance.json` extracts exact invocations,
input/toolchain identities and artifacts from passing native/JIT build reports;
large stdout/stderr fields are replaced by their SHA-256 hashes, and original
report hashes are retained. Windows application qualification is still running.

The initial local gate exposed two harness issues: interpreter execution of an
external assembly needed explicit module paths, and the Python HTTP peer needed
HTTP/1.1 rather than its HTTP/1.0 default. Neither failed run is recorded as a pass.
HTTP/1.0 response support remains a [separate compatibility gap](../../../http-capabilities.md#http-version-compatibility-gap--2026-10-13).
