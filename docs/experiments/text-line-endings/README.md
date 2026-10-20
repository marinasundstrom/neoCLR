# Exact configurable text line endings — 2026-10-20

The async fixture passes interpreter, JIT and AOT in [results.json](results.json).
The matching [sync fixture](../sync-text/Main.rvn) checks the same delimiter and
writer behaviors with direct results. Both use NewLine through text interfaces.

Coverage: LF, CRLF, CR, ASCII and Unicode delimiters across single-byte input;
overlapping prefixes; ordinal UTF-8 matching despite canonical String equality;
empty-reader rejection without consuming input; incomplete delimiter prefixes at
EOF; explicit preservation of CR with default LF; result byte bounds; writable
interface configuration; LF/CRLF/custom/empty output; and configuration snapshot
across a pending read. Console.Write exercises the borrowed console host path.

Compiler: Raven codex/neoclr 100f98a2f. Libraries: target/sync-text-libraries-v12.
Bundle: target/line-text-bundle-v2/bundle, including the 8192-method native load-set
cap (neoCLR 9d1207baf). Applications: target/line-text-lines-app-v2 and
 target/line-text-async-app-v3; sync evidence uses target/line-text-sync-app-v1
with the same library and the prior backend cap. The combined async fixture was
split for focused evidence, then the full supplied library metadata count exposed
the cap: 4046 library methods plus a small async consumer exceeded 4096 before
reachability selection. The type and dependency caps are unchanged.

```sh
python3 docs/experiments/text-line-endings/check.py \
  target/line-text-bundle-v2/bundle target/line-text-lines-app-v2 \
  --output target/line-text-lines-app-v2/results.json
```

The current policy is exact delimiter matching, default LF. This replaces implicit
CR stripping before LF: configure CRLF explicitly. A constants class and an enum
are unnecessary because configuration takes the actual string sequence.

The matcher caches a prefix table per reader configuration. It scans input in
linear time and holds possible delimiter prefixes outside the output buffer;
this is an algorithmic property, not a measured throughput claim. Line storage,
returned Strings, conversion sessions and async task machinery still allocate.
The synchronous path does not construct Tasks or wrap blocking console transfers
in Promises. Performance benchmarking is separate work.
