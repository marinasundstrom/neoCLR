# Readiness after interface dispatch — 2026-10-01

## Decision

We can move to larger, sample-driven integration milestones. The direct binary path
is working for a meaningful subset, including interface dispatch, but it is not yet a
class-library compiler. The highest-value next step is **connect the real neoCLR target
profile and library dependency contracts to the direct backend**, then use collection
applications to expand shared emission. Continuing only with isolated instruction
slices would leave the largest integration gate untouched.

This is an assessment and proposed sequence, not a claim that the remaining work is
implemented. It follows the author's request to reassess after dispatch and test the
existing samples. The earlier [constraint checkpoint](state-assessment-2026-10-01.md)
is preserved as historical evidence.

## What now works

Raven emits interfaces, nongeneric class implementations, inherited contracts, interface
method/property calls and reference-array flows through the independent metadata API.
Both source orders return 42 on ordinary .NET and binary neoCLR. The API validates
missing implementations and unrelated receivers; both runtimes fault on null dispatch.
This uses ordinary CLI InterfaceImpl/callvirt and existing native runtime lookup, with
no new ISA or runtime source changes in this round.

The producer has 69 passing C# contract groups. Twelve focused compiler tests passed.
[Dispatch evidence](interface-dispatch-validation.json) records the actual runtime hash.
Earlier math, Language, class/property/indexer/array/generic/constraint evidence remains
valid within its documented subset; none proves an entire System build.

## Fresh experiments

[Inventory](readiness-inventory-2026-10-01.json): 31 attempts covering grouped iterator,
collection and sequence contracts, unchanged Language/ArrayList/Option/Result/Math/GC
sources, twelve existing applications/library samples, and all 167 Raven library source
files together. Two configurations are deliberately distinguished:

- The direct backend's existing .NET primitive bootstrap, for foundational units.
- `CompilationOptions.NeoCLR` plus native Self, using the checked-in CLI declaration
  snapshot `api-docs/reference/NeoCLR.CoreProbe.dll`. This is an application reference
  contract, not a complete implementation-bootstrap seed or native metadata importer.

No source was simplified for the inventory. Failed emission is never executed.
The report preserves source/reference/runtime hashes and first diagnostics; complete
diagnostics remain in each run directory. Counts are diagnostic counts, not missing
features or a percentage of library completion.

| Experiment | Observed result | Meaning |
| --- | --- | --- |
| Iterator/Iterable/Disposable group, host bootstrap | Binary emitted and native verification passed | Foundational declarations/signatures work. Separate reference-flow/dispatch probes establish execution. |
| Unchanged Language, host bootstrap | Binary emitted and native verification passed | Prior whole-source execution remains the behavioral evidence. |
| Collection and Sequence groups, host bootstrap | Binding succeeds; direct emission rejects declarations | Constructed generic interface bases and interface indexers remain admission/metadata gaps. |
| All 12 selected apps with actual neoCLR profile | Binding and ordinary CLI emission succeed; direct native emission rejects with NEOMETA002 | The direct backend explicitly requires TargetPlatform.DotNet. Real target-profile integration is unfinished; do not bypass that guard and call it native support. |
| Option and Result source, target snapshot | Bind and emit ordinary CLI; direct profile gate rejects | Existing frontend/CLI union support is reusable, but the new declaration/body model still needs union/value/callback support. |
| ArrayList source, target snapshot | Three binding errors refer to missing System.Runtime support | The application reference snapshot lacks implementation dependencies, notably CheckedStorage. With a proper seed, generic interface implementation and callback/Option bodies are further known gates. |
| Full Math and GC source, target snapshot | RuntimeServices/import errors | These units need implementation intrinsic declarations and backend mappings, not a new parser for each function. |
| All 167 source files with the consumer snapshot | 702 diagnostics in this run; no emission | Not a supported bootstrap configuration. Missing RuntimeServices/CheckedStorage and target iteration/typeof/propagation identities cause cascades. This is not evidence of 702 independent compiler defects. |

### Runtime controls through the existing bridge

The [bridge controls](readiness-bridge-controls-2026-10-01.json) use the **same current
Raven CLI output**, current legacy importer, current generated System collection
library, and current neoCLR runtime. This isolates runtime capability from the new
backend's narrower coverage. It does not recompile every class-library source, and
its neoIL output is not acceptance evidence for the direct binary metadata backend.

- `application-interfaces.rvn`: verifies and runs, exact output `42`, `99`.
- `application-inheritance.rvn`: verifies and runs, exact output `7`, `42`.
- `application-delegates.rvn`: verifies and runs, all expected callback/capture output.
- `application-order-collections.rvn`: binds and emits CLI, but the importer rejects
  an invalid stack in `PendingOrder`; it never reaches runtime execution.

Inspection of the emitted CLI localizes the broad-sample failure **before native
import**. The source's `Option<Order>(None())` produces:

```text
IL_0040: newobj System.Option/None::.ctor()
IL_0045: newobj System.Option<Order>::.ctor(System.Option/Some<Order>)
```

The actual MemberRef uses the declaring-type parameter `!0` for Some's payload, as
normal CLI metadata does; the problem is the selected Some constructor after creating
None. The source/semantic selection versus CLI constructor-resolution root cause is
not yet isolated. Record this as a general compiler regression candidate, independent
of the new native emitter. Do not weaken the importer or rewrite the sample to hide it.

## Next larger milestones

1. **Real target and implementation bootstrap.** Replace the direct emitter's host-only
   admission with an explicit validated native target contract, retaining .NET defaults.
   Separate application declarations from the seed needed to compile System itself.
   Carry Unit, Self, iteration, propagation, typeof, primitive/core identities and
   RuntimeServices/CheckedStorage through explicit contracts. Add negative tests for
   missing/mismatched cores. Acceptance: unchanged simple target samples emit binary,
   verify and run with no accidental host dependencies.
2. **Collection library plus separate application.** Add constructed generic interface
   bases, interface indexers, generic implementations and dispatch. Generalize owned
   member handles to imported nominal/generic types and methods, reusing the current
   semantic importer/reference projection behind a source contract initially. Do not
   build a second symbol hierarchy. Drive this with Iterator/Iterable/Collection/Sequence,
   ArrayList, then the unchanged interface/iterable applications. Acceptance must cross
   an emitted library boundary, not embed every declaration into one test assembly.
3. **Broad application semantics.** First isolate/fix the Option constructor regression
   with a focused .NET compiler test. Then bring existing union/value, delegate/closure,
   propagation and iteration lowering into shared backend plans, with selective target
   capabilities. Use the unchanged order-collections application as the main acceptance
   gate: expected output, mutation/alias behavior and callback cases. Preserve ordinary
   CLI metadata and instruction semantics; extend native metadata only where required.
4. **Library subsystem batches.** Expand supported primitives/value layouts, byref/out,
   native service calls, virtual inheritance, records/enums, text and async state machines
   according to actual source dependencies. Compile coherent source modules, then their
   existing samples. Separate unsupported exceptions from shared instruction semantics.
   A full-library build and representative applications become the final gate.

These are implementation milestones, each with several separately committed slices;
they are not a request to rewrite the entire compiler or add target-specific shortcuts.
The runtime already executes more than the new backend can emit. Its missing work
must be established by binary consumers, not inferred from an adapter rejection.
The extended format still uses authoritative #Neo data plus a CLI reference projection;
this assessment does not claim arbitrary .NET executables run unchanged on neoCLR.

## Reproduction and scope

Tested implementation commits: neoCLR `56ad0410`, Raven `bc25c9e5d`, plus the inventory
and control tools accompanying this assessment. Both remain feature-branch work:
`codex/extended-cli-metadata` and `codex/metadata-consumer`.

Build `tools/NeoClrMetadataProbe` in the Raven worktree with `NeoClrMetadataProject`
pointing at this worktree's metadata project, then run:

```sh
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --readiness-inventory /absolute/neoclr /tmp/fresh-inventory /absolute/neoclr/target/release/neoclr
```

From neoCLR, build the current legacy Probe with `RavenRoot` pointing at the same Raven
checkout and `BuildProjectReferences=false` after its compiler build. Run:

```sh
python3 docs/experiments/raven-target/verify_readiness_controls.py \
  /tmp/fresh-inventory /tmp/fresh-controls \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --runtime target/release/neoclr
```

The inventory/control commands record expected gaps; their successful process exit
means the report was produced, not that all source programs passed. No full test suite,
website build, new benchmark or release qualification was performed.

## Independent Raven fixes integrated into local main

At the author's direction, `codex/compiler-fixes-from-neoclr` was created from
Raven main `d7040e21d`. Six general fixes were extracted by behavior, retaining
ordinary .NET tests and excluding native metadata/backend dependencies:

| Fix | Independent commit |
| --- | --- |
| Complete assignment RHS parsing | `43adcfca1` |
| Accessible setters on public read-only properties | `ea0f91399` |
| Stable property/accessor/backing-field identity and initializers | `1bca614cb` |
| Qualified external type accessibility | `85cb7f316` |
| Generic methods on constructed declaring types | `854adde3d` |
| Implied default-constructor metadata flag for struct constraints | `b4052b0aa` |

The new C# regression cases fail 16/23 on original main and pass 23/23 after
extraction; 174 surrounding tests also pass. Fresh-worktree build and focused
compiler build pass. Execution validation uses modern .NET/net11.0, not native
neoCLR, .NET Framework or NanoFramework. Runtime Contract configuration and target
selection remain unchanged. Raven's `docs/compiler/general-fixes-from-neoclr.md`
records provenance and reproduction filters.

Local Raven main was fast-forwarded to `e5607ca17`, including the validation record.
No remote push was performed. The native metadata API and backend remain on their
feature branches; main integration of these compiler fixes does not imply native
target readiness. Shared vector-loop/transfer lowering (`a9defade8`), shared field
initialization plans and backend abstractions still need separate extraction and
validation. The order-collections Option constructor mismatch remains unfixed.

## Native profile gate and refactor-parity follow-up

The author clarified that fixing behavior changes introduced by codegen refactoring
comes before continuing end-to-end emission, and that semantic decisions should move
into binding where appropriate rather than moving all backend work there.
Raven `87ff03d3f` records a bounded comparison against shared main `e5607ca17`:
21 existing constructor/field/loop/receiver cases pass on main, and 25 on integration
(four additional integration cases). Both pass 99 invocation/iteration cases; use
serial test collections for console-output cases. An initial parallel decimal-loop
output failure passes isolated and serial checks, so no codegen regression is established.
Five added parity cases pass on both lines, covering Debug/Release evaluation order,
short-circuit side effects, array collection replacement and null receiver faults.
This does not replace full regression or release qualification.

Two issues remain explicit: imported `Choice<Item>(None())` selects the Some constructor
in the semantic model on shared main; loop-variable callbacks return 0 on main and
333 on integration instead of 123. Both lines are incorrect in the capture probe;
shared array lowering changes its manifestation. These are not passing acceptance
cases and are not claimed fixed. Do not move constructor selection into the backend
to disguise the binding problem or claim all refactoring risks are closed.

Raven `6f46bbade` then implements explicit native profile admission. The temporary
CLI snapshot provides symbols under `CompilationOptions.NeoCLR`; primitive and Unit
symbols must originate from one imported core with the exact configured identity.
The independent metadata API still encodes native signatures and instructions, and
the runtime loads/verifies/executes the binary. Unlike ordinary .NET assembly execution,
no host core or declaration-snapshot method bodies execute here. The benefit is reuse
of existing binding with native output; the cost is dependence on the CLI declaration
snapshot and its PE identity provider until native metadata importing is implemented.
No binder or general .NET codegen policy changes in this slice.

[Native profile evidence](native-profile-validation-2026-10-01.json) records exact
core/runtime hashes and results: Hello World/function return 42, Unit entry return 0,
array iteration return 42 and owned interface dispatch return 42. All four binaries
verify and run. Wrong core names/versions reject without changing the output stream.
The existing host-bootstrap interface control still verifies and returns 42 on .NET
and neoCLR in both source orders. Build and run the C# probe with:

```sh
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --native-profile-runtime /absolute/neoclr /tmp/fresh-profile /absolute/neoclr/target/release/neoclr
```

The historical inventory's blanket profile rejection is superseded for this subset.
Its other unsupported-source results have not been rerun or declared solved. The
implementation-bootstrap seed, native symbol importing, generic interface dispatch,
imported nominal/generic member emission and broad collections/union/callback bodies
remain open. The API and compiler target stay on their respective feature branches;
only the independently proven compiler fixes were integrated into local Raven main.

## Collections constructor blocker resolved

Raven shared fix `46491585e` (local main; integration port `b2293c67e`) corrects
contextual argument typing before overload resolution: a concrete union case can
target only the matching case, while carrier parameters retain family lookup.
This fixes None selecting the Some constructor without changing codegen or Runtime
Contracts. Sixteen C# regressions cover four spellings, both case declaration orders
and Debug/Release, asserting semantic selection and execution; 323 surrounding cases
also pass. The original two-case test failed before the fix.

[Fresh collections evidence](order-collections-binding-validation.json) supersedes
the earlier carrier failure. The unchanged sample emits ordinary CLI, imports and
verifies, and produces its complete expected output on neoCLR. Both the App and the
existing generated System collection library were then assembled to native binaries;
loading, verification and execution with both binary inputs produce the same output.
No verifier rule was relaxed. This proves existing runtime binary capability for the
broader application, not direct Raven metadata-backend acceptance or System source
compilation. The native direct backend still rejects imported generic Register
signatures with NEOMETA001 and emits no bytes.

Use Raven's `--readiness-sample <neoclr-root> <fresh-output> <runtime> application-order-collections`
probe to repeat only this source, then the existing bridge's `--import` command on
`application-order-collections-target.cli.dll`. Generate System using
`collection_library.py` as in prior controls. Assemble each neoil input with
`neoclr assemble <source> <output.neox> --format neox` (pass the System source with
`--system` for App assembly), then `neoclr verify App.neox --system System.Collections.neox`
and `neoclr run App.neox --system System.Collections.neox`. Compare stdout with the
checked-in sample's `.expected.txt`. The native generic import, implementation-seed
and loop-capture gaps remain open; historical findings above remain preserved.
