# Native source HTTP gate — 2026-10-05

The complete HTTP source group compiles and executes through native metadata.
The driver compiles 17 unchanged production sources (Networking, Web and Uri) plus
two native service adapters into Http.dll, referencing the cumulative 111-source
System library. Consumers reference those artifacts without their library sources.
This establishes callback-based HTTP integration, not full-System bootstrap or native
async state-machine emission.

## Reproduction and evidence

Build the cumulative library and explicit dependencies using the
[cumulative library gate](cumulative-library-type-budget-2026-10-05.md) and its linked
bootstrap instructions. Then run:

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_http.py \
  --compiler /path/to/rvnc.dll --library /path/to/Numbers.dll \
  --ownership /path/to/cumulative-tasks/ownership.json \
  --seed /path/to/System.neox --core /path/to/IntrospectionCoreParams.dll \
  --runtime target/release/neoclr --output /tmp/http-consumers
```

The output directory must be new. The [recorded evidence](source-http-2026-10-05.json)
contains all 25 successful commands, stdout/stderr, source and artifact SHA256 hashes,
compiler/runtime revisions and tool versions. Tested Raven revision: 48ff053196;
NeoCLR: e076c118. The tooling/fixture changes are in the commit containing this record.
The compiler uses native semantic references, explicit ownership and retained seed;
no application reference is projected to CLI metadata. Execution uses the documented
100-million-instruction budget and the release runtime. On this machine the Rust build
required SDKROOT pointing to Xcode's MacOSX26.2.sdk because the automatically selected
27.0 SDK was incompatible with the installed linker. No system setting was changed.

Seven compiled/verified/executed consumers check exact output: headers, base addresses,
request association, JSON client helpers, routes, captured receiver identity and property
pattern evaluation. The native cancellation harness checks headers/body cancellation and
subsequent request reuse. Status checks cover valid and malformed responses, EOF-delimited
bodies and a source-built server (eight valid and two rejected responses). All eleven
stream-upload cases pass, including cancellation, stream errors, premature EOF and
ownership/disposal. The loopback harnesses retain their deadlines and live=0 resource
assertions; cancellation/status also run their existing .NET controls.

## Fixes and regression evidence

- Generic callback method binding preserves constructed arguments in existing metadata.
- Shared propagation lowering preserves receiver/argument order and early return.
  Its independently tested fix is integrated into local Raven main e33591945:
  31 propagation/runtime-contract tests pass. Integration propagation checks: 23 pass.
- Lexically nested closures retain captured self identity and private access without
  widening unrelated access (7 accessibility and 5 nesting tests pass).
- Native reference property patterns evaluate getters once, short-circuit and bind
  payloads. The focused regression and 59 existing shared-body checks pass.
- Boxed Int32-backed enums preserve nominal equality and existing formatting/hash behavior;
  focused enum display/equality regressions pass.
- Entry metadata accepts Main(string[]) using the ordinary CLI signature. Startup passes
  bounded managed user arguments, excluding argv[0]; Environment retains the full list.
  Three runtime argument tests and 155 C# metadata groups pass; a C#-authored image loads
  and returns the expected argument count.

The JSON fixture now checks supported Int32 mapping and uses Byte to test unsupported
mapping before side effects. The status fixture no longer treats a missing Content-Length
as malformed: the decoder already supports EOF-delimited bodies. It tests that behavior
explicitly and uses conflicting lengths for the malformed case. These correct stale
expectations; production HTTP sources were not rewritten to bypass compiler failures.

## Remaining scope

Native async state machines and imported method-group binding remain unsupported;
async-heavy historical HTTP samples are not claimed passing. Generic/value receiver
closures and value/field property patterns are separate emission gaps. Existing protocol
limits are unchanged. Full 166-file System source/bootstrap ownership and dual-target
source-library parity remain open; separately compiled source groups do not establish a
single full-System build.

Broader runtime checks found existing Array<T>/Object default display/equality/hash
instantiation failures (type parameter index outside arguments). Both display and equality
failures were reproduced with the boxed-enum changes removed; the remaining object tests
passed (8/9 display, 35/36 equality). Track this generic dispatch gap for full-library
expansion. Guest API snapshot validation still reports the previously recorded stale
snapshot; no subset replaced it. Host API XML was updated. No website build, publication,
performance claim or benchmark is part of this gate.
