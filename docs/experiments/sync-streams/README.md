# Optional synchronous stream capabilities

Development, 2026-10-20. `SyncInputStream` and `SyncOutputStream` are independent
capabilities with borrowed byte views and direct Result returns. MemoryStream
implements them explicitly, sharing its storage, quota, cursor and closed state
with the asynchronous interfaces. Sync calls do not create or wait on Tasks;
this does not claim that storage growth or Result values never allocate.

The executable tests a synchronous-only provider, mixed sync/async calls, borrowed
subranges, untouched destination bytes, empty reads, short reads, EOF, the storage
quota and close state shared across interface views. See `results.json` for engine
outcomes: interpreter, JIT and AOT pass. The compiler and native backend must include explicit-method fixes;
older bundles are not sufficient.

```sh
python3 docs/experiments/sync-streams/check.py \
  target/sync-stream-bundle-v4/bundle target/sync-stream-app-v7 \
  --output docs/experiments/sync-streams/results.json
```

Matching inputs: `sync-stream-compiler-v4`, `sync-stream-libraries-v6`,
`sync-stream-bundle-v4`; native docs use `sync-stream-api-audit-v1`. Compiler code
starts from Raven `3011ac3f7` with the native explicit-method patch (committed as
`69f91e49f` on `codex/neoclr`); the bundle build
records that working provenance instead of claiming an unbuilt commit.

The focused Raven `NeoClrMetadataProbe --explicit-methods <core> <output>` checks
native explicit mapping metadata, private visibility and rejected ordinary access.
The backend unit test `explicit_reference_methods_preserve_private_dispatch` checks
explicit-target selection against a competing public method and rejects public
mapping artifacts. The focused selection test checks mapping identities after unused functions are removed.
Production execution also covers private library-name rewriting,
which must update explicit interface targets as well as ordinary call targets.

Native admission is bounded to nongeneric classes and nongeneric instance methods.
Existing explicit property emission is unchanged. No new runtime opcode, Runtime
Contract setting or CLI bridge encoding is introduced; ordinary .NET is unaffected.

Compared with .NET Stream's coexisting synchronous/asynchronous methods, neoCLR
uses independent capabilities so a provider is not required to implement both.
Explicit methods avoid return-type-only overloads. The cost is selecting an interface
view and documenting per-instance capabilities for wrappers. See
[stream design](../../stream-design.md) and [manual API](../../../api-docs/streams.md).

Synchronous Console/StreamReader/StreamWriter integration is not implemented here.
The author's subsequent direction adds independent SyncTextReader/SyncTextWriter and
provider-dependent supported modes. That requires shared text state and honest mode
reporting, rather than sync-over-async or blocking I/O hidden inside a completed Task.

Validation: the release-mode backend `--lib explicit_` filter passes all 11 checks,
four native API snapshot tests pass, both API references validate, and the website
build checks 4,610 pages plus 124 retained legacy pages. Website publication is separate.
The initial focused unit fixture lacked an instantiated receiver (so its dispatch
set was correctly empty); the final fixture constructs the receiver and passes.
