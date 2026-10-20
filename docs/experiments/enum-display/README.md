# Native enum general formatting

Development follow-up, 2026-10-20. Int32-backed enums now use their names and values
for Object.ToString, including direct Raven calls inherited from Object. This closes
the conservative rejection recorded in the earlier [enum equality qualification](../enum-object/README.md).
No public overload, enum representation, interface or host ABI is added.

The [.NET general-format contract](https://learn.microsoft.com/en-us/dotnet/api/system.enum.tostring?view=net-10.0)
provides names for named values, a comma-separated representation for decomposable
flag values, and numeric text otherwise. Alias-name choice is unspecified by .NET.
neoCLR preserves its existing interpreter policy: prefer the first declared alias,
then choose nonzero masks in descending unsigned-value order and display the selected
names in ascending order. Exact names take precedence over decomposition. Unknown
bits, unnamed zero, negative values and Int32 endpoints use invariant integer text.

## Shared implementation and allocation

Native display reads the existing enum metadata and ordinary box payload. The
interpreter and native emitter share mask ordering and alias selection through the
existing internal metadata bridge. Numeric fallback reuses the integer formatter;
flag text reuses the same one-result materialization service as StringBuilder.
There is no enum-specific allocator, host callback or managed carrier.

Named values return immutable image text. For flag decomposition, the generated
function uses bounded stack scratch for at most 32 image-text pointers: each accepted
nonzero mask clears at least one remaining bit. The private initialized String-array
descriptor is borrowed only by the existing join service and never escapes into guest
code or the heap. Its pointers all refer to image literals, so it hides no managed
owners from collection. Selected names are copied into one final UTF-8 String.
Integer fallback also allocates only its result. Ordinary Object conversion still
allocates the enum box; this is not a boxing-elimination claim.

The native regression checks arena footprints for a fresh boxed value: 16 bytes for
named `Read`, 37 for `Read, Execute`, and 25 for unnamed `8`. The latter include the
16-byte box and one length-prefixed String (13 or 1 UTF-8 bytes). This verifies the
absence of managed intermediate parts arrays/text in the arena profile, not a general
throughput comparison or GC-header accounting.

## Validation and limits

`Main.rvn` covers named values, aliases, zero, negative/unnamed values, composite and
overlapping flags, unknown bits, the sign bit, all 32 selected names, direct/boxed
calls, repeated formatting and a retained result across explicit collection. The
same source passes interpreter, macOS ARM64 JIT and AOT; commands and exact artifacts
are in [validation-20261020.json](validation-20261020.json).

The [native enum regression](native-tests-20261020.txt) passes integer boundary hashes,
nominal identity, all 32 flag names and allocation footprints. The public API signature snapshot is unchanged;
the Object manual reference and website coverage notes describe this development
behavior. No website publication or replacement-release qualification is implied.

Other enum underlying types, explicit format specifiers, general record-component
formatting and boxed floating formatting remain separate gaps. The old private
EnumFormat reflection service is not newly admitted by this boxed-dispatch change.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/enum-display/EnumDisplay.rvnproj \
  --bundle target/enum-display-bundle/bundle \
  --aot target/enum-display-bundle/bundle/bin/neoclr-codegen \
  --output target/enum-display-final-app
python3 docs/experiments/enum-display/check.py \
  target/enum-display-bundle/bundle target/enum-display-final-app \
  --output docs/experiments/enum-display/validation-20261020.json
```
