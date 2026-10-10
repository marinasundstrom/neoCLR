# Native process entry arguments

Development, 2026-10-10. Native Raven applications may declare
`func Main(arguments: string[]) -> int` (or a unit-returning Main). Both interpreter
and native console/HTTP project hosts pass the arguments after the executable name.
No arguments produces an empty, non-null array. Empty strings, whitespace, quotes,
backslashes and Unicode text are preserved after the OS/CRT has separated arguments.
The runtime does not parse flags or shell quoting a second time.

This closes an AOT admission/host gap. Raven already emits the String[] signature;
no language, Runtime Contract configuration or CLI metadata change is needed.
Parameterless Main remains supported. The one-Int32 experimental root is a separate
private backend testing contract, not a recommended application entry signature.

## Contract and comparison

Reviewed 2026-10-10: [.NET Main arguments](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/program-structure/main-command-line)
exclude the executable name; Environment.GetCommandLineArgs includes it. We follow
that separation. The C# source signature rules, CLI entry metadata, runtime startup
argument conversion and application flag parsing are distinct layers. This change
implements neoCLR startup conversion and AOT admission, not a new parser or a claim
of supporting every .NET async entry signature.

Windows hosts use the [MSVC wide entry point](https://learn.microsoft.com/en-us/cpp/cpp/main-function-command-line-args)
and convert UTF-16 to UTF-8 with WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS).
This avoids dependence on the active ANSI code page. Unix hosts validate UTF-8.
Ill-formed text fails startup rather than being silently replaced; this deliberate
UTF-8 boundary policy costs compatibility with arbitrary Unix filename bytes or
unpaired Windows surrogates. Valid Unicode has the same guest representation on
both hosts. OS/CRT quoting conventions still differ; pass an argument vector when
launching programmatically.

Passing the array explicitly avoids global argument storage, hidden process state
and a testing-framework dependency on Environment.GetCommandLineArgs. AOT binding
for that API remains a separate gap. The rejected fallback is not retained in the
runner. The source-included TestRunner accepts `(suite, arguments)`.

## Temporary native ABI and ownership

The backend selects `neoclr_entry_args_v1(argc, result, context, argv)` for String[]
roots and records `entryArguments: true`/`nativeAbi: process-arguments-v1` in the
selection report. The build driver defines NEOCLR_ENTRY_ARGUMENTS for the matching
host. Parameterless/scalar exports retain their existing ABI; a mismatched legacy
host fails to link rather than calling an incompatible signature.

The private generated adapter checks entry admission, clears diagnostics/resets the
invocation heap, then invokes `neoclr_process_arguments_v1`. That service copies
argv[1..] into the managed String[] and text allocations, publishes the array only
on success and never collects. Guest entry prologues then publish normal GC roots.
No host pointers escape into guest values. The copies share ordinary managed-array
and string lifetime; there is no special command-line object model. Startup costs
one array and one string allocation per argument, plus temporary Windows conversion
storage. No performance improvement is claimed.

Failure preserves the caller's result and records the startup fault. Existing
bounds apply: at most 65,536 elements, text service limits and the host's invocation
heap budget. Allocation failure may consume heap space, reclaimed on a subsequent
admitted entry or host cleanup. Native-GC/reference-arena profiles are required.
The runtime/backend and native-host layers own this private ABI. A future stable
embedding/startup ABI should replace it; it is not a portable public hosting API.

The native metadata signature is already canonical `arrayref<String>`; the Raven
reference bridge exposes the existing CLI String[] representation without losing
entry-argument information. No new temporary marker or callable carrier is added.
Native Windows ARM64 execution remains outside the existing x64 project profile.

## Validation

`tools/aot-poc/tests/entry_arguments.rs` checks explicit GC admission, Windows COFF
entry/import symbols, argument exclusion/empty arrays, invalid UTF-8, count limits,
heap exhaustion, result preservation, repeat invocation and quiescent reclamation.
The macOS C probe uses undefined-behavior/bounds sanitizers.

The [Raven test runner](../runtime/raven/tests/README.md) exercises actual compiled
Main(string[]) in native and interpreted execution, including empty strings, spaces,
quotes, backslashes and non-ASCII/supplementary Unicode. Its existing Windows action
runs the same argument and filtering cases with the MSVC wide host. Local Windows
object emission is not a substitute for that execution gate.

Local qualification: both focused backend tests pass, including Int32, unit and
no-result entry execution on macOS ARM64 and Windows x64 COFF generation. The Raven
suite qualification is recorded with the filtering follow-up. Windows execution of
this revision remains pending its GitHub action; no Windows pass is inferred here.
