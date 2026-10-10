# Runtime attribute usage (development)

The Raven runtime now owns `System.AttributeTargets` and sealed
`System.AttributeUsageAttribute`. The flags use .NET values; `ValidOn` is read-only,
`AllowMultiple` defaults to false and `Inherited` defaults to true. The latter two
properties can be changed on an instance. The policy annotation on AttributeUsage
itself permits classes only. This does not implement inherited guest queries or
add emission support for every target named by the enum.

Run `verify.py --bundle BUNDLE --aot AOT --runtime VM --output NEW_DIRECTORY`.
It compiles the actual source-library consumer, runs a copied native executable in
a clean environment, compares interpreter output and checks zero live objects.
It checks all flag values, composition, defaults, mutation and independent instances,
then compiles an invalid imported annotation and requires RAV0502 with no artifact.
The Windows collections workflow runs the same gate; Windows qualification is pending.

The temporary primitive CLI bootstrap previously duplicated both declarations.
`prepare-native-bootstrap.py` now asks the metadata translator to remove those
scaffolds and their type-level policy annotations before source-library compilation.
The bootstrap remains metadata-only; constructors are never run. Explicit ownership
requires one runtime declaration. Reusing an unprepared old Core.dll is unsupported
for this slice. The complete API reference still retains both declarations.

Raven 0f09c350ab9d4dc6eeea021af91d31d6d6719560 separates bound attribute data from
usage validation, preventing recursive validation when AttributeUsage describes itself.
Its 24 focused .NET attribute tests pass, including self and mutual annotation cases.
No new Runtime Contract option is introduced; ordinary .NET lookup is unchanged.
Native metadata and eventual removal of the primitive CLI scaffold replace this
preparation step. Separate-library Attribute inheritance and TestAttribute discovery
remain subsequent work.

See [validation.json](validation.json) for the local input hashes and cases.

The [named-data regression](named-regression-validation.json) also passes with
this compiler and prepared bootstrap, including explicit retention failures and
no attribute constructor/accessor roots. Bootstrap preparation was checked for
byte-identical repeat preparation and rejection of in-place/existing outputs.
