# Outlining heap-store work: smaller entry, no demonstrated gain

2026-10-25. Follow-up to [CPU attribution](cpu-results.md) and the rejected
[counter-removal screen](counter-results.md). The original live store adapter
remains unchanged. These are internal implementation experiments; no .NET API,
Raven emission, runtime contract or ownership semantics change.

The store adapter performs common context/heap/width/extent admission before
handling out-of-arena writes. Most store calls in the retained plaintext reports
are out-of-arena. The original ARM64 entry reserves 192 bytes and saves six register
pairs, including registers needed only by managed-heap traversal. `split-store.py`
keeps admission and the out-of-arena write in the entry and moves the remaining
body unchanged to a private noinline continuation. No allocation, callback,
safepoint or epoch transition occurs between the two parts. All active-epoch
logging, cancellation, identity/bounds checks and diagnostic increments remain.
The prototype rejects borrowed-store configurations rather than silently changing
that separate ABI.

This reduces the entry stack frame to 16 bytes and one saved pair, but heap stores
now call another function and resolve thread-local state again. A second variant,
`--pass-state`, passes the already-resolved candidate-state pointer directly to the
private helper. It is valid only for this same-thread immediate call; no stored
handle, root-to-heap ownership association or lifetime extension is introduced.
The helper retains separate heap ownership validation machinery. This is distinct
from the earlier activation-wide borrowed-store experiments.

## Tests and performance screen

Both variants pass existing scalar, exact-store and record-store native fixtures
in optimized and UB/bounds builds: twelve fixture executions total. These cover
aligned/unaligned widths, non-marker writes, active-epoch changes, invalid stores,
range/cache lifetime and grouped record mutation. The first test link incorrectly
used the full native-scope adapter without its text provider and failed unresolved
linkage. The corrected tests use the explicit-epoch generator and the same store
transformation. Full HTTP hosts retain the automatic/native-scope collector.
An initial generator assertion also expected a trailing newline beyond its selected
function slice; this was corrected before any successful build.

Each experiment then runs three alternating original/candidate one-owner pairs,
32 client lanes and four-second windows, with the identical live managed object.
All **161,817 responses** across the two paired cohorts pass through EOF, with
positive live epochs, balanced shutdown and zero retained index metadata.

| Variant | Original requests/s | Candidate requests/s | Original CPU us/response | Candidate CPU us/response |
| --- | ---: | ---: | ---: | ---: |
| Outlined heap path | 2,717.5 | 2,664.5 | 385.86 | 393.08 |
| Outlined, direct state argument | 2,707.75 | 2,689.25 | 385.99 | 386.90 |

These are medians, with whole-process CPU including startup/warmup/drain/shutdown.
The plain split costs about 1.9% CPU and loses 2.0% throughput. Passing state is
approximately flat (+0.2% CPU, -0.7% throughput), not a demonstrated improvement.
Managed allocation counts remain approximately 208/response; no reduction is claimed.
A preliminary single state-argument run is retained separately as a screen, not
included in the paired medians. These results do not establish a universal regression.

Do not promote either variant or spend broader JIT/JSON/default qualification on
them without a materially different design. Existing synchronous and guarded-sync
controls remain available; this screen compares candidates to the same live path.
The production implementation, diagnostics and public documentation are unchanged.

## Evidence and next work

[split-evidence](split-evidence/) retains generated candidate sources, build commands
and hashes, ARM64 entry disassembly, native/sanitized results, the failed test link,
paired HTTP reports and the preliminary state screen. Exact native binaries remain
local. Regenerate either source with `split-store.py --source <generated-gc.c>
--output <fresh-path>` and optionally `--pass-state`; relink using the recorded
commands. The state-argument generator reproduces the measured source byte-for-byte.

The smaller assembly prologue did not translate into a whole-request gain.
Prioritize avoiding unnecessary adapter calls through bounded compiler provenance
or reducing admission work under an explicit lifetime proof, rather than more
function outlining. Keep foreign-context rejection and changing-root/active-epoch
correctness. Any promising candidate still needs standalone tasks/collections/arrays,
JSON, JIT/AOT, memory and pause qualification before default consideration.
