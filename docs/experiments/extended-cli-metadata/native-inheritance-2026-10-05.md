# Native local virtual hierarchy completion

The unchanged `application-inheritance.rvn` now compiles through Raven's ordinary
native driver and executes with stdout `7\n42\n`, exit 0 and empty stderr. Compiling
and executing the same source for .NET produces the same result. Collections and
interface controls retain their checked output. The ten original POC samples now
have execution evidence across the recorded slices; this is not full-System completion
or release publication. Native language-server/VS Code qualification remains next.

## Implementation and comparison with CLR

Metadata definitions/builders author ordinary public nongeneric class virtual and
abstract slots and exact local overrides. CLI Virtual/Abstract/NewSlot flags encode
the same supported behavior as CLR. Explicit virtual implementations retain their
slot flags instead of becoming Final merely because they implement an interface.
Concrete classes must satisfy inherited abstract members. Definition and builder
paths share attachment/writer validation. Interface conformance traverses local bases.
Native snapshots and introspection retain the declaration facts. No payload fields or
format-version changes are required.

The existing runtime already validates and executes virtual class hierarchies. Its
interface lookup now uses a validated declaration origin when an encoded interface
contract refers to an ordinary runtime-spelled class slot, preserving owner/signature
matching. Existing primitive/text implementations use the same path.

Raven admits this bounded shape through `AllowsClassVirtualSlots`. It retains a
`DirectInstanceCall` semantic operation for a bound base receiver. Native emission
uses symbols to author methods and emits direct base calls separately from virtual
calls. Abstract class methods remain bodyless; they are not runtime internal calls.
The importer/emitter boundary and ordinary .NET backend are unchanged.

External class overrides, generic virtual owners, re-abstraction, nonpublic virtual
slots and new-slot hiding remain unsupported. The reader's richer declaration facts
do not imply that every external class callable can already be emitted by Raven.
No temporary CLI projection or dependency fallback is added.

## Reproduction and evidence

[Machine-readable evidence](native-inheritance-2026-10-05.json) records source and
artifact hashes, commands, compiler/runtime base revisions and exact outputs. It labels
the working changes used for execution; this commit and its Raven counterpart contain
those changes. Raven counterpart: `448f25dfd` on `codex/metadata-consumer`; neoCLR
base: `3465aa70` on `codex/extended-cli-metadata`. Primitive core, retained System seed, ownership manifest and source-built
Numbers/Http dependencies are unchanged from the previous sample gate.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0 -- --class-virtual-runtime target/release/neoclr /tmp/class-virtual1005-final
```

The C# fixture emits and executes both manually authored and builder-authored graphs
on CLR/NeoCLR. It checks interface dispatch, virtual base dispatch, concrete base calls,
mutation and slot flags; missing implementations, incompatible overrides, inherited
hiding and abstract bodies are rejected before output. All 159 metadata contract groups
pass. `cargo test --release interface` passes 78 matching runtime tests. Raven's ten
focused existing inheritance/override checks and two new Debug/Release capability/CLR
execution tests pass.

`check-native-poc-samples.py` now checks the inheritance output when `--runtime` is
supplied. Focused execution covers inheritance, order collections and interfaces; the
other original sample evidence is reused, including the recorded HTTP localhost pair.
The inventory command remains a reporting tool: inspect each execution's `passed`
field rather than treating its own zero exit status as an acceptance assertion.

The stale reader rejection that removed Sealed from a static declaration now represents
a supported abstract class. It was replaced with an invalid value/static shape; this
is a test expectation adjustment, not an additional feature claim. Public host API
reference and website development status are updated; no guest API snapshot changed.
