# Direction and migration

The platform name is undecided; neoCLR names the runtime only. The existing code
is a small semantic testbed. It is not a commitment to Rust for every component,
JSON for distribution, or the exact instruction extensions used here.

## Next milestones

1. Add a control-flow verifier: typed stack states at joins, definite local
   initialization, valid return paths, and maximum stack calculation. Make
   `check` a meaningful pre-execution guarantee within its stated safety scope.
2. Settle array ownership and borrow rules, then implement the bounded experiment
   in [arrays and pointers](arrays-and-pointers.md).
3. Implement a CLI-based binary reader/writer for the supported subset, preserving
   standard table/heap/token and opcode encodings where semantics permit. Define
   versioned extensions only for required deviations; see [format direction](format-direction.md).
4. Replace intrinsic-only library scaffolding with loadable modules, structured
   error definitions, user-defined unions and generic definitions. Keep familiar
   namespaces while defining contracts around `Option` and `Result`.
5. Introduce interfaces without naming prefixes, explicit dispatch metadata, and
   a modest collections library; choose equality and mutation contracts deliberately.
6. Build a .NET metadata/IL inspection and translation tool for a supported subset,
   with actionable diagnostics for semantic differences.
7. Choose heap lifetime management and implement reclamation before treating this
   as a long-running runtime. Then evaluate layout, native interop, concurrency,
   and runtime async against measured needs.

## Migration principles

Use [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
metadata and instruction concepts as the baseline vocabulary. Preserve familiar
namespaces, signatures where meaning permits, and ordinary arithmetic/control-flow
structure. Each incompatible behavior needs an explicit mapping or diagnostic.

Prefer a source recompile path first. Imported .NET class instances generally need
explicit `Ref<T>` to retain aliasing and identity; converting them to frame-owned
copies silently would change programs. Imported structs can often remain owned
values, but boxing, reflection, interface dispatch, and layout still need work.

Translate void-return calls with their changed stack effects: neoCLR produces a
real Void value. Insert an explicit discard when adapting a CIL caller that
expects no result. Introduce `Option` for APIs whose nullable values mean absence;
do not rewrite null tests mechanically when null is a deliberate reference state.

Exception-heavy APIs need signature and control-flow adaptation to `Result`,
including explicit resource cleanup. A translation tool must diagnose unsupported
handlers rather than discard them. Faults cannot stand in for ordinary recoverable
exceptions without changing the contract. A future external .NET bridge may catch
host exceptions at that boundary and return structured Errors, but guest exception
semantics should not leak in.

Array covariance, byrefs, unsafe pointer arithmetic, reflection, dynamic code,
finalizers, disposal, async state machines, and unchecked integer overflow all
require deliberate treatment. Ordinary `add`/`sub`/`mul` already retain wrapping semantics; checked `.ovf`
operations terminate with Faults rather than throw. Surface required changes early
through a compatibility report rather than promise binary execution.

No importer, source compiler, bridge, binary metadata writer, or compatibility
analyzer is implemented yet. A useful migration success criterion is a small real
library recompiling with localized, explained changes and equivalent observable
behavior in the supported subset.
