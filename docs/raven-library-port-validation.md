# Raven library port: execution gate and branch audit

Development validation on 2026-09-19. This records the API-preserving source port;
proposal API alignment and the System.Runtime assembly identity remain subsequent
work. [Authoring status](raven-system-library.md) explicitly lists unported sources.
A runnable library and complete Raven source ownership are separate checks.

## String/Error follow-up

The separate opaque-value slice adds two source owners (49 total). All 11 admission
checks and 21 focused Rust string/error/default/interface checks pass. Native-status
injection covers successful slicing, both typed failures and an unknown-status fault.
Error.ToString retains delegation through Message. String retains method order,
receiver conventions and descriptive runtime parameter names; its Raven reference
named arguments remain unchanged. A graph budget of one now correctly rejects
String.Equals because generated Boolean adapters are reachable; service assertions
are retained with a larger fixture budget. No Raven compiler code changed; target
boundary documentation is on feature commit `981301d09`.

## Empty values and Void follow-up

Five empty error types and Void bring the total to 55 source slices. Nine admission
checks, 14 focused Rust error/calendar/clock tests, and four saved programs (error
values, generic unions, Void results and Void values) pass. Wrong-case extraction
faults and uninitialized carrier defaults are rejected. Raven main `c17cb8397` fixes
unit-contract assembly lookup independently with .NET ValueTuple (19 checks); the
neoCLR feature cherry-pick `b6f12353f` passes 24 focused target/unit checks.

## Typed error carriers follow-up

Seven carriers bring the source total to 62. All 14 admission cases pass, including
altered cases/storage, unsupported payloads, constructor side effects and invalid
defaults. All 25 focused Rust arithmetic/error/file-input/file-output/I/O/string checks
pass. All 64 saved-program cases pass against freshly generated consumer metadata and
the 62-slice library. These results supplement, rather than replace, the earlier full
Rust baseline; a full final run remains required after the remaining source ports.

## Propagation declaration follow-up

The 63rd source slice owns Propagatable. Five admission cases pass, checking exact
out metadata, payload positions and method names. No carrier body changes in this
slice. All 20 Rust propagation/union-output/generic-bound tests pass, including
non-overwriting misses and unproven-read rejection. Snapshot hashes match.

## Generic union follow-up

Option and Result bring the total to 65 source slices. All 12 admission cases pass,
including altered cases, constructor side effects, unsupported payloads, invalid
defaults, changed output names and an unassigned success return. All 41 focused Rust
error/generic-bound/result-factory/no-result/propagation/union-output tests pass.
All 64 saved-project cases pass with fresh consumer metadata and the regenerated
library. Readonly copy adapters retain source receiver promises; failed extraction
leaves destinations untouched. A final full Rust run remains required.

## Descriptor follow-up

The descriptor hierarchy brings the source total to 66 slices. Nine admission cases
check storage names/order/types, constructor visibility, sealing, exports, parameter
names and invalid defaults. All 23 distinct Rust reflection/hierarchy checks pass;
the final Raven-profile suite includes seven checks, with new array-copy independence
and nonpublic accessor coverage. A fresh consumer reference and library compile,
import, verify and run the saved reflection sample with its exact expected output.
All 64 saved-project cases also pass; the final field-name preservation adjustment
was rechecked by the seven Raven reflection tests and fresh reflection sample above.
Snapshot hashes and source/API audits pass. The legacy Neo profile remains separate.

## Native allocation follow-up

The 67th source slice owns NativeMemory overload composition. Five admission cases
check the complete export surface, parameter names/types and extra exports. All 28
native integer/allocation/pointer tests pass, including overflow, zero allocations,
null release, bounds and lifetime faults. The saved Raven allocation sample compiles,
imports, verifies and runs against the regenerated library. Snapshot hashes match.

## Terminal failure follow-up

System.Fault brings the source total to 68. Three admission cases and seven
fault/query tests pass. A newly compiled Raven program imports and verifies,
then faults with its exact Unicode diagnostic. Native guest-failure semantics and
host survival remain unchanged. Source/API audits and snapshot hashes match.

## Delegate declaration follow-up

Func brings the source total to 69 slices. Six admission cases reject missing/extra
arities, altered return/input positions and ordinary classes posing as delegates.
All 28 delegate/nominal-delegate tests pass. The saved Raven delegate sample compiles,
imports, verifies and executes all five arities, a Void callback and collection
callbacks with expected output. Snapshot hashes and source/API audits match.

## Normal BindingFlags enum follow-up

BindingFlags brings the source total to 70 slices. It is declared as a normal Raven
Flags enum. Seven admission cases cover the valid declaration, changed/missing/extra
literals, a different underlying type, a missing Flags marker and a struct substitute.
The compiled Raven flags program imports, verifies and executes with the expected
bitwise, conversion, equality and reflection-filtering results. Snapshot hashes and
source/API audits match (770 declaration candidates, 170 source files, 118 declaring
sources). All 31 enum/reflection Rust tests pass. Enum operations are intrinsic lowering of
the declaration, not handwritten managed-library methods.

Source enum emission exposed a missing RTSpecialName on `value__`. The independent
.NET PE-metadata regression failed before the fix. Raven main `266b457f5` and the
neoCLR feature cherry-pick `6219343b9` each pass 13 focused enum/target-core checks.
No Runtime Contract options or consumer enum semantics change. The host emitter's
reserved-bit masking is repaired in Raven's final metadata pass; target-specific
nominal enum lowering remains outside Raven main.

## Root and attribute marker follow-up

Object and UnionAttribute bring the total to 72 source slices. Nine source admission
cases reject storage, added methods, constructor side effects, wrong bases and
changed sealing. A fresh generic-union Raven program compiles, imports, verifies
and executes with expected results against the rebuilt library. The fieldless root
and marker ABI remain unchanged; compiler-facing Object members are recognition
metadata rather than executable stub bodies. Source/artifact hashes and ownership
audits match. All 22 attribute/ordinary-union/reflection tests pass. This adds no
compiler code or Runtime Contract configuration.

## Reproduce the program gate

Use the neoCLR feature compiler, built from the Raven repository's
`codex/neoclr-namespace-metadata` branch. General compiler fixes are developed and
validated independently against normal .NET metadata before entering Raven main.
Set `RAVEN_ROOT` to that feature checkout; commands below run from neoCLR's root.

```sh
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot="$RAVEN_ROOT"
cargo build --bin neoclr
python3 docs/experiments/raven-target/build_runtime_library.py \
  --compiler "$RAVEN_ROOT/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll" \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --check
python3 docs/experiments/raven-target/inventory_runtime_api.py --check
python3 docs/experiments/raven-target/audit_runtime_api.py --check
cargo test
```

The saved-project gate uses a fresh `--reference-core` output and a flattened
collection profile from `collection_library.py`, rather than previously installed
preview assets. See [the saved-project runner](experiments/raven-target/run_project.py).
The reference and executable library must be regenerated together after importer
changes. Published SDKs and extension packages are not updated by these commands.

Create that fixture in a new temporary directory (the profile generator refuses
to overwrite an existing file):

```sh
PORT_ROOT=$(mktemp -d)
mkdir -p "$PORT_ROOT/demo"
BRIDGE="$PWD/docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll"
dotnet "$BRIDGE" --reference-core "$PORT_ROOT/demo/NeoCLR.CoreProbe.dll"
python3 docs/experiments/raven-target/collection_library.py "$PORT_ROOT/System.neoil"
cat > "$PORT_ROOT/demo/Demo.rvnproj" <<EOF
<Project>
  <PropertyGroup><NeoCLRRoot>$PORT_ROOT</NeoCLRRoot></PropertyGroup>
  <Import Project="$PWD/build/NeoCLR.Raven.props" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>
EOF
python3 docs/experiments/raven-target/verify_project.py "$PORT_ROOT/demo/Demo.rvnproj" \
  --collections --bridge "$BRIDGE" --system "$PORT_ROOT/System.neoil" \
  --runtime target/debug/neoclr
```

Run `verify_project.py PROJECT --collections`, `verify_process.py PROJECT` and
`verify_file_project.py PROJECT`, passing `--bridge BRIDGE --system SYSTEM --runtime RUNTIME`
to each. The first compiles saved Raven sources, imports and verifies their emitted
code, checks execution output, then checks rejection without stale-output fallback.
The other two control input/environment and inspect real file bytes.
`verify_collection_capabilities.py` checks unavailable collection operations.

Additional checks:

- `verify_error_carrier_library.py --compiler COMPILER --bridge BRIDGE`: 14 carrier
  layout, case, constructor, payload and default admission checks.

- `verify_empty_library.py --compiler COMPILER --bridge BRIDGE`: nine empty-error/
  Void declaration cases, including mismatched fields and exports.
- `verify_opaque_library.py --compiler COMPILER --bridge BRIDGE`: 11 String/Error
  admission cases check storage, receivers, signatures, allocation/mutation rejection
  and initialized Error copies.
- `verify_foundation_library.py --compiler COMPILER --bridge BRIDGE`: 17 source
  admission cases, including wrong generic positions/arity/base contracts, calendar
  layout/visibility and unsupported erased payloads/defaults.
- `verify_interface_library.py --compiler COMPILER --bridge BRIDGE --runtime RUNTIME`:
  interface admission plus real fixed/system-clock consumers and host-local conversion.
- `verify_runtime_context.py --bridge BRIDGE --runtime RUNTIME`: the separate context
  POC executes source `typeof` through RuntimeContext and checks handle identity.
- `dotnet BRIDGE --library-signature-checks CORE`: nominal Void encodings agree;
  no-result returns, foreign/versioned core identities and nonempty Void are rejected.

The port retains Int32's method declaration order and parameter names for existing
introspection output. The reflection sample checks a nonnegative method definition
index rather than hard-coding its global ordinal, which changes as helpers are added.
Generated File/Int32 adapters increase the reachable function graph: two Rust fixture
budgets increase to 128, retaining explicit too-small-budget rejection. Runtime
production limits are unchanged. File input tests inject every native status and
require an unknown status to fault. The full Rust run exposed an inline managed-array
fixture that still embedded wrapper includes; it now embeds the generated contracts
because the text assembler does not resolve files. All eight fixture checks pass.

## Recorded results

- The final Rust runs cover all 178 integration-test binaries plus library/binary
  unit tests: 1241 tests pass, zero fail and zero are ignored. Documentation tests
  also complete successfully (no cases). After the inline fixture correction,
  tests run in independent batches; early groups are rerun with the final generated
  library so results cover the parameter-name fix throughout.

- All 64 saved-project cases pass, including executable API samples, saved edits,
  compiler/import rejection without stale execution, and expected runtime faults.

- All 65 slices compile and import. Inventory/coverage checks pass for 770 candidates
  and 116 declaring sources. The final clean regeneration gate remains required.
- All 17 foundation admission cases pass. Clock declaration rejection checks and
  fixed/system-clock execution pass, including host-local time conversion.
- Controlled process checks pass for fresh/copied arguments, environment values,
  missing/invalid names, input byte boundaries and EOF.
- Saved-source file checks pass for actual UTF-8 round trips, preserved contents
  after rejected writes, missing files, invalid UTF-8 and size limits.
- All 10 unavailable collection-capability checks reject before execution.
- RuntimeContext POC and signature-identity checks pass. The targeted reflection
  program preserves method parameter names after fixing the importer regression.

The Python-launched Rust batches select the installed Xcode 26.2 SDK for their
processes: Apple's Python launcher supplied a newer Command Line Tools SDK that
the active Xcode linker could not read. No global developer-tool or repository
configuration was changed for that host-only mismatch.

## Raven branch audit

Compare behavior and final file differences, not just `git cherry`: earlier general
fixes were consolidated or had documentation differences and may still appear with
`+` even though their implementation is on main. Existing main history includes
numeric conversions/operators, metadata method/field/constructor references, source
generic parameters, namespace lookup, interface dispatch, array elements and generic
unit handling. The generic-interface base-scope fix is `2e3856a6b` on main and
`0b6f11bb5` on the experimental branch; their compiler/test changes agree.

The audit also found the experimental delegate bridge fix absent from main.
An ordinary .NET regression reproduced InvalidProgramException without any target
configuration. Main commit `5a37cd56c` independently integrates that fix; all 19
focused delegate/unit tests pass. The experimental branch already has the equivalent
behavior in `8e0f6cb7d`; no target policies accompany the main commit.

The remaining target-specific policies include nominal Void value-position handling,
generic managed-array projection/covariance, propagation without CLR exceptions and
context-owned `typeof`. These remain separate from main. Existing reusable metadata,
iteration, propagation and unit configuration mechanisms on main do not authorize
merging the neoCLR branch wholesale.

The Option/Result source work exposed direct out-parameter forwarding being rejected
as unassigned. A regression using a source generic setter and ordinary .NET Math.DivRem
failed with RAV0269. General fix `5f6e17347` is on Raven main; feature cherry-pick
`2d2a1d586` keeps the target isolated. All 41 focused semantic/runtime parameter checks
pass on main, including observable execution and conditional/deferred negative cases.
The correction covers direct invocation expressions, not a redesign of all nested
expression flow analysis. It adds no Runtime Contract option.

General candidates requiring independent validation remain explicit:

- Direct value deconstruction without boxing: an emission optimization on the
  experimental branch; independently test value-copy/mutation and narrowed/null
  paths before moving it to main.
- Language-server reference injection under explicit metadata selection: verify
  normal project/editor contracts before extracting the experimental guards.
- Compound parenthesized comparison conditions: the previously recorded
  [parser candidate](raven-target-evaluation.md#deferred-general-parser-candidate--2026-09-15)
  remains deferred; this port uses existing valid guard syntax.

- Unqualified nested case names in source struct method signatures: the carrier
  authoring probe rejected them while explicit qualification compiled. Reduce this
  against ordinary .NET references and the language scope rules before classifying
  it as a general compiler defect; no fix is claimed.

No .NET Framework or NanoFramework execution result is implied by .NET 11 checks.
No remote branch push or release/package publication is part of this validation.
