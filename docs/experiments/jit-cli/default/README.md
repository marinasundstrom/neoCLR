# Installed default JIT qualification

Development checkpoint, 2026-10-12. `neoclr run` selects JIT on macOS ARM64 and
Windows x64; `--execution interpreter` remains explicit and other hosts retain
interpretation. This is eager closed-world compilation through the existing
external compiler backend, not lazy or tiered JIT compilation. AOT remains the
preferred publishing target.

The preparer stages the runtime, compiler, rebuilt helper and coherent standard
libraries together. Discovery uses the runtime installation, not the guest folder
or current directory. Missing dependencies fail without interpreter fallback.
Native limits remain bounded and configurable during preparation. The default
helper uses a 32 MiB heap and a 4 MiB byte-array allocation limit. Those two limits
are independent; reference-array and operation limits remain unchanged.

Validation exercises omitted execution mode, installed library discovery,
HTTP/DNS/connect/fragmentation/timeouts/fault cleanup, library and runner contracts,
and explicit interpreter controls. On macOS, the default also binds runtime-managed
suspension. Windows uses compiler-generated async state machines; Windows runtime
suspension and ARM64 JIT remain open.

The first Windows attempt used an obsolete Raven compiler and failed to read the
current async-iteration bootstrap contract, before guest execution. The gate now
pins Raven `e349f4783cb7534979ac36b31e81c0ffad3d13ef`, the native compiler code
revision used locally. Raven main and the native compiler branch remain separate.

Reproduce using [bundle preparation and test commands](../../../jit-execution.md)
and the [Windows gate](../../jit-windows/README.md). Historical explicit-mode
reports remain valid for their recorded binaries; they are not evidence of the
installed default. Debugging remains interpreted. This checkpoint does not imply
full interpreter/native parity or a production-ready host ABI.

## Results

- macOS ARM64: all 29 compiled library suites pass their expected contracts when
  replayed from an installation relocated to a path containing spaces, from an
  unrelated working directory. Their 265 test outcomes include 262 passes, two
  intentional failure controls and one intentional skip. Interpreter selection and
  runtime-await StackTrace/GC controls also pass: [31 replay cases](mac-installed.json).
- Both hosts: eight HTTP/lifetime cases pass using installed `neoclr run` without
  execution, compiler, helper or library flags: [macOS](mac-http.json),
  [Windows](windows-http.json).
- Windows x64: [run 38182286947](https://github.com/marinasundstrom/neoCLR/actions/runs/38182286947)
  passes at `04812f87`, including scalar/CLI checks and 29 framework, guard and
  selector records: [gate](windows-gate.json), [framework](windows-framework.json).

The initial macOS [library prefix report](mac-library-prefix.json) deliberately
retains its failed status: the first 23 suites passed before the time-zone loader
failure. The [remaining six suites](mac-library-remaining.json) passed after the
helper fix. The final replay validates all 29 assemblies against the completed
helper, checks compilation hashes, and records the excluded earlier loader failure.
No failed report is relabeled as passing. Preparation and manifest snapshots are
retained for [macOS](mac-preparation.json) and [Windows](windows-preparation.json).

These gates establish the development default on the two supported hosts. They do
not establish Windows ARM64, Windows runtime suspension, lazy compilation, native
debugger stepping or unrestricted native metadata support. The subsequent generic
async/scoped-cleanup compiler work is a separate qualification.
