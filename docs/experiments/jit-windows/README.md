# Windows x64 JIT qualification gate

2026-10-11 development work. The backend now selects the executing Windows x64
host and uses the existing native HTTP lowering and helper services. This gate is
implemented and passes on Windows x64 at `f993e94b` in
[run 38134756345](https://github.com/marinasundstrom/neoCLR/actions/runs/38134756345).
All five scalar JIT tests, CLI selection, 22 HTTP cases across interpreter/AOT/JIT,
and 23 framework/guard/selector cases pass. Downloaded HTTP DLL, assembly, object
and executable hashes match the retained report. See [HTTP evidence](http-validation.json),
[framework evidence](framework-validation.json) and [gate report](validation.json).
The subsequent shared overload fix is separately qualified by macOS JSON JIT/AOT
execution; it is not part of this Windows run.

Run from an x64 MSVC developer environment with Rust, Python 3.12 and the matching
.NET SDKs, or use [Windows JIT HTTP](../../../.github/workflows/windows-jit.yml):

```powershell
python docs/experiments/jit-windows/verify.py --output target/windows-jit-validation
```

The workflow pins and verifies the existing Preview 13 bootstrap bundle, rebuilds
the recorded Raven revision and current runtime libraries, builds a native HTTP
recipe, then exports its C and Rust-kernel helper symbols into a DLL. The JIT resolves every declared
import before executing guest code. The host shares rooted task/I/O teardown with
macOS; it restores the Windows CRT stream modes after each invocation. Generated
code uses Windows calling conventions and inline guard-page probes. Guest arguments
are converted to UTF-16 to match the existing Windows entry helper.

Required evidence: scalar JIT results and faults; rejection of foreign execution
targets; CLI mode validation; HTTP client/server, DNS, fragmentation, malformed headers,
timeouts and handler-fault cleanup; and JIT-only Index/Range plus runner-contract
suites, including Unicode/empty/option-like arguments, filtering and guest exit codes.
The HTTP gate compares relevant interpreter/AOT behavior for this new target. It does
not make a routine three-mode matrix mandatory for unrelated tests.

Reports and logs are uploaded even on failure. A workflow file or successful macOS
build is not a Windows pass. The current gate prepares an installed JIT bundle,
runs HTTP with no execution/tool/library flags and runs representative framework
suites with the omitted-mode JIT default. Windows ARM64 and Windows runtime
suspension remain separate work. The historical evidence above predates this
default-mode gate. See [CLI contract and readiness](../../jit-execution.md).

The completed omitted-mode gate passes at `04812f87` in
[run 38182286947](https://github.com/marinasundstrom/neoCLR/actions/runs/38182286947):
eight installed HTTP cases and 29 framework/guard/selector records, including
zoned-time services. [Retained default-mode evidence](../jit-cli/default/README.md)
is separate from the earlier explicit-mode results above.

## Managed SSL composition gate

The Windows runner now invokes the same managed SSL/HTTPS JSON consumer used on
macOS, in interpreter, default JIT and AOT, and uploads its validation/build reports.
The HTTP failure consumer separately checks hostname rejection/recovery, client
timeout/cancellation after ClientHello, and the server handshake deadline.
This extends the earlier private provider/host checks; adding the gate does not mean
Windows managed qualification has passed.

The development bundle pins Raven neoCLR compiler revision
`6fe7d630a25d0e27f3e9e89b1403987caf30d9d6`, including native async receiver/pattern
lowering required by SslStream. That revision must be available from the configured
Raven remote before CI can rebuild the bundle. It was local at this checkpoint;
publication and an actual Windows run remain pending. Raven main remains separate.

The managed HTTP interoperability gate (`tls-http-interop`) uses independent
Python/OpenSSL peers with TLS 1.2 and TLS 1.3, in both client/server directions.
It also exercises chunked responses crossing TLS record and UTF-8 boundaries.
Use a Python SSL build with TLS 1.3 enabled; credentials are generated per run and
verification remains enabled. Uploads include the validation and native build reports.

## Release candidate alignment — 2026-10-13

The development HTTP/JIT gates now select Raven native commit
`6b5e052ce3648f9247ac76aba275587a3a5a1bb4`, matching the installed macOS compiler
and its forward async-return declaration fix. The revision is available on the
Raven validation branch `codex/neoclr-release-20261013`; Raven main and the native
compiler line remain separate.

The native-toolchain workflow builds the matching Windows SDK/VSIX, then prepares
an actual default-JIT bundle before extracting and testing it. The new
`prepare-windows-jit-package.py` builds a Windows HTTP recipe, packages the current
backend/helper through `prepare-jit-bundle.py`, and archives only verified manifest
members. It records archive and manifest hashes, retains build logs and refreshes
notices. Two local archive regressions pass (stale output exclusion/hash mismatch
and out-of-bundle rejection); actual Windows execution remains a hosted gate.
