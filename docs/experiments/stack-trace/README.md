# Owned current-stack snapshots — development 2026-10-12

The author asks for a class that lets managed programs print their call stack.
`System.Diagnostics.StackTrace` captures an owned text snapshot in its constructor;
`ToString()` returns that snapshot, even after the original frames return or GC runs.

```raven
import System.Diagnostics.*

let trace = StackTrace()
Console.WriteLine(trace.ToString())
```

The first surface intentionally implements current capture and printing only.
Structured StackFrame/GetFrame/FrameCount, file/line lookup, skip-count constructors
and causal async-stack stitching remain gaps, not alternative permanent semantics.
No arguments, locals, arbitrary native frames or another operation's suspended frames
are captured. Generated async-body names can still appear. Faults remain terminal;
capture neither throws a guest exception nor modifies a prior Fault snapshot.

## Contract and comparison

Like [.NET StackTrace](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.stacktrace?view=net-10.0),
construction captures the current caller and formatting is independent of execution
continuing. Unlike .NET's richer StackFrame/exception/source overloads, this first
slice exposes only owned text. These are incomplete APIs to expand, not a deliberate
claim of improved semantics. The existing neoCLR 64-frame bound remains, with an
additional 64 KiB UTF-8 text bound and an explicit truncation marker. The limits bound
diagnostic work/memory but lose older/deeper frames; allocation exhaustion can still
terminate execution. Instruction positions are neoIL instruction indexes, not CLI
byte offsets or native addresses. Source locations are not fabricated.

Capture uses the active logical guest chain. After runtime suspension/resumption,
it captures the currently running body. It does not infer a parent from all parked
stacks or promise a complete causal async history. Task/Promise ownership and GC
reachability are unchanged. Captured text holds no runtime frame pointers or user
object references. The profiler and debugger remain separate tools; both can reuse
names, mappings and future stack-snapshot contracts.

## Implementation boundary

The private `neoCLR.Runtime.StackTraceCapture(Int32) -> String` service receives a
nonnegative skip count. The managed constructor skips its own frame. Interpreter
execution supplies the current VM frames. Native builds use published logical root
frames and an immutable image table of method names, requiring `--native-gc` and
explicit `--bind-stack-trace` admission. The standard JIT CLI supplies that admission.
Capture never scans machine-stack memory and allocation never collects in the native
service; the returned text becomes ordinary managed storage. Invalid native inputs
preserve the output pointer and Fault context. No stable hosting ABI is introduced.

Original source names are preferred when present. A diagnostic-only decoder handles
the exact reserved experimental `NeoMetadata_<hash>.F_/T_/M_` transport encoding;
malformed/unrecognized names remain unchanged. This is a temporary projection aid,
not name resolution. Native Fault formatting also benefits from readable names.
Production metadata should supply direct logical names and source/IL mappings, so
this decoder can eventually disappear. Raven lowering and Runtime Contract settings
are unchanged; compiler main and the native branch remain separate.

## Validation

`validate.py` accepts a passing async compiler recipe, benchmark host recipe, a
fresh four-library native bundle, an interpreter host and a fresh output directory.
It runs ordinary and pending-await consumers in interpreter/JIT/AOT, checks repeated
snapshot equality after GC, and verifies native allocation/stack teardown. The
sanitized C test additionally checks truncation, invalid arguments, Fault-context
preservation and snapshot retention after every source frame has returned.
Focused Rust tests cover the same current-stack/skip/truncation boundary plus
transport-name decoding and existing terminal-Fault stack behavior.

The [recorded macOS ARM64 run](validation.json) passes both consumers in all three
engines: owned snapshots remain identical after return/GC, and each of the sixteen
native invocations reclaims every allocated block and releases its native stacks.
The pending-await fixture deliberately asserts the emitted `$closure$0.Invoke`
body name; restoring its source-level `CaptureAfter` name needs compiler mappings.
Ten Rust stack tests, the exact native binding admission test, sanitized native
capture checks and both API snapshot checks pass. The [library build](library-build.json)
pins actual source/artifact hashes from main plus this uncommitted slice; the base
revision alone is not the complete library provenance. Raven compiler code is
`e349f4783`, with native-line documentation HEAD `7622d09e4` at build time.

A compiler gap observed during authoring remains deferred: spelling this class
`sealed` caused RAV0611 at construction in the tested native compiler. The ordinary
public class used here compiles and executes; no sealed-class fix is claimed.
