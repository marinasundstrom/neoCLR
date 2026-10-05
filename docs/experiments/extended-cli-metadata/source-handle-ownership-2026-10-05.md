# Source-owned RuntimeTypeHandle — 2026-10-05

The combined source library now includes RuntimeTypeHandle: 140 production sources
plus nine adapters compile together. Separate native JSON mapping and Tasks consumers
compile, verify and execute with their existing output/result assertions. Library
sources are absent from consumer commands. Source Object ownership and the remaining
core service families still block the complete 166-source library.

## Ownership and architecture

The source handle is an empty value declaration whose storage remains runtime-owned.
Explicit bootstrap ownership selects it as a native primitive provider. During its
source build, Raven canonicalizes handle signature references to the bootstrap handle;
without that selection the source name remains an ordinary nominal type. This fixes
strict typeof contract binding without relaxing the contract or broadening symbol
equality. Consumer import selects the emitted native primitive provider.

Metadata definitions and builders now accept RuntimeTypeHandle in SetNativePrimitive.
Native reading and introspection preserve that designation. Native encoding uses the
existing Runtime representation and RuntimeTypeHandle signature; CLI signatures retain
the ordinary core handle reference. No format version, opcode, reflection API or runtime
implementation change was needed. Like the CLR handle, this is an opaque runtime value;
neoCLR does not expose a private pointer-shaped field as its storage model.

The selected source library must be the only owner. The original retained seed rejected
with `Runtime seed duplicates a source-owned declaration: System.RuntimeTypeHandle`.
The gate removes exactly that empty seed declaration, retaining services and all other
bootstrap inputs. It does not silently substitute CLI metadata for the source library.
Raven emission rejects fields, generic/nested declarations and constructors for this
intrinsic; the metadata writer independently revalidates its storage contract.

## Reproduce

Start with the successful combined-library ownership manifest from the
[post-HTTP assessment](post-http-compilation-2026-10-05.md), and the retained seed source
from the [JSON gate](source-json-mapping-2026-10-05.md):

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_handle.py \
  --compiler /path/to/rvnc.dll --ownership /path/to/combined/ownership.json \
  --seed-source /path/to/System.neoil --core /path/to/IntrospectionCoreParams.dll \
  --runtime target/release/neoclr --output /tmp/source-handle-gate
```

The fresh output includes the adjusted ownership catalog and seed, native library,
consumer images and command/hash evidence. [Build evidence](source-handle-ownership-2026-10-05.json)
and [consumer execution](source-handle-consumers-2026-10-05.json) preserve the actual run.
Compiler implementation is Raven 1ac78fdfa. Revision fields name pre-commit working-tree
bases; artifact hashes identify the tested compiler/metadata builds. Execution uses the
existing release runtime from the HTTP gate (e076c118 build), confirming no new runtime
support is needed for these handle definitions. This run does not revalidate the later
array-dispatch change or all HTTP loopback cases.

Validation: 155/155 C# metadata groups pass, including builder/manual-definition handle
round trips, introspection and invalid identity/storage. Raven's 34 existing focused
metadata-import/typeof tests passed before the change; all 36 pass afterward, including
explicit-selection and unselected-source signature tests. Both JSON consumers and Tasks
execute through native references. Existing .NET controls stay passing; this target-only
ownership feature has no independent general compiler fix to backport to main.

Host public API documentation and XML are updated. The API snapshot check still reports
the previously recorded stale guest snapshot; no partial library replaced it. Website
content was reviewed: no guest API shape changed and no website build/publication was
needed. Native async, source Object ownership, full-library .NET parity and post-bootstrap
benchmarks remain separate milestones.
