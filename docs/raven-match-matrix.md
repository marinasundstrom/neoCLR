# Raven match forms on neoCLR

Recorded 2026-09-12 against Raven `22cea6fa1` on the isolated target branch.
This is an executable compatibility matrix for the bounded target profile, not a
claim that every Raven pattern is supported. No Raven compiler changes are made
in this slice.

## Verified forms

| Form | Evidence |
| --- | --- |
| Keyword-first expression: `return match value { ... }` | Result<Int32,OverflowError> selects an Int32 arm value on success and error |
| Postfix expression: `value match { ... }` | Option<Int32> selects a String for Some and None |
| Statement: `match value { ... }` with action/block arms | Result success/error invokes the selected Console action |
| Statement in implicit tail-return position | Int32 arm values become the enclosing function result |
| Typed case patterns with `when` equality guard | Matching guard, failed guard and error paths select distinct arms |
| Side-effecting scrutinee | Helper prints once before the selected result, proving one evaluation |
| Result<Void,OverflowError> typed cases | Completion and error remain distinguishable; no unit payload is printed |

The runnable [basic sample](experiments/raven-target/samples/library-match.rvn) uses:

```raven
return match Math.Abs(value) {
    Result.Ok<int> ok => ok.Value
    Result.Error<OverflowError> error => -1
}
```

The [Void-result sample](experiments/raven-target/samples/library-match-void.rvn)
combines `?` with a statement-form match. Both samples can replace Main.rvn in the
prepared [VS Code project](experiments/raven-target/VSCODE.md); use the dedicated
neoCLR task. Keep a copy of any edited Main.rvn first. The installed .4 compiler
already supports the verified forms; the source bridge now admits assigned string
locals used for expression results.

## Rejections and unresolved observations

| Tested shape | Current result |
| --- | --- |
| Missing Result error arm, expression or statement | Compile error RAV2100 |
| Arm following a catch-all | Compile error RAV2101 |
| Option.None pattern against Result | Compile error RAV2102 |
| `.Ok(let value)` / `.Error(_)` against the imported carrier | Compile error RAV2104, with related diagnostics; target-typed case shorthand is not admitted by this compiler/profile combination |
| `Result.Ok<int>(let value)` deconstruction | Compile error RAV1610: these declaration case types do not expose a deconstruction contract |
| Final statement match with explicit `return` in its arm blocks | Compile error RAV1503 (Unit-to-Int32 conversion) in the fixture |

A further fixture with an explicit `return` inside an *expression* arm block
currently compiles and runs, returning 42. This and the final statement-arm rejection
need reconciliation with Raven's general `match-forms.md`, which describes different
rules. These are characterized observations, not recommended patterns or intentional
neoCLR language decisions. Evaluate any Raven correction separately on its experiment
branch; do not change its default .NET behavior to accommodate this bridge.

## Runtime and CLR comparison

C# also separates [switch expressions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/switch-expression)
from [switch statements](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/selection-statements).
Its nonexhaustive expression can produce a warning and throw if no arm matches.
Sources consulted 2026-09-12. The tested Raven union matches reject missing coverage
at compile time. Exhaustiveness and syntax are compiler responsibilities; the bridge
executes ordinary emitted calls, locals and branches, with no new match opcode.

The defensive `throw null` path already admitted by the bridge remains a terminal
fault. Its message now says “Guest program reached a terminal failure”, covering
match failures as well as invalid propagation. It is not recoverable exception flow.

String expression temporaries are admitted only after definite assignment. The bridge
does not synthesize an empty string for CLI default-null storage. Reading an unassigned
string local is rejected, including when InitLocals is set. This is a conservative
profile boundary, not a change to .NET's string or local initialization semantics.

## Repeatable check

From the neoCLR repository, with Raven built at the revision above and a fresh output:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- --matches /tmp/neoclr-match-check
python3 docs/experiments/raven-target/verify_matches.py /tmp/neoclr-match-check --runtime /absolute/path/to/neoclr
```

The probe captures diagnostics, imports accepted programs, and rejects an injected
uninitialized string read. The Python check verifies/runs every admitted fixture and
checks the rejected cases produced no executable. Review and update the matrix when
compiler behavior changes; do not silently convert a recorded limitation into a claim
of support. The saved-project verification also includes both readable match samples.
