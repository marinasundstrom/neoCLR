# Native documentation provider boundary — 2026-10-07

**Deferred by the author later on 2026-10-07.** The native provider/model migration
and bridge assembly-label correction are not requirements for the upcoming release.
A possible rewrite of RavenDoc in Raven is exploratory future work. Retain the design
below as context; prioritize release reproduction, working samples and developer usability.

Author direction: stop presenting NeoCLR.CoreProbe.dll as production assembly ownership.
RavenDoc should consume NeoCLR metadata natively, directly or through the compiler API,
and use providers and standardized presentation models. This is a planned implementation
boundary, not a claim that the current website already imports native metadata.

## Inspected implementation

At Raven ce51cd941, Program.GenerateFromAssemblies constructs ordinary CLI references,
adds framework references and scans adjacent DLLs. DocumentationGenerator already accepts
multiple assembly symbols and renders from ISymbol; source/assembly lines use the
symbol's containing assembly and reference path. The displayed CoreProbe identity is
therefore a consequence of the actual input, not just a string-formatting defect.

## Bounded implementation order

1. Introduce an explicit input-provider boundary: CLI preserves its current behavior;
   native input selects a bundle/dependency catalog, the existing primitive bootstrap
   and Runtime Contract. It imports native artifacts through the current Raven adapter.
   Do not scan arbitrary sibling assemblies or insert .NET framework references on
   the native path. Required references must be validated before publishing pages.
2. Reuse the existing compiler-symbol renderer for the first native bundle acceptance.
   Documented roots are the four selected production assemblies; bootstrap dependencies
   resolve signatures but are not automatically documented. Preserve one namespace tree.
3. Introduce an immutable documentation model incrementally between symbol projection
   and rendering: assembly-qualified declaration identity; declaration kind/name and
   generic parameters; typed signatures and parameter modes; accessibility and flags;
   relationships/accessor associations; structured documentation; real declaring assembly
   and artifact identity; optional verified source locations. Keep display signatures
   separate from identity keys. Inherited/extension rows retain original declaration IDs.
4. Migrate the website input to native artifacts only after coverage, route compatibility,
   cross-assembly links and authored documentation association pass. Never fix provenance
   by changing CoreProbe labels or guessing ownership from namespaces.

The initial provider should use existing native introspection → compiler symbols, not
reimplement dependency resolution or create runtime Reflection objects. A later direct
introspection provider can produce the same documentation model if independence from
compiler semantics becomes useful. That introduces a second projection to maintain;
it is not a prerequisite. The CLI provider and existing .NET rendering remain controls.

## Acceptance

- Type and member pages show actual System.Runtime, System.Data, System.Networking or
  System.Web ownership. Assembly identity, artifact filename and bundle title are distinct.
- Cross-assembly parameter/result types link to canonical pages; one combined navigation
  and search model, without duplicate per-assembly namespace trees.
- Inherited/extension/accessor declarations keep their original provenance.
- Function signatures, unions, Self, native void and generics retain native semantic facts.
- Conflicting/missing identities reject; bootstrap and undocumented synthetic members
  do not silently enter the public API tree.
- External authored XML/Markdown is matched by canonical declaration identity; absent
  source locations remain absent rather than being guessed from documentation files.

This follows the existing assembly-bundle/source-link direction in api-docs/README.md.
It avoids the maintenance cost of a second resolver while leaving the presentation
boundary replaceable. Full website migration, API coverage and source-link qualification
remain release work; no native runtime/compiler semantics change is proposed here.
