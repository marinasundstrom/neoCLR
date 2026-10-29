# Historical Raven CLI bridge fixtures

This directory retains generated neoIL fragments, the old slice project and
historical validation receipts used by bootstrap/profile tests. The current
class library is authored in the runtime component projects, and active compiler
service templates live in `../../compiler-services/`.

Moved from `runtime/raven` on 2026-10-29. Generated comments and JSON receipts keep
their original source paths and hashes as provenance, not current build evidence.
See [runtime source ownership](../../README.md).

Development naming migration: the retained textual bootstrap fragments use
Result.Failure and Optional, matching current source APIs. Their checked-in JSON
generation receipts remain historical evidence from the original generation;
those receipts do not certify a fresh regeneration of the renamed text. Current
qualification uses the rebuilt native class-library bundle and API snapshots.
