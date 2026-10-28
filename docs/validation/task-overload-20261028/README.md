# Structural callback overload fix — 2026-10-28

Raven `codex/neoclr` commit **91c1edca8** fixes generic specificity for structural
function signatures. It extends the existing nominal delegate rule to native
function inputs and results, with no Task.Run name special case. Ordinary .NET
behavior and Runtime Contract configuration are unchanged; native metadata and
CLI documentation projection need no new encoding. Raven owns binding; neoCLR
executes the selected method through the existing Function/backend contract.

Captured test logs have trailing whitespace normalized.

## Compiler evidence

- Before: equivalent Box<T> callback repro passes on .NET, fails with RAV0121 on
  neoCLR in both declaration orders (`before.log`).
- After: **21/21** focused tests pass, including both orders, conflicting
  specificity, structural function semantics and Task.Run (`focused.log`).
- Broad overload suite: **419 pass, 2 fail**, both before and after. Existing
  `Lowerer_WithLoweringTrace_RecordsNestedQueryPipelinesAcrossExtensionSources`
  and `Lowerer_WithLoweringTrace_RecordsNestedExtensionInvocations` fail trace
  assertions. This is not a green full-suite claim.
- Native net10 compiler build succeeds with the neoCLR metadata project; the
  mixed HTTP consumer emits successfully with `Task.Run(() => Short(id))`.

Reproduce focused compiler checks in Raven with:

```sh
dotnet test test/Raven.CodeAnalysis.Tests/Raven.CodeAnalysis.Tests.csproj /property:WarningLevel=0 --filter 'FullyQualifiedName~StructuralFunctionOverloadSpecificityTests|FullyQualifiedName~NativeStructuralFunctionTests|FullyQualifiedName~TaskRunBlockLambdaTests'
```

## Native HTTP qualification

The current mixed-workload experiment was rebuilt using the fixed compiler and
libraries from `target/dynamic-access-20261028/bundle-generic`. That bundle's
packaged compiler was not replaced. `inputs.json` records the separately built
compiler, guest and preparation hashes; `consumer.rvn.txt` captures the source.
The experiment itself remains in development, separate from this compiler fix.

| Ready storage | Collector | Checked requests | Result |
| --- | --- | ---: | --- |
| Baseline | Synchronous | 20 | Pass |
| Shared | Synchronous | 20 | Pass |
| Baseline | Concurrent | 20 | Pass |
| Shared | Concurrent | 20 | Pass |

Each run requests four fanout endpoints once, then four clients request all four.
Each parent Task.Run returns a task from Short, which schedules four children,
awaits them, checks shared capture aliases/mutations and returns a checked result.
The outer await must unwrap that result. HTTP bodies, request counts, zero-error
shutdown and exit status are checked; full output is retained in `http.json`.
These are lazy-JIT diagnostic correctness runs, not throughput measurements or
proof of overlapping managed workers. No parent relationship tracking was added.

This deliberately isolates fanout endpoints. The broader mixed-I/O experiment
still has a separate DNS/concurrent-GC failure and is not qualified by these
results. Captured-parameter/implicit-self emitter restrictions remain separate;
this source still uses local captures and a static helper. AOT was not rerun for
this binder-only correction. Website publication remains manual.
