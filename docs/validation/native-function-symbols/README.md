# Native function symbols (integrated, 2026-10-16)

Integrated into neoCLR `main` and Raven `codex/neoclr`; original validation hashes below remain unchanged.

Run `python3 raven/run.py --compiler /path/to/rvnc.dll --bundle /path/to/bundle
--output /path/to/output --imported` from this directory (one command line).
The retained bundle is `target/variance-argument-sequence-20261016`.

The provider stores `(T) -> T` in `FunctionBox<T>`. The consumer imports
`FunctionBox<int>.Get()`, infers a local generic `Apply<int>` call and returns 42.
Its source-provider compilation also covers RuntimeAwait and RuntimeGenerator.
Interpreter, AOT with UBSan/bounds, and managed JIT pass; hashes and commands are
in `results.json`. No new runtime ABI is introduced.

An imported assembly-level generic Apply with a function parameter remains outside
the host SDK's authored function-reference profile. This fixture qualifies the
imported generic class/getter and local generic helper, not that separate gap.
