# Store diagnostic counter cost: no demonstrated improvement

2026-10-25. The [loaded CPU profiles](cpu-results.md) implicate the scalar-store
path. Before changing ownership admission, remove only diagnostic increments in a
separate generated collector and compare against the identical retained live object.
This tests a concrete potential cost without weakening validation or mutation logging.

`counter-control.py` removes exactly thirteen store/lookup counter increments from
the generated source and relinks the existing plaintext host. It preserves counter
storage and exports, which report zeros, plus all candidate/collection/allocation
counters. The count assertion catches source drift. The script currently builds the
live comparison; the guarded control is not part of this screen. No checked-in
collector source or default changes. The counters are read only for reporting;
this experiment does not remove write barriers, lifetime state or gate decisions.

Three alternating on/off pairs validate **80,855 plaintext responses** through EOF:

| Diagnostic increments | Median requests/s | Median CPU us/response |
| --- | ---: | ---: |
| Enabled | 2,703.75 | 386.14 |
| Disabled | 2,664.25 | 391.05 |

Disabling increments is 1.5% lower throughput and 1.3% higher CPU in these short
runs. This does not establish a general regression, but provides no reason to
promote the candidate. Managed allocations remain approximately 208/response;
small differences reflect workload lifetime/amortization. Both modes complete live
epochs and release index metadata. Candidate reports have zero store/lookup counts;
control counts are positive, confirming that the intervention was active.

`counter-standalone.py` then rebuilds matched live hosts for existing tasks,
collections and arrays, with the same thirteen-increment removal. Five alternating
pairs per workload pass expected output, positive completed epochs, balanced
candidate shutdown, zero retained index metadata and equal managed allocations/bytes.
These are 30 standalone AOT runs; their short duration limits timing conclusions.

| Sample | CPU seconds, enabled | Disabled | Managed allocations per run |
| --- | ---: | ---: | ---: |
| Tasks | 0.024069 | 0.023873 | 65,548 |
| Collections | 0.132364 | 0.133178 | 237,580 |
| Arrays | 0.058203 | 0.057780 | 86,028 |

Effects are mixed and below 1%. Retain counter-enabled behavior and the diagnostic
screen as evidence; do not add a production switch for an unproven gain. No JSON,
JIT, sanitizer or broader pause claim is made for this rejected candidate. Those
qualifications would be necessary before accepting a changed store implementation.
The existing runtime safety evidence is unaffected because no runtime implementation
is modified. Website/API claims remain unchanged.

## Next target

The guarded-sync CPU gap is not explained by these counter writes. Return to repeated
store ownership/extent admission and generated stack-store provenance, preserving
foreign-heap rejection and all active-epoch barriers. Avoid repeating earlier rejected
borrowing/header-cache designs without a materially different proof or implementation.
Compare future candidates across plaintext, JSON and standalone samples, with matching
JIT/AOT safety qualification before promotion.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-http-owners/counter-control.py \
  --build target/concurrent-http-owners-plain --output target/counters-off-new
python3 benchmarks/execution/concurrent-http-owners/measure.py \
  --build target/counters-off-new --go-client target/http-close-client \
  --owners 1 --modes live --seconds 4 --rounds 1 --output target/counters-off-new/cost.json
python3 benchmarks/execution/concurrent-http-owners/counter-standalone.py \
  --report target/concurrent-workloads/scalar-frame-ordinary-measure/results.json \
  --output target/counter-standalone-new
```

Use fresh paths; alternate the modified and original HTTP builds. The
[counter evidence](counter-evidence/) retains exact commands, link inputs, responses,
CPU/allocation reports and standalone output hashes. Compilation precedes timing;
HTTP and standalone measurements were run sequentially.
