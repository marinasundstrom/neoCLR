# Preview 16 release closure

Release targets: **macOS ARM64 and Windows x64**. SDK/extension version
`0.1.16-neoclr.20261017.1`, Raven `c0854cdda`. The
[qualification record](validation/release-20261018/README.md) owns exact revisions,
archive hashes, raw reports and publication verification.

| Gate | Evidence and status |
| --- | --- |
| Source | Canonical extracted-source full suite and 25 smoke programs pass at `efa1687c8`, with strict Clippy/formatting, release-host checks and minimum-Rust checks on Windows/macOS/Linux. The qualified-source archive is that exact snapshot. |
| Optimized packages | Both extracted platform packages pass. README-only publication repacking preserves every executable/library byte and verifies every manifest hash. |
| Native metadata | Production System.Runtime and dependent libraries are built from native metadata and explicit runtime-service contracts, without CoreProbe semantic references. The retained seed supplies execution support. |
| Runtime/library | Packaged collections, JSON/reflection, HTTP, GC and 24 await cases pass their supported interpreter/JIT/AOT matrices. Unchanged earlier focused type, checked-arithmetic and field/initializer evidence remains linked from the candidate record. |
| Editor | Both platforms pass installed-server completion/hover/navigation checks, exact Task<int> inlays and two ten-cycle two-project diagnostic-recovery runs. SDK/VSIX server files match. |
| HTTPS | Hosted security-provider checks pass on both platforms. Matching-package macOS HTTPS passes 15 AOT security/recovery/cleartext cases. Matching-package Windows JIT/AOT project qualification also passes. |
| Runtime performance | Final macOS CPU/JSON and plain/typed-JSON HTTP measurements pass correctness checks; final Windows measurements also pass against the exact qualified archive. Compilation is excluded from steady-state measurements. |
| API coverage | Native snapshot and website contract tests verify the reference and navigation. The route audit classifies 19 legacy-only hierarchical pages and validates 104 compatibility redirects; documented coverage limitations remain explicit. |
| Publication | Preview 16 and all eleven assets are public and verified. Manual website deployment and live download/benchmark/API checks pass. Published Preview 15 artifacts and notes remain unchanged. |

The [earlier candidate record](validation/release-candidate-20261017/README.md)
retains the six repaired source-test failures, UTF-8/array and JSON codegen fixes,
callback inference and repeated stale-project-diagnostic repairs. Failed candidate
runs are not relabeled as release qualification.

## Scope and measurement limits

The release keeps broader structural work parked. Supported structural types,
nominal interface variance, Function reflection, ranges, checked operations and
field/type initialization retain their documented bounded contracts. Raven/.NET
and Raven/neoCLR remain separate compiler lines.

CPU/JSON and HTTP measurements use fresh processes, five rounds and explicit
warmups. Native CPU batches contain 10,000 operations; smaller pilot batches are
not used for native CPU conclusions. Startup is separate from throughput. Preserve
raw data, correctness checks, host/toolchain details and artifact/input hashes.
Interactive macOS and hosted Windows measurements are separate series, not a
cross-platform ranking or a production capacity guarantee. The syntax-adapted
Preview 15 comparison documents the historical endpoint spelling; it does not
justify precise speedup claims under different host load. PostgreSQL, allocation
rates, sustained concurrency and HTTPS throughput are not newly measured.

All execution and publication gates pass. [Preview 16](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.16)
is published; [verification](validation/release-20261018/publication.json) records the
release commit and all asset identities.

### Qualification findings — 2026-10-21 editor and HTTP follow-up

Generated development editor tasks now make JIT execution explicit and provide
a separate native compilation action; Raven 6aa3aacf5 and neoCLR 7db517701 record
that contract. Six task/runner tests, exact collections JIT output, three CLI
rejection controls and 23 website tests pass. Refreshed installed-bundle and native
build-kit acceptance remain required; this is not release approval.

The HTTP JSON import failure is repaired in 033942653 and passes SDK contract
tests plus interpreter client/server requests. JIT qualification still fails at
the 2,048 selected-function budget before server startup. A diagnostic build with
a 4,096-function budget exceeded the 60-second startup deadline under concurrent
source qualification; that experiment does not establish a solution and the
source limit is unchanged. It also exposed a verifier cleanup gap: killing the
runtime parent can leave the JIT child holding its output pipes. The diagnostic
child was stopped explicitly. Both selection/startup behavior and process-tree
cleanup require follow-up before repeating package qualification.

The clean-source qualification snapshot predates the stale floating-display unit
assertion repair and additionally exposed an obsolete boxed-Double rejection
expectation in `tests/object_display.rs`. Keep the original failures as evidence;
do not describe the extracted full-suite run as passing.

The corrected boxed-Double integration test passes; production behavior is unchanged.

[Slice/Span release review](validation/release-readiness-20261020/slice-span-review-20261021.md) records the supported-type/lifetime gates, missing buffer helpers and by-reference indexing distinction. It is a review, not completed API expansion.

### Qualification cleanup repair — 2026-10-22

The HTTP validator cleanup gap recorded above is repaired on POSIX: launches own
a new session and cleanup kills that process group even after the launcher exits.
Pipe draining is bounded. HTTP client timeouts use the same ownership, and the
HTTP foundation/method/retained-session/project validators share the implementation.
Seven focused tests pass, including a real child retaining inherited pipes and a
listening socket. Windows uses bounded `taskkill /T /F` for a live launcher; the
exited-launcher case is not qualified there and needs target-host follow-up.

[Evidence and cleanup timings](validation/release-stabilization-20261022/http-process-cleanup.json)
record the exact working inputs. These observations measure validation cleanup,
not runtime request throughput. The selected-function budget/startup blocker,
fresh source suite, matched packages and installed-editor gates remain open.

### Specialization lookup optimization — 2026-10-22

Rust specialization now indexes immutable source methods by name once rather than
scanning the full source inventory for each call. All candidates still undergo the
existing instance/generic-arity/definition/closed-owner/parameter checks. Source-order
overload traversal and ambiguity rejection remain unchanged. The extra hash index
is temporary compiler storage, not a runtime object allocation policy change.

All 24 specialization tests pass. A five-round isolated unoptimized probe with
4,096 source functions and 1,024 reachable functions changes median expansion time
from 45.6 ms to 13.6 ms. [Raw evidence](validation/release-stabilization-20261022/specialization-lookup.json)
includes limitations: no optimized-build, HTTP startup or request-throughput gain
is established. The 2,048-function gate remains unchanged; the HTTP selection/startup
blocker still needs end-to-end analysis with matching source artifacts.

The author's preference for separate native support libraries informs subsequent
boundary reviews. The byte-only formatter in `tools/aot-native-text/src/lib.rs` is
a potential NativeImport-facing service; its `runtime/native/text/numeric-format.c`
adapter allocates rooted managed text and must retain runtime integration. The TLS
provider already has a separate native adapter with host-owned socket/scheduler
callbacks. Any move requires all-engine import support, matched packaging and
measured call/marshalling cost; no service was relocated in this increment.

### Reflection fixture gate repair — 2026-10-22

The current-source run reproduced all seven recorded Raven reflection failures.
The archived test profile lacked the three generic-method fields now supplied by
runtime snapshots. Its adaptation now follows the current Raven provider layout;
production Raven sources and generated legacy artifacts remain unchanged. The old
open-generic-token rejection expectation is replaced with descriptor identity and
open-storage rejection checks. All 26 reflection and seven type-inspection tests
pass. [Evidence](validation/release-stabilization-20261022/reflection-fixture-repair.json).

Performance review: this repair changes test fixtures only, so it introduces no
production Rust/Raven allocation or execution changes. Test-run duration is not a
runtime benchmark. This closes the focused failures, not the fresh extracted-source
release suite or pending HTTP startup/package gates.

### Explicit specialization root bounds — 2026-10-22

Review of the HTTP selected-function gate found that explicit host roots (including
reflection roots) bypassed the specialization-stage function budget. Admission now
checks each distinct host root against the same 2,048-function limit as discovered
calls; repeated roots and the entry point consume no extra slots. Later selection
already enforced its own bound, so this closes excess intermediate work rather
than expanding or reducing the supported executable budget.

This is an implementation guard for the experimental native backend, not a new
.NET/CLR type or reflection semantic. It adds a constant-time check per unique host
root, no production runtime allocation and no change to generated code. No HTTP
startup or throughput gain is claimed. Public API/website contracts are unchanged;
HTTP selection/startup and matched-package release qualification remain open.

Validation: the regression fails before the fix and passes afterward. All 24
specialization contract tests pass, including boundary native compilation; the
timing-only probe is intentionally excluded. Rust formatting passes.
[Before/after evidence](validation/release-stabilization-20261022/host-root-budget.json).

### HTTP selection diagnosis and interface lookup — 2026-10-22

The retained October 21 package cannot be combined directly with the current
backend: its primitive metadata predates the Void/Character migrations. Rebuilding
the unchanged HTTP JSON server sample with the matching native-layout compiler and
library overlay reproduces the selected-function limit at 2,048 functions, 932
clones and 661 shapes. The largest source-owner groups include Result, Promise,
Task and ArrayList specializations. These counts do not prove that any particular
body is redundant; the limit remains unchanged and HTTP startup is not qualified.

A two-second sample of the unoptimized diagnostic backend found repeated interface
owner resolution inside `has_self_contract` during source verification. The helper
now rejects signatures without Self before resolving their owners across inherited
interfaces. The same signature and exact definition-identity predicates remain;
no cache, new storage or generated-code change is introduced. This preserves
nominal interface contracts and the existing implementing-type extension; it does
not introduce a new divergence from the .NET/CLR model.

A five-round isolated unoptimized probe over approximately 4,100 methods reduces
median time for 100 negative Self-contract queries from 195 ms to 4.5 ms. This
synthetic lookup result is neither overall startup speedup nor HTTP throughput.
Public APIs and website capability claims are unchanged. Further reachability
analysis, matching native-host packaging and full release qualification remain open.

Validation: 92 integration tests pass across Self, nominal/boxed/class/default/explicit
interfaces, inheritance and variance, plus the new lookup contract test and timing
probe. A current-backend HTTP rerun preserves the same selection failure counts.
[Inputs, results and timing evidence](validation/release-stabilization-20261022/interface-self-lookup.json).

### Dispatch relationship reuse and HTTP inventory — 2026-10-22

Selection and value-profile admission now reuse immutable type-compatibility
answers within a compilation scope. Reachable receivers and dispatch targets
are still recomputed. Closed generic targets remain distinct and mutation requires
a new scope. Temporary compiler storage replaces repeated relationship traversal;
no cache or extra allocation is emitted into applications.

All 135 backend unit tests pass; six focused selection checks including a manual
timing probe pass. A diagnostic 4,096-budget build admits the rebuilt HTTP JSON
server with **2,055 functions and 662 types**, including 932 generic clones. Native
execution is not tested. The production budget and root-probe limit remain 2,048;
a coordinated bound update and matching host qualification are still required.
[Design comparison and evidence](design/native-reachability.md).

### Coordinated native method budget — 2026-10-22

The source-built HTTP JSON server requires 2,055 selected methods, seven above the
previous experimental bound. The compiler's selected-method/clone bounds and native
root-probe index bound now agree at 4,096. Input inventory, type, object and inline
layout limits are unchanged. This is a backend capacity change, not a managed type
system restriction or a change in method/GC frame layout. A newly compiled program
that exceeds the old bound requires the matching rebuilt host; an older installed
host is not qualified by this change.

A new compiler/header agreement test prevents drift. The native frame test now
checks the configured last valid index and the next invalid index rather than its
stale hard-coded 1,023/1,024 pair. The sanitized C test accepts 4,095 and aborts for
4,096, with GC enabled and disabled. Native compilation accepts 4,096 selected
methods and rejects 4,097 without writing an object. All 136 backend unit tests
pass, including explicit host-root, specialization and selection boundaries.

The current optimized backend and rebuilt native helper pass the four-request
HTTP JSON server check and the Raven client/server exchange. These are local macOS
ARM64 source-overlay checks, not an extracted/installed SDK or Windows qualification.

Standalone AOT now passes the same server and client/server checks. The server
selects 2,055 methods/662 types and the client 1,988 methods/674 types. Single
optimized compiler runs take about 19.7 and 19.8 seconds respectively (loading,
verification and emission, excluding linking). These are build observations, not
steady-state throughput. The bound adds no per-call storage or GC frame-layout
change; larger admitted programs can consume more compiler memory and output space.
[Exact artifacts, test logs and HTTP results](validation/release-stabilization-20261022/http-function-budget.json).
Fresh extracted-source, packaged/editor and Windows qualification remain open.
