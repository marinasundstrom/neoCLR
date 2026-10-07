# Managed array backing over source Object — 2026-10-07

Array authoring already accepted source Object as the local base, but the native
reader and runtime verifier rejected every base. Their rules now agree: a selected
array backing class may have no local base or the explicitly host-selected fieldless
Object root. The runtime compares the exact root definition identity and requires
fieldless storage. Ordinary bases remain rejected. No array layout or instruction
changes; no metadata format version change.

This preserves managed T[] storage and nominal Array<T> backing. Like CLR arrays,
assignment aliases storage and mutation is visible through either reference. This
bounded generic descriptor is still a neoCLR contract, not CLR System.Array metadata.
Inline/value arrays remain separate and outside this change.

## Evidence

- 163 C# metadata groups pass. The array test now reads a backing descriptor over
  source Object; an unrelated base rejects. Existing missing identity, incorrect
  storage, foreign/duplicate selection and post-selection mutation checks remain.
- [Runtime gate](array-root-2026-10-07.json): API-authored native PE library and
  separate digest-bound PE consumer. The host explicitly selects the library's
  Object root. Verification passes; allocation, alias assignment, element mutation
  through the alias and reading through the original return 42, with no output.
  Missing backing, invalid storage, duplicate selection and an unselected Object
  base all reject. The fixture uses ILGenerator and the real runtime.
- Runtime builds successfully; cargo fmt --check and API snapshot check pass.
  Arrays feature-page review found no user-facing API or example change required;
  no website build was run.
- [197-input System audit](native-bootstrap-array-root-2026-10-07.json) clears this
  reader check, still publishes no output, and now identifies `System.String ->
  System.Object` as an unsupported base category. Reader errors now identify both
  declarations to make that next failure actionable. Source String has intrinsic
  reference storage and needs its own bounded base treatment; do not enable arbitrary
  primitive inheritance to pass this check.

Runtime/metadata base is cd475f87 plus this slice. Raven compiler code remains
ca4aeccfb-equivalent, using the updated metadata DLL. The audit records actual hashes.
The execution gate is API-authored, not complete source Array<T> or full System
execution. Ordinary .NET codegen and CLI array signatures are unchanged.
