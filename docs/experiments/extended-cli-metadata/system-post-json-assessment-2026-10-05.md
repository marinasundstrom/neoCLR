# Full-System compilation reassessment — 2026-10-05

The complete runtime library does not yet compile. A fresh ordinary-driver attempt with
all 166 System Raven files plus the four native introspection adapters stops in binding:
220 diagnostics, including 69 distinct missing RuntimeServices member names. No assembly
is published. These are not 220 independent defects: missing services, source/bootstrap
identity collisions and failed expression binding produce cascades. This differs from
the earlier 416-diagnostic bootstrap profile, so the difference is not a controlled
measurement of compiler progress.

Runtime revision: `91ac32ed`; Raven revision: `8f015386e`. The existing separate-library
JSON execution gate remains valid. This reassessment changes no compiler/runtime/source
behavior and makes no new execution or .NET-parity claim.

## Compile results

[Complete commands, manifests, diagnostics and hashes](system-post-json-audit-2026-10-05.json).
Counts below include native adapter files where selected.

| Selection | Files compiled | Result |
|---|---:|---|
| Numeric/collections/text baseline | 75 | Emits Numbers.dll |
| Baseline plus encoding | 82 | 4 binding errors: SliceUtf8 lookup plus pattern cascades |
| Plus text streams | 87 | 8 binding errors, same first cause |
| Plus JSON/introspection and adapters | 108 | 12 binding errors, same first cause |
| Plus ResultOperators | 109 | Same 12 errors |
| Entire System plus adapters | 170 | 220 binding errors; no output |
| Tasks/Concurrency against emitted libraries | 6 | Emits Additional.dll |
| Remaining storage/IO against emitted libraries | 11 | 30 binding errors; 7 missing service names |
| Networking/HTTP/tasks/Uri against emitted libraries | 23 | 58 binding errors; 21 missing service names |

The artifact-reference rows do not include library sources in the consumer compilation.
The 109-file combination contains 105 production System files and four internal adapters.
Their separate-library JSON gate succeeds, but this single-assembly combination fails.
Compiling Tasks here is compilation evidence only; it was not executed in this audit.

## Highest-value next slices

1. **Repair source-owned primitive member lookup.** The checked-in four-line
   `bootstrap/source-string-member-probe.rvn` fails with exactly one RAV0117 when
   compiled alongside the 75-file source baseline, and emits when compiled against
   the corresponding native artifact. `String.rvn` already declares SliceUtf8.
   This isolates a source-versus-import binding discrepancy, not a missing text API
   or metadata instruction. Fix the canonical source symbol/member path, then rerun
   the cumulative encoding → streams → JSON cases and execute their consumers.
   The exact underlying compiler cause still needs investigation. If general rather
   than target-specific, reduce it independently on .NET/main before backporting.
2. **Complete storage service contracts as one family.** Missing declarations are
   StorageList, StorageNames, StorageKind, ReadAllText, WriteAllText, PathCombine and
   PathGetFileName. Six names already occur in the runtime native binding table;
   StorageNames is a payload-adapter boundary to investigate. Reuse the existing
   service ABI and checked source adapters. Match signatures and vector/erased-value
   payloads, then compile the 11-file addition and execute real filesystem round trips.
   Do not interpret the 21 generic comparison errors as independent operator gaps
   before the missing service return types are resolved.
3. **Complete DNS/socket service contracts, then reassess HTTP-specific binding.**
   The 21 missing names span lookup/cancellation/results, socket creation/connection/
   acceptance, transfer, deadlines and closing. Twenty occur directly in the native
   binding table; DnsAddresses is another payload-adapter boundary. Existing names
   prove implementation entry points, not source ABI compatibility. Compile and run
   a bounded loopback case with cancellation/cleanup before the HTTP sample. Four
   `Cannot convert from Error to Error` diagnostics also remain; reduce them after
   wiring the missing contracts to distinguish real identity/conversion issues from
   cascades. Do not rewrite the HTTP sources around them.
4. **Finish source-built core ownership.** The all-files attempt includes Object,
   Value, Void, RuntimeTypeHandle and other declarations still owned by the accepted
   bootstrap. It reports an invalid typeof contract as well as missing primitive/native
   service contracts. Give each declaration a single explicit owner and migrate the
   bootstrap in bounded steps. This is a full-System prerequisite, not evidence that
   the established native importer should be replaced or the .NET backend redesigned.

This order prioritizes a cross-family compiler blocker, then service families with
existing runtime implementations. Preserve the working artifact-only JSON gate. Defer
wider emission/runtime conclusions until binding admits the relevant sources; this
assessment cannot establish that the rest of code generation is complete.

Compared with .NET, ordinary source and imported declarations should expose the same
members after explicit target primitive selection. Preserve that semantic expectation
and existing .NET Reflection/Emit behavior. Runtime services use the existing explicit
InternalCall boundary rather than implicit desktop CLR bindings; ownership and payload
adapters are necessary integration work, not new language semantics or performance claims.

## Reproduce

First prepare the matching artifacts using the
[source JSON mapping gate](source-json-mapping-2026-10-05.md), then run:

```sh
python3 docs/experiments/extended-cli-metadata/audit_post_json_compilation.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCore.dll \
  --seed /path/to/json-gate/System.neox \
  --ownership /path/to/json-gate/ownership.json \
  --reference /path/to/text-gate/Numbers.dll \
  --reference /path/to/encoding-gate/Encoding.dll \
  --reference /path/to/streams-gate/TextStreams.dll \
  --reference /path/to/json-gate/JsonIntrospection.dll \
  --reference /path/to/json-gate/ResultOperators.dll \
  --output /tmp/system-post-json-fresh
```

The tool derives each cumulative diagnostic ownership manifest from the accepted gate,
removing primitive/typeof providers before their sources are included. It neither
rewrites sources nor treats these merged manifests as a production assembly layout.
Imported cases retain the gate's actual separate ownership. Source hashes, compiler
component hashes, dependency hashes, commands, emitted artifact hashes and diagnostics
are recorded. A successful audit process can contain deliberately failed compilations;
inspect each case's exitCode and outputPublished fields.
