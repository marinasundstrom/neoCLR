# Console terminal host foundation — 2026-10-20

This is the first host transport slice of the
[modern Console design](../../design/console-input-review-20261020.md).
The author-facing objective is key, character, line and counted-character reads
sharing buffered input. Those Raven APIs are not implemented by this slice.

Run from the repository root on a POSIX host:

```sh
cargo test --lib console::
cargo build --example terminal_input
python3 docs/experiments/console-terminal-host/check.py
```

The unit tests exercise input ownership, default signal mode, explicit capture,
nonblocking idle reads, exact raw byte preservation, empty/closed reads, close/drop
restoration, redirected input and terminal hangup. The consumer check launches the
actual host example with a PTY, sends UTF-8 and Ctrl+C, checks exact bytes and restored
terminal attributes, then verifies that redirected stdin is rejected. BSD automatically
sets PENDIN when canonical mode resumes; assertions exclude that kernel queue-state
flag while comparing all user modes and control characters.

No allocation, latency or throughput result is claimed. The diagnostic polls with
an explicit delay; it is not a prototype of the final scheduler-based async API.
Windows Unicode input, invocation cleanup, interrupt-to-cancellation handling,
Raven public APIs and interpreter/JIT/AOT guest parity remain pending.


Qualification host: macOS ARM64. The executable consumer and six transport tests
pass; Linux and Windows are not claimed as tested. The API snapshot check and
4572-page website build pass. Strict root Clippy reports two existing
`nonminimal_bool` diagnostics at `src/inheritance.rs:405`; no terminal diagnostics
were reported. This is not complete release qualification.
