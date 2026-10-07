# Imported Object authoring — 2026-10-07

The metadata API now selects an explicit output-owned external Object reference
before authoring signatures. Raven `d920e57c0` supplies that identity from the selected
root's semantic symbol and host artifact contract, before primitive-bootstrap signatures
are mapped. Readers/importers are not reopened by emission.

The [public host API](../../../api-docs/experimental-metadata.md#external-native-object-authoring-development-2026-10-07)
uses the existing scoped native module/type aliases, preserving the canonical
System.Object native name and exact assembly identity. No format-version bump or new
runtime instruction is required. CLI reference signatures retain ELEMENT_TYPE_OBJECT;
executable CLI writing rejects native root selection. Root shape/slot admission remains
an explicit runtime responsibility; setting an authoring identity does not validate an
arbitrary dependency's contents.

[Evidence](imported-object-authoring-2026-10-07.json):

- All 165 C# metadata groups pass, plus the focused root checks after adding authored
  external method-reference and reference-projection coverage. Manual definitions and
  builders share validation; unselected, foreign and conflicting roots reject.
- The API-authored consumer loads the separately built Runtime and executes Equals,
  returning 42; verification succeeds for 1,753 IL functions.
- The [ordinary Raven consumer](bootstrap/imported-object-override.rvn) compiles against
  that Runtime without library sources, verifies and returns 42. It exercises a source
  override with an imported Object signature and an observable result.
- The public API snapshot check passes. No guest API snapshot or website capability
  change is needed; host APIs are documented in the existing manual reference.
- Separate Networking compilation advances from an override-builder exception to an
  encoding diagnostic for **System.Value**, with no published output. Data's independent
  internal array-reflection boundary remains open.

## Reproduce

Use the Runtime candidate and finalized retained seed from the prior split gate.
Substitute local paths to matching artifacts; hashes in the evidence identify those
actually tested.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --object-roots

dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --external-object-consumer /tmp/runtime-split1007/runtime-owned/System.Runtime.dll \
  /tmp/failure1006b/Core.dll /tmp/external-object-new.dll

dotnet /tmp/external-root-final1007/rvnc.dll neoclr \
  --core-reference /tmp/failure1006b/Core.dll \
  --runtime-seed /tmp/runtime-split1007/runtime-owned/System.runtime.neox \
  --bootstrap-ownership /tmp/runtime-split1007/runtime-owned/ownership.json \
  --reference /tmp/runtime-split1007/runtime-owned/System.Runtime.dll \
  --object-library System.Runtime -o /tmp/external-raven-new.dll \
  docs/experiments/extended-cli-metadata/bootstrap/imported-object-override.rvn
```

Verify and run each output with `target/debug/neoclr verify` / `run`, passing
`--system System.runtime.neox --module System.Runtime.dll --object-root System.Runtime.dll`
using the full artifact paths above. Verification exits 0; execution exits 42 with no
stdout/stderr. Keep these explicit dependencies together; these are not standalone
bootstrap-free binaries.

External boxing/service contracts and broader inherited class slots are not completed
by root selection. The next optional-library blocker is imported Value ownership;
continue from the real Networking encoding diagnostic rather than relaxing metadata
validation or substituting platform services.
