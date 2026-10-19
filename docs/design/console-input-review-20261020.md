# Console key input and encoding review — 2026-10-20

Status: researched proposal, not implemented APIs or a release commitment. The author
requests console key/combinations/Ctrl+C support and asks about encoding. The current
console supplies byte streams and strict UTF-8 line/text wrappers; host I/O is blocking,
even where managed methods return inline-settled Tasks. There is no terminal key API.

## Modern Console direction

The author clarifies: “We want a modern Console APi.” Treat the work as a coherent
terminal interaction design, with Console as the convenient standard-stream entry
point and an optional owned interactive session. .NET supplies comparison points;
its global configuration and synchronous-only key-reading shape are not the goal.

Design principles: genuinely awaitable/cancellable input, structured events with
separate key and text identities, UTF-8 text boundaries, explicit capabilities,
scoped/restored modes, coordinated input ownership and value-oriented hot paths.
Redirected/piped/headless use must remain first-class. Later cursor/style/rendering
features should use the same session/capability model, while widgets, prompts and
shortcut command routing can remain higher-level libraries. Exact public signatures
and implementation sequencing remain proposals pending focused prototypes.

## Terms and proposed public responsibilities

- **Key**: logical terminal key identity (Enter, Escape, arrows, function keys, letters).
  This is not guaranteed to be a physical scan code or layout-independent key position.
- **Modifiers**: Control, Alt, Shift, and additional modifiers only where reported.
- **Key chord**: a key plus modifiers, such as Ctrl+Shift+P. A sequence such as Ctrl+K,
  Ctrl+C is a sequence of chords; shortcut bindings can be a later library layer.
- **Text input**: Unicode text produced by typing, composition or paste. It is distinct
  from the key event. Event boundaries need not be completed grapheme boundaries.
- **Interrupt**: host process-control notification, often triggered by Ctrl+C. It is
  not equivalent to every textual byte 0x03 arriving on stdin.

Keep System.Console as the familiar entry point. ConsoleKey, ConsoleModifiers and a
small ConsoleKeyInfo value are reasonable names, following .NET. An awaitable ReadKey
and a nonblocking TryReadKey convenience should share one input source; define EOF,
no-ready-event, unavailable terminal and cancellation distinctly before implementation.
A buffered availability property alone cannot guarantee a subsequent read won't wait.

For longer-lived interaction, a disposable Console.OpenTerminal-style session could
own input/echo/interrupt modes and restore the previous state on disposal and orderly
host shutdown. This name and exact signatures are provisional. A richer event union
can separate key, text, paste and resize events, with ReadKey as a convenience view.
Do not promise independent simultaneous consumers of ReadLine, raw byte input and
terminal events: one coordinated reader must own input and retain partial sequences.
No compiled Raven sample is claimed for these proposed APIs.

Normal mode preserves process-interrupt behavior. An explicit scoped input option
can request Ctrl+C as a key event. Separately expose interrupt observation for orderly
application cancellation/shutdown; reading a key must not be required for interrupts
to work. The decision to suppress normal termination needs a deterministic host
contract. Schedule guest callbacks outside OS signal-handler context. Cancellation
of a pending read is a third, distinct concept. Arbitrary key-up, held-key state,
physical scan codes and global shortcuts cannot be promised by a portable terminal API.

## Primary-source comparison

- [.NET ReadKey](https://learn.microsoft.com/en-us/dotnet/api/system.console.readkey?view=net-10.0)
  returns ConsoleKeyInfo with key, UTF-16 character and modifiers. Keep its familiar
  key/modifier concepts, but do not import one UTF-16 code unit as neoCLR's text model.
  .NET may read keyboard input independently of redirected stdin; a host-owned terminal
  handle should make that capability explicit instead of silently opening a device.
- [.NET CancelKeyPress](https://learn.microsoft.com/en-us/dotnet/api/system.console.cancelkeypress?view=net-10.0)
  and [TreatControlCAsInput](https://learn.microsoft.com/en-us/dotnet/api/system.console.treatcontrolcasinput?view=net-10.0)
  distinguish interrupt processing from input. A scoped mode avoids uncoordinated
  global toggles, at the cost of an explicit session lifetime and migration difference.
- [Terminal.Gui keyboard handling](https://tui-cs.github.io/Terminal.Gui/docs/keyboard.html)
  is useful for key/modifier modeling and a separate key-binding/command layer. Its
  richer UI routing should remain above the foundational console host contract.
- [Spectre.Console capabilities](https://spectreconsole.net/console/reference/capabilities-reference/)
  distinguish interactive/ANSI/Unicode rendering capabilities. Capability discovery
  is useful; a detected UTF-8 encoding does not establish font or glyph availability.
- [Kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)
  explains legacy ambiguity and optional richer key information. Legacy Ctrl+I/Tab
  and Ctrl+M/Enter may be indistinguishable. Escape can prefix another sequence.
  Key-release, repeat and disambiguated modifiers require supported/negotiated input;
  never invent information the transport did not report. Kitty is one protocol,
  not a universal baseline or a dependency decision.

## Encoding recommendation

Unicode is the character repertoire; UTF-8 is its byte encoding. neoCLR String stays
UTF-8 and Char stays one extended grapheme. A keystroke can produce no text, one scalar,
multiple scalars, or text that combines with earlier input. Do not force all events
into a Char or normalize incoming bytes. Text events and paste need their own payload.

Keep console text APIs UTF-8 by default, including redirected streams. A terminal host
adapter translates the actual native transport: UTF-8 bytes where configured/supported,
or native Unicode input/output APIs on Windows. Do not assume the Windows byte console
is already UTF-8: [Windows console code pages](https://learn.microsoft.com/en-us/windows/console/console-code-pages)
are configurable, and Unicode console APIs avoid that byte-code-page dependency.
Prefer adapters over mutating global process code pages as an incidental library action.
Legacy encodings should be explicit stream reader/writer configuration at byte boundaries;
those overloads, encoding discovery and malformed-input policy still require design.
Do not reinterpret arbitrary redirected bytes as keys or silently guess an encoding.

ANSI/VT escape sequences are a terminal control protocol, not a character set.
Font coverage and terminal cell widths are also separate from decoding and grapheme
counts. Report capabilities/limitations independently. Preserve the existing strict
UTF-8 stream error behavior until a reviewed change explicitly replaces it.

## Implementation alternatives and qualification

Use Raven for public types, ownership, key matching and parsing where suitable. Compare
extending the existing host console capability with a separate native terminal provider.
A provider can expose bounded bytes/events over a stable ABI; runtime integration is
needed for scheduler wakeups, cancellation, signal delivery and orderly teardown. Neither
placement is selected merely because it uses native OS calls. Embedders must be able to
supply synthetic input or no terminal; do not unconditionally capture process stdin.

Start with capabilities and host input ownership, then keys/modifiers, cancellable reads
and scoped mode restoration, followed by interrupts and richer text/paste events. Async
reads must actually suspend instead of wrapping a blocking byte read in a Task. Reuse
buffers/operation slots for hot input; measure rather than promise zero allocations.
Small key/modifier payloads should be values; optional Unicode scalars can be inline,
while multi-scalar text/paste may need owned bounded storage. Avoid one String or Promise
per key where the protocol needs neither; GC-safe payload lifetime remains mandatory.

Focused tests need synthetic host events plus PTY/native-console integration: fragmented
UTF-8 and escape sequences, legacy ambiguity, negotiated modifier/release support,
Ctrl+C interrupt/input modes, cancellation while idle, EOF versus no-ready-event,
redirected input, simultaneous-reader rejection, terminal restoration, bounded paste,
GC-safe event payloads and interpreter/JIT/AOT parity. Add allocation/latency evidence
for sustained input and pending-read cancellation before claiming efficiency.
