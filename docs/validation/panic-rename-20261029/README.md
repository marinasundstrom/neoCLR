# System.Panic rename — 2026-10-29

Author-directed naming change only. Runtime fault propagation, cleanup guarantees
and thread/task boundaries are unchanged. Broader panic behavior is deferred.
Raven compiler changes are committed on codex/neoclr as `b4c5790e1`.

## Validation

- 20 focused Raven terminal-flow/RuntimeFailureContract checks pass, including
  imported and qualified Panic, old Fail rejection and ordinary .NET controls.
- 17 runtime/frontend/query tests pass: `cargo test --test system_fault --test
  query_terminals --test neo`. The updated panic test separately passes all three
  system_fault cases, checking no old public symbol, message/code preservation,
  terminal execution and survival of the embedding host.
- Seven focused build-kit and native API snapshot Python checks pass.
- Separate native consumers compile both `Panic` and `System.Panic` in an
  int-returning function with no normal return. The removed System.Fail rejects.
  Interpreter execution reports UserFault with the computed Unicode diagnostic,
  stack frames, empty stdout and exit status 1. See consumer.json.
- All 14 native class libraries, including optional PostgreSQL, rebuild from the
  renamed source. A separate isolated tree also qualifies the rename independently
  of concurrent JSON/configuration work. After that work committed separately as
  `311f70549`, the final reference retains both changes. Native reference rendering and CLI
  reference/source-fingerprint checks pass; no website publication is performed.

The test logs and consumer sources are retained here. The local build artifacts
are in target/panic-rename-20261029. Reproduce the library build with
scripts/build-native-class-library.py and the native-enabled Raven compiler;
compile each consumer separately against System.Runtime.dll and ownership.json.
Run with the matching empty System.runtime.neox seed, System.Runtime dependency,
`--execution interpreter` and `--object-root` set to that dependency.

The legacy bootstrap neoIL fixtures are mechanically renamed and covered by the
focused runtime tests. Their historical provenance is not relabeled as a fresh
full legacy CLI library regeneration. Production native sources/bundles and the
regenerated API snapshots are the current contract.
