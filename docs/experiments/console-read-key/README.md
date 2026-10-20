# Public synchronous ReadKey — 2026-10-20

Console.OpenKeyInput([captureInterrupts]) and ConsoleKeyReader.ReadKey connect the
Raven decoder to invocation-owned terminal transport. The reader is disposable,
non-echoing and synchronous; results are values containing logical ConsoleKey,
Shift/Alt/Control and optional Unicode scalar text. Char remains a grapheme.

[Contract and limitations](../../../api-docs/console.md#terminal-keys-development) ·
[Design comparison](../../design/console-input-review-20261020.md#public-synchronous-key-reader-2026-10-20)

## Qualification

- Raven compiler `100f98a2f`, source library `target/readkey-libraries-v4`, matching
  bundle `target/readkey-bundle-v1/bundle`; macOS ARM64 POSIX terminals.
  The final API snapshot uses `target/readkey-libraries-v5`, rebuilt after adding
  XML descriptions to the six error cases. Executable behavior is unchanged;
  the focused execution evidence above is reused for that documentation-only edit.
- [Main.rvn](Main.rvn) passes **15 executions** across interpreter, JIT and AOT:
  redirected-input rejection; text PeekChar ownership; keys; direct unread-byte
  handoff to text; ordinary invocation fault restoration.
- Real PTY keys cover ASCII, captured Ctrl+C, Shift+Control+Left, slow fragmented
  two/four-byte UTF-8, Alt text, standalone Escape, unsupported CSI and malformed
  UTF-8. The following byte survives Close/reopen and key-to-text switching.
  Closing restores the original terminal settings. Repeated Close is harmless;
  subsequent ReadKey returns Closed. A second reader and text stealing are rejected.
- [results.json](results.json) records all commands and cases. `check.py` drives a
  pseudo-terminal, rather than replacing the runtime calls with test stubs.
- Focused release Rust tests: 23 terminal transport/invocation checks plus one native
  binding test covering all three signatures and six invalid mutations each.
  This includes byte/status mapping and stale handles; existing transport/decoder
  tests cover EOF and truncated sequences independently.
- Both public API references are refreshed. No compiler change is required; the
  paired bridge documents describe the private runtime boundary.

## Allocation result

[measure.py](measure.py) relinks the same AOT application with
[allocation-host.c](allocation-host.c), preserving the production terminal scope
and native collector. Both modes open/close the same reader and perform the same
setup/output. Only one mode performs 128 public ReadKey calls on ASCII input.

| Mode | Managed allocations | Collections | Reclaimed allocations |
| --- | ---: | ---: | ---: |
| Setup/baseline | 7 | 612 | 7 |
| 128 key reads | 7 | 12264 | 7 |

There are **zero additional managed allocations per key** in this diagnostic;
all allocations are reclaimed. This does not measure Rust/C host allocator calls,
interpreter allocations, latency or throughput. Eager collection checks increase
substantially. The byte-at-a-time host bridge also incurs extra host transitions
for multi-byte text and escape sequences. Bulk input with a shared cursor is a
future optimization requiring lossless ownership, not a claimed performance gain.

## Reproduce

Use a matching source compiler/library/runtime/backend bundle. On this machine,
select the Xcode compiler and MacOSX26.2 SDK (the default CLT SDK is incompatible).

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/console-read-key/Main.rvnproj \
  --bundle target/readkey-bundle-v1/bundle \
  --aot target/release/neoclr-codegen --output target/readkey-app-v2
python3 docs/experiments/console-read-key/check.py \
  target/readkey-bundle-v1/bundle target/readkey-app-v2 \
  --output docs/experiments/console-read-key/results.json
python3 docs/experiments/console-read-key/measure.py target/readkey-app-v2 \
  --output docs/experiments/console-read-key/allocation-results.json
cargo test --release -p neoclr -p neoclr-codegen terminal --lib
```

## Remaining gaps

- First ordinary byte input claims the text side for the invocation. Subsequent key
  acquisition returns Busy, even after EOF. This prevents silently discarding a
  buffered text decoder's state. Text-to-key handoff needs a shared cursor.
- Serialize console input operations; managed admission is not a synchronization
  primitive. Native worker attachment is not implemented; native key acquisition
  requires the entry thread. Windows terminal input is unsupported.
- Legacy Escape uses a 30 ms inter-byte timeout; Unicode scalar completion does
  not. Modifier ambiguity remains. No physical key-up, mouse or paste protocol.
- Close errors are retryable; Dispose attempts Close and invocation cleanup retries.
  Forceful process termination cannot guarantee restoration. Async readiness and a
  public cancellation contract are deferred; blocking input is not wrapped in Task.


## Small console tools

The author requested the familiar “press any key to exit” and explicit Ctrl+C
patterns. [AnyKey.rvn](AnyKey.rvn) exits after one key. [CtrlC.rvn](CtrlC.rvn) ignores
ordinary keys and exits on Ctrl+C. Both capture signal characters explicitly and
restore terminal settings through `use`. Their matching `.rvnproj` files compile
with the same bundle. Separate PTY reports cover all three engines:
[any-key-results.json](any-key-results.json) and [ctrl-c-results.json](ctrl-c-results.json).
The Ctrl+C test first sends an ordinary letter and verifies the process stays alive.

The application can choose to continue at the Ctrl+C branch; a general cancellable
shutdown-request/signal API is a separate future capability. No such API is added.
