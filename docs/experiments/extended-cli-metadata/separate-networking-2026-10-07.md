# Separate Networking execution — 2026-10-07

System.Networking builds independently against the 174-input System.Runtime candidate.
The unchanged network-cancellation sample then compiles with only those two library
artifacts, verifies and runs with exact stdout `Network token cancellation checks passed`
and exit 0. The sample exercises pre-cancellation, asynchronous cancellation callbacks,
DNS, local socket/listener operations and disposal. This is bounded networking acceptance,
not proof of every Networking API or complete platform packaging.

## Fix and architecture

The first CheckedStorage rejection was an acceptance-tool configuration omission:
class-library compilation requires explicit `--bootstrap-intrinsics`, already used for
Runtime. Enabling it exposed boxing validation that still demanded legacy seed Object.
The metadata writer now accepts a complete selected external Object contract. Raven
supplies all three slots from symbols, not just slots encountered at call sites. Missing
slots reject before bytes are published; runtime linking validates real definitions.
This preserves the normal CLR boxing/object-slot model with explicit NeoCLR root ownership;
no new metadata encoding or change to default .NET code generation is needed.

The primitive Core bootstrap, retained services seed and exact Runtime artifact remain
explicit. No importer object is reused by emission and no native library uses CLI fallback.
Consumer compilation does not enable class-library intrinsics or include library sources.

## Reproduce

Build Raven with the matching metadata project. Using the independently built Runtime
candidate documented in [the Runtime split gate](runtime-split-2026-10-07.md):

```sh
python3 scripts/audit-optional-libraries.py \
  --compiler /tmp/root-box-compiler1007/rvnc.dll \
  --compiler-revision 2262f8e13 \
  --core /tmp/failure1006b/Core.dll \
  --runtime-library-directory /tmp/runtime-split1007/runtime-owned \
  --output /tmp/network-gate1007-retry \
  --runtime target/debug/neoclr
```

The output directory must be new. The driver records commands, diagnostics, source and
artifact hashes, rejects nonzero compile/verify/run results, and compares exact stdout.
The report retains Data's independent compilation failure without counting it as success.
[Recorded commands and hashes](separate-networking-2026-10-07.json) identify the tested
compiler binaries; Raven integration source is committed as `2262f8e13`.

Validation: 165/165 C# metadata groups; API snapshot check; incomplete external boxing
contracts reject, complete contracts round-trip. The API external-root fixture also
boxes and type-tests an integer before imported Equals dispatch and returns 42 against
real System.Runtime. No runtime change was needed for this slice. No performance claim.

## Next boundary

Data still fails to access internal RuntimeServices.ReflectionArrayLength/Get/Create;
no Data artifact is published. Introduce the supported public metadata/reflection array
boundary and consume it from ObjectMapper, then execute JSON mapping across the split.
Web and project/LSP dependency catalogs follow. This does not finish bootstrap or release.
