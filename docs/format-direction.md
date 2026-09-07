# Assembly syntax and CLI format direction

Interpretation, JIT, and native AOT share the semantic metadata/IL contract described
in [execution architecture](execution-architecture.md). This format is an input to
multiple backends, not a serialization of interpreter-only execution state. Native
artifacts and exports need additional target/linkage information; their schema remains
open. Hosting consumes these contracts and does not define the execution modes.

## Guiding constraint

neoIL does not have to mimic CLR assembly syntax. The output should retain familiar
.NET metadata and CIL structure because there is valuable existing parsing and
tooling support. Change or extend the representation when the platform's semantics
require it, rather than invent a new encoding for an unchanged concept.

Use [ECMA-335 partitions II and III](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
as the binary metadata and instruction baseline. Assembly syntax is a front end;
metadata/CIL encoding is a separate backend contract. A syntax improvement should
not by itself require a binary format change.

Reuse CLR managed references, represented by T&/ByRef, as the explicit reference
feature. Extend the existing metadata, verification and address/load/store paths
rather than introduce a parallel public ownership type. Frame-backed and retained
storage implement the same reference abstraction. Automatic retention and escapes
beyond a defining frame require documented semantic extensions; they do not imply
compatibility with execution on an unmodified CLR.

Preview artifacts and APIs may change incompatibly to implement the selected model.
Ref<T> is a historical proposal and arena encoding, not a required future wrapper.
Reject incompatible formats and require reassembly where needed; compatibility
layers are not a prerequisite for removing obsolete preview representations.

## Intended representation policy

- Preserve ordinary metadata concepts such as modules, type/field/function
  signatures, references, tokens, strings, and blobs. Aim to reuse standard tables,
  heaps, tokens, and operand encodings where their constraints fit neoCLR.
- Preserve standard opcode meanings and byte encodings where possible. Source
  conveniences should lower to existing sequences before adding an opcode.
- Give required deviations explicit version/feature information. Unsupported
  semantic features must be diagnosed, not silently treated as standard CLI.
- Distinguish tools that can parse a container or decode instructions from tools
  that can correctly verify or execute neoCLR semantics. Familiar encoding alone
  does not make the latter compatible.

## Required semantic differences to resolve in binary design

| Requirement | Binary/tooling question |
| --- | --- |
| First-class `Void`, including generic arguments | How to extend legal signature contexts and call/return stack contracts without confusing CLI readers/verifiers |
| Free functions without a class container | How to represent function ownership directly while keeping method-like signatures and tokens familiar; no synthetic class requirement in the semantic model |
| No inherent value/reference type split | Which existing signature/type flags can be preserved and where explicit storage capabilities need extended representation |
| Default owned allocation and explicit heap identity | Which existing operations remain valid and which need lowering or declared extensions |
| Option/Result and no exceptions | How tagged data and branching lower to ordinary field/control-flow operations; how to reject exception clauses and represent terminal Faults |
| Future borrows/async | Which lifetime and suspension metadata is required for verification |

Do not assign speculative ECMA opcode numbers, custom table IDs, or binary flags
before an encoding proposal and round-trip tests exist. Some parsers assume fixed
table schemas and may require modification. A CLI-shaped file must not be sold as
an ordinary executable .NET assembly if a standard runtime would misinterpret it.

## What exists today

The assembler currently resolves labels and produces the interpreter's typed IR,
serialized as `.neo.json` for inspection and tests. It does **not** emit CLI binary
metadata or CIL byte streams. Its integer branch operands are instruction indices,
not CIL byte displacements; field operands are record indices, not metadata tokens.
`ldc.bool`, explicit value-storage operations, and heap operations are experimental
IR instructions, not allocations of new binary CIL opcodes. Format 4 removed the
earlier union-specific instructions; Option/Result use ordinary library methods.
`newobj`, `stfld`, `ceq`, and Void-return
behavior also have prototype differences documented in the semantic reference.

`add`, `sub`, and `mul` preserve wrapping behavior. The `.ovf` forms preserve the
explicit checked-operation distinction, with terminal Faults replacing exceptions.
There is no reason to change these arithmetic distinctions merely because the
assembly syntax is new.

The next format milestone should choose an existing CLI reader for a small corpus,
write a minimal binary module for the unchanged subset, and demonstrate disassembly
and metadata round trips. Then add only the semantic extensions that the experiments
prove necessary. Reader/tool selection and a final binary schema remain open.
