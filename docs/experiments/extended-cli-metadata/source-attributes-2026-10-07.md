# Source attribute hierarchy — 2026-10-07

Raven e492490dc (tested snapshot built from 622041224 plus this change) and neoCLR
67b09d90 plus this slice preserve source Attribute -> Object and UnionAttribute ->
Attribute. Both production attribute declarations are compiled unchanged by rvnc.
The C# introspection check resolves the union custom attribute to that same canonical
marker; a separate metadata-API consumer calls the constructor/inherited hash test
and exits 42. Invalid private marker constructors reject before publication. A
source-root union without a source marker retains embedded-marker behavior and runs.

This follows CLI nominal attribute inheritance and CustomAttribute constructor
references; no format extension or new attribute discovery service is introduced.
The ordinary .NET backend is unchanged (101 root, constructor and union tests pass).
The API reference now has the matching abstract/protected declaration and summaries.

Reproduce with `verify_generic_object_root.py --source-attributes`, supplying the
compiler, core, seed, runtime, compiler revision and a fresh output directory as in
[the recorded commands and hashes](source-attributes-2026-10-07.json). The fixture
assembles an explicit minimal String.Concat bootstrap using the real runtime internal
call; source Object remains the sole root. It does not import the released library's
competing Object graph. The consumer is API-authored: this does not claim ordinary
Raven imported-root support or full-System execution. No source API was rewritten to
hide an unsupported feature. Generated union display methods are encoded but not the
behavior being asserted by this constructor/identity fixture.

[The full audit](native-bootstrap-source-attributes-2026-10-07.json) includes 195
inputs, has zero binding errors and stops at NativeMemory.Alloc's pointer-to-source-Void
signature. No full-System artifact is published. Next resolve the unit/pointer
signature boundary, retaining CLI PTR VOID where applicable.
