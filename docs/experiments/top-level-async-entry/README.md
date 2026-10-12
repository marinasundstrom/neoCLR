# Top-level async entrypoint compiler follow-through

The wider async tests run after the unary-await fix exposed 15 existing failures.
Two shared compiler defects caused them:

- Synthesized MainAsync looked for framework Task types in the source assembly's
  own declarations. Framework Task types normally come from references; a failed
  lookup incorrectly fell back to Unit. Resolve through the compilation instead.
- Compilation-unit functions were registered during ordinary top-level body
  emission. Async state-machine emission could encounter calls before those
  functions had member definitions. Declare these functions alongside Program
  members before emitting any bodies.

Raven main `86bfabafd` fixes both, with a deliberate carry to the separate native
compiler line as `371d1d707`. All **148 functions/async tests pass**, including the
15 former failures and a new executable entrypoint regression. The semantic
regression explicitly checks that Task resolves from a referenced assembly.

[The CLI fixture](Program.rvn) awaits a top-level function after Task.Yield and
returns exit code 42. The native compiler branch's ordinary **.NET net10.0 path**
failed with RAV1503 before the fix and now compiles and executes with exit code 42,
empty stdout and empty stderr. [Evidence and hashes](validation.json).

```sh
dotnet /path/to/fixed/rvnc.dll \
  docs/experiments/top-level-async-entry/Program.rvn \
  --framework net10.0 -o /tmp/top-level-async.dll
dotnet /tmp/top-level-async.dll
# Expected exit status: 42
```

This restores .NET-style asynchronous process-entry behavior. No new Runtime
Contract setting, public API, native metadata or CLI bridge representation is
introduced. Target-owned native Main adaptation is unchanged; the native HTTPS
project still compiles. This is not new qualification of native top-level await,
Windows execution or TLS packaging. The prior HTTPS runtime evidence remains
applicable to its separately pinned compiler and explicit Main consumer.
