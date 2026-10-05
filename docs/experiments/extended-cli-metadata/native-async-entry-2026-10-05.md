# Native async entry completion — 2026-10-05

An internal Int32 startup function now adapts a selected native Task<unit> or Task<int>
entry. It forwards the source parameters, invokes the source function, calls the existing
explicitly bound DrainEntryTasks service, then invokes GetResult. Unit returns process
status zero; Int32 is preserved. The original method keeps its Task signature.

The existing runtime dispatcher retains the startup frame/task while queued or registered
host work finishes. Cancellation and an unresolved task with no work fault when GetResult
observes completion. This is entry adaptation, not a new scheduling mechanism, polling
await implementation, runtime suspension or green threads. It follows the previous bridge
entry adapter's drain-then-result behavior using native symbols and metadata references.
The .NET Reflection.Emit entry bridge is unchanged. Task<Result<...>> adaptation remains
explicitly unsupported; no result/error mapping is invented.

## Evidence and reproduction

Both unchanged samples pass ordinary native driver/runtime execution:

- `library-async`: exactly `Suspended\n42\n`, exit zero.
- `library-async-default-queue`: exactly `Hello on a worker\n`, exit zero.

Focused entry regressions verify String[] forwarding and a pending Task<int> result
becoming exit 23; cancelled/pending tasks exit 1 with their documented faults. The C#
probe inspects the separate Int32 startup signature and proves missing-runtime and
unsupported-result emission preserves the destination stream. All 60 focused .NET entry,
entry-diagnostic and async-method tests pass. No runtime or public metadata APIs changed.

[Full inventory](native-async-entry-2026-10-05.json) records source/compiler/runtime and
dependency hashes, ordinary commands, exact stdout, exit codes and expected fault checks.
The recorded revision is the parent of this slice; artifact hashes identify tested code.
Reproduce with the same explicit bootstrap/catalog inputs:

```sh
python3 scripts/check-native-poc-samples.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCoreParams.dll \
  --seed /path/to/System.neox --ownership /path/to/ownership.json \
  --reference /path/to/Numbers.dll --reference /path/to/Http.dll \
  --async-library Numbers --runtime target/release/neoclr \
  --output /tmp/fresh-native-entry-inventory
```

The inventory exits normally when cases are blocked; inspect each `compiled` and
`execution.passed`. All eleven non-network execution records pass, including the two
expected-fault controls. Thirteen of fifteen cases compile: eight of ten original samples
plus five focused regressions. The HTTP pair is compiled here and retains the separate
[localhost execution evidence](native-http-execution-2026-10-05.md).

## Next bounded work

The two original sample failures remain unchanged and publish nothing:

- `application-types`: Point constructor construction receiver cannot escape or be used
  for ordinary access. Investigate generated field/property initialization and fix its
  owning layer without weakening constructor validation.
- `application-inheritance`: native source type declaration capability rejects its shape.
  Establish the minimal required inheritance contract, then test runtime dispatch.

Also retain the independently recorded ordinary .NET field-return candidate for main
reproduction and isolated fix integration. Current changes are native entry adaptation;
there is no independently validated .NET fix to backport from this slice. Editor/LSP
acceptance and the remaining sample gate precede a release claim. No guest API snapshot
refresh or unrelated website build was needed.

Implemented compiler revision: Raven `259542217`. Runtime behavior is reused from the
recorded binary; no runtime source changes were required.
