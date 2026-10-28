# Persistent file transfer windows

Run the production Raven file adapters against a matching rebuilt interpreter and
five-library bundle:

```sh
python3 runtime/System.Runtime/test/FileMemoryTests/verify.py \
  --bundle <bundle> --output <fresh-output-directory>
```

The consumer writes a selected readonly window, checks the exact resulting file,
reads into a writable window, and checks untouched prefix/tail bytes, short reads,
EOF, end-position empty windows, invalid-range cursor preservation, closed-state
precedence and pre-cancellation. The current provider completes inline; the test
observes settled outcomes explicitly and does not claim asynchronous OS file I/O.
Temporary files are isolated and removed after execution.

`cargo test --lib host_call::tests` additionally verifies persistent destination
rooting while another heap participant collects, bounded write-back, root release,
readonly write snapshots excluding uninitialized surrounding storage, and exact
persistent byte capabilities. `cargo test --test file_streams` covers retained
legacy bootstrap bindings. Production Raven uses only the two-argument Memory
service forms. JIT/AOT file services and public stream-signature migration remain
separate work.
