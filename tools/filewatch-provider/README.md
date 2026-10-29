# Shared physical watcher provider

Private Rust implementation used by the interpreter and native host adapter.
`notify` is pinned to 8.2.0. The library owns OS subscriptions and bounded queues;
it never retains guest pointers or invokes guest callbacks. It is linked statically,
not discovered through general NativeImport/dynamic-library lookup.

The C header describes the native host transport. The interpreter uses the same
Rust open/read/close functions. Invocation registries enforce handle ownership and
close outstanding subscriptions on teardown. Queue overflow and OS rescan signals
are explicit errors. Unknown rename pairs are not guessed. A process-wide limit of
64 watchers bounds retained native resources; inactive handles never become valid
again. The callback queue is bounded to 1,024 records of at most 64 KiB each.

Windows and macOS are the current release priorities. Native backend availability
also includes Linux, but is not execution evidence. Unsupported platforms return
PlatformNotSupported; no polling fallback is selected automatically.

```sh
cargo test -p neoclr-filewatch-provider
python3 scripts/validate-filewatch.py --bundle /path/to/current/bundle \
  --output target/filewatch-validation
```

The standard native project builder selects this archive only when required;
JIT helper preparation includes it. Windows validation needs the matching MSVC
compiler/runtime/bundle and an x64 Rust toolchain. Run actual tests on Windows;
cross-compilation alone does not qualify notifications, teardown or filesystem behavior.
The configuration sample has its own native file-input dependency and is not a
substitute for the standalone Windows watcher tests.

See [design and limitations](../../docs/file-system-watcher.md) and
[public API](../../api-docs/file-system-watcher.md). The managed API is in
System.IO; System.Storage provider contracts are unchanged.
