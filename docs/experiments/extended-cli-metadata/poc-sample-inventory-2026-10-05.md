# Native POC sample inventory — 2026-10-05

The author clarified that the release goal is an end-to-end NeoCLR POC, with samples
required to compile. This bounded inventory uses ten unchanged representative samples,
not every repository experiment. It does not require full System source ownership.

Raven `ac702d7c0` and NeoCLR `bb5acdd2` were tested with explicit primitive core, retained
seed, cumulative source-built Numbers.dll and Http.dll. Applications compile through
`rvnc neoclr` using native library references only; no library sources or consumer stubs
are included. Those dependencies reuse the source-built HTTP gate rather than rebuilding
the whole library. Artifact hashes, sources, commands, diagnostics and runtime output
are recorded in [the evidence](poc-sample-inventory-2026-10-05.json).

| Selected sample | Native compile | Runtime evidence |
| --- | --- | --- |
| application-order-collections | Pass | Exit 0 and exact checked-in stdout |
| application-interfaces | Pass | Exit 0, `42` and `99` |
| json-object-mapping | Pass | Exit 0, mapping assertions and expected stdout |
| application-types | Blocked | Point constructor receiver cannot escape/use ordinary access |
| application-inheritance | Blocked | Native declaration capability rejection |
| library-async | Blocked | Native Task is not recognized as an async return/entry type |
| library-async-default-queue | Blocked | Same Task recognition failure |
| library-async-cancellation | Blocked | Same Task recognition failure |
| http-json-server | Blocked | Same Task recognition failure |
| http-json-client | Blocked | Task recognition plus propagation/conversion diagnostics |

No failed compilation published an artifact. The three successful executions compare
exit status, stderr and exact stdout. The broad order sample checks collection mutation,
callbacks, queries and shared identity through its unchanged observable output. This is
an actual native driver/runtime application case with separately emitted dependencies;
it does not establish the complete dual-target library or editor release gate.

## Priorities supported by this evidence

1. **Native async identity and lowering:** five samples share the earliest blocker.
   `NativeNamedTypeSymbol` currently classifies native scalar/grapheme primitives but
   leaves Task and builder declarations without special classifications.
   `AsyncReturnTypeUtilities` recognizes Task by its special identity. The existing
   NeoCLR bridge profile already selects heap state machines, and the production
   Tasks source supplies GetAwaiter, OnCompleted and an AsyncTaskMethodBuilder.
   Connect those facts through explicit native dependency/target contracts, preserving
   the default .NET path. Do not recognize arbitrary lookalikes solely by name.
   Recognition alone is insufficient: test completed and pending awaits, callback
   continuation, entry completion, cancellation, and synthesized-state emission next.
   No runtime suspension or green threads are required by this POC.
2. **Constructor receiver validation:** minimize the application-types failure and
   decide whether the emitter or metadata validator is wrong before changing either.
3. **Inheritance declaration/emission:** identify the exact rejected declaration in
   application-inheritance and admit only the relationships required by that sample.

Reassess the HTTP client propagation diagnostics after Task recognition; some may be
secondary to the invalid async contract. Source-root replacement and generic local bases
remain useful work, but are not prerequisites for samples already using the explicit
retained seed. This follows the author's POC scope, not exhaustive API completion.

## Reproduce

```sh
python3 scripts/check-native-poc-samples.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCoreParams.dll \
  --seed /path/to/System.neox --ownership /path/to/ownership.json \
  --reference /path/to/Numbers.dll --reference /path/to/Http.dll \
  --runtime target/release/neoclr --output /tmp/fresh-poc-inventory
```

Use the exact dependency hashes recorded in the evidence or rebuild the matching
[source HTTP gate](source-http-2026-10-05.md). The primitive bootstrap and existing
library artifacts remain explicit reproduction inputs, not installed SDK discovery.
This command inventories failures and therefore returns normally when samples are
blocked; inspect each `compiled` and `execution.passed` result. It is not a green
acceptance-test command. Every compiler/runtime invocation has a 120-second watchdog.
Runtime execution is limited to the three known non-network controls. No compiler or
runtime semantic change, .NET behavior change or performance claim is made by this slice.
