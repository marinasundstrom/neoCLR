# Enum ancestry regression

## Enum ancestry — 2026-10-18

The native target preserves the imported `System.Enum` base of application enums
and admits direct boxing to Enum through the existing native ancestry capability.
Enums retain their nominal Int32 payload identity; boxing/unboxing does not convert
them to Int32 or another enum. The runtime verifies the fieldless abstract
Enum → ValueType → selected Object chain in the foundation's defining module.
Native boxes retain the enum tag and reuse the existing rooted value-box layout.

This follows the [.NET enum contract](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/enums).
Ordinary .NET output and Runtime Contract configuration are unchanged. The temporary
CLI projection already extends Enum; native metadata now retains the resolved
foundation too. The native metadata/runtime and Raven native emitter own this repair.
The consumer and exact commands/hashes live in neoCLR's
`docs/experiments/enum-ancestry/`. Existing Int32 enum restrictions are unchanged;
primitive ancestry is the following slice.

Run `verify.py` with the six artifact/output arguments documented by `--help`.
`baseline.json.gz` retains the rejected native Enum boxing conversion.
`results.json.gz` retains matched interpreter/JIT/AOT output with native GC enabled.
Focused checks cover metadata roundtrips/rejections, foundation ownership, and the
existing enum/compiler suites. Installed Preview 16 is unchanged.
