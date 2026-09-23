# API documentation maintenance

The `/docs/` section is built with pinned DocFX 2.80.1 (.NET 10). It describes the
**development** API. The release goal is a navigable overview and useful descriptions
of the main async APIs, not complete documentation of every library member.

## Build the complete website

From the repository root:

```sh
dotnet tool restore
npm ci --prefix website --ignore-scripts
python3 scripts/build-website.py
```

Serve `target/website`; the reference is at `/docs/`. The Pages workflow builds and
uploads both sections as one artifact. Deployment remains the existing manual
workflow on main. A local build or a push does not publish either section.

## Refresh signatures and XML descriptions

Use a freshly built Raven-target bridge (see its README for prerequisites):

```sh
mkdir -p target/api-docs/input
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-core target/api-docs/input/NeoCLR.CoreProbe.dll
python3 scripts/build-api-docs.py \
  --refresh target/api-docs/input/NeoCLR.CoreProbe.dll --check
```

The DLL must come from the current declarations, not an old published bundle. The
bridge output folder may differ by configuration. Alternatively supply the freshly
generated `demo/NeoCLR.CoreProbe.dll` from a bundle built at the same source revision.

DocFX reads that compiler reference assembly and the companion
`NeoCLR.CoreProbe.xml`. We currently author this XML sidecar directly: it is **not**
claimed to be compiler-emitted from Raven comments. It uses standard documentation
IDs. Keep descriptions aligned with the Raven implementations and tested feature
samples. Moving descriptions into source comments and automatic XML emission can
follow when that path is validated for the library build.

Commit the generated `api/*.yml` and `snapshot.json`, never the DLL or generated HTML.
This lets website CI render the reference without a Raven checkout. The manifest
records the input DLL hash, source/configuration fingerprints and output hashes.
It catches stale snapshots, but does not certify that an arbitrary supplied DLL was
built from those sources. Regeneration must use the current bridge. The build checks
that each included type/member has an XML summary and treats DocFX warnings as errors.

## Current scope and limitation

`filter.yml` selects Task, Promise, TaskQueue and TaskState. Other feature areas have
an overview linking their on-site guides. TaskOutcome and supporting types are shown
in signatures without implying complete reference coverage.

DocFX 2.80.1 fails in `YamlModelGenerator.AddSpecReference` when Roslyn sees the
neoCLR-specific `Func<System.Void>` argument. The filter excludes exactly Post, Run
and OnCompleted; `callbacks.md` documents them in Raven notation and the index makes
the omission explicit. Do not substitute `Action` or another type in the reference
assembly merely to make DocFX accept it. Remove this workaround after verifying a
renderer that supports these signatures. The XML keeps descriptions for those IDs
ready for that change; update the callback guide alongside them in the meantime.

Generated C#/VB declarations are metadata notation, not supported application
frontends. This tradeoff reuses established .NET documentation tooling without
requiring neoCLR's type system to obey every C# restriction. A custom reference
renderer would avoid the notation mismatch but add maintenance; defer it for this
release. Primary source: [DocFX assembly input and XML documentation](https://dotnet.github.io/docfx/docs/dotnet-api-docs.html),
reviewed 2026-09-23. Task/.NET contract comparisons remain in the existing Task
feature guide and design records; no runtime contract changes are made here.
