# Public JSON native qualification — 2026-10-19

The complete Public.rvn consumer passes interpreter, JIT and AOT on macOS ARM64
with Raven native compiler 2d22055dc and five coherently rebuilt Raven libraries.
Build and execution commands, dependency hashes and exact output are retained.
This is development evidence, not extracted release-package qualification.

Native lowering now admits nested array references through specialization,
selection, traced element storage and reflective array creation/access. Existing
reserved storage preserves initialization checks; ordinary creation initializes
inner references to null. Four variants verify survival across forced/paced GC,
null defaults, uninitialized reads and bounds faults. Five reflection tests pass,
including nested-array identity and supported access in the presence of unsupported
Int64-array metadata. Unsupported reflective Get/Create shapes still fault at use;
retaining their metadata no longer rejects unrelated supported operations.

Verified direct value ToString overrides may have the recognized System.ValueType
foundation. This permits JsonError diagnostics without broadening arbitrary base
inheritance or reference dispatch. Public JSON exercises these display paths.
These changes restore existing .NET-like array/reference and value-member behavior;
they do not introduce array covariance or a new reflection capability contract.
Nested arrays reuse traced snapshot storage rather than a new native ABI, at the
cost of retaining that bounded backend representation.

Together with the synchronous-host borrowed-view fix and nullable payload compiler
fix, this closes the full-fixture failures recorded in the earlier expression
qualification. No throughput improvement, universal array element support or
release completion is claimed.
