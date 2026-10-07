# Independently built Runtime candidate — 2026-10-07

The ordinary Raven native driver compiles 174 source inputs to `System.Runtime.dll`
(5,079,040 bytes), excluding all Data, Networking and Web sources and the two native
network adapters. Source Object supplies GetType, so the seed-only ObjectIntrospection
adapter remains excluded. This is the first executable boundary from the
[library inventory](library-boundaries-2026-10-07.md), not a final minimal core.

The unchanged `application-order-collections` then compiles against the emitted
native reference with all library sources absent. Runtime verification passes for
1,767 IL functions (maximum stack 7); execution matches the checked-in expected
output, with empty stderr and exit 0. Missing and conflicting Object owner selections
both reject before publishing output. No compiler, metadata or runtime changes were
needed. Existing .NET behavior is unaffected; this packaging check runs NeoCLR only.

[Recorded evidence](runtime-split-2026-10-07.json) includes source selection/hashes,
compiler and runtime hashes, commands, generated ownership and retained seed hashes,
artifact hashes, diagnostics and output. Compiler source contract: Raven `3e5406d8f`;
actual executable snapshot is identified by hashes. Source/runtime baseline is neoCLR
`1365c231` plus this audit/acceptance tooling change.

## Reproduce

Use the explicit primitive core and released seed/libraries from the existing
bootstrap setup; the released libraries are hashed audit control inputs, not Runtime
build references. Replace paths with the corresponding local artifacts:

```sh
python3 scripts/audit-native-bootstrap.py \
  --compiler /tmp/imported-root-compiler1007/rvnc.dll \
  --compiler-revision 3e5406d8f --core /tmp/failure1006b/Core.dll \
  --seed /tmp/preview12-bootstrap/System.neox \
  --libraries /tmp/native-poc-libraries1005 --runtime target/debug/neoclr \
  --output /tmp/runtime-split-new --case runtime-owned

python3 docs/experiments/extended-cli-metadata/verify_source_owned_orders.py \
  --compiler /tmp/imported-root-compiler1007/rvnc.dll \
  --compiler-revision 3e5406d8f --core /tmp/failure1006b/Core.dll \
  --audit /tmp/runtime-split-new --runtime target/debug/neoclr \
  --output /tmp/runtime-split-orders-new --case runtime-owned
```

The audit writes the selected ownership manifest, compiles the candidate, then
finalizes the retained service seed with its exact emitted dependency identity.
The consumer selects `--object-library System.Runtime`; runtime loading selects
that same assembly through `--object-root`. Primitive CLI bootstrap and retained
native service seed remain required. Namespace partitioning here selects sources;
it does not introduce namespace semantics or automatically split encoded assemblies.

Next compile Data and Networking independently against this Runtime, then Web.
Their accessibility, linking and execution are unproven by this result. Project/LSP
catalogs and release layout still use the earlier bundle. The website's existing
aggregate-development statement remains accurate; no new public APIs or published
capabilities are introduced by this development packaging evidence.
