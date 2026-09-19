# Raven library port: execution gate and branch audit

Development follow-up (2026-09-19): the legacy System.Error wrapper and its runtime
support are retired. Historical Error port notes below describe the earlier slice.
Current opaque admission coverage has six String cases; simple message errors are
ordinary strings carried by Result. See [the migration](errors.md).


Development validation on 2026-09-19. This records the API-preserving source port;
proposal API alignment remains subsequent work. The System.Runtime authoring
project and implementation identity are now established. [Authoring status](raven-system-library.md) records the completed source boundary.
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

## Managed array implementation follow-up

Array brings the total to 73 slices. Eight authoring admission checks cover exact
intrinsic storage, private empty construction, forbidden backing-field writes and
allocation, and named-parameter contracts. All 25 array shape/iteration/collection
Rust tests pass, including the real Raven iterator observing a later element write
and rejecting Current before positioning, after exhaustion and after disposal.
Four saved array programs preserve expected metadata, interface dispatch, callbacks
and element results. Full bootstrap regeneration and snapshot hashes match.
The source-body ownership audit leaves only the TypeOf<T>.Of helper to migrate.

## Obsolete declared-type helper removal

At the author's direction, TypeOf<T>.Of is removed rather than ported. typeof(T)
provides declared-type inspection without an unused value argument. Runtime and
consumer declarations and the importer mapping are removed, and active examples
and tests are migrated. Existing compiled helper calls must be rebuilt. Published
preview notes remain historical. Thirteen type/reflection tests and the saved
reflection program pass; an attempted old helper call is rejected during compilation.

The ownership gate now checks all 724 selected method/function declarations against
generated fragments or the explicit native-service catalog (54 service declarations).
No handwritten managed method body remains in the Raven profile. The historical
Neo profile retains its representation-specific implementations. Final execution
results below remain a separate acceptance gate.

## Final port metadata correction

The complete post-port Rust run exposed one semantic regression: generated
Option/Result cases had qualified names but no declaring-type identity. The importer
now preserves their lexical nesting inside the already validated companion
containers. This restores metadata ownership without changing case storage,
constructors, conditional outputs or managed method bodies. The regression checks
all four cases against the exact resolved owner definition, including nongeneric
Option.None. All 34 focused nesting, union, factory and reflection tests pass. It does not infer ownership from a display-name prefix.

This is a target importer correction. Raven already emits nested CLI metadata;
compiler semantics and Runtime Contract settings are unchanged. General application
nested-type admission is not broadened. All 73 slices regenerate, source/snapshot
checks pass, and all 12 generic-union admission cases pass. The API inventory retains
768 candidates in 173 source files with 118 declaring sources; case entries now
record the proper lexical context.

## Reproduce the program gate

Use the neoCLR feature compiler, built from the Raven repository's
`neoclr` branch. General compiler fixes are developed and
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

## Earlier intermediate results (65 slices)

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

## Final source-port gate — 2026-09-19

The API-preserving source port now has 73 Raven slices and no handwritten managed
method bodies in the selected Raven profile. System.Runtime.rvnproj is the authoring
project, System.Runtime.dll is the implementation input, NeoCLR.CoreProbe remains
the explicitly mapped bootstrap reference, and System.neoil remains the executable
library module. This milestone does not claim that the subsequent RuntimeContext /
Info-interface API migration is implemented.

The final Rust baseline covers all 178 integration-test binaries plus library/binary
unit tests: 1246 tests pass across batches, with no remaining failed or ignored cases.
The Console graph-budget fixture was corrected and rerun (11 tests); the final
system_companions failure was a real lost-declaring-type regression, fixed above
and rechecked in a 34-test focused batch. Documentation tests complete with no cases.
The unaffected binaries were not rerun after that bounded metadata correction.

All 65 saved-project cases pass against the corrected, freshly generated library:
compile/import/verify/execute, saved edits, rejected builds with no stale execution,
and expected runtime faults. The post-correction fixture was generated separately
from the user's VS Code workspace; user edits were not overwritten.

All 25 source-admission suite files pass. The primitive fixture now expects the
preserved Int64 parameter name. All 73 slices regenerate, snapshot hashes match,
and the source-ownership gate accounts for 724 declarations: 54 explicit native
services and the remaining declarations supplied by generated snapshots. The
inventory/coverage gates account for 768 candidates and 118 declaring sources.
Controlled process/file checks and the isolated VS Code baseline pass as recorded
in [the local-build guide](raven-port-local-build.md). That workspace still exposes
the pre-alignment descriptor classes; its successful execution is not evidence for
the new interface-based API.

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

The final review integrated editor reference isolation on main (`6c2ccb13e`):
65 workspace integration tests pass with ordinary .NET references. Before the fix,
the editor injected Raven host support assemblies despite explicit metadata import.
No new Runtime Contract setting or neoCLR policy was added.

Direct value deconstruction is a correctness fix, not merely an optimization:
a reduced ordinary .NET ref-struct pattern threw InvalidProgramException before
main `f7f3f0c6d`. All 31 focused cases pass on main and neoclr, including value-copy
mutation and narrowed/null reference inputs. The existing general path remains for
type parameters; no broader generic ref-struct capability is claimed. The current
.NET [ref struct restriction](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/ref-struct)
forbids boxing; source pattern semantics remain unchanged.

The remaining production differences are target array shape/covariance and their
project options, nominal Void value positions, target propagation, and context-owned
typeof. The project service already enforces explicit reference isolation on main;
the feature's extra evaluator guard was redundant. All 47 main and 52 neoclr
project-system checks pass after retaining both unit and array policy coverage. These target policies and their
fixtures stay on neoclr. The branches were not merged wholesale.

The eight temporary fix branches created during the port were removed after checking
that their commits are ancestors of main. The integration branch was renamed to
`neoclr` as requested. Existing main, dev and old/* branches were preserved.

General candidates requiring independent validation remain explicit:

- Compound parenthesized comparison conditions: the previously recorded
  [parser candidate](raven-target-evaluation.md#deferred-general-parser-candidate--2026-09-15)
  remains deferred; this port uses existing valid guard syntax.

- Unqualified nested case names in source struct method signatures: the carrier
  authoring probe rejected them while explicit qualification compiled. Reduce this
  against ordinary .NET references and the language scope rules before classifying
  it as a general compiler defect; no fix is claimed.

No .NET Framework or NanoFramework execution result is implied by .NET 11 checks.
No remote branch push or release/package publication is part of this validation.
