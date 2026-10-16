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

Development native discovery is qualified with explicit assembly/module retention.
The consumer also enumerates its logical module and compares its generic definition
identities with assembly discovery. Run on macOS ARM64:

```sh
python3 docs/validation/interface-variance/introspection/native-run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output target/open-generic-introspection
```

The runner compiles once, then executes that image in the interpreter, AOT and
managed JIT. It writes schema-6 reflection roots retaining the exact assembly's
type catalog and module names. The host checks faults, root cleanup and collection;
AOT also uses undefined-behavior and bounds sanitizers. `native-results.json` records
the matching artifact hashes and outcomes. Reified generic metadata remains
required, matching .NET. Open generic member/relationship queries, method generic
definition discovery and intrinsic String variance remain separate gaps.

The [closed-construction consumer](closed/README.md) separately qualifies indexed
argument queries and identity across interpreter, AOT and managed JIT.

`sequence-results.json` records the open-definition interpreter regression against
the fresh class library after moving sequence construction to the shared count
and indexed queries.
