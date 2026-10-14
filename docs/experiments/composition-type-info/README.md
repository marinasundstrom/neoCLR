# Structural descriptor probe

Development-only on `codex/structural-union-intersection`, with
`experimental-compositions` and a matching native System.Runtime source build.
This validates UnionTypeInfo and IntersectionTypeInfo through ordinary Raven
TypeInfo acquisition, type tests, Types enumeration, equality and assignment.
Mixed nesting and a structural generic argument exercise both factory construction
and runtime snapshot-provider selection. Repeated enumeration exercises allocation.

The fixture replaces four explicit placeholder type tokens after compilation.
Current Raven typeof(A | B) produces a nominal Union<A,B> token; this probe does
not qualify source-authored structural typeof. It changes no consumer call, cast,
property or enumeration body. `examples/composition_info_fixture.rs` asserts the
exact replacement count and preserves native metadata source identities.

Build `neoclr` with `cargo build --features experimental-compositions --bin neoclr`,
then run from the repository root:

```sh
python3 docs/experiments/composition-type-info/verify.py \
  --compiler target/preview15-extracted-macos/neoclr-native-poc/sdk/tools/rvnc/rvnc.dll \
  --library target/composition-info-libraries-v2 \
  --output target/composition-info-validation
```

The library was built by `scripts/build-native-class-library.py` with compiler
revision `793219d855b17f01fb805e215968b78871eb3804`. The runner records hashes for
the actual runtime, native fixture, source and library inputs. Regenerate the
library from matching sources rather than mixing a published bundle with these
new descriptor contracts.

Direct composition member, hierarchy and custom-attribute queries fault explicitly.
No nominal definition token is invented. Open generic compositions, structural
source emission, legacy CLI provider generation and JIT/AOT execution remain
unqualified. Closed reference admission and final-class intersection restrictions
are unchanged. Default builds keep the composition feature gate.

The native-source consumer passes (exit 0, `Composition descriptors passed`).
[Retained evidence](results.json) records the exact runtime/library/source hashes.
Only System.Runtime is loaded because it is the probe's only library dependency.
An initial run loading unrelated Data/Networking/Web libraries exceeded its
180-second bound; it is not counted as successful validation.

Focused checks: all 37 structural execution tests, the internal identity/constituent
query check, seven native reflection-lowering tests and four native API snapshot
tests pass. The API reference fingerprint check also passes.
