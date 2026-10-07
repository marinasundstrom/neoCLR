# Native-width integer metadata prerequisite — 2026-10-06

The Console bootstrap audit exposed an absent metadata category: the runtime already
supports IntPtr/UIntPtr but the host writer admitted only fixed-width integer primitives.
The library now preserves native-width method, field, property and local signatures
through CLI/native reading and introspection, and emits conv.i/conv.u through ILGenerator.

This follows [ECMA-335 sixth edition](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
II.23.1.16 (ELEMENT_TYPE_I/U) and III.3.27 (conversions). Like .NET, these signatures
retain target width rather than promising 64-bit storage. There is no intentional
representation divergence or new metadata schema. Native arithmetic admission and
source primitive ownership are not part of this prerequisite.

Validation: 161/161 C# metadata groups pass. NativeIntegerChecks verifies actual CLR
execution and native/CLI signatures, locals, fields, properties, facade results and
rejection of nonnumeric conversions. Its exported native artifact verifies and exits
42 with target/debug/neoclr. Signed -42 and unsigned 2^32 on this 64-bit host exercise
more than a positive Int32 round trip; the test selects 42 on 32-bit hosts, which have
not been qualified by this run. Guest API snapshot checks pass.

```sh
NEOCLR_NATIVE_INTEGER_ARTIFACT=/tmp/nativeintegers.dll \
  dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
target/debug/neoclr verify /tmp/nativeintegers.dll
target/debug/neoclr run /tmp/nativeintegers.dll # expected exit 42
```

Raven's target mappings and ownership catalog still need to admit these categories;
Console has not yet passed its source-built gate. The full-System diagnostic baseline
remains 35 from the calendar audit. This is metadata infrastructure progress, not a
claim that the next source group compiles. No Raven shared fix is introduced here.
