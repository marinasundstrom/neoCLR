# Target-specific language contracts in Raven

Proposed 2026-09-12 after the author's request to configure changed library names and
contracts per target. This is planned compiler work, not an implemented Raven option.
The ordinary .NET target must retain its existing behavior; neoCLR is an explicit
experimental target. Neo is outside this exercise.

## Boundary and comparison

A compiler needs to recognize a few library protocols when lowering language features.
Most library APIs need no special treatment: source names bind to supplied metadata.
Dropping the interface `I` prefix is a library naming policy, not a runtime opcode or
a reason to rewrite every type name. Configure compiler-recognized roles where needed.

C# synchronous foreach uses the GetEnumerator/MoveNext/Current pattern and specified
interface/disposal rules. That is a language/compiler contract; CLI execution uses
ordinary method calls. See [C# specification §13.9.5](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/statements#1395-the-foreach-statement)
(primary source consulted 2026-09-12). Reusing that division lets Raven select different
library symbols without adding iteration instructions to neoCLR. It does not imply
identical cleanup semantics: CLR exception-based unwinding is not neoCLR's fault model.

| Role | Existing .NET spelling | Candidate neoCLR spelling |
| --- | --- | --- |
| Iteration source | System.Collections.Generic.IEnumerable<T> | System.Collections.Iterable<T> |
| Iteration cursor | System.Collections.Generic.IEnumerator<T> | System.Collections.Iterator<T> |
| Cursor acquisition | GetEnumerator() | GetIterator() |
| Advance / element | MoveNext() / Current | MoveNext() / Current |
| Explicit disposal | System.IDisposable.Dispose() | System.Disposable.Dispose() |

The iterable is the source; the iterator is the cursor. These are two roles, not an
interchangeable name mapping. Similarly, ArrayList is a concrete collection and List
is its contract in neoCLR; a compiler need not recognize either merely to call Add.

## Proposed design

Use a compilation-owned target contract description, selected by project configuration
and shared by binding, lowering, emission and the language server. Start with iteration
and disposal rather than a general mechanism for arbitrary semantic rewrites. Preserve
the default .NET description; make the experimental neoCLR description opt-in.

Each role should identify a supplied assembly, metadata type name and generic arity.
Each required member should specify its name, instance/static shape, generic parameters,
parameter/return types and property accessor shape. Resolve to actual symbols once per
compilation and validate relationships, including Iterator<T> implementing Disposable.
Do not fall back to host .NET assemblies when a target contract is absent or malformed.
Missing, ambiguous or incompatible members need target-specific diagnostics before IL
emission. A name match is insufficient evidence of an equivalent contract.

Runtime metadata still declares the real type names and relationships. Emit normal
CLI signatures/calls for the selected symbols. The import bridge binds only admitted
library methods; it must not guess compatibility by stripping an I prefix. Keep target
contract resolution distinct from `MetadataImportOptions`, which currently selects
supplied metadata/core identity rather than defining language protocols.

Initial inspection at Raven revision `5b773ae3536f52ef077c8897867950249d6dde90` found
explicit .NET enumerable/enumerator names in `TypeSymbolExtensionsForCodeGen.cs` and
iteration information in `ForIterationInfo.cs`. Audit binder, lowering and synthesized
iterator paths before implementing this option; replacing a single lookup is not enough.

## Alternatives, costs and next validation

Hard-coding neoCLR names into Raven would risk the default .NET target. Renaming neoCLR
interfaces to match .NET would discard the chosen library API and still not resolve
member/cleanup differences. A small validated target description centralizes those
choices, at the cost of another compiler configuration surface and a test matrix.
Its schema and project syntax remain provisional; no arbitrary plugin callbacks or
user-authored code execution are needed to select symbols.

Explicit Raven collection calls now execute against the adapted runtime library.
Next implement contract resolution on Raven's feature branch, with .NET regression
coverage, neoCLR iteration binding/emission and malformed-profile diagnostics. Verify
VS Code uses the same project target as command-line compilation. Test normal completion,
early exit and explicit disposal; define fault cleanup separately before claiming full
foreach equivalence. Async protocols, delegates, awaitables and further well-known
library roles should be added only when the next feature requires them.
