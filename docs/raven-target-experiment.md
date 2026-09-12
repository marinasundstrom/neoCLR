# Raven target and binary artifact experiment

Planned 2026-09-12 on `codex/raven-neoclr-target`, starting from `64024d6`.
This document scopes an experiment. No Raven backend, binary format or loader support
is implemented by the planning slice. The subsequent
[slice 1 integration map](raven-backend-integration-map.md) now pins and inspects Raven's
backend: it identifies existing core-library retargeting support, reflection/PE coupling,
framework discovery and the Void/Unit and exception-projection gaps. Binary format and
runtime-target contracts remain undecided.

## Objective and compatibility boundary

Use the existing Raven compiler, which currently targets .NET according to the author,
to compile a small program for neoCLR through a new target. The program must resolve
neoCLR's own runtime-library declarations, emit an artifact, and execute against that
library on neoCLR. Neo remains a demonstration and regression frontend; the runtime is
the main focus. This experiment tests interoperability with an independently developed
compiler rather than only improving Neo's source projection.

Seek the smallest useful compatibility surface across metadata, IL behavior, calls,
type/member resolution and library APIs. “ABI” includes only part of this boundary;
matching binary layouts or opcode names alone does not establish compatible execution.
C# targeting remains a longer-term goal, not a second compiler implementation in this
experiment. Running existing .NET binaries unchanged is not the acceptance criterion.

The author explicitly accepts source/API adaptation for deliberate differences:
expected failures should use Result, absence should generally use Option, and terminal
Faults should not recreate .NET's ordinary catchable exception flow. Do not silently
translate arbitrary try/catch into Result or claim unchanged source compatibility.
Fault containment and cleanup rules still need precise contracts. Nullability remains
a capability the platform should support, even where library APIs prefer Option;
this experiment does not settle its representation.

The [reference-defaults evaluation](reference-defaults-evaluation.md) remains an
investigation. This branch does not select a new class/value model, require a ValueType
hierarchy, or authorize changing all current storage contracts as a prerequisite.
Record actual frontend obstacles before deciding which runtime differences to change.

## First end-to-end milestone

A Raven source program calls neoCLR's System.Console.WriteLine with a string and
returns normally. It compiles using the new target, produces a documented binary
artifact, loads through neoCLR, and prints the expected text using neoCLR's System
implementation. A compiler-readable library declaration artifact and executable
library may be distinct if the backend requires it; their identities and signatures
must agree. The runtime library may remain authored in neoIL.

The milestone must demonstrate:

- An explicit Raven target selection and reproducible build/run commands.
- Compilation against neoCLR's library surface, without silently resolving the call
  to the installed .NET framework instead.
- Binary metadata and method-body emission/loading sufficient for the program and
  its actual library dependencies, with clear rejection of unsupported inputs.
- Correct binding to the loaded neoCLR library, observable output and normal completion.
- A regression path for both a successful call and a missing/mismatched member reference.

Do not assume HelloWorld implies a tiny loader change: inspect the dependency closure
of the System library. Choose whether to support that closure or produce a documented
minimal subset from the same library. A temporary JSON transport can diagnose compiler
integration, but does not complete the binary-artifact milestone.

## Investigation and slice order

1. **Inspect Raven's current backend and framework discovery.** Locate its repository
   and instructions, identify metadata/assembly emission dependencies and their pinned
   versions, and trace a simple program through symbol resolution, lowering and output.
   Determine where it assumes CLI core types, value/reference categories, Void, object
   construction, assembly identities, generics and exception handling. Do not assume
   Roslyn, Reflection.Emit or a particular metadata writer is used before inspection.
2. **Map the minimal target contract.** Compare the required emitted operations and
   library signatures to neoCLR. Classify each difference as reusable behavior, compiler
   adaptation, artifact encoding, library projection or missing runtime contract.
   Preserve useful CLR semantics and justify each deviation with a concrete need.
3. **Choose a binary strategy from evidence.** Evaluate reuse of a CLI container and
   compatible metadata/IL subset versus a versioned neoCLR container reusing compatible
   tables/signatures/operands. Include the cost to Raven's actual writer and neoCLR's
   loader. Define library-reference artifacts if needed. Do not select a format merely
   because it serializes current Rust structures conveniently.
4. **Implement and test the bounded artifact path.** Encode identities, references,
   signatures and method bodies needed by the milestone, including its library closure.
   Define feature/version rejection, malformed-input limits and diagnostics. Keep the
   existing source/JSON path useful as a control while bringing up the binary path.
5. **Add the Raven target and execute HelloWorld.** Compile against neoCLR declarations,
   load the resulting artifacts and bind to neoCLR's System. Document exact toolchains,
   source, commands, expected output and remaining unsupported language constructs.
6. **Expand only after the first path works.** Add a user-defined type and call, then
   a generic Result/Option API with success/error/absence cases. Use these to expose
   semantic gaps instead of promising all Raven features at once.

Keep implementation slices separate and update the changelog for each. The present
request creates the branch and documents the experiment; the listed implementations
are planned work, not reported completion.

## Binary format decisions that need explicit answers

Reuse the existing [format direction](format-direction.md) and
[design research process](design-research.md), with ECMA-335 Partitions II and III as
the metadata/IL baseline. Deepen the primary-source comparison against the actual
Raven emitter before implementing a contract; no new binary conformance claim is made here.

| Question | Why it matters |
| --- | --- |
| Standard CLI container, adapted container or custom header? | Determines writer/tool reuse and whether unsupported features are distinguishable before execution. |
| What identifies neoCLR core types and System? | Prevents accidental .NET core-library binding; must include module/assembly identity policy. |
| How are free functions, inhabited Void and extended managed references encoded? | Current platform semantics cannot be assumed to fit every standard CLI consumer. |
| Which signatures and opcodes retain standard meaning? | Compatible spelling/bytes must not hide different construction, storage or call behavior. |
| How are library declarations exposed to Raven? | The compiler needs usable symbols, not merely a runtime-readable file. |
| What happens to unknown features, invalid tokens and unsupported exception regions? | Reject clearly rather than interpreting them with different semantics. |
| How do versioning, linking and diagnostics survive serialization? | Source and binary loading must select the same definitions and report useful failures. |

No native machine-code ABI, JIT, AOT, full PE ecosystem interoperability or .NET binary
execution is promised by this first artifact experiment.

## Evidence and decisions to retain

Record Raven revision/toolchain, neoCLR revision, imported System identity, emitted
references, binary version/features, exact commands and validation results. Compare
source/JSON and binary execution where possible. Include unresolved compatibility gaps
and explicitly distinguish expected Result errors from terminal Faults.

Success means an existing compiler demonstrably targets our runtime and library.
It does not mean all its language features work or that migration requires no changes.
