# Terminal failures with System.Fault

`System.Fault(message: String) -> void` is a namespace function. It terminates the
current guest execution and preserves a computed UTF-8 diagnostic message. It does
not return a Result, throw an exception object or resume guest execution. Use Result
for errors the caller can handle and Option for ordinary absence.

```raven
import System.*

func Stop(reason: string) {
    System.Fault("Cannot continue: " + reason)
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

## Implementation and compiler boundary

The public neoIL function calls a signature-checked InternalCall accepting String.
The native implementation produces the existing Fault outcome. This requires no new
opcode, exception hierarchy or change to normal return-stack conventions. The host
binding uses the existing Void value ABI; the public wrapper has a no-result return.
It cannot actually reach the wrapper's pop/ret instructions.

The Raven reference assembly projects this as a method on a namespace container
marked with Raven's existing TopLevelAttribute contract. Consumers can call
`System.Fault(message)` or import `System.*` and call `Fault(message)`. The experimental
importer validates the declaring assembly, marker, receiver and complete signature.
Raven itself and its Runtime Contract configuration are unchanged.

The signature is ordinary CLI void. This slice does **not** add compiler recognition
of non-returning calls: a non-void function must still satisfy Raven's existing return
analysis. A future general non-returning-call contract should be evaluated separately;
this API alone is not sufficient to remove all deferred-query migration blockers.

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
Compiler-visible non-returning semantics remain open rather than encoding a special
case based on the function's name in Raven.

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
