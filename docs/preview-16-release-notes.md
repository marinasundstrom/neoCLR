# Preview 16: native metadata from compiler to runtime

Draft for the next release; publication and final platform qualification are pending.

## Native library construction

Raven builds System.Runtime and dependent libraries from native neoCLR metadata
and explicit runtime-service contracts. Production compilation no longer needs
the NeoCLR.CoreProbe/CLI semantic bootstrap. The editor imports those same native
references, so completion describes the selected neoCLR libraries.

The Raven compiler and language server still run on .NET. That tooling requirement
is separate from the metadata and runtime used by compiled neoCLR programs.

## Types and runtime contracts

Native metadata represents structural unions, intersections, tuples and function
types as types rather than ad-hoc nominal carriers. Supported shapes retain runtime
introspection. This remains bounded support: consult the type guides for generic,
conversion and backend restrictions. Further structural expansion is parked.

Nominal interfaces support reference-only variance. Interface conformance remains
explicit; intersections do not introduce implicit duck-typed interface matching.
Function types support typed patterns and provisional reflection invocation.

Checked operators and contexts, consistent exclusive/inclusive range endpoints,
and overflow-safe signed integer iteration extend the language/runtime contracts.
Readonly fields and type initializers retain native metadata and execution rules.
Introspection describes fields, initializers and generic variance; reflection checks
supported operations at runtime.

## SDK and editor

The matched SDK and extension use native references for completion, hover,
navigation and type inference. Callback inference preserves results such as
`Task.Run(() => 41)` as `Task<int>`. Project configuration and native reference
changes invalidate diagnostics and recover after a failed project reload.

Runtime-owned await is the default. Explicitly clearing the runtime-async helper
setting selects existing heap state-machine lowering, including when the setting
contains only whitespace. Ordinary Raven/.NET async behavior is unchanged.

## Updating and scope

Install the matching platform bundle and its VS Code extension together. Rebuild
applications and native hosts; do not combine an older SDK/library bundle with the
new backend. Source syntax using the former range endpoint conventions also needs
review: `..` is exclusive and `..=` is inclusive in this compiler.

neoCLR remains experimental, with evolving APIs and metadata. Intended platforms
are macOS ARM64 and Windows x64. JIT is the development default; interpreter execution
and bounded AOT publishing remain available. Source debugging is not available.
HTTP supports HTTP/1.1, without HTTP/2/3 or connection pooling. The optional
PostgreSQL provider targets local loopback/trust development, not general production
connectivity.

The macOS bundle includes PostgreSQL; the Windows bundle contains the four base
libraries. Compiler and editor prerequisites include .NET 11 SDK
`11.0.100-rc.1.26425.128` and .NET 10. Python 3 supports the bundled build/run tools.

## Qualification and measurements

Final notes will link the exact two-platform package qualification, component
hashes and checksums. Fresh macOS CPU/JSON and HTTP measurements retain correctness
checks, startup separation and host-load limitations. Windows measurements are
pending. These exploratory results do not establish production capacity or a
cross-platform speed ranking.
