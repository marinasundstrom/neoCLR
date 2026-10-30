# Native Result entry points (development, 2026-10-30)

This closes a gap in Raven's direct native emitter. The earlier CLI-import path
already adapted Result entry points; the native emitter previously accepted only
Task<int>/Task<()> for async Main, and did not adapt synchronous Result process
statuses. Raven's language rules remain unchanged.

Static, nongeneric Main can take no arguments or one string[] argument. Supported
return families are (), int, Task<()>, Task<int>, Result<(), E>, Result<int, E>,
Task<Result<(), E>> and Task<Result<int, E>>. Ordinary methods returning Tasks and
async methods use the same entry adaptation. The target has no separate nongeneric
Task class. Only one eligible entry may be declared.

| Outcome | Process behavior |
| --- | --- |
| Unit or Ok(()) | Exit zero. |
| int or Ok(int) | Use that integer status, subject to host OS truncation. |
| Failure(error) | Write normal Object/ToString display and a newline to stderr; exit one. |
| Null error | Write an empty line to stderr; exit one. |
| Terminal fault / cancelled or incomplete entry task | Existing runtime fault behavior; not converted to Failure. |

User-defined implicit error conversions and conversion to System.Error remain
ordinary Raven `?` binding. Entry adaptation does not reinterpret or restrict E
through a new error interface. Implementing Error does not automatically invoke
FormatError; override ToString or explicitly format a string error when a richer
diagnostic is wanted. Normal stdout is unaffected.

## Native implementation and comparison

The native target emits an Int32 process adapter and a private generated Raven
helper. The helper is bound and lowered normally, using Result patterns, ordinary
error boxing/virtual display and Console.Error. Structural-union errors are
extracted through TryGetResidual, matched against their concrete alternatives and
formatted as the selected payload, avoiding whole-union Object boxing. Separate integer/unit success
helpers avoid boxing a successful unit. No new metadata feature, custom attribute,
public runtime API or Runtime Contract option is introduced.

Async adaptation keeps the Task alive while the existing DrainEntryTasks contract
drives the default queue and registered host operations, then calls GetResult and
adapts its Result. It retains drain-to-quiescence behavior, not a CLR thread pool or
blocking Task.GetResult policy. Application-owned private queues remain owner-driven.

The policy matches Raven's
[entry specification](https://github.com/marinasundstrom/raven/blob/main/docs/lang/spec/top-level-code-and-entry-points.md)
and its ordinary .NET Result bridge. C# Main provides int/void/Task/Task<int>;
Result entry returns are a Raven extension, not a System API compatibility claim.
The CLI-import implementation and historical qualification remain recorded under
[entry results](experiments/entry-results/README.md). The new change is target-specific
on Raven codex/neoclr; it requires no general main change or wholesale branch merge.

## Qualification

`runtime/System.Runtime/test/ResultEntryPointTests` contains process fixtures for all
four Result return forms. These are Main functions rather than test-method attributes
because process exit status and stderr are the assertions. The runner compiles each
fixture and checks native/interpreter success/failure statuses and exact separated
output. It also checks null errors, a user-defined error conversion, Error-interface
propagation, and both branches of structural error unions.

```sh
python3 scripts/validate-result-entrypoints.py --bundle /path/to/bundle \
  --runtime /path/to/neoclr --aot /path/to/neoclr-codegen --output target/entry-check
```

The integrated extensions sample now returns Task<Result<(), ServiceError>> from Main
and exercises real suspension through timer-driven configuration observations.
See [the dated receipt](validation/result-entrypoints-20261030/README.md) for exact
compiler/runtime evidence and platform scope.
