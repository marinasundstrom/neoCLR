# Task namespace migration (development, 2026-10-12)

The author selects `System.Concurrency.Tasks` after the asynchronous stream
migration, before SSL and `System.Security.Cryptography`. The module groups
`Task`, `Task<T>`, `Promise<T>`, task outcomes, states and operators beneath the
existing concurrency module. Its assembly remains `System.Runtime`. Scheduling
implementation remains runtime-owned; the namespace does not change scheduling,
completion, cancellation, suspension or task lifetime.

.NET uses `System.Threading.Tasks`. neoCLR deliberately uses `Concurrency` to group
operations and scheduling without implying one operating-system thread per task.
The benefit is consistent organization with existing Scheduler and cancellation
APIs; the cost is a different import from .NET and a breaking metadata identity
change for existing neoCLR clients. There is no forwarding alias: rebuild libraries,
applications, compiler bindings and tools together. No performance claim follows
from this change. The existing [scheduler comparison](runtime-scheduling-design.md) remains
the behavioral reference.

Raven's explicit neoCLR Runtime Contract maps the new Task identity only for the
selected async provider. Ordinary .NET compilation still uses
`System.Threading.Tasks.Task<T>`. Source-owned and imported native providers must
agree with the mapping. The temporary CLI facade uses the same new metadata name;
no new encoding or lost information is introduced. Native metadata carries the
ordinary fully qualified identity and module association.

Validation must cover source-owned library compilation, imported async methods and
entrypoints, task completion/cancellation/dispatch, interpreter/JIT/AOT consumers,
and both API snapshots. Existing historical evidence records its original names.
Legacy generated neoIL fragments receive a mechanical identity migration, not a
claim that their historical bodies were recompiled from today's library sources.

## Qualification

[Retained evidence](../runtime/raven/tests/task-namespace-validation.json) records
seven memory-stream and eleven text-stream cases passing in interpreter, JIT and
AOT on macOS ARM64, eleven deferred HTTP upload peer cases in JIT, six runtime
atomic/dispatch checks and eighteen Raven provider/entrypoint checks. All five
native libraries compile, including PostgreSQL. Both API references are refreshed;
22 website tests and two snapshot checks pass. The renderer check caught and fixed
the namespace article UID and extension namespace selection. Windows execution is
not claimed by this local migration gate.
