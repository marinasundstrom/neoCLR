# Managed entry arguments

The development Raven collection profile accepts a static, nongeneric, no-result
entry with no parameters or one string array: `func Main(arguments: string[])`.

The bridge emits a parameterless startup adapter that copies the host argument
vector after index zero into a fresh managed array. An empty host vector or a
vector containing only the executable produces an empty array. Empty strings,
whitespace, Unicode and option-like strings retain their values. Mutating the
array does not change Environment.GetCommandLineArgs().

This matches [.NET Main arguments](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/program-structure/main-command-line):
Main excludes the executable; Environment.GetCommandLineArgs includes it.
The host continues to supply that full vector. The adapter reuses managed arrays
and GC tracking at the cost of one extra array and startup copy. The runtime's
parameterless entry ABI and Runtime Contract configuration remain unchanged.
The restricted legacy importer remains parameterless. Direct integer-, Result-
and Task-returning entries remain outside this slice.

Run `python3 docs/experiments/entry-arguments/verify.py --toolchain-root BUNDLE --runner target/release/examples/measure_async`.
The three cases cover empty host startup, executable-only startup, mixed values,
array independence and collection (live=0). SignatureProbe checks valid and invalid
metadata shapes. The JSON DOM corpus now uses Main(arguments: string[]).
