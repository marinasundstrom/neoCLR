# Generic parameter variance introspection

`Main.rvn` discovers the three declared generic types in its own namespace and
reads `GenericParameterTypeInfo.Variance` through both generic-argument APIs.
It checks invariant/covariant/contravariant values, descriptor equality and that
`Fixed<int>` exposes the concrete argument rather than a declaration parameter.
The temporary compiler can emit additional support types; those are deliberately
outside this sample's namespace filter.

Build a native Raven project containing Main.rvn and importing the fresh bundle's
`NeoCLR.ClassLibrary.props`, then run its image with neoCLR's interpreter, supplying
`System.Runtime.dll` as `--module` and `--object-root`, and `System.runtime.neox` as
`--system`. Success exits 42. `results.json` records the tested matching bundle and
artifact hashes. The runtime unit test `generic_parameter_variance_follows_declaration_identity`
also checks the raw query contract; codegen reflection tests check closed shape queries.

Native open-definition discovery is an existing reflection gap. AOT/JIT interface
variance calls are independently qualified in the parent directory. Reified generic
metadata remains required, matching .NET; build-time catalogues alone do not prove
complete runtime reflection support.

The [closed-construction consumer](closed/README.md) separately qualifies indexed
argument queries and identity across interpreter, AOT and managed JIT.
