# Explicit owned-heap JIT host — 2026-10-24

The [warm HTTP ownership experiment](../native-host-ownership/README.md) is now
available as an explicit macOS ARM64 development-bundle configuration. Add
`--owned-heap` to `scripts/prepare-jit-bundle.py`; it defines
`NEOCLR_GC_OWNED_HEAP` for the existing JIT entry host and records
`jit.heapOwnership: "exclusive-thread"` in the bundle manifest. Without the option,
the manifest records `external` and host behavior stays unchanged.

This opts into the existing native arena contract, not a new collector. The empty
invocation arena is acquired before socket/task/terminal scopes. All returning
post-acquisition exits release it before freeing the original buffer, including
scope admission failure and fault/terminal-rendering outcomes. Failed acquisition
frees only the new buffer and preserves an incumbent owner. Async shutdown, root
checks, final collections and scope teardown remain intact; broken root/cleanup
invariants continue to abort.

Only runtime helpers may modify descriptors or arena data/capacity/used state.
Admitted native helpers may access payloads but cannot overwrite headers or use
the arena across threads. Nested acquisition is rejected. Independent invocations
on separate threads have separate owned heaps. This does not qualify shared-heap
workers, ownership transfer or unrestricted FFI. JIT bundle opt-in is rejected on
Windows pending native qualification; its existing default path is unchanged.

Preparation removes joined or split ownership defines/undefines inherited from a reused
recipe unless explicitly requested. This includes `=0`, because C `#ifdef` treats
that spelling as enabled. Unrelated flags are retained. Ownership does not enable
pacing: `NEOCLR_GC_PACED` remains a separate recipe choice. Both policies are tested;
the earlier throughput gain was measured with pacing, not forced collection.

## Validation

- Eight failure-injection executables pass: external/owned × forced/paced ×
  optimized/UBSan-bounds. They inject host allocation failure, socket/task/terminal
  admission failure, terminal-leave failure and fault-rendering failure. Each
  asserts scope balance, exactly one buffer free where applicable, release before
  free, and successful subsequent invocation. Injection wraps host calls and
  preserves actual helper cleanup; it does not add production hooks.
- Four lifecycle executables pass against the integrated host: forced/paced ×
  optimized/UBSan-bounds. They cover allocation zeroing, fault/re-entry, invalid
  entry admission, incumbent ownership, nested entry refusal and repeated
  independent-thread invocations. Existing fixture code is reused.
- Five policy tests pass: no implicit inherited ownership, a single explicit
  define, split flags/unrelated flags, recipe undefines, and rejection on the unqualified platform.
- The disabled host object is byte-identical to `71d448f2e` under the recorded
  macOS native flags. The enabled host object is byte-identical to the previously
  measured generated ownership experiment. No new capacity trial is needed to
  establish host-code equivalence; this is not a fresh full-bundle speed claim.
- Two bundles are staged with coherent existing compiler/library inputs and
  current runtime/backend/helper sources, one forced and one paced. Four ordinary
  invocations (task and collection samples in each bundle) and four server
  invocations (plain/JSON in each bundle) pass through the installed CLI with no
  helper overrides. Sixteen responses check status/body, including UTF-8 JSON;
  all server completion counts, failure counts and control shutdowns pass.

The full packaged-source verifier was also attempted. It stopped before JIT
execution because the retained bundle's collections source still uses old Option
imports/cases against its current Optional contract. The failed report is retained;
this is not presented as full packaged-sample acceptance or a releasable bundle.
The fallback consumer checks use the current admitted assemblies from the existing
mode/HTTP corpus, validate the staged manifest hashes and use the installed CLI's
default dependency/helper selection. This slice does not repair unrelated packaged
source or the earlier JSON DOM admission gap.

## Comparison, scope and remaining work

Reuse the [.NET/Java/Go allocator research](../../../docs/design/allocation-metadata-runtime-comparison.md):
explicit ownership amortizes bookkeeping without enlarging object headers. Unlike
CoreCLR allocation contexts, this arena has a strict same-thread operational
contract; its benefit does not establish general cross-thread object support.
The measured benefit and remaining JIT/AOT gap stay in the prior experiment.
This slice makes that existing path selectable and qualifies admission/cleanup.

Public managed APIs, emitted code, compiler bridge and runtime CLI semantics are
unchanged. The new preparation option and manifest field are development tooling.
No bundle was installed globally or published. Follow-ups include fixing packaged
source coherence, broader helper/Windows qualification, and isolating warm JIT
long-range call/layout costs while retaining relocation correctness.

## Evidence and reproduction

`qualification.json.gz`, `equivalence.json.gz`, both preparation reports,
`consumers.json.gz`, `packaged-sample-failure.json.gz`, `paced-recipe.json.gz` and
`validation.json` retain commands, outcomes, source/artifact hashes and manifests'
policy selections. Baseline is `71d448f2e`. The paced recipe is derived from the
passing external-helper build in the prior ownership experiment and records that
source report's hash; it preserves the existing pacing flag.

Use the matching retained compiler/library bundle and host recipe documented by
that experiment, with fresh output directories. Example owned staging:

```sh
python3 scripts/prepare-jit-bundle.py --bundle target/uri-efficiency-bundle/bundle --runtime target/release/neoclr --jit-compiler target/release/neoclr-codegen --host-recipe target/primitive-comparison-build/build.json --heap-bytes 1048576 --byte-array-limit 65536 --owned-heap --output target/owned-jit-bundle
gzip -dc benchmarks/execution/owned-jit-host/paced-recipe.json.gz > target/owned-jit-paced-recipe.json
python3 scripts/prepare-jit-bundle.py --bundle target/uri-efficiency-bundle/bundle --runtime target/release/neoclr --jit-compiler target/release/neoclr-codegen --host-recipe target/owned-jit-paced-recipe.json --heap-bytes 1048576 --byte-array-limit 65536 --owned-heap --output target/owned-jit-paced-bundle
python3 benchmarks/execution/owned-jit-host/verify.py
python3 benchmarks/execution/owned-jit-host/consumers.py
python3 -m unittest discover -s scripts -p test_jit_host_policy.py
```

`consumers.py` also expects the separately staged paced bundle named in its source;
the commands above prepare both policies, and `paced-recipe.json.gz` preserves
the reusable host command.
The local verification harness uses retained artifacts, not a clean-checkout
installer. Xcode DEVELOPER_DIR/SDKROOT must select the recorded native toolchain.
