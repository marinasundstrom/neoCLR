# Console key input and encoding review — 2026-10-20

Status: researched public API proposal; POSIX host transport and an internal Raven
decoder are implemented, while public key reads and native-backend integration remain open.
See [implementation and focused evidence](../console-io.md#development-terminal-host-transport--2026-10-20). The author
requests console key/combinations/Ctrl+C support and asks about encoding. The current
console supplies byte streams and strict UTF-8 line/text wrappers through explicit
synchronous interfaces. Host I/O is blocking. There is no public terminal key API.

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


## Author clarification: read-oriented public API

The author subsequently clarifies the objective: read a key, a character, a line,
or a requested number of characters. The primary public experience should therefore
be these read operations, with terminal ownership underneath them. A session/provider
API alone does not fulfill that objective.

Proposed integration contracts (not yet implemented):

- `ReadKey` returns logical key/modifier information and any reported text, without
  confusing keys with Char or inferring unreported modifiers.
- `ReadChar` returns one neoCLR grapheme, or EOF. A UTF-8 byte or a Unicode scalar is
  not sufficient. Streaming graphemes may need lookahead: an arriving combining mark
  can extend the previous scalar. Input-idle timeout must not silently redefine Char.
- `ReadLine` returns text with the line terminator consumed, or EOF; an empty line
  remains distinct from EOF.
- A counted character read returns text containing the requested number of graphemes,
  or the shorter remainder at EOF. A zero count returns empty without consuming input;
  negative counts are invalid. Independent byte limits bound potentially large graphemes.
- These methods share one input cursor, decoder and retained lookahead, preserving
  unread data when switching methods. Concurrent reads must be coordinated or rejected.
  Separate transient Console.In wrappers cannot own independent buffered lookahead.

The assistant identifies the host transport as a prerequisite, not the canonical
application surface. Async readiness/cancellation, framing/key decoding and shared
Raven text-reader state still need implementation and cross-engine validation.


### Peek clarification and text reader implementation

The author asks whether TextReader should have peek functionality. The assistant
recommends awaitable PeekChar: the same next grapheme as ReadChar, without advancing
the logical cursor. It can consume and retain underlying bytes to establish a boundary.
Repeated peeks should reuse retained text, and line/count reads must consume that text.
The current implementation adds these Raven operations and a shared Console.In reader;
[qualification](../experiments/console-text-reading/README.md) distinguishes them from
pending terminal keys, cancellation and nonblocking standard-input integration.

.NET's [TextReader.Peek](https://learn.microsoft.com/en-us/dotnet/api/system.io.textreader.peek?view=net-10.0)
provides the non-consuming cursor baseline; neoCLR uses asynchronous Option<Char>
instead of a synchronous integer sentinel. Counted reads use the fill-or-EOF rule
of [ReadBlockAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.textreader.readblockasync?view=net-10.0).
UTF-8 String output and grapheme counting preserve the platform's existing text model;
they cost owned result storage, boundary lookahead and possible waiting on live input.
Buffer-target and cancellation overloads remain explicit gaps, not permanent exclusions.

## Internal decoder checkpoint — 2026-10-20

The author narrows scope to a small performant foundation, without requiring .NET's
exact Console shape. An internal Raven decoder now implements a bounded legacy
UTF-8/xterm profile over ReadOnlySlice<byte>, returning a value and caller-buffer
text offsets instead of allocating strings. [Contract and evidence](../experiments/console-key-decoder/README.md)
cover ambiguity, malformed input, fragmentation and measured native allocations.
This remains groundwork; it does not expose Console.ReadKey or acquire a terminal.

Compared with .NET ConsoleKeyInfo's key/UTF-16 character/modifiers, byte windows
preserve UTF-8 without constructing a string or misrepresenting a scalar as neoCLR
Char. The cost is explicit buffer lifetime and separate grapheme assembly. The
[xterm control-sequence reference](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
is the source for the supported CSI/SS3 and modifier encodings. Only modifiers 1–8
are admitted; Meta and other protocol extensions remain unsupported. Escape-prefixed
Alt is a profile convention, not reliable physical-key identification. Unknown
sequences are not silently promoted to known keys. This profile is provisional
until the public session/provider contract selects supported terminal protocols.

## Invocation lifetime integration — 2026-10-20

The next internal step chooses invocation ownership for terminal resources, reusing
the root runtime shutdown path. A separate FFI provider could own the OS descriptor,
but would still need runtime registration to release it on root failure or stop
idle workers before joining them. The private Rust service supplies that lifecycle
contract now; it does not define a native ABI or add guest InternalCalls yet. OS
transport remains replaceable through the existing Console/TerminalInput host traits.
The tradeoff is a runtime service and future cross-engine provider work rather than
an independently usable foreign terminal library. No compiler mapping changes occur.

Compared with .NET's process Console key APIs, ownership is invocation-scoped, with
one active session and caller-buffer reads. This fits embedding and isolates session
lifetimes, but requires deliberate coordination with buffered Console.In before a
public key API can be added. Interim cancellation uses bounded 20 ms OS waits; this
limits normal cancellation delay at the cost of periodic idle wakeups. A wakeable
native wait is a future optimization, not shipped asynchronous console support.

## Native provider transport checkpoint — 2026-10-20

The native C host now supplies a private caller-buffer POSIX transport and removes
legacy FILE read-ahead. The same input cursor must remain reachable when switching
between standard reads and a terminal session; retaining a hidden stdio input queue
would violate that requirement. The provider's explicit session storage and retryable
close/final-dispose split make ownership visible without allocating a String or task
for input. Its fixed-state design is a transport seam, not a new public Console API.

This layer performs OS operations without managed heap access, so it belongs in the
replaceable host adapter rather than adding key parsing to runtime intrinsics. Native
invocation registration and guest handle binding still require lifecycle integration.
No compiler mappings, CLI metadata or guest signatures change in this checkpoint.
Compared with .NET's public key/text abstraction, these private bytes deliberately
leave Unicode and key interpretation in Raven. Costs include a private native ABI,
explicit caller cleanup, platform-specific termios storage, and currently unsupported
Windows terminal acquisition. The Rust/C transports are tested separately; concurrent
mixed-provider process-stdin ownership is not yet coordinated.

## Native ownership decision — 2026-10-20

Inspection of the existing collector resource contract rules out restoring terminal
modes in a GC cleanup callback: those callbacks explicitly prohibit I/O and failure.
Native entry hosts instead own a separate terminal scope and leave it after guest
success/fault return. This follows the existing native task/socket scope pattern
without extending the native context layout or adding managed roots. Its benefit is
predictable terminal restoration before diagnostics and heap disposal; its costs
are explicit host entry integration and an interim entry-thread-only lookup.

Worker attachment remains an implementation gap compared with the shared Rust
invocation service, not a deliberate new public Console rule. No guest API or
compiler bridge is enabled until that contract and input cursor ownership are
qualified. The provider lifetime bounds handle uniqueness; reloading a private
native provider is not a promise that old host-held integer handles remain usable.

## Public synchronous key reader (2026-10-20)

The implemented entry is `Console.OpenKeyInput([captureInterrupts])`, returning a
fallible disposable `ConsoleKeyReader`; `ReadKey()` blocks without echo and returns
`Result<Option<ConsoleKeyInfo>, ConsoleKeyError>`. This follows .NET's logical key
and modifier concepts, but a scope owns terminal mode and signal policy, errors are
typed, and EOF is explicit. Unlike .NET's UTF-16 KeyChar, `TextScalar: Option<int>`
represents one Unicode scalar without allocating text. It does not claim a complete
grapheme, physical key position, key-up event or keyboard layout mapping. Shift,
Alt and Control are Boolean value properties; no flags allocation or new text type
is needed. The record constructor accepts user-supplied values; decoder-produced
scalars are validated Unicode. Future Rune integration can improve that static type.

The decoder consumes a reusable 32-byte buffer. The private host bridge transfers
one byte as Int32 per call, avoiding managed buffer pinning and retained native
pointers. This trades additional host transitions/syscalls for simple lossless
ownership; bulk reads with one shared input queue are a future optimization, not a
throughput claim. Unsupported sequences and invalid UTF-8 return errors. Any
unconsumed bytes survive Close and reopen, or subsequent Console byte/text input.
Legacy Escape syntax uses a 30 ms **inter-byte** timeout, bounded by the 32-byte
sequence limit. An Alt-prefixed UTF-8 scalar is not subject to that idle timeout.

Text-to-key handoff remains a gap: after the first ordinary byte read, acquisition
returns Busy for the rest of that invocation. This conservatively protects all
text decoder/lookahead state, including independently created wrappers. Key-to-text
handoff is supported. Callers must serialize console input/session access; the
public managed admission flag is not a thread synchronization primitive. Native
scope lookup currently admits only the invocation entry thread; Windows terminal
acquisition remains unsupported. These are qualification gaps, not permanent
language or console contracts. Default opening preserves signal handling; callers
must explicitly choose capture to receive Ctrl+C as a key. Scope exit and ordinary
invocation faults restore modes; forced termination cannot guarantee cleanup.
