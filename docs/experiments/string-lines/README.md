# String.ReplaceLineEndings — 2026-10-19

The Raven API uses the exact private StringReplaceLineEndings(String, String)
service. A two-pass interpreter/native scan sizes the result, reuses unchanged
owners and writes retained bytes plus replacements directly into the final text.
No intermediate byte arrays or strings per match are created. Rust's host-owned
String/Arc allocations are distinct from native managed GC counters.

Match CRLF as one sequence, then CR, LF, FF, NEL, LS and PS. VT, NUL and other
bytes are retained. Empty replacement deletes newlines. Replacement text is never
rescanned, even if it contains newlines. The default is LF on every OS, matching
neoCLR AppendLine; this deliberately differs from .NET's environment default.
This general Unicode operation is not HTTP framing or protocol validation.

Main.rvn exercises every recognized sequence, adjacent CR/CRLF/LF, deletion,
replacement containing newlines, decomposed accents, emoji, embedded NUL, empty
text and no-change owner identity. kernel.c additionally checks bounded-arena
exhaustion, output preservation on failure, integer overflow and null input.
The native registry test rejects wrong argument counts and result types.

```sh
python3 docs/experiments/string-lines/check.py \
  target/string-lines-bundle-v1/bundle target/string-lines-public-v1 \
  --output target/string-lines-validation.json
python3 docs/experiments/string-lines/measure.py \
  target/string-lines-allocation-v1 --output target/string-lines-allocation.json
clang -std=c11 -Wall -Wextra -Werror -I runtime/native/memory \
  docs/experiments/string-lines/kernel.c runtime/native/memory/text-arena.c \
  -o target/string-lines-kernel
target/string-lines-kernel
```

Use build-native-class-library.py, prepare-jit-bundle.py and
build-native-project.py's console profile for fresh artifacts. Qualification uses
macOS ARM64 with the pinned Xcode SDK, Raven code 2d22055dc and metadata e4709c451,
plus the runtime/backend/library working tree atop dc3dae6d1. Native GC uses eager
boundaries. New libraries require the matching new runtime and backend; no compiler
Runtime Contract or ordinary .NET behavior changes. The legacy documentation
reference is scaffolding, not the runtime implementation.

The [design review](../../design/string-api-review-20261019.md#replacelineendings-implementation-2026-10-19)
compares managed staging, the privileged runtime service and a separate FFI provider.
String-as-view/value representation remains deferred. Release-package qualification
and website publication are separate from these development checks.


## Results

All three public executions pass; see [validation.json](validation.json). The native
kernel boundary/fault check and exact registry-signature test pass. API fingerprints,
four native snapshot tests and the 4,558-page local website check pass.

The [128-iteration native allocation diagnostic](allocations.json) reports:

| Mode | Allocations | Reclaimed | Collections |
| --- | ---: | ---: | ---: |
| Baseline | 2 | 2 | 1286 |
| Unchanged | 2 | 2 | 1286 |
| Changed | 130 | 130 | 1798 |

Unchanged replacement adds no managed allocations; changed replacement adds one
per result. All tracked allocations are reclaimed. Whole-process eager-GC counts
are not a throughput measurement, and they do not count host Rust allocations.
