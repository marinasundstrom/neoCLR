# Collection element release probe

This interpreter-only consumer keeps an eight-slot ArrayQueue alive, fills it with
eight distinct objects, drains it through a separate method, then forces collection.
HeapObjectCount returns to its pre-fill baseline. The source and assembly hashes
and passing output are in [validation.json](validation.json).

Compile `Native.rvnproj` with the matching bundle's rvnc `neoclr --project` command
and NeoClrBundleRoot environment variable, then run the emitted assembly with that
bundle's modules and the source-built interpreter. Native AOT currently rejects
GCHeapObjectCount; this is not evidence of native GC reclamation. Native functional
checks are in the separate [collections consumer](../native-collections/).
