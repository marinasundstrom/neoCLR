# Separate Data / JSON execution — 2026-10-07

The 175-source System.Runtime candidate and six-source System.Data build independently.
Unchanged `bootstrap/json-object-consumer.rvn` imports only emitted native references
and executes nested models, getter/setter behavior, scalar and jagged arrays, mutation,
shared identity, Unicode content and rejection before model construction. Exact stdout
matches the established six-line transcript and exit is 42. The new public-array consumer
also returns 42 after checking copied storage, shared reference identity, null elements,
boxed scalar access and empty arrays.

## Changes and dependency ownership

- System.Runtime owns ArrayReflection.GetLength/GetValue/Create over its existing checked
  vector services. Data owns serialization policy and no longer accesses those internals.
- Nonvirtual imported Object.GetType references use canonical root owner/member names,
  preserving ordinary call dispatch. Before this fix they named a nonexistent encoded root.
- Raven binds bootstrap `Object` syntax to an explicitly selected native root even through
  parent namespace lookup. A reduced C# symbol probe covers Object/System.Object/object.
  Default .NET paths and user lookalikes are not redirected.

The resulting Runtime assembly references only the explicit primitive Core bootstrap;
there are no Data, Networking or Web dependencies. This realizes the author's foundation
rule without declaring the current Runtime surface minimal or its package layout final.
The documented retained service seed and primitive bootstrap remain required. This is
not a bootstrap-free release, full API qualification, or a new package distribution system.

## Reproduce

Build Raven at `65f554a49` with this metadata project. Start with the Runtime split inputs
recorded in [Runtime split](runtime-split-2026-10-07.md), using fresh output directories:

```sh
python3 scripts/audit-native-bootstrap.py \
  --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 \
  --core /tmp/failure1006b/Core.dll --seed /tmp/preview12-bootstrap/System.neox \
  --libraries /tmp/native-poc-libraries1005 --runtime target/debug/neoclr \
  --output /tmp/data-runtime-retry --case runtime-owned
python3 scripts/audit-optional-libraries.py \
  --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 \
  --core /tmp/failure1006b/Core.dll --runtime-library-directory /tmp/data-runtime-retry/runtime-owned \
  --output /tmp/data-library-retry
python3 scripts/verify-separate-json.py \
  --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 \
  --core /tmp/failure1006b/Core.dll --runtime-library-directory /tmp/data-runtime-retry/runtime-owned \
  --data /tmp/data-library-retry/System.Data.dll --runtime target/debug/neoclr \
  --output /tmp/data-consumer-retry
```

[Full evidence](separate-data-2026-10-07.json) records the actual build/consumer commands,
source/compiler/runtime/artifact hashes, ownership manifest and seed. Runtime was built
before the final importer correction; its source definitions did not require that
correction. The source-free Data/application builds use the corrected compiler/metadata.
No Runtime or Data source file participates in a consumer invocation.

Execution uses an explicit 100-million-instruction budget, as existing JSON gates do.
The default budget exhausted after `Invalid input begins`; defaults are unchanged and
this records no performance improvement. The public array API retains terminal runtime
Faults for invalid inputs rather than inventing a Result contract over uncatchable faults.
See [the API and .NET comparison](../../../api-docs/reflection.md).

Validation: 165 metadata C# groups, 35 reflection signature checks, three runtime array
checks (including wrong boxed type and bounds rejection), imported-root namespace probe,
47 focused compiler regressions, API snapshot, two source-free compiler/verify/run gates,
and an API-authored Object.GetType/boxing/virtual Equals consumer (42). Website/API
content is updated; no routine website build or publication was performed.

Next: independently compile and execute System.Web against Runtime/Networking/Data,
then integrate the ownership catalog into project/editor/distribution workflows.
