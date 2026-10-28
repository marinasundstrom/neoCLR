# Comparable owner-affine I/O handoff screen

This experiment compares a common application contract across neoCLR's native
runtime (both collectors), .NET 10 and Go. One owner acknowledges real TCP input
while independent CPU work runs inline or is explicitly offloaded. It is not a
runtime ranking, generated neoCLR application, HTTP capacity or UI benchmark.

From the repository root on macOS with Xcode, Go and .NET SDK 10.0.100 installed:

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/runtime-handoff/compare.py --output target/runtime-handoff
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/runtime-handoff/compare.py --output target/runtime-handoff-tiny --iterations 3200
python3 docs/experiments/runtime-handoff/profile.py --report target/runtime-handoff/report.json --output target/runtime-handoff-profiles
```

Use fresh output directories. The harness copies the .NET project into the output
directory and pins SDK 10.0.100 there; it does not alter repository SDK selection.
Each server publishes a loopback port, uses a bounded readiness/byte/ack protocol,
and prints final CPU/wall/checksum JSON. The common external peer keeps timestamps
in one clock domain. Native optimized/UBSan controls precede rotated timing runs;
profile processes run separately and their timings are excluded.

Results and architectural constraints are in the
[qualification record](../../validation/runtime-handoff-20261028/README.md).
No general managed-worker API, cross-thread async carrier or production tuning
preset is introduced. The next question is nonblocking bounded completion delivery
under several independent requests, preserving affinity and roots.
