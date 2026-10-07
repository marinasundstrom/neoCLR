# Imported erased Value ownership — 2026-10-07

The selected Runtime now owns System.Value throughout emission of bootstrap helper
calls. Previously the adapter demanded a competing seed declaration, then rejected
selected external Value as lacking a legacy dependency binding. The metadata library
now maps bootstrap Value references to the explicitly selected external carrier and
matches its canonical storage tag in retained runtime-service signatures. Native
module/type aliases preserve exact assembly ownership without a format change.

Raven `74c50ad2f` establishes this selection before importing callable signatures,
using the existing semantic erased-carrier contract and native artifact identities.
It does not reopen importer objects or introduce a CLR SpecialType. Ordinary .NET
behavior and unselected nominal types are unchanged.

[Recorded evidence](imported-value-2026-10-07.json):

- All 165 C# metadata groups pass. New cases cover both producer authoring paths,
  selected external Value parameter/return introspection identity and competing-owner
  rejection. The API snapshot check passes.
- The [focused Raven consumer](bootstrap/imported-value-services.rvn) calls actual
  retained ParseInt32, IsValue and UnpackValue services. It checks successful integer
  parsing, a failed parse with a byte status payload, and type discrimination. It
  imports emitted Runtime with library sources absent; verification succeeds for
  1,751 IL functions and execution returns 42 with empty stdout/stderr.
- Networking advances to `NEOMETA001` for imported virtual Object.ToString. Neither
  the previous metadata-builder crash nor Value signature rejection remains. This
  does not claim Networking/socket execution or completion of Data's reflection gap.

## Reproduce

Use the recorded compiler snapshot and the independently built Runtime/retained seed:

```sh
dotnet /tmp/external-value-compiler1007/rvnc.dll neoclr \
  --core-reference /tmp/failure1006b/Core.dll \
  --runtime-seed /tmp/runtime-split1007/runtime-owned/System.runtime.neox \
  --bootstrap-ownership /tmp/runtime-split1007/runtime-owned/ownership.json \
  --reference /tmp/runtime-split1007/runtime-owned/System.Runtime.dll \
  --object-library System.Runtime -o /tmp/value-services-new.dll \
  docs/experiments/extended-cli-metadata/bootstrap/imported-value-services.rvn
```

Verify/run the output with `target/debug/neoclr`, passing the explicit `--system`
retained seed, `--module` Runtime DLL and `--object-root` same Runtime DLL. Full
commands and dependency/source/toolchain hashes are in the evidence. Verification
exits 0; execution exits 42. This is a development service-contract probe, not a
recommendation for application code to depend directly on bootstrap RuntimeServices.

Next support imported Object slot references in the symbol-only emitter and metadata
API with actual virtual dispatch tests. Keep the service seed explicit and continue
Networking from its real sources; do not bypass the remaining virtual-call boundary.
