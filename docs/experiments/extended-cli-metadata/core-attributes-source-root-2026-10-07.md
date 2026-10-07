# Core attributes with a source Object root — 2026-10-07

Raven **622041224** fixes two native admission checks: FlagsAttribute and
MethodImpl(InternalCall) now resolve ownership against the exact host-provided core
assembly identity, using the existing NeoClrBindingContract comparison. Previously,
they compared attribute ownership to System.Object's owner, which changes when the
source library supplies Object. The initial source-root regression failed with
NEOMETA001 for the otherwise valid bootstrap FlagsAttribute.

No attribute meaning, metadata encoding, Runtime Contract option or public API changed.
Namespace/name, constructor arguments and supported declarations remain checked.
Same-named source attributes cannot impersonate the selected bootstrap declarations.
The .NET emitter is untouched; no general Raven fix needs backporting from this slice.

## Executed validation

`verify_generic_object_root.py --core-attributes` compiles the unchanged production
BindingFlags declaration together with the existing source Object fixture and a small
runtime-service caller. The caller checks present/absent bits and invokes WriteLine
through MethodImpl(InternalCall). Its separate API-authored consumer prints exactly
`source root attributes` followed by a newline and exits **42**. Both source-defined
FlagsAttribute and MethodImplAttribute lookalikes fail the exact owner check with no
output assembly. [Commands, sources and hashes](core-attributes-source-root-2026-10-07.json).

Run that script with `--compiler`, `--compiler-revision`, `--core`, `--seed`, `--runtime`
and a fresh `--output` directory, as for the earlier generic-root gate. The consumer
remains API-authored: ordinary Raven imported-root consumers are still unsupported.

The ordinary-bootstrap `verify_source_native_memory.py` gate was rerun against the
same compiler: both Alloc overloads and Free execute with exit **42**, double-free and
checked multiplication overflow fault, and unsupported pointers reject before output.
[Control evidence](native-memory-core-attributes-2026-10-07.json).

The native compiler build and API snapshot check pass. The metadata library and ordinary
.NET backend are unchanged; their previous focused evidence is reused rather than
claiming a new broad suite run. No website build was needed for this internal fix.

## Next full-System boundary

The 194-input full-owned-handle audit clears binding, generic source-root admission and
core attribute validation. It next rejects `System.Runtime.CompilerServices.UnionAttribute`,
whose base is the bootstrap's external `System.Attribute`. The current class emission
contract admits the local source-root relationships, not this external base.
[Audit evidence](native-bootstrap-core-attributes-2026-10-07.json).

Next establish coherent Attribute ownership/inheritance for source System and metadata
emission, with a minimal constructor/marker-class regression before retrying the entire
library. Do not silently erase the base or omit the source declaration to pass the audit.
Full System still emits no artifact; later blockers remain unknown.
