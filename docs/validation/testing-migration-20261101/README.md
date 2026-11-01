# Post-release testing migration — 2026-11-01

## Bounded scope

The author deferred applicable Python-driven behavior-fixture migration until after
Preview 19. The first slice is
[RangeIterationTests](../../../runtime/System.Runtime/test/RangeIterationTests/README.md).
It preserves the [existing compiler contract and .NET comparison](../range-iteration/README.md)
without changing runtime/compiler code, APIs, release assets or execution defaults.

| Original checks | Discovered cases |
| --- | --- |
| Eight signed-limit loops, including overshoot and minimum steps | Four cases per Int32/Int64 width, with bounded iteration guards and count assertions |
| Inclusive bounds and step side effects | Once-only source-order and iteration-count assertions |
| Break at signed maximum | Immediate break and one-iteration assertion |
| Exclusive maximum, empty and reversed bounds | One case per width, preserving yielded-value checks and rejecting any empty-range body execution |
| Exclusive bound side effects | Evaluation-order and bounded-count assertions |
| Continue/break interaction | Exactly one counted body between continue and break |

The standard discovered-test registry and runner replace a handwritten Main and
numeric failure exits. Python retains bundle hash verification before/after the run
and orchestration. The legacy command remains supported; `results.json` now copies
the common runner's `report.json` schema. Existing historical evidence is unchanged.

## Remaining applicability review

- HTTP and socket memory consumers contain behavior suitable for framework tests,
  but asynchronous lifecycle/cancellation observation and native host profiles need
  deliberate preservation. The inspected HTTP wrapper has a stale project name
  and old stream signatures; migrate and validate as a separate bounded task.
- HTTP custom-method wire peers, compiler visibility rejection and terminal-failure
  subprocess probes retain legitimate Python host orchestration. Do not mechanically
  convert these into in-process assertions.
- Published sample output checks and package/file-integrity checks remain distinct
  from framework behavior tests. This slice does not retire them wholesale.

## Validation

[The common runner report](three-mode-report.json.gz) and [transcript](three-mode.log.gz)
pass all 14 tests in interpreter, managed JIT and native AOT: 42 behavior-test
executions on macOS ARM64. Discovery, registration verification, compilation and
bundle-integrity checks pass. Inputs are fingerprinted in the report. The matching
development bundle is `target/http-message-surfaces/inputs9/bundle`; this verifies
the migrated tests against that qualified compiler/library/runtime composition,
not a new release or a claim of freshly rebuilt unified Raven main.

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  python3 scripts/validate-native-range-iteration.py \
  --bundle target/http-message-surfaces/inputs9/bundle \
  --output target/range-testing-migration-20261101
```

No website build is needed for this test-only migration. Existing .NET/compiler
qualification and historical negative probes are retained; no compiler contract
or platform-support claim changes.

A [fresh-output negative control](fresh-output-control.json) rejects a reused
validation directory and proves every existing evidence file remains unchanged.
That early wrapper guard was added after the three-mode run; it changes only the
precondition, while the recorded runner and Raven test inputs are unchanged.
