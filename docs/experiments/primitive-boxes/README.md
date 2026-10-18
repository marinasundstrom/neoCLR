# Scalar box ancestry and identity

## Complete scalar-box ancestry profile — 2026-10-18

Native value-box lowering now admits all twelve numeric primitive kinds, Boolean
and Char. Additional primitive tags use the existing rooted record-payload path;
they retain exact primitive identity for type tests, unboxing and GetType, and
inherit the verified ValueType foundation. Equal payload widths never imply equal
types. Char keeps neoCLR's grapheme representation and is traced through the
existing managed payload layout. This is a backend repair; Raven syntax, Runtime
Contract configuration and CLI encoding are unchanged.

The interpreter/JIT/AOT regression in neoCLR's `docs/experiments/primitive-boxes/`
checks all fourteen scalar kinds, signed/narrow values, floating-point payloads,
wrong-kind type tests and a Char box retained across repeated native-GC allocation
points. The existing default Object-display limitation for the newly admitted
primitive kinds is explicitly diagnosed instead of exposing synthetic box names.
Void/unit, erased Value and RuntimeTypeHandle are outside this scalar-box profile.

`baseline.json.gz` records interpreter success and the original native SByte boxing
rejection. The consumer uses explicit integer widening for payload comparison and
Char.Equals so independent comparison-operator admission limits do not obscure
boxing checks. These comparison limits are not changed by this slice. The final
consumer also checks `10b`, `255b`, `42L`, `1.5f` and `1.5d` literal types through
boxing and payload checks; decimal literals are outside this runtime profile.
The retained baseline predates these equivalent suffix spellings.
`enum-results.json.gz` rechecks the existing enum consumer with the new backend.

Run `verify.py --help` for artifact inputs. The compiler/library cohort is the same
as the preceding primitive ancestry slice; only the native backend changes here.
Source commands, hashes and output are retained in `results.json.gz`. The native
AOT executable uses the host's runtime services and contains precompiled code.
