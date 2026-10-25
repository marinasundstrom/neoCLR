# Bounded stack-argument audit (2026-10-25)

The post-inlining profiles found 81–85% of scalar stores outside the managed
arena. That is not proof of stack ownership. Native result/output parameters can
alias existing heap objects; their barriers must remain correct. This slice adds
a read-only, test-only compiler diagnostic, not guard elimination.

## Method and boundaries

After the existing live call-closure check, `NEOCLR_LIVE_STACK_ARGUMENT_AUDIT`
reports direct callers of each local function. A parameter is a candidate only
when every supplied direct caller passes a compiler stack address with a known
remaining extent. The minimum extent across callers bounds stores. Existing SSA
alias, unique incoming edge and checked constant-offset analysis is reused.
Nonlocal functions, IR address-taken functions and functions with no callers are
excluded. Destination analysis stops at loads, ambiguous joins and unknown
forwarded parameters; there is no interprocedural fixed point.

Count only existing scalar-store hook calls with a direct parameter destination,
nonnegative constant offset, fixed width 1/2/4/8 and an end within that minimum
extent. These are static potential sites, not executed-call counts or safe-to-remove
admission decisions. The report assumes the supplied checked closure; it is not a
general whole-module escape analysis. Any future transformation must establish
complete callers, function-address/data-relocation escapes and storage lifetime,
including asynchronous/native boundaries, before using these observations.

The comparison with CLR remains the existing distinction between compiler-known
storage and managed heap barriers. This diagnostic changes no .NET API semantics,
Runtime Contract configuration, temporary CLI encoding, emission, root publication,
heap ownership or collector policy. It adds no Raven compiler integration change.

## Results

| Retained live selection | Functions | Guarded static stores | Candidate sites | Functions with candidates |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,137 | 82,428 | 13,807 | 559 |
| JSON | 1,571 | 112,134 | 20,543 | 711 |

Both complete HTTP compilations pass, and both resulting objects are byte-identical
to `optimizer-http-build` retained live objects. Two focused tests pass, covering
minimum caller bounds, too-small extents, negative offsets, unknown callers,
exported/address-taken/unreferenced functions and unchanged IR. Three earlier
pre-lowering plaintext IR dumps also produce byte-identical objects.

The previously profiled header accessor (`neoclr_value_458` / JSON `680`) has
172 direct call sites, but no parameter passes the all-callers stack test. Keep
its checked path. The Result constructors (`855` / `1153`) have bounded receiver
and output arguments, but their receiver stores use reloaded pointers; none are
classified as candidates. Proving an immutable spill would require separate
alias/write analysis. ViewLength (`839` / `1061`) has one candidate output store
out of three guarded sites in each selection. These examples explain why counting
outside-arena calls alone would overstate an immediately usable optimization.

No runtime performance benchmark was repeated for byte-identical code. Throughput,
CPU and managed allocations are unchanged by this diagnostic; the prior ordinary
sample and HTTP comparison evidence still applies. Synchronous GC remains faster
and the default. The next bounded implementation is to qualify guard elimination
for proven stack destinations, then measure all three factors with the retained
tasks/collections/arrays and plaintext/JSON controls. Loaded receiver proofs and
unknown forwarding remain separate work.

## Diagnostic overhead and evidence

The first whole-closure audit was intentionally cancelled after a host profile
showed it rebuilding the same incoming-edge map for each store. Reuse that map
once per function. The cancelled run, source and sample are retained separately;
it is a diagnostic implementation problem, not a GC execution failure.

`stack-argument-results.json.gz` retains commands, output, input/object hashes and
byte comparisons. `stack-argument-{plain,json}-audit.json.gz` contain all function
rows. `stack-argument-tests.log` contains focused test results. The three
`stack-argument-neoclr_value_*.clif.gz` dumps and `stack-argument-ir-results.json.gz`
retain source attribution. Identical object binaries are not duplicated in Git.
`stack-argument-cancelled-*` preserves the interrupted diagnostic evidence.

Reproduce with a signed test runner containing the diagnostic:

```sh
python3 benchmarks/execution/concurrent-workloads/audit-stack-arguments.py \
  --build target/concurrent-workloads/optimizer-http-build \
  --runner target/concurrent-workloads/test-runner-stack-audit-reuse \
  --output target/concurrent-workloads/stack-argument-audit-repeat
```

These are internal retained-artifact recipes, not standalone distribution tools.
