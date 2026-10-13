# HTTP allocation attribution

Development diagnostics, 2026-10-14. This tool attributes successful native managed
allocations to published logical root frames. It does not sample machine stacks for
attribution or alter guest code, collector policy, roots or scheduling. It is a
provisional profiling aid, not a public runtime ABI.

```sh
python3 benchmarks/http-server/profile.py --build BUILD_JSON \
  --workload plain --allocation-sites --output NEW_DIRECTORY
```

Use `--workload json` for the JSON fixture. The controller relinks the guest object
from the passing native build, records input hashes, validates 32 responses and
measures the 24 requests between connection-close boundaries 8 and 32. Separate
processes avoid retained counter state. The existing native stack sampler remains
independent of allocation attribution. This entry point currently targets macOS.

Exclusive rows identify the innermost published managed frame; their allocation,
payload-byte and text totals must reconcile exactly with the request counters.
Inclusive rows attribute an allocation to each function on that logical stack,
counting recursive appearances only once. Inclusive rows overlap and must not be
summed. Async causal ancestry absent from the active stack is not reconstructed.
Native helpers without their own managed frame are charged to their active caller.
This tool expects one compiled guest image; nested images need image-qualified IDs.

Function identities come from the matching build's compiled-function table and
separate generated boxing-helper records. Unknown identities remain explicitly
unmapped. Names are display labels, not public symbol identities; numeric compiled
indices and original names remain in the report. The first JSON attempt rejected
extra generated helper indices; the final tool includes their explicit helper records
and retains an unmapped fallback rather than inventing a member name.

The counters use fixed native storage and walk active frames on each allocation.
Their timing and extra native memory are diagnostic overhead, not benchmark results.
`bytes` means requested managed payload bytes, excluding allocator descriptors,
native resources and retained memory. `textBlocks` is the native TEXT allocation
kind, not the number of managed String objects.

## Findings in the retained fixtures

These qualified AOT guests use the earlier state-machine library fixture with the
current collector. They isolate allocation attribution; they are not new SDK,
runtime-await, concurrency or release measurements. Raw profiles retain hashes,
commands, whole-process GC counters and per-function data.

| Per measured request | Plain text | JSON |
| --- | ---: | ---: |
| Managed allocations | 510 | 1,198 |
| Requested payload bytes | 20,681 | 36,990 |
| TEXT blocks | 137 | 504 |
| Allocations under `Uri.At` | 124 | 124 |
| Allocations under request-decoder `Text` | 26 | 26 |

Both 32-response processes pass with zero GC failures and complete reclamation:
16,340 allocations plain and 38,358 JSON, including startup and shutdown. The two
last rows are inclusive subsets of the total, not additional allocations. URI
source inspection confirms `At` constructs a one-byte UTF-8 substring per probe.
This is a concrete candidate for byte-based parsing with bounded scratch storage;
no URI implementation change or resulting speedup is claimed here. Header parsing
already scans bytes but eagerly decodes names/values and lowercases via a temporary
list. Known-name constants and byte matching are the subsequent HTTP candidate.
See [ownership and external-runtime comparison](../../docs/http-foundation.md#byte-ownership-and-lazy-materialization-direction--2026-10-14).

Focused tests exercise ignored warmup/teardown allocations, exclusive/inclusive
accounting, recursion, unframed allocation, generated/unmapped identities and
reconciliation failure. Run:

```sh
python3 -m unittest discover -s benchmarks/http-server -p test_allocation_sites.py
```

The subsequent [URI byte-parsing refinement](uri-byte-parsing.md) rebuilds matched
libraries, removes the parser temporaries and records separate timing/correctness
evidence. The original attribution profiles above remain unchanged.
