# Owner-local real TCP completion/resume

Run from the repository root on macOS with an Xcode SDK:

```sh
SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/io-resume/validate.py --output target/io-resume-check
python3 docs/experiments/io-resume/profile.py --binaries target/io-resume-check --output target/io-resume-profiles
```

Use new output directories. The harness compiles both collector configurations,
runs optimized/UBSan checks, captures separate async traces and executes interleaved
untraced timing pairs. The trace-only binary writes `trace.json` in its working
directory; the harness runs it in the requested output directory and renames it.
The profiler uses macOS `sample`; those timings never enter the comparison table.

Four shapes: idle receive, sixteen short CPU callbacks, those callbacks with
allocation churn, and one long CPU callback with the same total loop count. The
peer sends over real loopback TCP; scheduler callbacks consume the transfer and
complete the operation awaited by a rooted parked runtime context. Native callbacks
isolate the common runtime layer; no generated JIT/AOT/HTTP/UI claim follows.

[Measurements, definitions, limitations and retained evidence](../../validation/io-resume-20261028/README.md).
The design follows existing [.NET/Go scheduling research](../../design/go-inspired-scheduling.md):
continuation readiness, dispatch and execution are distinct, and owner affinity
must remain explicit. Permission to redesign internals does not establish safe
stack migration, preemption or an arbitrary parallel-mutator contract.
