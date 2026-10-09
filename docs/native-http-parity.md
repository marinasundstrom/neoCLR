# Native HTTP parity — development plan, 2026-10-09

The author requests HttpClient and HttpServer as the next native showcase, with
consistent, predictable macOS/Windows behavior. This extends the completed
Windows console project work. It does not establish general native networking.

## Contract and sequence

1. Qualify the same private listener, accept and transfer consumers on both OSes:
   strict IPv4 input, bounded slots, opaque identities, nonblocking operations,
   deferred exactly-once callback delivery, rooted receive buffers, snapshotted
   sends, cancellation, monotonic deadlines and complete scope cleanup.
2. Integrate the shared adapter with Windows guarded heap/stack admission and the
   HTTP project workflow. Compare the same Raven server and interpreter scenarios:
   greeting, fragmented input, duplicate lengths, handler errors and guest faults.
3. Add native DNS and outbound-connect bindings. Both are currently absent from
   the native backend, including macOS. Qualify success, refused connections,
   deadline/cancellation races and cleanup before claiming native HttpClient.
4. Run paired Raven client/server projects on both platforms with byte-exact
   responses and equivalent typed failures. Preserve standalone deployment and
   artifact/toolchain provenance. Keep performance claims separate from correctness.

TLS, HTTP/2, general asynchronous OS backends and scheduler migration are not
implemented by these steps. Existing HTTP parsing/library policy remains shared;
this work must not silently select different policies on each OS.

## Platform boundary and tradeoffs

Reuse the .NET comparison and UTF-8/bounded HTTP decisions in the
[native execution investigation](native-execution-investigation.md) and
[existing web evidence](../benchmarks/native-web/README.md). .NET provides portable
HTTP/socket APIs over OS-specific services. This slice addresses missing neoCLR
host support, not a demonstrated limitation or performance improvement over .NET.

Keep a single socket operation state machine. The private `socket-os.h` translates
native handle width, nonblocking/noninherited configuration, send/receive, errors,
clock and scope initialization. Windows uses SOCKET/closesocket with balanced
WSAStartup/WSACleanup for each admitted scope; POSIX uses descriptors/close. Both
map errors into the existing guest SocketError contract, matching the interpreter
where OS error categories are equivalent. Do not expose errno or WSA codes.

QPC and CLOCK_MONOTONIC supply elapsed time; wall-clock adjustments must not alter
deadlines. Millisecond deadlines have the same interpretation but OS scheduling
can delay observation: identical latency is not promised. Shared polling avoids
two diverging lifetime policies at the cost of polling overhead and bounded
capacity. IOCP/kqueue and asynchronous DNS workers remain alternatives to evaluate
when implementing client completion and runtime scheduling. A blocking resolver
must not be presented as cancellation/deadline support.

Thread-local scopes and roots remain temporary host ownership. Shutdown closes
sockets and releases operations/roots before heap destruction. This does not
qualify fibers, suspended activation migration or a public scheduler contract.
There is no compiler bridge encoding or public API change in this adapter slice.

Primary OS references reviewed 2026-10-09:

- [WSAStartup](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-wsastartup): version negotiation and balanced ownership.
- [closesocket](https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-closesocket): Windows socket lifetime.
- [QPC guidance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/acquiring-high-resolution-time-stamps): interval measurement independent of wall time; also used by .NET Stopwatch.
- [connect](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect) and [select](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-select): pending nonblocking connection success/failure must be observed explicitly.

## Evidence

`scripts/validate-native-network.py` compiles and executes the same three C
consumers on macOS and Windows. The macOS build uses undefined/bounds sanitizers;
Windows uses MSVC /W4 /WX and the static CRT. Reports retain source/artifact hashes,
commands and logs, and cannot pass when any consumer is skipped. The dedicated
`Native network parity` Action runs both platforms.
[Run 37960722734](https://github.com/marinasundstrom/neoCLR/actions/runs/37960722734)
at `da8ca1f2` passes all three consumers on macOS ARM64 and Windows x64. All
45 downloaded artifact hashes and 14 source inputs per platform are verified in
[the retained evidence](native-network-parity-validation.json), allowing Windows
checkout line endings. These C consumers are prerequisites, not evidence that
Raven HttpClient or a Windows native HTTP project already works.

## Windows ARM64 follow-up

The author requests Windows on ARM support (2026-10-09), then explicitly directs
completion of Windows HTTP project integration for the showcase. Preserve that
sequence. Winsock and most host-service policy should be reusable, but current
target selection, MSVC host guards and stack accounting explicitly assume x64.
Cranelift 0.121.2 has AArch64 inline probes; its outlined probe path is unimplemented.
Neither fact establishes Windows ARM64 generated-code correctness. Validate target
ABI/COFF linking, page probing, final-frame bounds, host admission and managed
root/fault cleanup on native ARM64 before admitting the project profile.

Unlike .NET's established per-architecture deployment model, this remains a
missing neoCLR target qualification, not a proposed semantic difference. Use
`aarch64-pc-windows-msvc` as the prospective native target, separate from ARM64EC
or x64 emulation. [Microsoft's ARM64 ABI](https://learn.microsoft.com/en-us/cpp/build/arm64-windows-abi-conventions?view=msvc-170)
specifies stack and calling conventions; [GitHub's runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
lists `windows-11-arm` for native execution (reviewed 2026-10-09). Adding that
qualification costs another toolchain/artifact/test lane; no difficulty or
performance claim is established yet.

## Windows HTTP project integration (development)

`--profile windows-http` selects the explicit `--windows-http-experiment` backend
contract: Windows x64, closed-world compilation, reference arena, GC frames, stack
budget and task/listener/accept/transfer bindings. File/path/character services
remain rejected. This is a private backend/host contract with no Raven compiler
or CLI metadata encoding change. The macOS `http` profile remains the default
HTTP target. Both use the same callback loop and 15-second host completion bound.
Windows supplies guarded heap/stack admission and binary CRT streams; the EXE may
import only KERNEL32 and WS2_32. Scope/root cleanup precedes heap destruction.

`scripts/validate-native-http-project.py` builds fresh greeting and callback-fault
projects, compares five real-request scenarios against the interpreter, executes
with only the native binary in its directory and no runtime/SDK on PATH, and
rejects overwrite/stale-output publication. The dedicated Windows HTTP Action
retains build logs and hashed artifacts. Windows execution is pending. The
HttpClient DNS/connect work and ARM64 qualification remain separate.
