# Cumulative System library type budget — 2026-10-05

The cumulative source-built library now includes Tasks/Concurrency: 115 inputs,
comprising 111 production System sources and four native introspection adapters.
The ordinary driver emits one Numbers.dll. Artifact-only JSON and Tasks consumers
load that same native library and execute successfully.

## Fix and compatibility

After the source primitive member lookup fix, this build reached the host metadata
writer's 256-type limit. Manual definitions, builders and native reader materialization
now share a limit of 4,095 authored declarations. This fits the CLI reader's existing
4,096 TypeDef-row budget including the synthetic module row. Nested declarations count
toward the same limit. The limit is host-library policy, not the maximum of the CLI
format. Tokens, instructions and native schema are unchanged. Older host-library
versions still reject larger native inputs.

All other bounds remain in force: byte/image/node/depth limits, fields, properties,
methods, signatures and dependencies. Large libraries use the existing schema-3
WriteLibraryBinary profile; this change does not expand the application envelope.
No runtime change or performance claim is involved. Compared with .NET metadata,
ordinary TypeDef indexing already accommodates these declarations; the mismatch was
between our authoring and reader policies, not a different type representation.

## Validation

154/154 C# metadata test groups pass. The new boundary group authors 4,095 classes
through alternating builder/manual paths, rejects an additional declaration through
both paths without changing the collection, reads CLI and native/library images,
resolves facade types, and rejects an excessive native type array before materialization.
The pre-existing smaller binary-envelope limit remains enforced; boundary tests use
its explicit library profile.

[Compilation and executable evidence](cumulative-library-type-budget-2026-10-05.json)
records the exact library inputs, command, dependency/compiler/runtime hashes and
three separately compiled consumer commands. The nested JSON case returns 42 with
exact output; unchanged Mapping.rvn/Main.rvn returns 0 with its success line. The
unchanged Tasks consumer returns 42 with empty stdout, covering mapped/chained tasks,
callbacks, thread/worker results, cancellation and shared state-machine mutation.
Library sources are absent from those consumer invocations. The retained bootstrap
and 100-million-instruction execution budget remain explicit.

Raven source-binding implementation: de5645e96; matching capacity documentation:
7e7f11524. The metadata change is in the commit containing this record. Evidence
revision fields name working-tree bases; hashes identify tested binaries. Ordinary
.NET controls from the source-binding slice remain 25/25; no further binder/emitter
behavior changed. API snapshot validation still reports the previously recorded stale
full guest snapshot; host API XML/manual limits are updated without replacing it with
a partial runtime library.

## Reproduce and remaining work

Run audit_post_json_compilation.py with the arguments in the
[assessment](system-post-json-assessment-2026-10-05.md). The cumulative-tasks case now
emits. Use its Numbers.dll and ownership.json with:

```sh
python3 docs/experiments/extended-cli-metadata/verify_cumulative_json.py \
  --compiler /path/to/rvnc.dll --library /path/to/cumulative-tasks/Numbers.dll \
  --ownership /path/to/cumulative-tasks/ownership.json \
  --seed /path/to/json-gate/System.neox --core /path/to/IntrospectionCore.dll \
  --runtime target/debug/neoclr --tasks --output /tmp/cumulative-library-consumers
```

Storage still lacks seven compiler-facing service declarations, and DNS/socket code
lacks twenty-one; most corresponding runtime entry points already exist. The full
all-source build also retains source/bootstrap ownership conflicts. Those are the next
families to resolve. Neither this library build nor these consumers establish complete
System compilation, complete .NET source-library parity or socket/HTTP execution.
