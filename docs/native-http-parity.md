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
3. Add native DNS and outbound-connect bindings. These are now bound in
   both native profiles. Qualify success, refused connections,
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
retains build logs and hashed artifacts. Windows run
[37961490213](https://github.com/marinasundstrom/neoCLR/actions/runs/37961490213) at
`353188bf` passes all five cases with exact interpreter parity, as does the
macOS ARM64 run. [Retained evidence](windows-native-http-project-validation.json)
verifies all 53 Windows and 29 macOS artifact hashes; all 29 Windows and 25 macOS
tracked build inputs match the commit, including checkout line endings. The
32 Windows bundle inputs match the pinned bundle and both build reports use
the same recorded backend hash. The x64 console regression also passes in run
`37961490084`. HttpClient DNS/connect and ARM64 qualification remain separate.

## Native client adapter (in development)

Reuse interpreter `src/name_resolution.rs` and `src/socket_io.rs` policy: four
process-wide resolver workers, eight DNS operations per scope, sixteen unique
IPv4 addresses and at most 256 resolver records. Name validation is ASCII
alphanumeric/dot/hyphen with a 253-byte maximum. Blocking getaddrinfo runs only
on detached workers owning copied names/results and a refcounted host allocation.
Workers never touch guest pointers, callbacks or contexts. Cancellation/timeout
retires delivery without joining the worker; its capacity permit lasts until
completion. Workers balance their own Winsock startup, including after scope exit.
This follows the existing bounded interpreter policy rather than claiming an
improvement over .NET resolver scheduling. Native activation migration stays open.

Connect snapshots/deduplicates addresses and tries them in order. Poll observes
write/error readiness plus SO_ERROR, not getpeername. Each phase has a five-second
cap, clamped to a supplied shared deadline; fallback attempts have a one-second
cap while more addresses remain. A completed callback is delivered once, with
results consumed on the owner thread. Scope exit closes in-flight connects and
releases roots; late DNS completion owns no heap or scope storage.

The private native ABI represents a successful DNS snapshot as erased tag 7 with
a traced String[] pointer; DnsAddresses returns a fresh shallow array snapshot.
This is an adapter-owned encoding, not a new public Value kind or CLI metadata
rule. Existing string-array backing identities must be preserved at the generated
service return. The eventual native metadata/runtime-service ABI should replace
this private representation. No Raven compiler or public library API change is
required. Exact reserved service signatures gate the backend binding.

The focused native client consumer tests numeric resolution, rooted snapshot
retention, cancellation/timeout while workers are held, capacity retained after
cancellation, context teardown before worker completion, address fallback,
expired connect admission and connection refusal. macOS passes with sanitizers;
Windows run `37963921861` also passes all four socket consumers on both platforms;
end-to-end await-based Raven client and server gates also pass at `fbd73677`,
with ten client and five server cases on Windows x64 and matching macOS cases.
[Downloaded Windows evidence](native-http-await-validation.json) verifies 42 client
and 53 server artifact hashes.


## Await-first native entry integration (2026-10-09, in development)

The author requires `await` as the application default. The client showcase uses
async `Main` and awaits `HttpClient.GetString`; explicit callbacks remain appropriate
inside library awaiters and host-boundary tests. The native task scope now has an
opt-in host poll hook and a saved published entry-frame boundary. See the
[entry lifecycle contract](experiments/aot-console/task-queue.md). Queue-only hosts
need no socket linkage. Missing required I/O pumping faults deterministically.
The existing cancellation contract is unchanged: cancelling an awaited task propagates
to the entry task, whose result is a cancellation fault under the current bridge.

The backend binds the exact existing DNS/connect InternalCalls. Closed-world
reference layouts now preserve inherited field prefixes and assignability, with
leading direct-base constructor forwarding. An inherited virtual `ToString` body
can be selected only when every loaded descendant has no competing member; even
an unconstructed descendant override rejects monomorphic admission. This closes
the `IPAddress` formatting path without claiming general virtual dispatch. Relative
to CLR reference inheritance, this is a deliberately narrower admission proof,
with a 32-lane object layout bound. Closed selection allows 512 types. The original 512-clone allowance was aligned
with the existing 1024-function cap on 2026-10-10 for generic JSON collection adapters;
each clone still consumes that same total function budget and a 128-level dependency nesting guard; these are compiler work limits, not language
limits. No performance improvement is claimed. General dispatch remains separate.

The private DNS snapshot tag and these backend admission rules do not change Raven
Runtime Contract configuration or emitted CLI metadata. Compiler bridge ownership
and replacement remain as documented above. Rebuild generated objects and native
adapters together; their entry-scope ABI is private and has changed.

## Serial host reuse and recovery (development)

The next bounded hosting slice re-enters the same generated HTTP program three
times on one native heap/context. The private correctness host renders each guest
Fault before releasing its diagnostic roots, verifies scope/root teardown, clears
that fault, collects the invocation heap and requires zero retained bytes before
admitting the next entry. It preserves the first guest failure as process exit
status while allowing later entries; a host lifecycle failure stops immediately.
The default executable still invokes Main once. `NEOCLR_HTTP_INVOCATIONS=3` is a
private qualification build option, not a public persistent-host API.

The shared client validator compares success → fault/cancellation → success with
three fresh interpreter processes. It covers a continuation fault, a fault while
DNS/request completion remains pending, and entry cancellation. Late resolver
workers own copied host memory only; teardown does not wait for a blocked resolver
or allow it to retain guest pointers. The same fixture runs on macOS and Windows.
Standalone execution still requires only OS libraries.

This advances the lifecycle foundation in the [roadmap](platform-roadmap.md), not
retained application state, concurrency, reload or green-thread scheduling. Relative
to the .NET hosting/Native AOT baseline already reviewed in the roadmap, the useful
new guarantee is explicit reusable entry cleanup; costs include full collection
between entries and a deliberately stateless boundary. Persistent roots, retained
code/data ownership and suspension migration remain separate design/validation work.

[Shared reuse evidence](native-http-reuse-validation.json) records all 14 passing
client/factory/recovery cases on macOS ARM64 and Windows x64. Windows client run
[37970610749](https://github.com/marinasundstrom/neoCLR/actions/runs/37970610749)
and the five-case server run
[37970610944](https://github.com/marinasundstrom/neoCLR/actions/runs/37970610944)
both pass at `15ebc8b5`, using rebuilt source libraries. Downloaded binary, library,
compiler and validation hashes are verified against the reports; tracked source
inputs match that commit, allowing Windows checkout line endings. Ten non-execution
`.vscode` files per toolchain were omitted by artifact upload; the evidence records
that exact gap and subsequent uploads include hidden files.
