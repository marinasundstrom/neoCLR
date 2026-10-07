# Imported Object slot calls — 2026-10-07

Raven can now emit calls to the selected imported Object root's ToString, GetHashCode
and Equals slots using only semantic signatures and output-owned metadata references.
These references require virtual dispatch; they are not mislabeled as override or
interface declarations. Actual dependency/slot admission remains a runtime check.

[Evidence](imported-object-slots-2026-10-07.json):

- The [Raven consumer](bootstrap/imported-object-slots.rvn) uses an `object` receiver
  holding a source Item. All three calls reach Item's overrides. Compilation against
  the separate Runtime succeeds; runtime verification exits 0 and execution exits 42,
  with no stdout/stderr. The combined set has 1,755 IL functions.
- The API fixture now calls the imported Equals slot rather than the local override
  directly. It verifies and returns 42; the root's default implementation would return
  false, so success proves dispatch to the derived override.
- All 165 C# groups pass. Added checks cover slot interning, virtual category, invalid
  signatures, incompatible ordinary/virtual contracts and rejecting Call for a slot.
  Receiver validation admits ordinary reference classes to the selected root without
  admitting unboxed values. API snapshot validation passes.
- Networking advances to `CheckedStorage.Reserve<T>` mapping: the bootstrap declaration
  cannot be validated against a matching retained native type. It still publishes no
  output. Networking execution and Data's array-reflection boundary remain open.

Source baseline: neoCLR `45d76411` plus this change; Raven `1c2ccd638`. The evidence
records actual compiler, dependency, source, runtime and output hashes. The Raven
compiler snapshot preceded the final API-fixture receiver-validation addition; the
C# suite/API runtime fixture validate that addition separately. No .NET backend or
runtime implementation change is made.

## Reproduce

Build matching compiler/metadata artifacts and use the established Runtime candidate:

```sh
dotnet /tmp/object-slot-compiler1007/rvnc.dll neoclr \
  --core-reference /tmp/failure1006b/Core.dll \
  --runtime-seed /tmp/runtime-split1007/runtime-owned/System.runtime.neox \
  --bootstrap-ownership /tmp/runtime-split1007/runtime-owned/ownership.json \
  --reference /tmp/runtime-split1007/runtime-owned/System.Runtime.dll \
  --object-library System.Runtime -o /tmp/object-slots-new.dll \
  docs/experiments/extended-cli-metadata/bootstrap/imported-object-slots.rvn
```

Run `target/debug/neoclr verify` and `run` with that output, the retained seed as
`--system`, and Runtime as both `--module` and `--object-root`. Full commands are in
the evidence. Verification exits 0; execution exits 42. Generate the API fixture with
`tools/metadata/NeoCLR.Metadata.Experimental.Tests --external-object-consumer` using
the same Runtime/core and a fresh output path, as in the earlier root-authoring gate.

The [host API reference](../../../api-docs/experimental-metadata.md#imported-object-slot-references-development-2026-10-07)
documents the bounded contract. General imported virtual class methods are not enabled
by this change. No new native instruction or metadata format version is introduced.
