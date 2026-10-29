# Terminal failures with System.Panic

`System.Panic(message: string) -> unit` is a non-returning namespace function for
an unrecoverable guest error, distinct from recoverable `Result.Failure`.

```raven
import System.*

func Stop(reason: string) {
    Panic("Cannot continue: " + reason)
}
```

## Rename — 2026-10-29

Replace `System.Fail(message)` with `Panic(message)` after `import System.*`.
Qualified `System.Panic(message)` remains supported. No compatibility alias is
retained. Rebuild consumers with the matching Raven `codex/neoclr` compiler,
reference assembly and runtime library.

This change preserves existing behavior: the call terminates guest execution with
`UserFault`, preserving the computed UTF-8 message and fault diagnostics. The CLI
reports the fault and exits unsuccessfully; embedding hosts receive the existing
`Fault` outcome and survive. Empty messages remain permitted. Guest code cannot
catch the fault or choose its code. No guest Dispose/finally/destructor execution
is guaranteed. No new thread or task failure boundary is introduced.

The author identifies Panic with Rust's panic facility. Cleanup-aware stack
unwinding and revised thread/task termination semantics are a separate, deferred
behavior decision, not capabilities implemented by this rename. Unlike .NET
`throw`, this API does not introduce catchable exception objects. Use `Result`
for errors callers can handle and `Optional` for ordinary absence.

Raven's explicit RuntimeFailureContract now selects the public `Panic` namespace
function. The temporary CLI bridge recognizes the exact namespace marker, core
assembly, static non-generic signature and name. CLI metadata still encodes a
void method; native metadata uses the existing no-result call and terminal fault
path. Ordinary .NET methods named Panic remain ordinary calls. Internal
`neoCLR.Runtime.Fail`/`Fault` service identities, `FaultCode`, the neoIL `fault`
instruction and metadata builder `Fail` are unchanged. Target-owned metadata
continues to be the replacement for the temporary CLI identity check.

Rename validation: [compiler, runtime, native consumer and reference evidence](validation/panic-rename-20261029/README.md).

The historical checkpoints below describe their original bundles and spellings.

## Development rename — 2026-10-02

The namespace function is now `System.Fail(message)`: Fail is the action and Fault is
its runtime outcome. Replace `System.Fault(...)` (or imported `Fault(...)`) with
`System.Fail(...)` (or `Fail(...)`) and rebuild with the matching Raven compiler,
reference assembly and runtime library. No public compatibility alias is retained.
The Rust `Fault` type, `FaultCode` categories and neoIL `fault` instruction are unchanged.
The internal bootstrap binding `neoCLR.Runtime.Fault` and existing artifact filenames
remain implementation details. This is a naming change, not exception handling or a
change to host-process survival.

Raven's explicit neoCLR compatibility contract recognizes the exact imported Fail
signature and namespace marker as non-returning. Ordinary .NET methods named Fail,
and the old Fault spelling, do not acquire that target-specific behavior. Runtime
Contract configuration is unchanged; a general target-owned non-returning contract
remains the future replacement for this temporary CLI identity check.

## Implementation and compiler boundary

The public neoIL function calls a signature-checked InternalCall accepting String.
The native implementation produces the existing Fault outcome. This requires no new
opcode, exception hierarchy or change to normal return-stack conventions. The host
binding uses the existing Void value ABI; the public wrapper has a no-result return.
It cannot actually reach the wrapper's pop/ret instructions.

The Raven reference assembly projects this as a method on a namespace container
marked with Raven's existing TopLevelAttribute contract. Consumers can call
`System.Fail(message)` or import `System.*` and call `Fail(message)`. The experimental
importer validates the declaring assembly, marker, receiver and complete signature.
Raven's ordinary .NET behavior and Runtime Contract configuration are unchanged;
the matching neoCLR terminal-call recognition uses the new API name.

The signature remains ordinary CLI void; the matching neoCLR compiler contract supplies
terminal flow recognition. No ordinary .NET non-returning API policy is introduced.

## Comparison and tradeoffs — 2026-09-15

.NET's [Environment.FailFast](https://learn.microsoft.com/en-us/dotnet/api/system.environment.failfast)
terminates the process without running active finally blocks or finalizers (Microsoft
reference documentation, retrieved 2026-09-15). neoCLR reuses its existing terminal
execution-fault boundary instead of forcing process termination on embedding hosts.
The benefit is a consistent diagnostic outcome and host isolation; the cost is that
it is not a process-abort guarantee. Namespace placement follows the project's
preference for functions over utility classes. No operating-system crash dump or
event-log contract is introduced.

Alternatives were a new dynamic fault opcode, a fake exception object, or an
Environment-style static utility method. The existing InternalCall ABI plus a public
namespace function supplies the needed behavior with fewer new runtime mechanisms.
A general metadata-based non-returning contract remains open. The temporary neoCLR
compatibility rule currently checks the complete imported identity and signature,
not just a method name.

## Validation

`cargo test --test system_fault --test query_terminals` checks terminal execution,
message preservation, host survival, invalid signatures and existing query fault
boundaries. The independent Raven consumer check also verifies a computed Unicode
message and that code after the call does not execute:

```sh
python3 docs/experiments/raven-target/verify_fault.py \
  --compiler /path/to/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Release/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

Development rename validation (2026-10-02): all eight Raven terminal-flow tests, four
source-export admission cases and seven release-profile runtime tests pass. Both
qualified/imported Fail consumers preserve a computed Unicode message and terminate
before subsequent output. All 150 library slices were freshly regenerated; source/
artifact hashes and the regenerated API reference pass their snapshot checks. One
FileSystem helper also reflects the current compiler's local layout; a native consumer
verifies two-entry enumeration and the bounded-error path (42). The direct Raven
FileSystem-local consumer remains rejected by the existing CLI importer and was not
claimed as an end-to-end success. See [recorded evidence](experiments/extended-cli-metadata/system-fail-rename-validation-2026-10-02.json).


## Native source bootstrap — 2026-10-06

Unchanged System.Fail now compiles with an internal source RuntimeFailure adapter.
The new exact `neoCLR.Runtime.Fail(String) -> void` service raises the same UserFault;
wrong argument or inhabited-result declarations reject. Legacy `neoCLR.Runtime.Fault`
keeps its inhabited Void convention so the existing Numbers library and seed continue
to link. This separate service spelling avoids mixing incompatible return conventions
under one runtime identity. No new opcode or exception behavior is introduced.

The source-failure primitive profile omits the bridge Fail declaration. A separate
native consumer imports Failure.dll without its sources and checks unsuccessful exit,
empty stdout and the supplied fault message. Raven still recognizes terminal flow only
for the legacy core declaration: source/imported native Fail in let-else needs the next
compiler contract slice. The runtime guarantee already holds, but do not treat that as
proof of source-library flow analysis. [Evidence](experiments/extended-cli-metadata/source-failure-2026-10-06.md).


Development update (2026-10-07): the source-owned terminal-flow limitation described
above is resolved with Raven's explicit RuntimeFailureContract. The selected native
owner now works in let-else, including separate compilation. Ordinary .NET methods
remain unchanged; missing or incompatible owners reject before output. This changes
compiler configuration, not the public Fail signature or runtime fault behavior.
