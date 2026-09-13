# Raven extension methods on neoCLR

The source experiment supports nongeneric application extension methods using
ordinary emitted static calls. No runtime opcode or Raven compiler change was
needed for this slice. The declaration image now supplies `ExtensionAttribute`
and the metadata-only `NotImplementedException` constructor referenced by Raven's
extension marker stub. This closes the emitted metadata graph; it does not add an
executable exception API. Reachable construction of that placeholder is rejected.

The readable [extension sample](experiments/raven-target/samples/application-extensions.rvn)
demonstrates chained integer operations, mutation through a shared ArrayList
reference, an Iterable receiver, and captured Func callbacks. Its `Visit` helper
runs immediately and explicitly disposes its iterator on normal completion. It is
not a LINQ operator. Callback faults remain terminal and do not acquire cleanup
semantics through this helper.

## Comparison and boundaries

Microsoft's [extension member documentation](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/extension-methods)
(consulted 2026-09-13) describes compiler-bound static methods invoked with member
syntax. Raven's extension containers likewise lower calls with a leading receiver.
Preserving that division keeps lookup in the compiler and execution in ordinary
runtime calls. Adding a dedicated extension opcode would duplicate compiler work.
The cost here is declaration metadata for generated, non-executable marker members;
the importer still validates every reachable method body.

This slice verifies source declarations in the application assembly, not the full
extension feature set. Generic application methods/types remain rejected by the
bounded bridge, including generic extension containers. Referenced extension
assemblies, extension properties, constrained receivers and static extensions need
separate end-to-end coverage. This is a bridge limit, not evidence that the neoCLR
runtime lacks generics or that Raven cannot compile generic extensions.

## Query follow-up

The intended starting contract remains deferred `Where` and `Select`, followed by
explicit materialization. [.NET Where](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.where?view=net-10.0)
(consulted 2026-09-13) defers execution until enumeration. Eager filtering would
change callback timing and allocate an intermediate collection, so it is not a
substitute for that contract.

The [prototype query API](raven-query-api.md) now implements these operators in the
Raven-target runtime library with validated closed generic bindings. It uses ordinary
runtime iterators without importing arbitrary generic application bodies. Its API
document records validation, costs and the remaining cleanup/compiler boundaries.

## Running and verification

Build the current source bridge and generate fresh declarations using
`--interfaces OUTPUT` as described in the [experiment instructions](experiments/raven-target/README.md).
Copy the sample to `Main.rvn` in a disposable collection-profile project with that
`NeoCLR.CoreProbe.dll`; run it using the saved-project tasks or `run_project.py`.
The expected output is `43`, `41`, `2`, `7`, `42`, `2`, `2`, on separate lines.
Archived SDK/extension .7 bundles do not contain the new declarations.

`verify_application.py` checks execution, shared references/captures, empty iteration,
rejection of generic extensions and rejection of executable marker construction,
alongside existing application cases. `verify_editor.py --extensions` adds member
completion for an integer extension; use the other profile flags matching the
project's declaration image. The latest source language server needs no further
compiler changes for this completion check.
