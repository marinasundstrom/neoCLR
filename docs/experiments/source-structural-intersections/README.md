# Raven source intersections

Development experiment, 2026-10-14, on the paired structural composition branches.
`Main.rvn` compiles directly to native intersection signatures. It exercises
Animal & Quacks parameters, returns and locals, guaranteed interface dispatch,
IntersectionTypeInfo constituent snapshots, IsInstanceOfType and
Box<Animal & Quacks> generic argument identity. The interpreter exits 0.

Run `python3 docs/experiments/source-structural-intersections/verify.py --compiler
/path/to/rvnc.dll --library /path/to/native-library --output target/intersections`
with the experimental-compositions interpreter. Project configuration explicitly
sets RavenNeoClrStructuralCompositions. No signature/token substitution is used.

Raven's `NeoClrMetadataProbe --source-compositions <library> <output>` additionally
compiles public source union/intersection libraries, reads their native signatures,
rejects default catalog import, then compiles separate consumers through an opted-in
catalog. Both consumers return 42 when linked to their producer and System.Runtime.
Results retain artifact hashes. The 86 focused semantic tests include open/abstract/
sealed-hierarchy bounds, closed/value rejection and default .NET gates.

Only interfaces and classes open to extension are intersection bounds; Raven sealed
hierarchies remain allowed. Closed nominal generic support uses the existing native
profile. Open generic execution, generic method specialization, mixed nested bounds,
composition member enumeration and JIT/AOT remain limitations, not permanent rules.
