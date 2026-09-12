# Bounded CIL instruction decoder

Implemented 2026-09-12 as the next part of the [Raven binary experiment](raven-binary-profile.md).
`neoclr::cil::decode` reads standard CIL **code bytes**, excluding the method header.
It preserves byte offsets and module-scoped metadata tokens. It does not load a PE,
resolve a token, verify a signature or execute an assembly.

## Contract and comparison

The baseline is ECMA-335 Partition III's instruction encodings. Microsoft's
[`ldc.i4.s`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldc_i4_s?view=net-10.0)
and [`call`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.call?view=net-10.0)
document the signed short operand and method-token operand (consulted 2026-09-12).
The [.NET method-body API](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.methodbodyblock?view=net-10.0)
separates instruction bytes from locals, exception regions and other body information.
This decoder follows that separation; admission of that surrounding information remains
required before execution.

The supported subset is `nop`, `ldarg`, `ldloc`, `stloc` (compact, short and two-byte
forms), `ldc.i4` (all forms), `dup`, `pop`, `call`, `ret` and `ldstr`. Short integer
operands are sign-extended; multibyte operands are little-endian. Calls admit non-nil
MethodDef and MemberRef tokens; MethodSpec is excluded until generic input is supported.
String operands must name a nonzero user-string heap offset. These checks establish token
**kind**, not existence, identity or signature. A MemberRef can still designate a field;
the metadata layer must reject that use as a call target.

Code size is limited to 64 KiB, bounding decoded instruction count and allocations.
Empty streams, truncated operands and unsupported opcodes are errors. The entire stream
is decoded, including unreachable instructions after `ret`. No prefixes, branches,
exception instructions or object construction are admitted in this first subset.
Errors identify the starting byte offset of the offending instruction. Returning a
partially decoded body or guessing operand lengths would make later validation unreliable.

Reusing standard bytes avoids a new compiler writer. Decoding them to a small intermediate
representation separates binary framing from the existing runtime representation and
retains `nop` offsets for future diagnostics. The cost is another representation and a
subsequent binding step. This is compatibility groundwork, not a claimed performance
improvement or replacement binary format.

## Validation

Run `cargo test --test cil --test no_result`. Six decoder tests cover offsets/tokens,
integer boundaries, slot encodings, every truncation boundary in representative operands,
unsupported instructions, nil/wrong-kind tokens and the size limit. Seven no-result
regressions exercise the preceding runtime contract.

The [.NET probe](experiments/cil-decoder/Program.cs) emits two methods through
Reflection.Emit, invokes them on .NET, and records their actual IL bytes and result.
To reproduce using the pinned .NET 10.0.100 SDK:

```sh
cd docs/experiments/cil-decoder
dotnet run --project Probe.csproj
```

The checked-in `result.json` is consumed by the Rust test. Both CLR execution and neoCLR
execution of the decoded, test-bound instructions return 42. The test resolves exactly
one known token and discards `nop` during test-only lowering. It is not a production
linker and does not demonstrate loading Raven's PE artifacts.

Next: admit PE headers and metadata with explicit dependency resolution, select reachable
methods without ignoring unresolved helper metadata, bind the real System library,
validate signatures/local declarations/maxstack, and connect decoded offsets to runtime
instruction diagnostics. The decoder alone must never be treated as input verification.
