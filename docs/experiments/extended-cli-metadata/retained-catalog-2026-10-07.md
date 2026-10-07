# Retained bootstrap dependency catalog — 2026-10-07

Raven e769eb4b8 and neoCLR 75240a79 plus this tooling slice. No compiler or runtime
code changes. [Commands, inputs, hashes and results](retained-catalog-2026-10-07.json).

After removing bootstrap Object, retained services still reference the source Object.
The seed's empty reference list correctly failed runtime validation. The audit now
retains an explicit compile-time seed and finalizes a separate runtime seed after
source emission. The translator reads the emitted native artifact's module identity
and revision through metadata APIs and adds that direct reference. No name-hash
reimplementation, implicit dependency discovery, or reference-check bypass is used.

Compared with CLR's explicitly resolved assembly references, this is the same ownership
requirement at the native module boundary. The extra bootstrap stage is necessary because
the retained service signatures and the source-owned root refer to each other before
source compilation has produced an artifact. It is an audit/bootstrap arrangement,
not a proposed permanent packaging layout. Dependencies remain native NEOX/PE at runtime;
JSON is only explicit seed preparation.

## Reproduce and assess

Run the full-owned-handle audit as recorded in the evidence. It emits all 197 inputs
and records both System.neox (compile time) and System.runtime.neox (runtime).
`verify_retained_catalog.py --audit <audit-directory> --runtime <neoclr> --app
<expanded-library control app> --module <expanded-library control library> --output
<new-directory>` loads the full source artifact using the selected object root.

The combined load set passes typed-stack/control-flow verification for **2,433 IL
functions**, maximum stack 7. Control execution returns **42**, with empty stdout/stderr.
The control is API-authored and does not exercise all System bodies. A wrong dependency
revision rejects; missing, malformed and duplicate translator references publish no
output. The existing translator's two-argument path is exercised by the wrong-revision
fixture preparation. Production finalization uses native reader APIs; disassembly is
used only to prepare that negative fixture.

The unchanged orders source, compiled against the emitted library with its sources
absent, now reports `Requested value 'System_Value' was not found.` Raven's native named
type importer attempts an Enum.Parse into SpecialType for the explicitly represented
System.Value. Its semantic classification is the next blocker. No consumer artifact is
published. Full-library bootstrap, broad application execution and separate optional
library packaging remain open. No independent compiler fix was produced to backport.
