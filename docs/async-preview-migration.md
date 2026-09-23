# Async preview migration review

Draft for the upcoming async/Tasks preview, reviewed 2026-09-23. No release number,
date or final candidate is selected. The baseline is published `v0.1.0-preview.8`;
the initial review covers changes through `556fd7a`. Later candidate changes must
be reviewed too. Published Preview 8 notes and artifacts remain unchanged.

## Upgrade the toolchain as a set

Rebuild application and library projects with matching Raven SDK, reference assembly,
import bridge and System implementation. Rebuild dependent projects too. Do not mix
Preview 8 references or generated application artifacts with development Tasks.
Use the bundle's `.rvnproj` assets: they select heap async state machines, cancellation
propagation and exception-free lowering through `RavenHeapAsyncStateMachines`,
`RavenPropagateAsyncCancellation` and `RavenCaptureAsyncExceptions=false`.
These are neoCLR target policies, not changes to Raven's ordinary .NET behavior.

Raven's `unit` and `()` still map to `System.Void`; this mapping already existed in
Preview 8. The new work supports it through async completion and Result propagation.
Use the matched compiler rather than substituting .NET Task or builder assemblies.

## Changes from published Preview 8

| Area | Migration / behavior |
| --- | --- |
| Queries | Replace `Where` with `Filter` and `Select` with `Map`. No legacy aliases. Deferred traversal remains; see the [operator comparison](../website/features/collections/index.html). |
| File and path APIs | Replace `System.IO` imports/qualified names with `System.Storage`, including file error types. `ConsoleReadError` moves to `System`. This rename does not introduce storage providers or change local synchronous file behavior. |
| Message-wrapper errors | Replace `System.Error` payloads with `string` or domain-specific error types. Remove `Error.FromMessage` and `.Message`. `Result.Error(...)` remains a union case and is not the removed wrapper type. |
| Old low-level error artifacts | Rebuild metadata/JSON artifacts using the retired Error intrinsic. Replace neoIL `error "text"` with `ldstr "text"` when the desired payload is text. Update Rust uses of `Value::Error`, `Type::Error`, `Instruction::Error` and `RuntimeService::ErrorValues`; those variants are removed. |
| Runtime service inventory | Exhaustive Rust matches must handle the new `TaskDispatch` and `IsolatedWorkers` service variants. They identify runtime requirements, not an application permission sandbox. |
| Runtime limits | Exhaustive Rust `Limits` literals need `worker_result_bytes`, or use `..Default::default()`. It defaults to 1 MiB per worker for successful result text plus captured output. This is not a total-memory limit. |

Tasks and isolated workers are new relative to Preview 8. Their limits are new-feature
constraints, not regressions in a previously published Task API. Constructor storage
now permits erased/delegate payload fields to remain uninitialized while the constructor
assigns them. Reading unassigned storage or publishing an incompletely initialized
object still faults; explicit unsupported defaults remain rejected.

Additional iterable and Option/Result operators are additive. Their vocabulary,
short-circuiting, callback and disposal contracts are documented in the existing
[query](raven-query-api.md) and [outcome](raven-outcome-operators.md) design records.

## Migration from intermediate development builds

These names appeared during development, not in published Preview 8:

- Replace `System.Threading.Tasks` imports with `System.Tasks`.
- Replace `TaskCompletionSource<T>` with `Promise<T>` and `TrySetResult(value)` with
  `Complete(value)`. `Complete` and `Cancel` return false after a terminal transition.
- Use `Task.State` or `Task.Outcome` to distinguish success from cancellation.
  `IsCompleted` is true for either terminal state. `GetResult()` faults on pending
  or cancelled tasks; it does not block.
- Prefer ordinary Promise construction and async calls for default dispatch. Explicit
  `TaskQueue.Run/Drain` remains useful for controlled ordering. The invocation pumps
  its default queue before returning, so posted callbacks can add observable work
  after the entry function returns. A pending Promise alone does not keep it alive.
- `Map` transforms the entire payload. `MapResult` maps only an Ok payload and preserves
  Error; `Then` chains tasks. Cancellation bypasses their user callbacks. Awaiting a
  cancelled task cancels the enclosing async task; it does not invent a value.
- Use `(await input)?` with the current postfix-first precedence for awaited Result
  propagation. Await inside `for` loops remains diagnosed until iterator cleanup is
  suspension-aware. These compiler protocols remain provisional.

## Worker and host boundaries

`Thread.Start` and `ThreadPool.Queue` take static String-to-String callbacks without
captures. Each job has isolated guest state. The pool has two threads; submissions
are bounded to 64 per invocation and nested workers are rejected. Ordinary completion
uses queued blocking joins in submission order, not readiness order.

Worker output is buffered and forwarded on completion. The result/output quota counts
UTF-8 bytes plus one byte per output line; exceeding it faults. Console input and
native P/Invoke are unavailable in workers. A worker Fault is terminal, not a Task
failure value. Parent host cancellation is checked around joins and between output
lines; already visible output is retained. Teardown joins workers cooperatively and
can wait for a blocking host operation.

Applications using earlier development workers with larger outputs must raise the
host limit deliberately or reduce their payloads. Promise cancellation changes the
consumer outcome; it does not stop worker execution or cancel the whole host invocation.
The notification adapter remains experimental and is not the ordinary worker path.

See [Task contracts](task-contracts.md), [isolated workers](isolated-workers.md),
[host cancellation](cancellation.md) and [error migration](errors.md) for the existing
.NET comparisons, tradeoffs and tested behavior. This review adds no API contract.

## Distribution review still required

The local package evidence is [recorded separately](async-preview-readiness.md).
Before publishing, select and check the exact candidate, rerun affected package
checks after runtime changes, finish interactive editor validation, and validate
all six platform/toolchain source jobs. Binary support must match actual package
execution evidence. Review complete release notes, notices and final checksums;
this migration inventory is not a provenance or publication certification.

The API documentation requirement is a useful overview and main async descriptions,
not exhaustive coverage. Stream, Storage-provider, Encoding and JSON experiments
must remain labelled experiments; networking is later roadmap work.
