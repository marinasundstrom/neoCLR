# Value auto-property construction — 2026-10-05

`application-types` now compiles through the native driver and executes unchanged,
printing exactly `42\n99\n7\n42\n7\n`, with exit zero and empty stderr. This proves class
alias identity, independent struct copies and collection value-copy storage. The broad
order-collections consumer also still executes with its checked-in expected output.

The portable constructor path initializes an owned auto-property backing field directly
when a value type is under construction and the receiver is explicit/implicit self.
Previously it called the setter, which correctly failed construction-receiver escape
validation. Auto-properties have no user setter behavior to skip. Custom setters, other
receivers, reference constructors and ordinary writes are not redirected. No metadata
encoding or runtime validation was weakened.

Compared with the .NET backend, this is the native representation of initializing the
same property state, not a new source-language rule. The ordinary .NET generator remains
in place and its conservative portable profile still rejects value owners. Canonical
property/accessor identity and .NET execution are checked in both Debug and Release.

## Evidence

- [Native sample commands, hashes and output](native-value-properties-2026-10-05.json).
- [C# driver probe on both targets](value-properties-dual-target-2026-10-05.json): explicit
  and implicit self initialization, subsequent mutation and independent property copies,
  alongside existing generic payload checks. Both processes return 42 with empty output.
- All 17 focused auto-property, constructor, value receiver and struct semantic tests pass.

The C# probe's old negative boxing expectation was stale: native boxing exists but requires
an explicit System binding. It now verifies that binding error and no output publication;
this correction does not claim new boxing support. That probe uses the checked-in CLI
bootstrap core, separately from the native library consumer's primitive/ownership inputs.
Both configurations and hashes are preserved in their evidence; they are not interchangeable.

Reproduce the native consumer with `scripts/check-native-poc-samples.py --case
application-types --case application-order-collections` and the explicit compiler/core/
seed/ownership/native references and runtime from the evidence. Reproduce the dual-target
probe from Raven:

```sh
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --source-value-driver /path/to/rvnc.dll /path/to/neoclr \
  /path/to/neoclr/api-docs/reference/NeoCLR.CoreProbe.dll /tmp/fresh-value-check
```

Nine of ten original POC samples now have compilation/execution evidence, reusing the
unaffected prior async and HTTP results. The remaining original sample is inheritance:
abstract base types, ordinary virtual/override slots and interface dispatch through a
base reference. Keep the separate ordinary .NET field-return candidate for main validation
and isolated repair. This fix changes a portable path absent from local main; it does not
require an ordinary .NET behavior backport. No public APIs or guest snapshot changes were
introduced; the website development note was reviewed without an unrelated site build.

Compiler implementation: Raven `36dad5820`. Evidence records its parent revision plus
exact tested binary hashes; no runtime rebuild or source changes were required.
