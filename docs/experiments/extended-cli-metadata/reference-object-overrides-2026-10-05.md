# Object slots for source descriptors — 2026-10-05

The production descriptor classes override Equals, GetHashCode and ToString. This slice
adds those exact slots to metadata authoring and Raven's native capability contract.
It does not admit arbitrary virtual classes or generic reference-owner overrides.

## Contract and comparison

CLI flags reuse inherited slots exactly as on .NET. Equals parameters use the dedicated
CLI Object signature element; encoding a CLASS reference to System.Object caused real
CLR override dispatch to fall back to Object.Equals, found by the new C# test. Native
rootless classes already have an Object view without physical Object ancestry; runtime
dispatch now searches actual local base lineage for explicit overrides before taking
the Object default. Values retain their existing managed-box receiver path. Invalid
contracts still reject and closed dispatch analysis includes inherited targets.

The explicit retained seed adds Object.Equals as reference identity using the existing
ObjectReferenceEquals runtime service. The checked-in union seed and derived retained
seeds must include this slot; the emitter validates it rather than assuming availability.
No duplicate descriptor types are introduced. Imported member references are authored
from Raven symbol facts and host dependency identities, not importer definitions.

## Reproduction

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_object_overrides.py \
 --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
 --core /path/to/Core.dll --seed /path/to/System.neox \
 --ownership /path/to/ownership.json --base-library /path/to/Numbers.dll \
 --output /tmp/object-overrides
```

The library is compiled first; the consumer references its artifact with no provider
sources. Native verification succeeds and execution returns 42 with empty stdout.
It checks Object.Equals for matching/mismatching runtime classes, display, hash, a
derived override and inherited behavior. C# tests cover definition/builder parity, actual
CLR execution and rejection of another core's Object identity. 150 metadata groups and
32 focused runtime boxing/inheritance/dispatch tests pass. The guest API snapshot
check retains its recorded stale-snapshot failure; public host API changes are documented
in the manual reference. No website production capability claim changes.
[Commands, revisions and artifact hashes](reference-object-overrides-evidence-2026-10-05.json).

Production binding/emission now reaches the Flags enum attribute requirement. Descriptor
materialization and the full JSON mapping gate remain open.

Compiler commit: `11409a822`. Of 44 focused .NET override tests, 43 pass. The nullable
`default` return in `Emit_StaticInterfaceImplementation_EmitsMethodOverrideMapping`
also fails at unchanged integration HEAD `1a1759d43`, before this slice, but passes on
main `e1df355a2`. The existing arrow-body binding difference is recorded in Raven's
integration docs for independent assessment; native slot work does not change binding.
