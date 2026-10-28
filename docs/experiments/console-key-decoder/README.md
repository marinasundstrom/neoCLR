# Internal Raven terminal key decoder — 2026-10-20

This is a tested internal kernel, **not** a public Console.ReadKey implementation.
The production source is `runtime/System.Runtime/src/System.Runtime/System/Console/TerminalKeyDecoder.rvn`.
The consumer compiles that same source privately, avoiding a friend-assembly grant.
The complete source class library also builds with matching Raven `100f98a2f`.

## Contract

Decode borrows ReadOnlySlice<byte> and returns a six-integer value: state, consumed
byte count, logical key, modifiers, text offset, text byte count. Text windows refer
to the caller's input; copy or consume them before reusing that buffer. There is no
String, Char, array, Task or Promise construction in the kernel. Numeric keys and
states are internal implementation details, not a public ABI or API proposal.

States distinguish incomplete, decoded, unsupported and invalid UTF-8. Incomplete
input consumes nothing. One UTF-8 scalar is text, not necessarily a grapheme or a
physical key. ASCII letters use uppercase logical identities, without inventing
Shift from capital letters. Tab, Enter and Backspace keep conventional identities;
the decoder cannot distinguish Ctrl+I/M/H. Captured Ctrl+C yields logical C+Control.
Signal capture itself belongs to the host session, not this decoder.

The deliberately limited xterm-style profile supports CSI arrows/Home/End,
Insert/Delete/PageUp/PageDown, SS3 navigation/F1–F4, tilde F5–F12, and encoded
Shift/Alt/Control modifiers 1–8. Escape-prefixed text uses the legacy Alt convention;
that is ambiguous with rapidly typed Escape followed by text. A lone Escape waits
for explicit resolution. `finish` resolves a final prefix at EOF or an escape
sequence deadline; callers must not apply an idle timeout to incomplete UTF-8.
Two Escapes consume one at a time. Unsupported completed sequences are returned as
unsupported rather than invented key events. Parsing examines at most 32 bytes per
call and reports the consumed prefix; callers must choose how to handle an oversized
unsupported sequence and its remainder. There is no unbounded parameter accumulation.

No mouse, paste framing, key release/repeat, protocol negotiation, physical key
location or terminal encoding discovery is implemented. This decoder must not be
advertised as a general terminal event parser. Public key/modifier types, input
ownership across buffered text/key switches, invocation cleanup and all host/native
provider wiring remain required before exposing Console key reads.

## Evidence and reproduction

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/console-key-decoder/Main.rvnproj \
  --bundle target/line-text-bundle-v2/bundle \
  --aot target/release/neoclr-codegen --output target/console-key-decoder-v4
python3 docs/experiments/console-key-decoder/check.py \
  target/line-text-bundle-v2/bundle target/console-key-decoder-v4 \
  --output docs/experiments/console-key-decoder/results.json
```

The checked-in execution report covers interpreter/JIT/AOT: key/modifier values,
all split points in a modified navigation sequence and a four-byte UTF-8 scalar,
combining text, Escape resolution, text offsets, truncated/malformed UTF-8,
surrogates/out-of-range scalars, unsupported modifiers and bounded sequences.

Build `Allocation.rvnproj` using the same bundle and backend, then run:

```sh
python3 docs/experiments/console-key-decoder/measure.py \
  target/console-key-allocation-v1 \
  --output docs/experiments/console-key-decoder/allocations.json
```

For 128 iterations of text, modified-key and incomplete decoding (384 calls), the
native managed allocation count remains **2**, identical to baseline, with both
allocations reclaimed. Eager collection checks rise from 518 to 10374: no throughput
or CPU improvement is claimed. These counters do not measure host/interpreter
allocations. The production wait transport has separate PTY allocation evidence.
