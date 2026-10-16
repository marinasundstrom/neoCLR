# Raven imported class constraints — 2026-10-16 development

`Library.rvn` declares reference/value, default-constructor and nominal class
constraints. A separate `Main.rvn` consumer exercises reordered symbolic special
base requirements, subclass arguments for nominal bounds, local and external
concrete constructor arguments, and reference identity. Successful execution
returns 42. `Rejected.rvn` requires nine RAV0320 constraint diagnostics: missing
reference/constructor requirements, wrong reference/value/nominal arguments,
and absent, abstract or private constructors.

```sh
python3 docs/validation/imported-special-constraints/raven/run.py --imported \
  --compiler /Users/robert/Projects/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/raven-all-constraints-final-20261016
```

The matching compiler is built with the repository SDK via
`-p:NeoClrMetadataProject=/Users/robert/Projects/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj`.
[Results](results.json) retain exact compiler, runtime, bundle and source hashes,
diagnostic output and interpreter/AOT/JIT outcomes. AOT uses UBSan/bounds; native
hosts check cleanup. Qualification is macOS ARM64 development on Raven
`codex/neoclr` commit `9302f2e4c` and neoCLR `main`, not Raven main or a different runtime revision.

These are .NET-style nominal constraint checks. No new syntax, TextMate grammar,
Runtime Contract setting or CLI bridge encoding is introduced; existing semantic
symbols carry the imported flags and bounds. Broader constraint profiles remain
listed in the [parent record](../README.md). Structural types remain parked.
