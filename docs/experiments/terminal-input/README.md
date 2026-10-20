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
