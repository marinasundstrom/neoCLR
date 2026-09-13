# Raven match forms on neoCLR

Updated 2026-09-13 for the source experiment. This is an executable compatibility
matrix for the bounded target profile. The initial 2026-09-12 matrix used Raven
`22cea6fa1`; the additions below require the updated experimental Raven branch and
bridge (Raven `04c953d67` on `codex/neoclr-target-resolution`). Archived Preview 4 and local .7 packages do not gain these changes automatically.

## Verified forms

| Form | Evidence |
| --- | --- |
| Imported cases: `import System.Result.*`, then `Ok(let value)` / `Error(_)` | Generic payload types inferred from the scrutinee; both outcomes execute |
| Target-typed `.Ok(let value)` / `.Error(_)` | Existing member-union extraction and deconstruction contracts |
| `Result.Ok<int>(let value)` | Explicit nominal deconstruction remains supported |
| Option `.Some(let value)` / `.None` | Present and absent cases execute; `.None` has no payload |
| Keyword-first expression: `return match value { ... }` | Result<Int32,OverflowError> selects an Int32 arm value on success and error |
| Postfix expression: `value match { ... }` | Option<Int32> selects a String for Some and None |
| Statement: `match value { ... }` with action/block arms | Result success/error invokes the selected Console action |
| Statement in implicit tail-return position | Int32 arm values become the enclosing function result |
| Typed case patterns with `when` equality guard | Matching guard, failed guard and error paths select distinct arms |
| Side-effecting scrutinee | Helper prints once before the selected result, proving one evaluation |
| Result<Void,OverflowError> typed cases | Completion and error remain distinguishable; no unit payload is printed |

The [pattern sample](experiments/raven-target/samples/library-patterns.rvn) uses:

```raven
import System.Result.*

return match Math.Abs(value) {
    Ok(let amount) => amount
    Error(_) => -1
}
```

The [basic sample](experiments/raven-target/samples/library-match.rvn) retains explicit
typed case patterns as regression coverage. The [order workflow](raven-order-workflow.md)
uses imported cases to destructure file results and `Some(let order)` to retrieve a
shared application object. A Void success can use `.Ok` without binding a unit payload.

The [Void-result sample](experiments/raven-target/samples/library-match-void.rvn)
combines `?` with a statement-form match. Both samples can replace Main.rvn in the
prepared [VS Code project](experiments/raven-target/VSCODE.md); use the dedicated
neoCLR task. Keep a copy of any edited Main.rvn first. The original .4 compiler
supports the original typed forms; the source bridge now admits assigned string
locals used for expression results.

## Rejections and unresolved observations

| Tested shape | Current result |
| --- | --- |
| Missing Result error arm, expression or statement | Compile error RAV2100 |
| Arm following a catch-all | Compile error RAV2101 |
| Option.None pattern against Result | Compile error RAV2102 |
| Incompatible deconstruction arity | Compile error RAV1610 in the fixture |
| `.None()` / `.Ok()` with empty parentheses | No matching Deconstruct contract; use `.None` / `.Ok` for a payload-free case test |
| Final statement match with explicit `return` in its arm blocks | Compile error RAV1503 (Unit-to-Int32 conversion) in the fixture |

A further fixture with an explicit `return` inside an *expression* arm block
currently compiles and runs, returning 42. This and the final statement-arm rejection
need reconciliation with Raven's general `match-forms.md`, which describes different
rules. These are characterized observations, not recommended patterns or intentional
neoCLR language decisions. Evaluate any Raven correction separately on its experiment
branch; do not change its default .NET behavior to accommodate this bridge.

The 2026-09-12 matrix rejected shorthand and nominal deconstruction. Those two
limitations are resolved in the source experiment; the statement-return observation
remains open. No new syntax was added to Neo.

## Runtime and CLR comparison

C# also separates [switch expressions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/switch-expression)
from [switch statements](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/selection-statements).
Its nonexhaustive expression can produce a warning and throw if no arm matches.
Sources consulted 2026-09-12. The tested Raven union matches reject missing coverage
at compile time. Exhaustiveness and syntax are compiler responsibilities; the bridge
executes ordinary emitted calls, locals and branches, with no new match opcode.

Case declarations project `Deconstruct(out T)` for Some/Ok/Error. The bridge implements
these adapters using the existing runtime payload getter, and recognizes their
unconditional out assignments. These are target projection methods, not new native
runtime-library methods or named-case attributes. Known value receivers deconstruct
on a copy without boxing; reference payloads preserve their shared identity.

Nested pattern branches retain definite assignment, including String payloads. The
checks still reject an injected read of an uninitialized string. Generic CLR metadata
and primitive signatures are also tested by actually invoking emitted .NET methods.

The defensive `throw null` path already admitted by the bridge remains a terminal
fault. Its message now says “Guest program reached a terminal failure”, covering
match failures as well as invalid propagation. It is not recoverable exception flow.

String expression temporaries are admitted only after definite assignment. The bridge
does not synthesize an empty string for CLI default-null storage. Reading an unassigned
string local is rejected, including when InitLocals is set. This is a conservative
profile boundary, not a change to .NET's string or local initialization semantics.

## Repeatable check

From the neoCLR repository, with Raven built on the updated experiment branch and a fresh output:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- --matches /tmp/neoclr-match-check
python3 docs/experiments/raven-target/verify_matches.py /tmp/neoclr-match-check --runtime /absolute/path/to/neoclr
```

The probe captures diagnostics, imports accepted programs, and rejects an injected
uninitialized string read. The Python check verifies/runs every admitted fixture and
checks the rejected cases produced no executable. Review and update the matrix when
compiler behavior changes; do not silently convert a recorded limitation into a claim
of support. The saved-project verification also includes both readable match samples.


## Local pattern demo

On the development machine, a separate source-backed demo is prepared at
`/Users/robert/.neoclr/experiments/patterns-20260913/editor`. It uses refreshed
metadata and an updated language server, with the existing .7 extension frontend.
The neoCLR task compiles through the source bridge; this is not a repackaged SDK or
release. The earlier edited application demo and archived tools remain unchanged.

```sh
code --new-window --user-data-dir "$HOME/.neoclr/vscode/application-poc-20260913" --extensions-dir "$HOME/.neoclr/vscode/application-poc-20260913/extensions" "$HOME/.neoclr/experiments/patterns-20260913/editor"
```

Run **Tasks: Run Task** → **neoCLR: Run saved project**. Completion and hover for a
destructured String payload use the neoCLR String API (for example IsEmpty and
GetUtf8ByteCount). The stdio LSP check accepts `--patterns` to exercise both imported
and target-typed cases; pass the other flags matching the supplied library profile.
For another machine, use `--interfaces` followed by `prepare_editor.py --collections`
with the matching source-built language server, and copy `application-orders.rvn`
or `library-patterns.rvn` into the new project's Main.rvn.


Validation for this slice: 299 focused Raven tests, 50 saved-project checks, all
17 match-matrix fixtures, the order workflow's four state/file checks, and stdio LSP
completion/hover checks passed. The matrix includes compile rejections and an injected
uninitialized-string read; it is not a claim of complete Raven pattern coverage.
