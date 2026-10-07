# Standalone native bootstrap preparation — 2026-10-07

prepare-native-bootstrap.py generates Core.dll, a compile-time System.neox and the
retained System JSON model from checked-in sources. It needs a built runtime,
metadata translator and the existing narrow CoreDeclarations generator, but no old
Numbers/Http assembly or previous Runtime output. See the exact commands in
[runtime projects](../../../runtime/raven/projects/README.md).

Core remains the permitted CLI bootstrap, generated with source-owned Fail selection.
The seed source removes the source-owned Void, Value and RuntimeTypeHandle declarations.
The assembler temporarily supplies Object to resolve signatures; the retained model
removes that root and its methods, then clears dense definition IDs before encoding.
This is the same ownership procedure used by the earlier audit, now separated from
its legacy inputs and compilation matrix. A runtime seed is finalized only after
building System.Runtime against its actual emitted identity.

The preparation manifest is published last and records output hashes. The class-library
builder validates prepared files and the explicit Core selection before creating its
output. Historical audit directories without this manifest remain accepted explicitly.
Generator commands, source/tool hashes and the retained source model remain available
for inspection. Native consumer/library references still use native metadata.

This closes the undocumented temporary-input dependency in the build instructions,
not full clean-checkout qualification. The Core generator currently lives in the old
probe project and links Raven; extracting that narrow generator is future tooling work.
No metadata encoding, runtime instruction or compiler semantics change is made.
.NET is still the host toolchain; this is not compiler self-hosting on NeoCLR.


## Validation

The generator probe builds from current sources at Raven ce51cd941. Preparation
succeeds; all four projects build using the fresh Core/seed with the previously
qualified SDK. The archive is extracted into a fresh path with spaces. Collections,
Tasks/await and JSON mapping compile/run with exact output; HTTP server/client builds
and both HTTP request checks pass. All nine verifier commands exit 0.
Corrupted retained JSON and a different selected Core both reject before output creation.
[Exact commands and hashes](native-bootstrap-preparation-2026-10-07.json).

The existing packaged-editor evidence is reused: this slice changes preparation and
validation tooling, not the compiler/server/extension. It does not establish a fresh
editor run with these particular newly generated artifact bytes. Full clean-checkout
reproduction and native RavenDoc input remain subsequent gates.
