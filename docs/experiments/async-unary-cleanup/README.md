# Unary await and resource capture regression

The HTTPS project exposed a shared Raven compiler gap: `use` followed by
`if !(await task)` failed native definite-assignment verification. The tree walker
skipped unary operands, so capture analysis missed suspension and left the resource
in a local. Resume could then read an uninitialized local; suspension could dispose
the resource too early. The verifier was correctly rejecting invalid output.

Raven main `99ae011b4` traverses the operand. Native branch
`codex/source-object-metadata-resolution` carries it as `c71baae94`, without merging
branches. Four .NET behavior regressions cover true/false and completed/suspended
Tasks, asserting the branch result, no early disposal and exactly-once disposal.
They all failed before the fix; all 18 resource-lifetime tests now pass.

[Recorded evidence](validation.json) also identifies 15 broader async test failures
that reproduce with the unchanged walker (132/147 pass with the fix). These remain
separate main-branch top-level async/closure/entrypoint gaps; this slice does not
claim the entire async suite passes.

The native fixture compiles through its project with the fixed compiler. The
original [HTTPS consumer](../https-server/Server.rvn), including its awaited unary
condition, additionally passes cancellation/resumption and recovery in the
[project host validation](../https-server/project-validation.json).

```sh
NeoClrBundleRoot=/path/to/qualified/bundle \
  dotnet /path/to/fixed/rvnc.dll neoclr \
  --project docs/experiments/async-unary-cleanup/Native.rvnproj
```

This is a general compiler correction matching .NET resource lifetime semantics.
No Runtime Contract configuration, metadata format, public API, native ABI or CLI
bridge encoding changes. Runtime async remains independently selected. Native
execution evidence is macOS ARM64, not Windows qualification.
