# Candidate runtime-library boundaries — 2026-10-07

The author suggested splitting System.Runtime from System.Data, System.Networking
and System.Web. The aggregate 197-input compile remains a useful coverage gate;
its diagnostic Numbers assembly is not a proposed distribution layout.

## Evidence-based starting point

The [inventory](library-boundaries-2026-10-07.json) walks emitted local nominal and
function references in declarations, signatures and bodies. Nested generated union
cases follow their declaring type; overloaded methods are all counted. A namespace
partition produces these **candidates**, not independently compiled artifacts:

| Candidate | Types | Functions | Existing declaration CBOR bytes |
| --- | ---: | ---: | ---: |
| System.Runtime remainder | 334 | 1,999 | 4,896,542 |
| System.Data | 26 | 142 | 812,782 |
| System.Networking | 33 | 189 | 711,626 |
| System.Web | 37 | 313 | 2,348,997 |

The remainder includes IO, storage, reflection, introspection, tasks and adapters;
this is not a final claim that everything in it belongs in the minimal core.
Sizes exclude manifests, new external-reference contracts and PE overhead, so they
are planning evidence rather than future artifact-size predictions.

Observed local dependency edges:

- Data → Runtime: collections, Option/Result, reflection/introspection and primitives.
- Networking → Runtime: tasks, cancellation, deadlines, buffers and native services.
- Web → Runtime, Networking and Data: HTTP uses sockets and JSON extensions use Json.

No Runtime → optional-library edge appears in this local inventory. Imported seed
and bootstrap dependencies are outside this scan. Network service declarations and
`NetworkRuntimeServices` currently live under runtime namespaces; assign the
`RuntimeNetworkCalls.rvn`/`RuntimeNetworkServices.rvn` adapter sources explicitly when
building Networking rather than equating namespace with assembly ownership. Keep
nested types with their declaring assembly. Later, a separate Web.Json integration
assembly could remove the mandatory Data dependency; that is not decided here.

## Ordered implementation gates

1. **Reconcile retained-service ownership and ABI.** The aggregate artifact now
   emits, but runtime loading must succeed before it can serve as a reference core.
   Remove competing declarations through explicit ownership; preserve value-return
   versus no-result behavior at calls. Do not suppress duplicate-contract validation.
2. **Build the Runtime candidate alone.** Give it the intended assembly identity,
   compile without Data/Networking/Web sources, and keep explicit primitive/seed
   inputs. Then compile and run a source-free native consumer. Inventory any remaining
   core-to-optional dependency rather than substituting declarations.
3. **Build Data and Networking independently against Runtime.** Move their service
   adapters with them where appropriate; prove JSON mapping and socket/task consumers
   import emitted artifacts with library sources absent. Keep .NET controls.
4. **Build Web against the three emitted dependencies.** Execute the HTTP and JSON
   samples. Current source dependencies require Data; avoid silently treating it as
   optional. Host/runtime catalogs must have one owner per identity.
5. **Replace the diagnostic bundle layout in CLI/editor/release tooling.** Record
   hashes and ownership, test missing/conflicting dependencies, and update project
   templates, API-reference source selections and sample deployment sets together.
   Retain the aggregate compile as an independent regression.

This follows the existing .NET/CLI assembly-reference model rather than introducing
new namespace semantics. Splitting can reduce mandatory dependency sets and keep
ordinary artifacts below existing envelope limits; it also adds assembly references,
accessibility boundaries and packaging/versioning work. No execution-time improvement
is claimed. See the [metadata design baseline](README.md) and
[larger-library profile tradeoffs](expanded-library-2026-10-07.md).

## First ownership correction and next blocker

The audit seed still supplied System.Object even with the source root selected.
The runtime therefore found the wrong parent identity while validating Array backing.
The audit now assembles its temporary bootstrap seed, removes the Object type and its
methods from the retained metadata, clears affected dense definition identities for
normalization, and writes a native NEOX seed with the existing translation tool.
The intermediate JSON is a host-side preparation artifact, not a runtime fallback.
The final seed deliberately requires the source Object owner. The bootstrap input,
retained model, encoder command and artifact hashes remain explicit.

[All 197 sources still emit](native-bootstrap-retained-root-2026-10-07.json).
[Runtime admission](retained-root-2026-10-07.json) clears the Array/root mismatch and
now reports `neoCLR.Runtime.WriteLine`: retained seed has Void **value** return,
source declaration has **no-result**, both with the same String parameter. The
runtime diagnostic now names the conflicting declarations and both result contracts.
Next reconcile this ABI and other retained/source services without discarding stack
results from callers. Full-System execution and a separate-assembly bootstrap remain
unproven.

## Reproduction

The compiler audit command and exact inputs are recorded in the linked JSON evidence.
For the planning inventory, decode an emitted PE using the existing metadata API:

```csharp
File.WriteAllBytes("native.json", RuntimeAssemblyContainer.Read(File.ReadAllBytes("Numbers.dll")));
```

Then run:

```sh
python3 scripts/audit-library-boundaries.py --native-json native.json --output boundaries.json
```

The inventory rejects duplicate native type identities and cyclic/foreign declaring
owners. Recorded checks count all 430 types and 2,643 methods, retain the nested union
owners, reproduce the five edges above and reject a synthetic declaring-type cycle.
This is read-only planning tooling, not an assembly splitter or a semantic importer.
