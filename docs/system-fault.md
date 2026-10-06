# Terminal failures with System.Fail

`System.Fail(message: String) -> void` is a namespace function. It terminates the
current guest execution and preserves a computed UTF-8 diagnostic message. It does
not return a Result, throw an exception object or resume guest execution. Use Result
for errors the caller can handle and Option for ordinary absence.

```raven
import System.*

func Stop(reason: string) {
    System.Fail("Cannot continue: " + reason)
}
```

The command-line runner reports the fault and exits unsuccessfully. Embedding hosts
receive the existing runtime Fault outcome; the runtime does not abort their process.
No guest Dispose/finally/destructor execution is promised on this path. The API
requests a terminal failure; it is not an orderly shutdown or cleanup facility.
Empty messages are permitted, though descriptive messages are more useful.

**Development 2026-09-23:** this API and the explicit fault instruction always
produce `FaultCode::UserFault`. Guest code cannot supply a code; a message such as
`StackOverflow` does not change that classification. Runtime detection sites assign
specific codes for frame limits, arithmetic, memory and cancellation failures.
See the complete [on-site fault reference](../api-docs/faults.md) for code identifiers,
host API fields and debugger transport. Message text remains diagnostic, not a
machine-readable discriminator.

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
The new exact `neoCLR.Runtime.Fail(String) -> noresult` service raises the same UserFault;
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
