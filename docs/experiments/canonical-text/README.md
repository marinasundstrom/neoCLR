# Canonical public text qualification — 2026-10-19

Development qualification on macOS ARM64, not an extracted release-package check.
[Results](validation.json) record six passing executions and artifact hashes.

Public.rvn exercises typed and boxed String/Char canonical equality, matching
hashes, canonical order, constant patterns, explicit ordinal/simple-fold policies,
collection keys, original UTF-8 bytes and ToString owner/spelling preservation.
JSON and SQL names, paths and URI identity remain ordinal. HashCode.Add(string)
uses the canonical service without an intermediate encoded array.
NullReceiver.rvn requires an ordinary NullReference termination in all engines;
a native signal/abort is not accepted.

The compiler code baseline is Raven `2d22055dc`, with metadata adapter `e4709c451`;
Raven documentation revision is `b7d54e946`. Runtime baseline `f3ba4f0d4` plus the
coordinated canonical integration working tree produced the matched artifacts.
Libraries: `target/canonical-text-libraries-v9`; bundle:
`target/canonical-text-bundle-v10/bundle`. Public build v13 and null build v4 use
that bundle. Native builds use the pinned Xcode toolchain/SDK and native eager
boundary GC. No concurrent-GC or HTTP throughput improvement is claimed.

```sh
python3 docs/experiments/canonical-text/check.py \
  target/canonical-text-bundle-v10/bundle \
  target/canonical-text-public-v13 target/canonical-text-null-v4 \
  --output target/canonical-text-validation-final.json
```

Use build-native-class-library.py, prepare-jit-bundle.py and
build-native-project.py with a matching compiler/metadata adapter to recreate the
artifacts; see the repository native build workflow. Do not mix released libraries
with development text overrides. The unchanged Unicode 17 kernel has separate
20,034-row conformance and allocation evidence in the String review. Focused Rust
checks pass: 37 object-equality tests and two root-object override tests.
API snapshot validation, four native snapshot tests and the local website build
(4,544 pages) pass. Website publication remains separate.

## Existing record compiler gap

RecordGap.rvn reproduces native positional-record generated equality rejection.
Compile it in place of Public.rvn using the same project settings. The native
emitter explicitly exports record storage/deconstruction only; invoking generated
`==` reports NEOMETA001, an unregistered dependency on the current assembly.
Generated equality, hashing and formatting remain unqualified. This is not a
canonical-text regression or a successful record contract test. A future compiler
slice should implement those helpers and improve the rejection diagnostic.

Other limits: canonical ordering is NFD scalar order, not linguistic collation;
simple folding does not normalize; hashes are not stable persistent identifiers.
Canonical dictionary identity changes from the previous ordinal default. Use
StringComparer.Ordinal where exact spelling is significant. Char extraction may
still allocate; this work avoids normalized copies, not all text allocations.
