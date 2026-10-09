# Completed task factories (development, 2026-10-09)

`System.Tasks.Task.CompletedTask` returns `Task<()>` already successfully completed.
`Task.FromResult<T>(value)` returns `Task<T>` already containing the supplied value;
ordinary calls infer T. Both live beside Task.Run on the static, non-generic helper.
Task<T> remains the consumer handle. Neither submits work or supplies default(T).

Like [.NET Task.CompletedTask](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.completedtask)
and [Task.FromResult](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.fromresult),
these distinguish existing completion from future producer work. neoCLR represents
completion-only work with a unit result rather than a non-generic task base class.
The benefit is one task/await protocol; the cost is generic unit specialization.
Task identity is unspecified. The current implementation allocates a Promise and
Task each time, using the invocation's current dispatcher; it does not cache across
invocations. Allocation reduction is separate future work, not a performance claim.

IsCompleted is true and IsCancelled false before return. Await requires no scheduled
work. Explicit OnCompleted observers retain deferred dispatch, including for a task
already completed. Reference results preserve identity; Result.Error is a completed
value, not cancellation or a Fault. System.Fail still ends the invocation; a
following CompletedTask return only supplies an unreachable, well-typed fallback.

## Implementation and validation

The helpers are ordinary Raven library code with no new runtime services or native
metadata contract. The legacy CLI reference temporarily projects unit as System.Void
in both accessor and property signatures, and maps exact factory signatures to the
same library implementation. Ordinary .NET compiler behavior is unchanged. Native
source libraries already encode the actual unit owner through the existing explicit
Runtime Contract; the legacy projection disappears with that bridge.

The focused [consumer](../benchmarks/native-web/TaskFactories.rvn) checks immediate
unit/value completion, integer/string inference, reference identity and deferred
callback ordering in the interpreter, sanitized native execution and a standalone
native executable. HTTP request/fault tests cover the factory-based server handler.
The website sample gate compiles all changed setup fallbacks against rebuilt libraries.
Detailed local outputs are retained under `target/task-factories-*`.

Published Preview 13 libraries do not contain these helpers. Development HTTP gates
use `prepare-native-development-bundle.py` to verify a base bundle, rebuild all four
source libraries against its pinned compiler and primitive core, finalize the matching
runtime seed, and publish a separate development manifest. Original bundle inputs
remain unchanged; logs and hashes distinguish base tools from rebuilt libraries.
