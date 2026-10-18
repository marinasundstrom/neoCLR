# Direct incoming request construction — 2026-10-18

## Fresh baseline

Rebuilt all four managed libraries from current source at 4014257ba, using
the qualified Raven compiler 478e0dd85 (compiler-primitive-4), current release
backend and current native sources. Recompiled Server.rvn and rebuilt Go.
This replaces the older managed guest identified in the concurrent-GC review.
Build manifests include source/tool/artifact hashes and commands.

The baseline uses paced synchronous GC, the existing 32 MiB native heap and
unchanged Serve callback fixture. The initial 10-second 300/sec smoke completes
all 3,000 measured requests; it verifies the rebuilt fixture, not sustained capacity.
Matched baseline/candidate/Go load measurements follow in a separate slice.

Rebuild with scripts/build-native-class-library.py and the shared
../task-completion-20261018/build.py, using the retained final-load/native link
recipe. Evidence contains both library build provenance and guest build commands.
No public HTTP API or GC policy changes in this baseline slice.
