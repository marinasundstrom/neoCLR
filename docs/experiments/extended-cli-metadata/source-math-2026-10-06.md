# Native source Math gate — 2026-10-06

The unchanged System.Math functions and InvalidRangeError now compile into Math.dll
against the released source-built Numbers library. Two internal native adapters expose
all 15 existing floating math services. A separate consumer imports only the emitted
artifacts, verifies and runs with exit 0 and exact output `Native source math passed`.
[Commands, source/dependency hashes and full-library audit](source-math-2026-10-06.json).

All 20 public functions are exercised: integer Abs/Min/Max/Sign/Clamp and the 15 Double
operations. Assertions include both ties-to-even cases, floating domain NaN and Min
propagation, integer Abs overflow and invalid Clamp bounds. These are observable
execution assertions, not emitted instruction snapshots. Existing runtime implementations
are reused without changes; transcendental accuracy and performance are not new claims.

## Boundaries and compatibility

The [Math contracts and .NET comparison](../../math.md) remain unchanged. Public function
signatures and their XML documentation are untouched. Native module functions and
InternalCall declarations use the existing writer, loader and runtime linkage. Raven
symbols supply emission facts; no importer objects cross into code generation. No
compiler fix, metadata format revision, public API addition or .NET behavior change
is part of this slice.

The retained primitive bootstrap contains a System.Math type. Raven's wildcard import
selects that type ahead of the same-named native namespace, so the consumer explicitly
uses `alias NativeMath = System.Math` and `NativeMath.Sqrt(...)`. This selects the real
native namespace functions; it does not translate the library to CLI metadata or replace
its implementation. Keep this bootstrap collision visible until native core ownership
removes the competing type. This gate does not claim ordinary wildcard imports work in
that mixed environment or change existing .NET import precedence.

Before the adapters, the isolated library reported 15 missing RuntimeServices members.
Afterward, all four driver/runtime commands pass. The full owned-handle diagnostic
build includes 180 files and drops from 74 to 59 errors, with missing service-member
errors reduced from 45 to 30. It still publishes no full System assembly. Remaining
console, environment, GC, pointer and timezone service inputs take priority before
interpreting generic/scope diagnostics as independent compiler defects.

## Reproduce

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_math.py \
  --compiler /path/to/rvnc.dll --library /path/to/Numbers.dll \
  --ownership runtime/raven/native/poc-ownership.json \
  --seed /path/to/System.neox --core /path/to/Core.dll \
  --runtime target/release/neoclr --output /tmp/source-math-gate
```

Use the matching Preview 12 bootstrap and Numbers inputs recorded in the evidence.
The script creates a fresh directory and records failing commands before stopping.
The consumer command contains no library sources. This is development support, not a
modification of published Preview 12 bundles.
