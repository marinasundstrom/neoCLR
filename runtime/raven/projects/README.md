# Native class-library projects

These are native Raven projects. The older `../System.Runtime.rvnproj` remains the
legacy CLI bridge slice project; it is not this native build.

`System.Runtime/System.Runtime.rvnproj` selects the audited 175-source Runtime
foundation. Data, Networking and Web do not belong to this project. Platform helpers
remain in Runtime for now; extracting their service boundaries is a separate gate.
The colocated ownership manifest records source and semantic ownership explicitly.

Build with a Raven compiler containing the NeoCLR metadata adapter:

```sh
RavenNeoClrCoreReference=/absolute/path/Core.dll \
RavenNeoClrRuntimeSeed=/absolute/path/bootstrap/System.neox \
  dotnet /absolute/path/rvnc.dll neoclr \
  --project runtime/raven/projects/System.Runtime/System.Runtime.rvnproj
```

These environment variables are evaluated MSBuild properties. They must refer to
matching primitive-bootstrap and compile-time retained-seed artifacts; there is no
implicit download, host-framework fallback or application metadata projection.
The existing `scripts/audit-native-bootstrap.py --case runtime-owned` preparation
produces the documented bootstrap inputs. They remain explicit dependencies, not a
claim of bootstrap-free compilation.

The output is `System.Runtime/bin/neoclr/System.Runtime.dll`. Before execution,
finalize the retained seed against that exact artifact:

```sh
dotnet /absolute/path/NeoCLR.Metadata.Translate.dll \
  /absolute/path/bootstrap/System.retained.json /absolute/path/System.runtime.neox \
  --reference runtime/raven/projects/System.Runtime/bin/neoclr/System.Runtime.dll
```

Use `System.runtime.neox` for consumers and runtime execution, with the emitted
Runtime artifact selected as Object owner. The compile-time seed and finalized seed
serve different purposes. A seed finalized against an older Runtime is not silently
reused. This prototype finalization remains explicit tooling; it does not translate
application/library references into .NET metadata.

`scripts/verify-native-runtime-project.py` reproduces the project build, finalization
and source-free execution of unchanged `application-order-collections`, with hashes
and exact stdout/exit checks. Higher-level native projects, automatic bootstrap
orchestration, Platform ownership and shipping layouts remain subsequent work.

Compared with the existing .NET workflow, the native project uses the same evaluated
project/source model but requires explicit primitive and retained-seed inputs. This
makes ownership reviewable at the cost of an additional finalization stage. No .NET
compiler defaults or native metadata encodings change in this slice.
