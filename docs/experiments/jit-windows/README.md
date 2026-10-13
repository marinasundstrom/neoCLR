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

### Mixed callback dispatcher repair

Candidate `a9adc97cb` reached managed SSL stream compilation in
[Windows run 38317601422](https://github.com/marinasundstrom/neoCLR/actions/runs/38317601422)
after passing compiler/library preparation, crypto and direct TLS checks. It then
failed at the combined host callback dispatcher: inhabited-unit and no-result
callbacks each fit the existing per-shape bound, but their union exceeded 32.

The generated dispatcher uses branches and has no fixed-size host table. Remove
only the redundant combined bound; retain independent shape admission, managed
function selection bounds, handle validation and GC ownership. This is a backend
admission repair, with no managed API or callback ABI change. Like .NET delegates,
valid callbacks do not gain a different invocation contract when unrelated
callbacks are linked; remaining native POC bounds are implementation gaps.

Focused regression coverage emits Windows x64 COFF for mixed shapes and executes
both shapes on macOS ARM64 with more than 32 combined targets. The sanitized host
session checks late-target invocation, retained roots, collection, fault propagation
and invalid/stale handles. The full Windows SSL/HTTP consumer still requires a
fresh hosted run; object generation alone does not qualify Windows execution.

### Windows AOT host heap admission

[Run 38319332994](https://github.com/marinasundstrom/neoCLR/actions/runs/38319332994)
gets past callback compilation and passes the SSL JSON consumer in interpreter and
default JIT. The standalone AOT executable exits with code 2 before guest execution:
the shared build accepted an 8 MiB heap, but the private Windows allocator still
rejected anything above its original 1 MiB experiment bound. [Retained report](../release-20261013/windows-ssl-before-heap-fix.json).

Align Windows capacity admission with the shared native HTTP build's maximum
aligned Int32 byte budget (2,147,483,640 bytes), preserving its 1 MiB default and
caller-selected capacities. Like a CLR host's resource budget, this ceiling does
not promise successful OS allocation. Guard pages, zero initialization, failure
atomicity and ownership remain unchanged. Windows x64 arithmetic stays bounded;
this does not qualify Windows ARM64 or alter public managed APIs.

The Windows memory test now covers 18 lifecycles, including 8 MiB SSL and 32 MiB
framework heaps, both guard pages, zeroing, duplicate create rejection, release and
invalid capacities. It does not allocate the multi-gigabyte admission ceiling.
The JIT/HTTP gate runs that check before its consumers. Host initialization failures
now print their status and requested heap size rather than silently returning 2.
Python syntax checks pass locally; real guarded-allocation and AOT execution remain
Windows-host validation requirements.

### Current qualification — 2026-10-13

[Run 38321019998](https://github.com/marinasundstrom/neoCLR/actions/runs/38321019998)
passes on `a77e20dba`, including the mixed callback and heap admission repairs.
All guarded-heap, SSL/HTTPS, cryptography, independent TLS interoperability, HTTP,
framework and async cleanup stages pass. Both ordinary and generic async cleanup
consumers pass interpreter, JIT and AOT. The matching default-JIT SDK package and
all 26 installed VS Code checks pass separately on `566fb34dc` in
[run 38319479140](https://github.com/marinasundstrom/neoCLR/actions/runs/38319479140).
See the [release qualification record](../release-20261013/README.md) for retained
reports, component hashes and precise revision boundaries. This is qualification,
not release publication or Windows ARM64 support.

### Per-shape callback admission — 2026-10-13

The root-observation Windows qualification passed native HTTP but exposed the
remaining per-signature 32-target restriction while compiling managed SSL streams
([run 38399690376](https://github.com/marinasundstrom/neoCLR/actions/runs/38399690376)).
Remove that redundant restriction too: dispatch emits branches, with targets bounded
by the existing 2,048 selected-function limit. This aligns with .NET delegate
semantics: linking another callback with the same signature does not invalidate an
invocation. More targets can increase generated code and compilation time; selected
function, type, stack and heap bounds still apply. No callback ABI, Raven lowering
or CLI bridge encoding changes. This is a native backend admission correction.

Focused tests cover 65 same-signature targets for managed invocation in ARM64 Mach-O
and Windows x64 COFF, plus host invocation with more than 32 targets per unit/no-result
shape, retained roots and fault propagation. Recursion/borrowed-shape rejection remains.
Hosted SSL stream and HTTP execution is required in addition to object generation.

`windows-jit.yml` accepts `http_focus=true` for a bounded rerun: shared JIT preparation,
scalar/CLI checks, SSL streams and default-JIT HTTP. It records scope
`http-and-ssl-streams`; it does not claim the skipped full library suite passed.
The ordinary full gate remains the default.

The focused rerun at `55b53daed` now passes on Windows Server 2022 x64/MSVC
([run 38403029211](https://github.com/marinasundstrom/neoCLR/actions/runs/38403029211)).
It validates the formerly rejected SSL-stream HTTPS JSON consumer in interpreter,
default JIT and AOT, plus eight default-JIT HTTP cases. Shared JIT preparation,
scalar and CLI checks also pass. [Retained reports and scope](../../../benchmarks/http-server/root-observation.md#windows-qualification--2026-10-13)
keep the initial full-gate failure distinct from the focused rerun; the skipped
library suites are not newly qualified. Windows performance and ARM64 remain open.
