# Terminal transport wait and allocation check

Host-only POSIX evidence, 2026-10-20; not a Raven key API or throughput benchmark.

```sh
cargo test --release -p neoclr --lib console::
cargo build --release --example terminal_input
python3 docs/experiments/terminal-input/check.py
```

Nine transport tests pass on macOS arm64. The executable check sends five separate
fragments: the two bytes of é, Escape, `[A`, and captured Ctrl+C. All five reads
report zero Rust allocator calls; the session restores terminal modes on close.
The single-threaded example counts alloc/realloc calls only around `read`, excluding
session acquisition and printing. This does not measure kernel allocation, a future
key decoder, errors, guest runtime allocation, or native JIT/AOT providers.

Input remains raw bytes. The transport neither invents grapheme/key boundaries nor
allocates Strings, Tasks, or Promises. Scoped acquisition may allocate once.

## Invocation ownership checkpoint

`cargo test --release -p neoclr --lib invocation` now includes synthetic terminal
lifecycle tests and a real PTY root-scope restoration test, alongside the existing
invocation/worker regressions (31 checks pass on macOS arm64). It checks cancellation and shutdown during idle
reads, retained invocation references, host read panic/unwind, stale handles,
overlap, empty buffers, timeout preservation, retryable restoration and cancellation
racing acquisition. If acquisition loses that race and restoration fails, the
invocation retains the otherwise-unreturned resource for shutdown to retry.

The existing `console-key-decoder` executable was also replayed successfully in the
interpreter built with the new root teardown path. That consumer exercises normal
VM teardown; only the private lifecycle tests acquire a terminal through the new
service. No guest acquisition binding or standalone native provider is implemented.

## Native POSIX transport checkpoint

```sh
python3 docs/experiments/terminal-input/check_native.py --output target/native-terminal-v3
```

On macOS, use a matching Xcode compiler and SDK in PATH/SDKROOT. The checked-in
`native-results.json` records six passing executable cases on macOS arm64, compiled
with C11 and warnings as errors: redirected input rejection, all 256 legacy byte
values/EOF, legacy-to-terminal switching without read-ahead, captured control and
fragmented UTF-8/escape bytes, hangup/final disposal, and replacement of stdin while
the original terminal is restored through its owned descriptor. The key-input case
also checks empty/oversized buffers, invalid timeouts, nonblocking and timed idle
reads, same/second-session rejection, legacy-reader exclusion, mode flags,
close idempotency, closed reads and reacquisition.

The matching native console output/range regression also passes:

```sh
cargo test --release -p neoclr-codegen --test console \
  linked_stream_adapter_preserves_ranges_limits_and_channels
```

This qualifies the C transport directly, not guest key reads or native invocation
registration. The native legacy byte reader now uses unbuffered POSIX read; there
is no hidden FILE input buffer to strand bytes on a mode switch. The provider has
no explicit heap allocation in its read path, but this checkpoint does not measure
libc/kernel allocations or claim end-to-end throughput. Windows terminal input
remains unsupported; its existing legacy byte reader is retained.

The existing Raven console-text consumer was freshly linked against this C provider
and passes as a standalone AOT executable with its Unicode/line input fixture.

## Native invocation scope checkpoint

```sh
python3 docs/experiments/terminal-input/check_native.py \
  --output target/native-terminal-scope-v5 \
  --jit-host target/terminal-scope-bundle/bundle/bin/neoclr-jit-host.dylib
```

The updated report has **11 passing cases** on macOS arm64. Scope checks cover
context identity, stale handles, duplicate registration, nested LIFO teardown,
abandoned open sessions, reacquisition and rejection on unattached worker threads.
The real console entry host is linked against an instrumented C guest entry and
stub collector/renderer to assert restoration before rendering/collection. Both
ordinary and fault returns pass. The real rebuilt JIT helper runs that C entry
with its actual collector/services; both success and fault cases run twice in one
process and restore the terminal each time. These are host-boundary tests, not
claims that a Raven guest can yet acquire a terminal.

The existing Raven console-text consumer passes interpreter/JIT/AOT with the
rebuilt helper (`text-results.json`). Its AOT executable also passes under the HTTP
host. The retained-session HTTP host passes a C11 warnings-as-errors syntax check
with its native-GC/async configuration; that is not an execution test. Windows
host wiring is updated but not qualified here; terminal acquisition remains
unsupported there. No guest terminal bindings or worker attachment are added.

The scope is caller-owned stack storage and performs no heap allocation itself.
This checkpoint measures correctness, not allocation counts or throughput.
