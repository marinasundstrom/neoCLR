# Persistent file transfer windows

Run the production Raven file adapters against a matching rebuilt interpreter and
native library bundle:

```sh
python3 runtime/System.Runtime/test/FileMemoryTests/verify.py \
  --bundle <bundle> --output <fresh-output-directory>
```

Two discovered `NeoClr.Testing` cases replace the numeric-exit consumer. They
write a selected readonly window, read into a writable window, and check untouched prefix/tail bytes, short reads,
EOF, end-position empty windows, invalid-range cursor preservation, closed-state
precedence and pre-cancellation. The current provider completes inline; the test
observes settled outcomes explicitly and does not claim asynchronous OS file I/O.
The host verifier independently checks the exact bytes `28 02` and that the
closed-state fixture remains empty. Temporary files are isolated and removed after
execution. The custom entry point supplies an isolated path, then delegates to
standard discovery/registration and `TestRunner`; optional runner selectors follow
the path argument. Inline-completion failures now return `TestFailure` rather than
panicking. The two tests use separate files and do not depend on discovery order.

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#file-memory-windows)
also records a deliberately preexisting file: the runner reports one named failure,
returns nonzero and preserves the existing file's bytes.

`cargo test --lib host_call::tests` additionally verifies persistent destination
rooting while another heap participant collects, bounded write-back, root release,
readonly write snapshots excluding uninitialized surrounding storage, and exact
persistent byte capabilities. `cargo test --test file_streams` covers retained
legacy bootstrap bindings. Production Raven uses only the two-argument Memory
service forms. This fixture remains interpreter-only; no JIT/AOT file-service coverage is
claimed by this migration. Array-range public signatures and the underlying Memory
service contracts are unchanged.
