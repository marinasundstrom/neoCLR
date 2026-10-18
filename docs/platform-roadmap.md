# neoCLR platform roadmap

**Preview 16 compiler investigation (author direction, 2026-10-18):** reproduce reported record and Object-display failures, repair native boxing/generic display with GC-safe value semantics, then investigate adjacent gaps and commit each slice. [Reproduction and bounds](experiments/preview16-record-display/README.md) distinguish the unreproduced emitter message from confirmed backend failures and separate JSON mapping/retention limits. This bounded corrective work does not permanently reorder the platform roadmap. The author next requests [JSON record serialization](experiments/json-records/README.md), preserving interpreter/JIT/AOT behavior, and raises ValueType ancestry as a separate familiar-semantics foundation question. Imported ValueType ancestry for application structs/record structs is now repaired with [three-mode semantics evidence](experiments/type-semantics/README.md); [Application enum ancestry and boxed identity](experiments/enum-ancestry/README.md) now also pass interpreter/JIT/AOT; [Intrinsic primitive and core-enum ancestry](experiments/primitive-ancestry/README.md) now also passes interpreter/JIT/AOT against a freshly rebuilt four-library cohort. [All fourteen numeric/Boolean/Char box kinds](experiments/primitive-boxes/README.md) now preserve exact identity and ValueType ancestry across the three modes. Additional-kind default Object display and local ordinary value-record authoring remain separate.

**HTTP buffer investigation (author direction, 2026-10-18):** [matched profiles and validation](../benchmarks/http-server/buffers-20261018/README.md) target small transfer blocks and response snapshot growth while preserving ownership. The author clarifies that the broader objective is measured sustainable capacity: follow with offered-rate/concurrency sweeps, error/rejection accounting, latency and memory/CPU attribution. Sequential microbenchmarks remain isolated evidence, not maximum server capacity. The [offered-load follow-up](../benchmarks/http-server/load/README.md) records worker/GC attribution, explicit paced-policy comparisons and a validated response timeout classification repair; collection defaults remain unchanged. The [lazy-accept comparison](../benchmarks/http-server/lazy-accept/README.md) now measures the author-preferred `ServeOne` callback path with an awaited handler, while preserving queued cancellation and admission. The author subsequently selected implementation of [Serve(handler)](../benchmarks/http-server/serve/README.md): server-owned bounded callbacks, lifecycle supervision and measured load behavior. [Sustained-rate profiling](../benchmarks/http-server/serve/capacity-20261018/README.md) now qualifies a local quieter-host 400/sec paced-GC point, retains noisy failures and 500/sec overload losses, and identifies native memory bookkeeping as the next CPU target. The author reaffirms a matched Go comparison as the benchmark objective; [alternating Go net/http controls](../benchmarks/http-server/serve/go/README.md) now quantify the CPU/latency gap under explicit execution and GC settings, with higher-rate Go results labelled as tested lower bounds. The author next directs [root metadata validation optimization](../benchmarks/http-server/serve/root-validation/README.md), preserving GC safety and measuring the same Serve/Go contract; Green Tea page-local tracing is recorded as research input, not a collector replacement decision. After the ancestry repairs, the author directs returning to GC/HTTP benchmarks and the website; [the fixed-guest repeat](../benchmarks/http-server/serve/gc-return-20261018/README.md) passes short 300/sec checks, fails both 400/sec repeats and retains separate GC sampling with matched Go controls. The subsequent [bounded intern-seed optimization](../benchmarks/http-server/serve/seed-walk-20261018/README.md) improves measured collection throughput, with an explicit overflow cost and no higher sustained HTTP rate established. The next [Task allocation slice](../benchmarks/http-server/serve/task-completion-20261018/README.md) removes empty continuation-list replacement and passes three-mode contracts, but reversed 60-second Serve/Go comparisons establish no repeatable CPU gain or higher reliable rate. The [lazy callback follow-up](../benchmarks/http-server/serve/callback-storage-20261018/README.md) reduces diagnostic allocations 13.37%; its requested rerun shows 5.6–7.2% lower CPU/completion, while retained earlier losses prevent a reliable 400/sec claim. The [reusable TaskQueue](../benchmarks/http-server/serve/task-queue-20261018/README.md) removes a further 11.71% of diagnostic allocations; two matched 400/sec trials pass with 1.5–3.0% lower CPU/completion and higher p99 latency. A sizing diagnostic peaks at 15 callbacks/capacity 16. The first [bounded HTTP receive-cache slice](../benchmarks/http-server/serve/receive-pool-20261018/README.md) saves 0.70% of diagnostic allocations and 3.43% of payload bytes, with three-mode ownership validation and explicit interpreter fixture-budget limits. Both candidate load trials pass at 400/sec, but the cleaner CPU pair differs by only 0.3%, establishing no convincing speedup or higher rate. The [encoding-provider follow-up](../benchmarks/http-server/serve/encoding-cache-20261018/README.md) removes repeated stateless provider allocation while preserving independent sessions: 3.07% fewer diagnostic allocations, three-mode contracts, and 1.1–1.2% lower CPU/completion in passing matched 400/sec trials. No higher sustained rate is established; an exposed native concrete-static-field interface-selection gap remains separately recorded. The [bounded header reservation](../benchmarks/http-server/serve/header-reserve-20261018/README.md) removes repeated startup growth, with three-mode boundary tests and 1.76% fewer diagnostic allocations despite 3.57% more collections. All matched 400/sec trials pass with 0.9–1.0% lower CPU/completion, without establishing a higher rate. The [caller-profile investigation](../benchmarks/http-server/serve/caller-profile-20261018/README.md) identifies repeated first-callback growth and substantial Promise result storage. [First-callback reservation](../benchmarks/http-server/serve/callback-reserve-20261018/README.md) saves 4.36% of diagnostic allocations with three-mode validation. Initial and post-activity-change load comparisons retain missed arrivals and mixed CPU results; no new sustained rate or repeatable speedup is established, and admission/latency investigation remains open. The author directs fixing the discovered allocation/GC issues first, then a ready-to-use HttpContext callback for low-level Serve, with website/sample migration accompanying implementation.

**Native bootstrap release (author direction, 2026-10-17; qualification 2026-10-18):** production core and dependent libraries use native metadata and explicit runtime-service contracts without CoreProbe semantic references. [Preview 16 is published](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.16) for macOS ARM64 and Windows x64 with Raven `c0854cdda`, matching optimized runtime/SDK/VSIX bundles, passing source, installed-editor, async/JSON/GC/HTTP/security qualification and fresh two-platform runtime measurements. See [release qualification](validation/release-20261018/README.md), [release notes](preview-16-release-notes.md) and [native migration evidence](validation/native-core-removal/README.md). All eleven public assets verify; manual website deployment and live download/API/benchmark checks pass. Broader structural work remains parked; Raven main and `codex/neoclr` remain separate.

**Mutable collection helper slice (author direction, 2026-10-16):** implemented
`MutableMap<K,V>.Entry(key)` and `MutableSet<T>.GetOrInsert(value)` as extensions,
with live key locators, explicit synchronous callback semantics and canonical set
lookup. Integrated on neoCLR main, Raven main and Raven `codex/neoclr`
(`afd5fe6e3`); 13 focused tests pass on the final Raven target branch.
Concrete collection names stay unchanged; GetItem/list work is deferred.
See the [contract and .NET/Rust comparison](collection-entry-design.md) and
[qualification record](validation/map-entry/README.md). This bounded library task
does not reprioritize the active milestone or the parked structural work.

**Structural type integration (author direction, 2026-10-15):** stabilize and
integrate the experimental structural type work, remove experimental opt-in flags,
and retire the nominal System.Union/System.Tuple ad-hoc carriers. Migrate dependent
library APIs, native Raven mappings, generated metadata and documentation together.
Declared nominal unions and ordinary .NET target behavior remain distinct. The validated implementation is integrated into neoCLR main and Raven’s neoCLR
line. Runtime/compiler/reader opt-ins and nominal carriers are removed. The
[qualification record](validation/structural-integration-20261015.md) distinguishes
validated behavior from remaining unsupported forms. This integration does not
reprioritize unrelated roadmap milestones or declare a release.


**Structural composition experiment (author direction, 2026-10-14):** the isolated
[closed-reference POC](experiments/structural-poc/README.md) on
`codex/structural-union-intersection` and Raven `codex/neoclr-union-intersection`
covers source native signatures, structural descriptors, Task union delivery,
intersection trait calls and closed method constraints with separate consumers.
Interpreter callable union alternatives now have focused storage/narrowing and
receiver-collection checks; bounded native callable-union storage now has AOT
receiver-retention and exact allocation checks. The artifact-only Raven callable
consumer now passes on freshly rebuilt libraries in interpreter, AOT and managed
JIT, including eager/paced collection and cleanup before code release. Broader
conversions and escaped async retention remain open. Closed structural generic arguments
now pass specialization. Checked structural array storage has collector/fault
qualification, and structural fields in nominal value records now pass metadata,
copy and collector checks. Managed value-record methods now admit structural signatures. Fresh direct and
closed-generic Task union consumers pass interpreter, AOT and managed JIT with
eager/paced collection and cleanup; see the [retained Task evidence](experiments/native-structural-types/README.md#fresh-task-union-native-qualification-2026-10-15).
Single/Double union alternatives now pass focused interpreter/AOT exact-type and widening
checks plus a fresh Raven interpreter/AOT/JIT consumer without arena allocation. Int32-backed nominal enum alternatives also have focused interpreter/AOT identity,
payload and widening checks. Broader primitive/enum qualification remains open.
Intrinsic String union alternatives now have metadata, interpreter and native collector checks
with separate traced text slots; see the [String checkpoint](experiments/native-structural-types/README.md#intrinsic-string-union-alternatives-2026-10-15).
Exact-alternative union widening now repacks differing native layouts without a
carrier allocation and passes a fresh interpreter/AOT/JIT consumer plus collector
checks. A bounded scalar-union microbenchmark measures about 5.3% higher median call cost
than an explicit nominal tagged record on Apple M1, with no native imports or
arena allocation; mixed/async throughput remains open. Written intersection bounds require interfaces or classes open to extension;
Raven sealed hierarchies remain valid. Full open-generic integration, broader
constraint authoring/reflection and broad JIT/AOT support remain follow-up work.
Bounded native tuple storage now passes macOS ARM64 AOT copy/call/address and
256-slot boundary checks; see [native tuple progress](structural-tuples.md#native-inline-storage--2026-10-15). The consolidated
acceptance runner records the exact tested artifacts. This author-directed
experiment does not change the active performance milestone. Bounded intersection
reference views and reference-alternative union payloads now have AOT/interpreter
differential checks. Scalar-only Int32/Int64/Boolean unions now have tagged
inline storage, exact adjacent box/cast elision and bounded scalar type tests; bounded mixed scalar/reference storage now has interpreter/AOT checks with a
separate traced reference slot. Closed nominal value-record unions now have bounded copy/extraction and embedded
reference checks. A [fresh imported Raven struct consumer](experiments/native-structural-types/record-import-project/README.md)
passes seven cases with no native imports or arena use. Record/reference mixtures now have bounded native checks and a
[fresh imported Raven consumer](experiments/native-structural-types/record-reference-import-project/README.md)
with eight inputs and only the explicit object allocation. A
[closed-generic record variant](experiments/native-structural-types/generic-record-reference-import-project/README.md)
also preserves Response<int>/Response<long> identity through nine inputs after a
general Raven pattern comparison fix. A
[reference-valued substitution](experiments/native-structural-types/reference-generic-record-import-project/README.md)
also preserves shared Cell mutation through copied records with exactly the two
explicit allocations. [Bounded native collector stress](experiments/native-structural-types/README.md#native-collector-stress-for-union-roots-2026-10-15)
now preserves union-held objects through eager/paced AOT collection and fault
unwind. The imported Response<Cell> consumer also passes repeated eager/paced
nonmoving collection with full reclamation; broader JIT/async/application
qualification was pending at that checkpoint; the integration record above supersedes
that status for the tested workloads. A
[fresh Raven scalar-pattern consumer](experiments/native-structural-types/scalar-pattern-project/README.md)
now passes guarded/fallback arms and result joins in interpreter and macOS ARM64 AOT.
An [artifact-only library consumer](experiments/native-structural-types/scalar-import-project/README.md)
also passes union returns/parameters with reordered alternatives and zero arena use.
A [fresh mixed-pattern consumer](experiments/native-structural-types/mixed-pattern-project/README.md)
passes seven scalar/reference inputs with only its explicit Response allocation.
An [artifact-only mixed library consumer](experiments/native-structural-types/mixed-import-project/README.md)
also passes imported union returns/parameters and Response property access with
only the explicit object allocation. The [closed-generic imported variant](experiments/native-structural-types/generic-mixed-import-project/README.md)
also preserves a stored Response<int> payload through mixed union calls.


**Type relationships (author direction, 2026-10-14):** TypeInfo assignment, subclass
and instance predicates share runtime semantics across interpreter/JIT/AOT. The
[focused qualification](validation/type-relationships-20261014/README.md) records
macOS checks and remaining compiler/variance limits. This bounded API addition does
not reprioritize the integrated-runtime performance milestone.

**Preview 15 published (2026-10-14):** the composed JSON error carrier now fits a
shared 256-lane native inline-value bound. GC and runtime-product qualification pass
on macOS and Windows; local three-mode JSON and 132 HTTP checks pass. The
[release tracker](tracking/toolchain-release.md) records passing source, Windows
package/editor and final JSON/editor delta gates, plus the installed macOS samples.
The release and website are published, with verified download and API source links. This author-directed release fix does not change
the performance milestone or promise unbounded native value layouts.

**Atomic GC marking (2026-10-14):** [directly retaining leaf payloads](../benchmarks/gc-index/README.md#direct-marking-of-atomic-payloads--2026-10-14)
removes empty tracing work while preserving dependent-owner completion. All 35 local
GC/root and 132 HTTP checks pass. Matched stress-GC AOT HTTP runs improve 4.7%/4.8%
plain/JSON, approximately recovering the static-catalog cost; isolated atomic-buffer
collection improves 12.2%. This is local sequential evidence, not Windows or sustained
load qualification. Continue profiling the integrated runtime against the milestone.

**Static HTTP header catalog (author direction, 2026-10-14):** the internal static
catalog now lazily shares 13 known names. Interpreter and native contexts own the
static roots, including separate storage for closed generic types; Raven native
branch `fac4ac736` emits the fields. All 132 HTTP checks pass across interpreter,
JIT and AOT with runtime-await. [Storage scope and remaining gaps](runtime-static-storage.md)
include type initialization/readonly static writes and Windows execution qualification.
[Measurements](../benchmarks/http-server/header-static-names.md) distinguish the
new shared values from the earlier seven-name byte matcher. The catalog uses
fixed-name intern publication on cache misses, not a general Lazy implementation.
This author-directed work remains part of the performance milestone.

**URI allocation refinement (2026-10-14):** [matched rebuilt HTTP fixtures](../benchmarks/http-server/uri-byte-parsing.md) replace per-character URI substrings with one bounded encoded buffer. Plain/JSON allocations decrease 23.9%/31.6%; seven alternating AOT rounds improve sequential throughput 1.56×/1.24× with 1,024 validated responses. All 12 URI tests pass in interpreter/JIT/AOT and four HTTP profiles reclaim every allocation. This is macOS stress-GC, state-machine fixture evidence, not Windows, sustained-load or runtime-await performance qualification. The following header matching slice is recorded above.

**HTTP allocation attribution (2026-10-14):** [published-frame diagnostics](../benchmarks/http-server/allocation-sites.md) reconcile every measured allocation and identify URI one-byte substrings (124 allocations/request in retained plain/JSON fixtures) before header decoding (26/request). Prioritize removing parser temporaries and fixed known-header identities within the author-directed HTTP byte-buffer work; preserve eager framing validation and ownership across await. The URI candidate is now implemented and validated above; header byte matching and shared fixed identities are implemented above. Neither replaces sustained/concurrent GC pacing qualification.

**Profiling follow-up (2026-10-14):** [dense index refinement](../benchmarks/gc-index/README.md#reuse-validated-spans-for-the-dense-index--2026-10-14) reuses the first complete validation pass within a synchronous collection. Longer local paired measurements improve dense collection throughput by 2.1–3.3%, with small/sparse controls within 1%; all 35 local GC/root checks pass. The matched HTTP regression check passes 768 responses with overlapping timing ranges; hosted macOS/Windows correctness gates pass. Windows dense gains are 3.6–6.9%; noisy hosted macOS timings are retained without a precise speedup claim. This bounded improvement preserves collection policy and does not establish application throughput gains.

**Integrated runtime product (author direction, 2026-10-13):** interpreter, JIT,
AOT and GC are maintained as one platform, rather than independent experiments.
The [integration direction](runtime-product-integration.md) starts with a shared
workspace/build and validation entry point, now qualified locally and on hosted
macOS/Windows. The shared native collector and root-frame services now live under `runtime/native/gc`,
and arena/context services under `runtime/native/memory`, with compatibility
forwarding paths and GC/non-GC package relocation checks. Fault layout/rendering
and opt-in POSIX diagnostics also have runtime-owned sources; remaining native
services and package consolidation follow. The JIT/AOT backend now exposes an
internal synchronous library driver. Its product package/tool is `neoclr-codegen`,
owned under `runtime/codegen`; active SDK and build paths use that name. Integration means one maintained, measurable
product, not one component or process (author clarification, 2026-10-13). Keep
separate components where useful; the installed process-based launcher is not
itself an integration defect. Memory management must permit future build-selected collectors and
constrained profiles without requiring tracing GC; no live collector swapping is
planned. Alternate profiles remain future work with explicit lifetime/capability
requirements. The author directed returning to measured runtime performance work once the current
integration qualification is complete (2026-10-13); this is not an open-ended
restructuring prerequisite. This integration direction complements the performance work below;
prototype pacing and unqualified targets are not promoted by packaging changes.

**SDK runtime-await integration (author direction, 2026-10-13):** rebuilt SDKs
select the existing explicit native compiler contract and carry the private await
and generator providers. The runtime-owned carrier now includes Windows x64 fibers
alongside macOS ARM64; GC/root and Windows stack-host gates pass. All 21 SDK
application cases pass on both platforms in interpreter/JIT/AOT. Exact build
requirements and qualification are tracked in the
[integration record](runtime-product-integration.md#sdk-default-runtime-await-integration--2026-10-13).
Keep explicit state-machine fallback and existing library bootstrap lowering.
Ordinary Raven/.NET lowering and collection pacing defaults are unchanged.

**Native handle admission (2026-10-13):** [one-walk exact-base admission](../benchmarks/gc-handles/README.md)
removes repeated heap walks for host handles and related owner checks while retaining
trailing-corruption rejection and failure atomicity. Local late-base cases improve
1.76–2.59× at 256/4,096 objects; early-base controls remain near baseline. All 21
local GC/root checks pass, and hosted macOS/Windows GC/root, socket and package
gates pass. Hosted late-base gains are 1.56–2.60×/1.43–2.26× respectively; early-base
control costs and variation remain documented. The AOT HTTP comparison is within
0.3% of baseline. This bounded runtime improvement does not establish an HTTP gain
or change collection pacing.

**Allocation and admission follow-up (2026-10-13):** [single-walk allocation](../benchmarks/gc-allocation/README.md)
retains complete heap validation while removing the repeated first-fit search:
matched hosted gains are 1.18–1.60× on macOS and 1.32–1.84× on Windows. Native-GC
code generation now selects concat/join helpers that protect borrowed inputs during
allocation retry; noncollecting native entry points remain available. Local compiled
GC/non-GC tests and 128 fresh JIT/AOT HTTP responses pass. The focused Windows
application gate also passes eight default-JIT HTTP cases and SSL-stream HTTPS JSON
in interpreter/JIT/AOT. The
[closed rooted pacing experiment](../benchmarks/http-gc-pacing/README.md#rooted-closed-workload-pacing-follow-up-2026-10-13)
passes large-request admission, exhaustion, explicit idle cleanup and suspended-owner
abandonment. The follow-up integrates the audited helpers and actual scheduler idle
cleanup into an opt-in runtime policy. Default promotion still requires managed
workload qualification. The completed [helper-family audit](experiments/aot-console/native-gc.md#explicit-helper-temporary-values)
(UTF-8 slicing now also protects borrowed input through retry; local pressure tests
and 270 compiled GC/non-GC comparisons pass; Windows GC/root and both-platform
socket checks also pass). Integer formatting now has audited stack-scratch allocation
retry in generated GC code, with 22 macOS/Windows GC/root checks and focused
compiled/boxed consumers passing. Byte/scalar array allocation and reservation
now also use rooted retry, with 23 macOS/Windows GC/root checks and 107 local
compiled interpreter comparisons passing. Managed UTF-8 encode/decode now retain
the source text/array through allocation retry, with 24 macOS/Windows GC/root
checks and 14 local compiled tests passing; raw native-buffer callers remain
noncollecting. Reference-array construction now also uses rooted retry, with header/payload
initialization completed before publication. Lexical paths and ordinal replacement now retain all copied string inputs
through retry, with 26 macOS/Windows GC/root checks and four local compiled
path/replacement tests passing. Unicode casing, grapheme access and sequence construction now retain source/partial
results through retry. Process-argument construction now roots its partial array and native decode
scratch is kept stable. POSIX file input now retries only final decoding after closing the descriptor.
Socket/TLS/crypto adapters do not allocate managed results; PostgreSQL uses the
managed socket/encoding layers. Culture/time-zone producers now retain native snapshots through rooted output
allocation. Interning now retains its source during pool-entry allocation; object allocation now retries with published argument/receiver roots and initialized payloads (see the producer ownership inventory). The subsequent pacing gate passes all 33 GC/root executions on hosted macOS and Windows, including these helper checks ([evidence](validation/runtime-product/native-scheduler-pacing/README.md)). Native scheduler acquisition now collects before returning Idle/Pending, with pending/abandoned-owner lifetime tests. The retained-stack pump refreshes ownership/readiness after idle collection to avoid diagnosing expired bodies as unresolved work. The runtime now supplies an opt-in 16-allocation/64-boundary policy with unconditional idle collection. Hosted macOS/Windows GC/root and stress/paced scheduler checks pass; integrated closed-workload gains are 3.61–4.14× with about 23% more peak arena extent. Fresh macOS and Windows managed plain-text/JSON AOT HTTP qualification each pass 640 response checks and complete diagnostic reclamation under both policies ([evidence](../benchmarks/http-gc-pacing/README.md#managed-http-qualification)); local timing overlaps compiler work and is not a speedup qualification. Hosted Windows measures 2.20× plain-text and 2.64× JSON gains on short sequential workloads with bounded peak arena growth. The macOS paced runtime-await matrix also passes all 24 JIT/AOT language, cleanup, client/server and lifecycle cases, plus interpreter lifetime and repeated-entry controls ([report](validation/runtime-product/native-scheduler-pacing/runtime-await-paced-macos-20261013.json)). Sustained/concurrent workload qualification precedes pacing default promotion. Interpreter policy remains independent.
The author also proposed a [platform abstraction layer](platform-direction.md#platform-abstraction-direction--2026-10-13); evolve existing OS boundaries incrementally without replacing this milestone. Linux ARM64 is an explicit future expansion target; separate OS and CPU/ABI concerns without claiming current support.
The author additionally proposed generations to reduce collection pressure/latency;
[the next architecture evaluation](runtime-gc.md#generational-collection-evaluation--2026-10-13)
compares survival, write barriers, promotion and pauses. It does not yet select or
implement a generational collector.

**Dense graph GC milestone (2026-10-13):** [adaptive lookup and retained evidence](../benchmarks/gc-index/README.md)
reduce native collection time by about 4.4–5.2× on 16,384-object chains/wide graphs
in final local/Windows runs. Exact root/payload validation, suspension ownership,
cleanup and collection frequency are preserved. Smaller controls expose bookkeeping
costs and hosted variability; this is not a universal speedup. The separate local
HTTP regression stays within 2% with overlapping ranges. The dense path adds a
validated walk and 32 KiB of stack. All platform GC/socket contracts and the focused Windows HTTP/HTTPS gate pass. Next
investigate allocation and root-publication overhead across general workloads;
complete temporary-root admission before changing collection pacing.

**Runtime-wide performance scope (author clarification, 2026-10-13):** optimize GC
and other runtime components for general workloads. HTTP is an important consumer
and benchmark, not the sole performance target. Include allocation pressure,
reference-heavy live graphs, suspended operations and cleanup in validation;
separate runtime gains from HTTP protocol/library improvements.

**Windows wait attribution (2026-10-13):** [matched diagnostics and benchmarks](../benchmarks/http-scheduling/README.md#matched-windows-results--2026-10-13)
identify unconditional scheduler sleeps as a major avoidable cost. Bounded socket
readiness waits improve plain HTTP by 2.09–2.23× and JSON by 1.58–1.64× in paired
Windows x64 JIT/AOT runs. All 1,536 comparison and 256 diagnostic responses pass;
final reclamation is complete with zero GC/poll failures. GC elapsed time is largely
unchanged and remains the next large work interval. DNS/timer-only and legacy TLS
waits retain bounded polling; macOS policy is unchanged. Hosted, sequential results
do not qualify sustained concurrency or Windows ARM64.

**HTTP design input (2026-10-17):** the [architecture review](http-server-design.md#performance-architecture-review--2026-10-17)
records author-supplied optimization guidance, existing managed HTTP capabilities,
and the assistant's proposed concurrent/slow-client measurement slice. Native
provider selection, buffer leases and persistent connection work remain explicit
open decisions; this input does not displace current release qualification.

**HTTP performance objective (2026-10-13):** the author targets competitive
throughput with Go and .NET, especially AOT; landing between their equivalent
implementations is an aspiration, not a current result. Retain sequential
connection-close controls and extend matched same-host comparisons to persistent
connections and concurrency sweeps, including latency, CPU, allocations, GC pauses
and response correctness. Short pacing measurements do not establish saturation
capacity. This objective does not skip the runtime-await integration gates below.

**Author clarification — runtime-await default (2026-10-13):** runtime-await is
our intended default neoCLR lowering, independently of collection pacing.
Compiler state machines remain an explicit fallback; Raven/.NET defaults stay
separate. After this scheduler/pacing qualification, integrate the source-included
provider behind the runtime capability contract, qualify native Windows suspension
and library lowering, then switch SDK/project defaults. The current macOS ARM64
carrier gate does not authorize a platform-wide default flip; unsupported targets
must have explicit capability behavior. See the [conversation record](development-timeline.md#2026-10-13--runtime-await-as-the-default-lowering).

**Windows root/HTTP qualification (2026-10-13):** [retained Windows x64 evidence](../benchmarks/http-server/root-observation.md#windows-qualification--2026-10-13)
passes GC/root/profiler and socket contracts, native HTTP, eight default-JIT HTTP
cases, and managed SSL-stream HTTPS JSON in interpreter/JIT/AOT. This exposed and
fixed a POSIX-only profiler clock and the obsolete native per-shape 32-callback cap;
the existing selected-function budget remains. The JIT rerun is explicitly focused,
not a full library-suite pass. A [matched Windows x64 benchmark](../benchmarks/http-server/root-observation.md#windows-measurements--2026-10-13)
passes 1,536 responses: plain-text throughput is unchanged within variability; JSON
medians rise 9–10% with overlapping ranges. Next attribute Windows collector versus
socket/scheduler wait time before choosing an optimization. ARM64 remains unmeasured.

**Root observation improvement (2026-10-13):** [matched JIT/AOT evidence](../benchmarks/http-server/root-observation.md)
shows 29–35% higher plain/JSON HTTP throughput after removing duplicate diagnostic
root-value reads. Validation and full collection boundaries remain; all 1,536 timed
comparison responses, 192 diagnostic responses and focused native lifetime/root
checks pass. These fixed-fixture macOS results are development evidence, not new
published release numbers. A separate newer streaming-library AOT comparison also
passes 768 responses and shows 27.5% plain/30.4% JSON improvement. The rooted admission primitive is implemented and tested,
but helper-root integration is still required before reconsidering GC pacing.

**Collection pacing experiment (2026-10-13):** [isolated AOT evidence](../benchmarks/http-gc-pacing/README.md)
shows substantial plain/JSON HTTP potential from collecting less often, but the
reserve/count policy is **rejected for production**: it faults on a large allocation
that the stress collector admits. Final HTTP reclamation and the small-arena fallback
pass; paced abandonment and idle cleanup remain unqualified. Next define and test
size-aware rooted allocation admission/retry, including native helper temporaries,
while preserving explicit/stress collection. No shipping GC policy or published
benchmark changed; generational/concurrent collector selection remains open.

**Collector phase attribution (2026-10-13):** [diagnostic HTTP evidence](../benchmarks/http-scheduling/README.md#boundary-and-collector-phase-attribution--2026-10-13)
separates root-boundary observation from validation, root seeding, tracing, owner
cleanup and sweep. Eight AOT runs validate 192 responses and complete reclamation;
86–91% of collections have no intervening allocation, which is not proof they can
be skipped. Next evaluate a pressure policy with explicit/stress collection, bounded
memory and idle/owner cleanup intact. Author-requested .NET generations and Java ZGC
research is recorded alongside Go/V8; collector replacement remains unselected.

**Runtime suspension follow-up (2026-10-13):** [matched application lowerings and idle-drain correction](../benchmarks/http-scheduling/README.md#idle-drain-correction--2026-10-13)
qualify the same concurrent HTTP fixture with state machines and runtime suspension.
The macOS carrier now avoids draining after empty polls; retained-stack/collector
checks pass and idle drain fibers fall from 358–359 to 90. Alternating uninstrumented
runs show no established throughput improvement. The author also requested learning
from other collectors: phase attribution and collection-pressure policy are the next
research targets, with .NET/Go/V8 references and tradeoffs recorded; no replacement
collector has been selected. JIT/interpreter comparisons remain open.

**Scheduling diagnosis (2026-10-13):** [retained diagnostic tooling and results](../benchmarks/http-scheduling/README.md)
now separate native socket polling, host pauses, process CPU and GC counters.
Eight AOT processes validate 192 responses and complete allocation reclamation.
Socket polling is small in these runs; slow peers trigger the expected 1 ms sleeps.
These guest builds use compiler state machines, so runtime-suspension comparisons,
readiness-to-resume latency and JIT/interpreter instrumentation remain open. No
scheduler optimization or throughput gain is claimed from this diagnostic slice.

**Concurrent execution direction (2026-10-13):** the author asks to inspect runtime
scheduling alongside HTTP work: scheduling and suspension are intended to improve
concurrent asynchronous execution. Evaluate wakeup-to-resume latency, ready-work
fairness, idle CPU, dispatch/GC boundary costs and HTTP throughput together. Runtime
suspension must release execution capacity while waiting; it does not by itself
prove a speedup or parallel execution. Preserve completed-task fast paths and
compare equivalent pending-work cases with compiler state machines.

Initial source inspection finds that `src/scheduler.rs` uses a 10 ms fallback:
socket readiness does not signal its wake latch. `src/invocation.rs` parks without
holding graph or dispatch access. The native host's `socket-listener.c` separately
polls its operation table and delivers one callback per safe boundary. These are
optimization candidates, not measured causes of JIT/AOT HTTP overhead. Next gather
ready/empty poll counts, wakeup latency, resumes and boundary costs under concurrent
plain-text/JSON requests, including slow peers. Compare execution modes separately;
do not replace waits with busy spinning or batch deliveries across unverified root
ownership boundaries. No scheduler implementation change or speedup is claimed.

**HTTP direction (2026-10-13):** the author reaffirms HTTP specification compliance
and established byte-oriented, bounded-I/O optimizations. [Streaming evidence and
next conformance slice](experiments/http-body-stream/README.md#author-direction-protocol-first)
distinguish current gaps from intended semantics; no relaxation for benchmark gains.

**HTTP admission (2026-10-13):** [concurrent exchange evidence](../benchmarks/http-concurrency/README.md)
qualifies bounded FIFO acceptance, cancellation and shutdown in interpreter, JIT
and AOT on macOS. Slow headers and bodies no longer prevent other admitted
connections from completing. Streaming request/response bodies are the next
author-directed step. [Known-length server response streams](experiments/http-stream-response/README.md)
now reuse the client-upload content contract. [Incoming streams](experiments/http-body-stream/README.md)
add explicit headers-first APIs on both sides, qualified by 54 local interpreter/JIT/AOT
cases, buffered controls and full diagnostic reclamation. Existing default calls
remain buffered; Windows/TLS streaming qualification and streaming JSON are open.

**HTTP comparison matrix (2026-10-13):** Author-directed [retained direct-handler projects](../benchmarks/http-server/platforms.md)
cover neoCLR JIT/AOT, Kestrel JIT/AOT, Go native and Node V8 without routing. All
2,304 pilot responses pass. The large observed neoCLR gap reinforces investigation
of collector/root-boundary overhead; short-run results are not saturation rankings.
[Fixed ASCII framing](../benchmarks/http-server/ascii-framing.md) lowers allocations
further to 510/plain and 1,198/JSON request, with full diagnostic reclamation.
Published baselines remain unchanged; no permanent milestone reorder is inferred.

**HTTP byte paths (2026-10-13):** [matched allocation and throughput evidence](../benchmarks/http-server/byte-paths.md)
records block decoder entry and direct byte response framing. Warm AOT allocations
fall from 559 to 519 per plain request and 1,256 to 1,210 per JSON request. Native
throughput improves locally, with overlapping plain-JIT ranges requiring a quieter
repeat. All 63 focused checks and 1,536 comparison responses pass; full reclamation
is observed in four diagnostic runs. Public APIs, installed SDK and published
baselines are unchanged. Buffer views/pooling remain future design work.

**HTTP collector optimization (2026-10-13):** Author-directed profiling identifies
collection as the shared plain-text/JSON cost. A [single-pass sweep/coalescing change](../benchmarks/http-server/profiling.md)
improves matched local JIT/AOT throughput by 7–8% with identical guest inputs and
unchanged collection policy. All 1,536 responses and sanitized GC/retained-stack
checks pass. Interpreter behavior, Windows qualification and published Preview 14
baselines are unchanged. Root-boundary overhead and safe pacing remain follow-ups.

**Benchmark baselines (2026-10-13):** The author requested published measurements
and retained history. [Preview 14 HTTP baselines](../benchmarks/http-server/README.md)
now cover plain text, object JSON and PostgreSQL/query/map/JSON across interpreter,
JIT and AOT, with 1,728 validated responses. [CPU and JSON node measurements](../benchmarks/execution/README.md)
retain startup, round ranges and the short native CPU pilot alongside longer batches.
The [public history](../website/content/benchmarks/history/index.md) and
[history policy](benchmark-history.md) preserve evidence and define comparable series.
Desktop/indexing contention makes these exploratory; controlled-host repeatability,
regression thresholds, Windows and cross-runtime comparisons remain open. SQL AOT
uses the shared native host. This measurement slice does not reorder the active milestone.

**Release preflight (2026-10-13):** [qualification evidence](experiments/release-20261013/README.md)
now includes passing hosted source/format/Clippy, minimum-Rust and macOS/Windows
host gates. The mixed-callback backend repair passes focused Windows object and
macOS SSL/HTTP checks; the author installation is refreshed. Windows default-JIT
package/editor qualification passes. The Windows AOT heap-admission repair now
passes guarded allocation and the full HTTP/HTTPS/framework gate. [Draft notes](next-preview-release-notes.md)
are ready for review; no publication or milestone reorder is implied.

**Local default-JIT preview (2026-10-12):** the [author-testing installation](local-preview-20261012.md)
now passes extracted HTTP JSON checks and repeated installed client/server runs.
Heap-object layout is bounded independently from inline values; matching SDK/VSIX
0.1.13-neoclr.20261012.2 is installed on macOS ARM64. Publication and fresh Windows
qualification remain separate gates; this does not reorder the platform milestone.

**Managed HTTPS composition (2026-10-12):** public development SslStream and
explicit PEM authentication options now compose HttpClient and HttpServer over the
shared provider. Nested JSON, lifetime checks and GC pass locally in interpreter,
default JIT and standalone AOT; cleartext HTTP also passes. Bounded compiler and
host admission fixes preserve explicit memory budgets rather than introducing
trimming. See [qualification and remaining HTTP work](experiments/shared-tls/README.md#managed-ssl-and-http-composition).
HTTP hostname rejection/recovery, handshake deadlines and caller cancellation also
pass in all three local modes; see [HTTP evidence](experiments/shared-tls/http-failures-validation.json).
Independent OpenSSL peers now pass TLS 1.2/1.3 JSON exchange in both directions
across all three local modes, after fixing short-record output pumping; see
[interoperability evidence](experiments/shared-tls/interop-validation.json).
Windows managed qualification and packaged build kits remain gates.

**Gzip foundation (2026-10-12):** the author requests System.IO.Compression and
identifies HTTP as a likely consumer. [GZipStream](gzip-compression.md) adds incremental
async compression and bounded validated decompression over existing stream contracts.
[Local qualification](../runtime/raven/tests/gzip-validation.json) covers native/JIT
contracts and external interoperability; interpreter qualification is recorded separately.
Automatic HTTP encoding/negotiation and incremental decoding remain follow-through;
this author-directed slice does not reorder the active milestone.

**Import/export and AOT sidecar direction (2026-10-12):** the author clarifies that
neoCLR uses direct structural function signatures, not delegates, and targets AOT
code artifacts with sidecar public-member and ABI mapping metadata consumable by
compatible runtimes and native tools. The [contract design](design/native-import-export-contract.md)
separates public type descriptions from native layouts/entry stubs and records
versioning, ownership and runtime-entry requirements. At the author's explicit
direction, main now includes interpreter address binding, call-scoped managed callbacks and
native callable results with independent ABI descriptors. Captured state, scalar
marshalling and runtime reentry are covered by the [callable contract](native-interop.md#callable-parameters-and-results-development).
Retained registrations, source compiler mapping, JIT/AOT, export generation and a
stable sidecar remain future expansion. The author explicitly requests merging this
bounded FFI slice now, deferring its compiler-readiness gate without promoting other
structural-type experiments or reordering the active platform milestone.

**Foreign function interface direction (2026-10-12):** the author requests platform
invocation/FFI beyond a Windows-oriented interface, retains `extern` plus attributes,
and calls for explicit calling conventions and individual value marshalling. The
[implemented source-to-interpreter slice](native-interop.md) adds NativeImport,
parameter/result MarshalAs, Cdecl/System, UTF-8/UTF-16 input buffers and Bool8/Bool32.
Native metadata preserves descriptors; lookup is lazy, and static class imports work.
Loaded metadata inspection retains the ABI descriptor and Raven imported symbols retain
the extern classification for both foreign imports and runtime internal calls.
The [focused source/native evidence](experiments/foreign-interop/README.md) covers
actual Raven attribute source and real foreign calls. JIT/AOT, installed bundles,
other-host qualification, bounded buffers/ownership, error capture and callbacks remain
gates. This author-directed slice does not permanently reorder the active milestone.

**End-to-end showcase target (2026-10-12):** the author identifies a neoCLR HTTP
server and client exchanging JSON as the ultimate showcase, possibly backed by
PostgreSQL running in Docker, with both applications possibly containerized too.
Continue toward common APIs rather than platform-specific public surfaces. The
[common async transport design](stream-design.md#common-async-transport-direction--2026-10-12)
connects Windows HTTPS qualification to reusable streams, explicit TLS and verified
client/server JSON round trips. PostgreSQL persistence and application containers
are optional later slices; qualify the selected container/runtime target separately.
This records a concrete consumer goal without replacing the active correctness gate
or claiming Linux JIT/AOT or Docker application support.

**Foundational security APIs (2026-10-12):** the author identifies async streams,
`System.Security.Cryptography` and certificate handling as necessary foundations
for further development. Follow the [bounded security API sequence](stream-design.md#security-and-certificate-foundations--2026-10-12):
common async byte I/O, certificate/key ownership and explicit trust, then managed
TLS composition and verified client/server JSON. Expose cryptographic primitives
as reusable capabilities backed by established implementations, with scope driven
by concrete consumers. The canonical InputStream/OutputStream contracts now return
Tasks, with text/JSON adapters and deferred HTTP uploads migrated. Memory, text and
JSON tests pass in JIT/AOT; memory also passes in the interpreter. File and console
host calls still block before returning settled Tasks. System.Tasks has moved to System.Concurrency.Tasks, including compiler/runtime
identities and consumers; [qualification](task-namespace-migration.md) covers the
three local execution modes. The [shared private TLS provider](shared-security-provider.md) now supports client/server
configuration and explicit authentication; its macOS/Windows CI gate is added, with
local macOS validation complete. Private secure-random/SHA-256/HMAC-SHA-256 primitives also pass local provider
tests, with [typed interpreter/JIT/AOT service qualification](experiments/native-cryptography/README.md)
now passing locally on macOS using a standalone native cryptography archive without
TLS dependencies. Prepared JIT bundles now include the provider, with private
cryptography cases passing through default CLI execution. Windows uses the pending
CI gates. Source AOT projects now link the provider on demand, with a Raven consumer
qualified in all three local modes. Packaged build kits remain open; managed SSL composition is qualified above. Typed private certificate/configuration errors and rejection
qualification now prepare that boundary. Private pollable close-notify shutdown
also passes TLS 1.2/1.3 backpressure and lifecycle tests. A checked resource context
now owns configurations/sessions behind non-reused IDs and bounded quotas; managed
ownership and runtime-service bindings now work in the interpreter through
heap-owned tokens. The native collector now has tested resource cleanup registrations
and teardown guards. A tested native TLS adapter now connects owner tokens to those
hooks. Explicit native JIT/AOT bindings now pass seven cases in all three local
modes. Prepared development JIT hosts and source AOT projects now integrate TLS
ownership/teardown; nine default-JIT cases and a Raven project pass locally.
A borrowed-buffer ciphertext pump now passes fragmented transport, backpressure,
shutdown and truncation checks on macOS, including its private C entry point; see
[transport evidence](experiments/shared-tls/buffered-transport-validation.json).
Managed pump bindings and SSL composition now pass locally; released build kits remain open. See [host evidence](experiments/shared-tls/host-integration-validation.json). See
[binding evidence](experiments/shared-tls/bindings-validation.json). See [native lifetime evidence](experiments/shared-tls/native-lifetime-validation.json). HTTP and SQL may share concepts/types
without requiring one native TLS backend; see the [backend boundary](shared-security-provider.md#shared-concepts-independent-backends). A bounded public System.Security.Cryptography facade
now adds destination-buffer SHA256/HMACSHA256, tag verification and secure random
fill; allocating/streaming cryptography overloads and broader certificate inspection
remain open. Explicit PEM credentials and the managed TLS adapter are qualified above. Compare .NET,
Windows Runtime and other platforms, retaining .NET as the ergonomics target.
See the [first async contract](stream-design.md#first-async-contract--2026-10-12).
The subsequent [async-first naming direction](stream-design.md#async-first-naming-direction--2026-10-12)
reserves ordinary operation names for asynchronous I/O and Sync for synchronous
alternatives when distinction is needed; the initial Async-prefixed stream interfaces
are removed. See [migration evidence](../runtime/raven/tests/default-stream-validation.json).

**Windows HTTPS follow-through (2026-10-12):** the author explicitly requests
Windows behavior consistent with macOS. Source linking and a shared validation
harness are implemented; a dedicated Windows CI gate now covers TLS versions,
rejection, fragmentation, timeout/cancellation, EOF/recovery and cleartext isolation.
[Windows AOT/JIT now passes 27 host cases plus two Rust transport tests](experiments/https-server/README.md#windowsmacos-parity-milestone--2026-10-12);
the shared 15 AOT cases match macOS. This closes the bounded source-host integration
gate, not managed TLS APIs or installed packaging. The author also requests async stream concepts
and a reusable secure-stream equivalent, clarifying that .NET is a reference rather
than a required hierarchy; evaluate other platforms before settling that API.

**HTTPS server direction (2026-10-12):** after discussing shared TLS, the author
explicitly selects HTTPS server support. The [native hosting proof](experiments/https-server/README.md)
puts Rustls behind runtime-owned nonblocking I/O, preserving Raven HttpServer handling,
deadlines, cancellation and recovery. It qualifies an explicit HTTPS-only host on
macOS ARM64 JIT/AOT, followed by Windows x64 above; managed per-listener TLS
configuration, Windows ARM64/interpreter integration and installed packaging remain open. The source project builder now
accepts an explicit `--https-server` switch, qualified with the shared Raven unary
await capture fix; [project evidence](experiments/https-server/project-validation.json)
covers twelve host cases plus fragmented transport. The subsequent
[authenticated-shutdown checkpoint](experiments/https-server/README.md#authenticated-shutdown-qualification--2026-10-12)
passes 14 project AOT cases, 27 runtime-async AOT/JIT cases and two Rust transport
tests, distinguishing clean TLS EOF from truncation while preserving buffered data. Client and PostgreSQL TLS are not
implemented by this server slice. This is bounded author-directed networking work;
the runtime correctness/performance milestone remains active.
Compiler follow-through also closes the 15 broader .NET async baseline failures:
[all 148 selected tests pass](experiments/top-level-async-entry/README.md), with
separate main/native-branch commits and no change to target-owned runtime startup.

**SQL data access direction (2026-10-12):** the author requests a small SQL provider,
PostgreSQL binding and Dapper-like mapper, explicitly avoiding an ADO.NET-shaped
abstraction, and selects submodules under System.Data. The bounded
[SQL connection slice](sql-data-access.md) defines common contracts in System.Data.Sql
and an optional local PostgreSQL provider in System.Data.PostgreSql. The author
clarifies that these are common SQL-engine contracts with explicit provider
extensions: keep PostgreSQL binding rules out of the mapper. Connection/statement and SQL-string query overloads now accept named-object and
positional inputs. [Local qualification](experiments/sql-postgresql/query-validation.json)
passes interpreter/JIT and shared-host AOT against PostgreSQL; standalone packaging
and Windows SQL remain unqualified. Process-isolated benchmarks remain next. Authentication/TLS,
remote servers and streaming remain follow-ups; this one-off direction does not
permanently reprioritize the active runtime milestone.

**Benchmark publication direction (2026-10-12):** publish HTTP request-processing
results as a plain-text, JSON and JSON-plus-database matrix across interpreter,
JIT and AOT. The native-compilation page should cover general execution performance.
The [website structure](design/feature-pages.md#benchmark-placement--author-direction-2026-10-12)
records graph/evidence requirements. Website reports now separate these concerns;
matched composed HTTP measurements and graphs remain pending. Standalone SQL
qualification or timing is not HTTP benchmark evidence.

**Collector and JIT follow-through (2026-10-12):** the author asks to reduce collector
cost with profiles, correctness tests and benchmarks, and reiterates that JIT should
become the default soon. The [collector traversal slice](experiments/runtime-async/native-profiling.md#fewer-collector-traversals--2026-10-12)
reduces controlled AOT runtime-async time about 10–13%, without changing collection
frequency. The author then explicitly directs completing a working default JIT.
The implementation selects JIT on macOS ARM64 and Windows x64, stages matching
helpers and libraries, and makes runtime-library tests use the omitted-mode default.
The [installed-default qualification](experiments/jit-cli/default/README.md) now
passes: 29 relocated macOS library suites, current Windows scalar/CLI/framework
gates and eight HTTP cases on each host. JIT is the qualified development default
on these hosts; Windows ARM64 and Windows runtime suspension remain open. Native
benchmark noise and the runtime-async performance gap remain separate work.
The author next explicitly requests fixing generic async and async `use` compiler
limits. The [compiler slice](experiments/runtime-async/generic-cleanup/README.md)
now passes both lowerings across interpreter/JIT/AOT on macOS, including retained
cleanup and generic captures. Windows x64 state-machine projects now also pass
interpreter/default-JIT/AOT, after fixing project cleanup admission and decoupling
explicit collection from runtime suspension. This compiler slice is complete;
Windows runtime suspension remains separate.

**Managed stack diagnostics (2026-10-12):** the author requests a StackTrace class
for printing the current call stack, then continued work on separate debugger and
profiler tools to understand the runtime. The bounded
[StackTrace slice](experiments/stack-trace/README.md) captures owned current-stack
text; structured frames, source mapping and causal async stitching remain follow-ups.
Readable native profile symbols and warmup-excluded GC summaries now pass a
200-invocation diagnostic run. Collection remains about two thirds of measured
diagnostic time; investigate collection work/pacing next with stress-GC validation
retained. Native debugger stepping, retention paths and causal stacks remain open.

**Attachable diagnostics direction (2026-10-12):** the author requests separate
tools, then clarifies that working runtime integration and useful provisional
development tools take priority over standalone packaging or a settled attachment
architecture. The [architecture and attach gate](runtime-diagnostics.md) remain
provisional. The [native GC attachment proof](experiments/runtime-diagnostics/README.md)
now passes repeated attachment during 128 HTTP requests in each of macOS ARM64
AOT and JIT, with fresh snapshots and verified request progress. Installed tooling,
other hosts/modes, multi-owner aggregation and debugger control remain open. The
[payload/pause measurement slice](experiments/runtime-async/native-profiling.md#payload-and-pause-measurements--2026-10-12)
provides bounded instrumentation and sanitized ownership checks; the attachment
layer builds on it without changing collection pacing.

**Runtime diagnostics direction (2026-10-11):** the author requests profiling the
remaining native runtime-async gap and building useful profiler/GC/debugger tools.
Start with [native sampling and collector diagnostics](experiments/runtime-async/native-profiling.md),
then measured bottleneck fixes and matched benchmarks. Keep native source mapping,
suspended logical stacks and GC lifetime/pause inspection visible as follow-ups.
Keep profiler and debugger as separate tools, as the author suggests, sharing
low-level diagnostic contracts where useful;
this is not a claim that the interpreter debugger already supports native execution.

**HTTP hosting direction (2026-10-11):** the author requests stronger HttpClient/
HttpServer support toward modern web applications and eventual WebSockets, followed
by better HttpContent/JSON ergonomics. The [bounded foundation](http-foundation.md)
adds practical buffered limits, shared request/response chunk parsing, informational
response handling and content-oriented JSON helpers. Streaming, TLS, persistence and
upgrade ownership remain subsequent gates; this does not qualify a production host
or permanently reorder the runtime-async milestone. The follow-up adds validated
[per-server admission and deadline options](http-server-options.md) within the
existing ceilings. Runtime ownership/GC changes remain coordinated with the
platform workstream; the 1 MiB transfer boundary remains unqualified.

**Concurrency ambition (2026-10-11):** the author aims for concurrency support
beyond .NET and at least on par with Go. This is a target, not a measured claim.
Assess usability and cancellation/cleanup correctness alongside suspended-operation
memory, multicore throughput, fairness and tail latency. Preserve matched .NET/Go
comparisons and compiler-state-machine controls; runtime-owned async alone is not
proof of an improvement. This reinforces the existing runtime-async milestone,
without bypassing lifetime or performance gates.

**Future GC tuning consideration (2026-10-11):** the author suggests evaluating
collector tuning later. Keep this exploratory; it does not reprioritize the current
compiler/runtime-async work or select a collector design. Establish suspended-root
correctness and abandoned-operation reclamation before tuning. Then measure allocation
rate, live/retained memory, collector CPU cost and pause distribution alongside scheduler
fairness and request tail latency under HTTP and JSON workloads. Compare .NET and Go
under matched workloads and resource budgets; distinguish fewer application allocations
from collector improvements. Collector policy, pacing, stack scanning and native carrier
memory are separate candidates to investigate, not implemented capabilities.

**Async iteration implementation (2026-10-11):** author-directed sequencing is the
managed AsyncIterable/AsyncIterator contract and await-for consumption, followed by
generator yield/resume. The interface and retained-body await-for slices pass all three local execution modes.
Native Raven `yield` and delegation now pass them using a source-local generic
producer helper. Nested producer cleanup also passes with pending disposal and
cancellation. Generic generator methods now also work on generic reference owners,
with nested generic metadata preserving lexical access in interpreter/JIT/AOT.
A private [native conditional-root collector primitive](experiments/aot-console/native-gc.md#conditional-ownervalue-roots--private-experiment-2026-10-11)
now passes sanitized ownership/cycle tests. An opt-in
[owned parked-frame registration](experiments/aot-console/native-gc.md#managed-ownership-of-parked-frames--private-experiment-2026-10-11)
also participates in automatic safepoints and expires unreachable stacks safely.
[Dependent host handles](experiments/aot-console/native-gc.md#dependent-host-handles--private-experiment-2026-10-11)
now share that marking fixed point, with explicit strong ownership required for
dispatch. The opt-in [native carrier now reclaims abandoned owned bodies](experiments/runtime-async/async-generators.md#owned-native-body-reclamation--2026-10-11)
in a sanitized 160-suspension contract test, including stack mappings and operation
slots. The provider now adds
[pending-task ownership links](experiments/runtime-async/async-generators.md#pending-managed-task-ownership-links--2026-10-11)
for results, advancement and disposal. The subsequent
[managed ownership integration](experiments/runtime-async/async-generators.md#managed-ownership-integration--2026-10-11)
connects external completions, automatic body/signal release and interpreter parity.
The returned Task now directly owns its producer registration, avoiding a per-body
lifetime notification. Runnable work retains strong ownership until dispatch; pending
abandoned cycles can be reclaimed without executing cleanup. The
[matched ownership benchmark](experiments/runtime-async/managed-ownership-benchmark.md)
closes the large completed-path callback regression; native runtime async remains
slower than the compiler-state-machine control. Platform and performance gates remain open. See
[scope, evidence and remaining work](experiments/runtime-async/async-generators.md).

**Compiler lines (2026-10-11):** the author directs keeping Raven main and the
neoCLR compiler branch separate. General fixes can be carried across deliberately;
main qualification does not qualify the native compiler bundle. See the
[branch audit and binder fix](raven-cli-bridge.md#compiler-lines-and-property-binding-fix-2026-10-11).

**Product and execution review (2026-10-11):** target simple, productive, fast native
web/backend development with familiar C#/.NET ergonomics, future language support,
and concurrency as a core capability. Evaluate suspension and green threads with
scheduling and memory ownership. The author selects JIT as the eventual default on
supported platforms, AOT as the preferred deployment target, and a fully supported
interpreter for constrained devices and explicitly supported dynamic-code scenarios.
Current execution defaults remain unchanged pending qualification. The author authorizes
switching to JIT when ready, with JIT as the routine test mode and selective relevant
interpreter/AOT checks. [CLI integration and readiness](jit-execution.md) now provide
explicit mode selection and test-harness selectors. Windows x64 backend/host support
passes its [Windows execution gate](experiments/jit-windows/README.md) at `f993e94b`.
Prior JIT deferral below is historical context, not a prohibition on this evaluation.
With the Index/Range slice complete locally, assess memory views and shared execution
requirements without treating any proposed API or backend as selected. See
[product direction](platform-direction.md#product-direction-clarified-2026-10-11)
and the [JIT evaluation brief](native-execution-investigation.md#jit-for-development-iteration-2026-10-11).

**Green-thread direction (2026-10-11):** the author clarifies that runtime async is
moving toward green threads. The interpreter POC now resumes detached callbacks and parents independently;
this is groundwork, not a completed green-thread implementation. The immediate
author-directed goal is runtime await as Raven’s default, with stability and
measured improvement. The POC now includes eager body invocation and runtime-owned integer-result
completion, plus retained outcomes for repeated/shared waits. Unit-result bodies
and ordinary `Task<()>` adapter consumption now pass the interpreter POC
([evidence](experiments/runtime-async/unit-validation.json)). Managed lifetime
projection remains open. Ordinary Raven await now passes through a source-local
compatibility adapter retaining generated state machines. The same-file attributed
intrinsic binder fix is now qualified on the separate native compiler line
(`f51d0561f`); [evidence](experiments/runtime-async/single-source-validation.json).
The adapter now propagates cancellation to managed Tasks without cancelling the
invocation ([evidence](experiments/runtime-async/cancellation-validation.json)).
Pending self-await now faults promptly; independent callback waits remain valid
([qualification](experiments/runtime-async/README.md#pending-self-await-diagnostic-2026-10-11)).
The author reaffirms that Fault remains non-recoverable and invocation-terminal.
Runtime async is the canonical model; compiler-generated state machines are a
migration path and a useful compiler regression oracle, with eventual retirement
as the goal. Qualification still precedes default switching.
Raven now has an explicit native runtime-async language POC: ordinary async methods
and Main lower to retained bodies without generated state machines. The
[language, fault and GC samples](experiments/runtime-async/README.md#raven-runtime-async-language-poc-2026-10-11)
pass, including zero final live objects and bounded heap use across repeated nested
suspension. The provider remains source-local; compiled support is qualified below.
[Matched interpreter benchmarks](experiments/runtime-async/README.md#matched-language-benchmark--2026-10-11)
now show about 16–21% less time at 100 operations, with 13–18% more allocations.
Improve notification/lifetime costs and broaden qualification before switching defaults.
The author further selects a [cross-language operation contract](runtime-scheduling-design.md#cross-language-asynchronous-operation-contract--2026-10-11),
independent of the concrete managed Task class, as a future design direction. A
[second runtime-backed operation](experiments/runtime-async/README.md#independent-operation-proof--2026-10-11)
now passes ordinary await, shared readers, cancellation and GC checks through explicit
provider mapping. An [imported generic interface proof](experiments/runtime-async/README.md#imported-operation-contract-proof--2026-10-11)
now lets pending and immediate implementations share one mapping across an assembly
boundary. The author clarifies that an awaitable-operation interface should be the
shared runtime interop contract, with platform-owned concrete implementations such
as managed Task; interfaces are primary, not exclusive, interop mechanisms.
**Subsequent author correction (2026-10-11):** defer pursuing or stabilizing this
interop contract. The immediate primary target is managed programs compiled to VM IL,
including Raven. Interpreter, JIT and native AOT all remain relevant targets for those
managed programs: managed describes the program model, not an interpreter-only
execution choice. Make those execution paths work before making major interface/ABI
design decisions. Preserve the interface proofs as exploratory evidence, not the
next milestone. End-to-end [HTTP showcase source qualification](experiments/runtime-async/README.md#managed-http-showcase-and-scoped-cleanup--2026-10-11)
now passes for both client/server and a combined eight-exchange consumer. The exposed
async-use gap is fixed behind an explicit retained-scope capability, with cancellation
cleanup and GC evidence. [Project-level opt-in](experiments/runtime-async/README.md#http-showcase-project-integration--2026-10-11)
also now qualifies both actual showcase builds, cleanup and configuration rejection,
with unchanged default lowering. [HTTP timeout/cancellation and terminal failure cases](experiments/runtime-async/README.md#http-lifecycle-qualification--2026-10-11)
now pass interpreter qualification. The author explicitly directs continuation until
JIT and AOT runtime suspension work. The [compiled POC](experiments/runtime-async/README.md#compiled-runtime-suspension-jit-and-aot--2026-10-11)
now qualifies both HTTP projects and lifecycle cases on macOS ARM64, with explicit
parked-stack GC roots and terminal teardown. Windows and a production stack carrier
remain open, as do native Task.Run and full library migration. A
[matched three-mode benchmark](experiments/runtime-async/README.md#matched-interpreter-jit-and-aot-measurement--2026-10-11)
now measures both lowerings: interpreter invocation time improves, but the native
POC is slower than state machines. [Bounded stack reuse](experiments/runtime-async/README.md#bounded-native-stack-reuse--2026-10-11)
now cuts about 41–45% from the uncached AOT microbenchmark time and records reserved
stack bytes with zero reservation after teardown. Native suspension remains slower
than state machines; context-switch/notification/GC costs and resident memory remain
open. The author-directed await-in-for compiler fix now passes
[loop lifetime qualification](experiments/runtime-async/README.md#await-inside-for-both-lowering-paths--2026-10-11)
in both async lowerings and all three local execution modes, including cancellation
and outward loop exits. Shared Raven fixes remain separate from native integration.
The author also requested [async-generator investigation](experiments/runtime-async/async-generators.md):
Raven/.NET support is distinct from missing neoCLR async-iteration contracts and
retained-yield lifetime. The proposed follow-up is exploratory and does not change
this milestone's readiness priorities.
Retain cancellation, terminal Fault, GC and benchmark checks. Broaden
platform coverage and measure costs before considering defaults. This is explicit
POC support, not a default-mode or native build-kit change. General native interoperability, runtime interface recognition,
native stack representation, preemption and multi-core scheduling remain open.
Task/Promise keep operation/producer roles.

**Runtime async groundwork (2026-10-11):** the author directs implementation while
keeping Raven's CLR lowering separate from neoCLR's execution choices. A private
[interpreter retained-frame yield](experiments/runtime-async/README.md) now exercises
ordinary call-chain retention and resumption. The subsequent interpreter POC parks
on private timer/manual operations and resumes with a result. Detached activation
handshakes now pass; public Task/HTTP integration, native support and compiler await integration remain
open. This does not complete public Task migration or the scheduler measurement gates.

**Latest author clarification (2026-10-11):** document testing and benchmarking as
[independent](../tools/testing/README.md) [projects](../tools/benchmarking/README.md),
including their runners/APIs (completed), then continue scheduler work followed by suspension.
The first resumed scheduler slice centralizes interpreter wait policy and timer
deadline hints. Native JIT/AOT hosts now share a
[completion-state boundary](experiments/native-scheduler/README.md), with explicit
ready/pending/idle results. Outer HTTP hosts now use
[owned delivery tickets](experiments/native-scheduler/README.md#owned-dispatch-slice--2026-10-11)
with explicit completion independent of operation-result consumption. [Generated-entry acknowledgment](experiments/native-scheduler/README.md#owned-entry-delivery--2026-10-11)
now covers JIT and HTTP AOT entry adapters, including fault cleanup. Await-site routing
and cross-mode wait policy remain open.
Migrate more cases as needed; remaining benchmark/framework features are follow-ups,
not prerequisites for beginning the scheduler slice. Parallel-by-default testing and
class grouping are proposals to develop with scheduling, not enabled behavior.

**Queue visibility clarification (2026-10-11):** task queues are implementation
details. Migrate away from the visible TaskQueue dependency; do not rename it into
an application-facing Scheduler without changing its responsibilities. The author
subsequently selects a managed `Scheduler` API backed by runtime implementation;
Task/Promise remain managed and queues private. The runtime exposes its own ABI hooks;
Task and Scheduler project them in the managed environment, with native-language
projections possible later. ABI signatures remain unselected. Exact members and customization
remain open. The next [routing migration](runtime-scheduling-design.md#queue-independent-scheduling--2026-10-11)
separates runtime-owned continuation destinations from producer queues, with coordinated
consumer/API migration. Interpreter completion admission now retains invocation-owned ready work without a
queue binding; its VM dispatch adapter still resolves the default queue at installation.
A [private native submission hook](experiments/native-scheduler/README.md#runtime-submission-prototype--2026-10-11)
now defers a rooted managed HTTP callback through runtime-owned admission/dispatch
without a Task or managed queue. A [source-included managed Scheduler prototype](experiments/scheduler-projection/README.md)
now projects a reserved submission service in interpreter/AOT/JIT and preserves
compiler-generated state machines in pending/completed await and fault cases. It is
not yet the public class-library Scheduler API or a stable foreign-language ABI. Current public TaskQueue behavior remains implemented
until the replacement is qualified. A [continuation-routing experiment](experiments/scheduler-continuations/README.md)
now qualifies pending/completed awaits, multiple awaiters, cancellation and terminal
faults without pumping an explicit producer queue in interpreter/JIT/AOT. It uses an
internal awaitable; it now captures an invocation destination per registration and
rejects foreign/expired destinations before retaining callbacks. Production Task
registration and cross-invocation routing remain open. The internal awaitable now
uses runtime-owned dormant registrations with one-shot activation/cancellation;
Task remains the operation/state abstraction and Promise its producer-side completion
source. Explicit observers now have scoped disposable registrations, qualified for
capacity reuse, early return and no retraction after activation in all three modes.
Automatic await abandonment cleanup and race/lifetime rules remain production-migration gates. A
[conditional-root collector experiment](runtime-scheduling-design.md#conditional-continuation-roots--collector-experiment-2026-10-11)
now validates live-owner retention and abandoned-cycle reclamation in the interpreter
heap. A [managed-storage alternative](experiments/scheduler-continuations/README.md#managed-pending-ownership-alternative--2026-10-11)
now proves reclamation of abandoned Raven awaits and preservation of live operation,
submission and timer roots under interpreter collection. Shared lifecycle cases pass in
all three local modes. Prefer this simpler ownership provisionally; native reclamation,
completion-capacity policy and production migration remain open. A
[batch adapter](experiments/scheduler-continuations/README.md#completion-batch-admission--2026-10-11)
now admits 128 awaiters using one runnable slot in all three modes, before outcome
publication. The adapter now returns to runtime arbitration between callbacks;
[incremental evidence](experiments/scheduler-continuations/README.md#incremental-batch-turns--2026-10-11)
qualifies older-work progress in all three modes. Interpreter runnable storage now grows
while preserving reserved activation slots. Native submissions now use an
[independent rooted backlog](experiments/scheduler-continuations/README.md#independent-native-runnable-backlog--2026-10-11),
qualified with a full timer table, 128 queued callbacks and root-budget saturation.
Native resource budgets, explicit-GC diagnostics, concurrency and performance qualification
remain gates before public migration. The original runtime
registration service still retains strong roots. The interpreter readiness query now includes
submitted work and keeps source/submission arbitration private;
[quiescence contract and validation](runtime-scheduling-design.md#unified-interpreter-readiness--2026-10-11).

**Benchmark requirement (2026-10-11):** retain a compiler-generated state-machine
baseline and measure scheduler/suspension changes as they develop. Compare equivalent
workloads within each execution mode, isolate scheduler changes from suspension changes,
and separate startup from steady-state latency/throughput, allocation and retained
memory. See the [measurement gates](runtime-scheduling-design.md#scheduler-and-suspension-benchmark-requirement--2026-10-11).
Correctness qualification alone establishes no performance improvement. A first
[state-machine baseline](../benchmarks/scheduler/README.md) now measures completed and
pending sequential awaits through the existing external runner in interpreter/JIT/AOT.
A [matched registration comparison](../benchmarks/scheduler/registrations.md) now
measures direct callbacks against captured invocation registrations in the same
pending-await workload. All eighteen runs pass correctness checks, but timing
variation leaves capture overhead inconclusive. Allocation and retained-memory instrumentation, broader
workloads and the future runtime-suspension comparison remain open.

**Common language runtime clarification (2026-10-11):** the primary target is managed
languages running bytecode through the interpreter or JIT, with native AOT as the
publishing target. Runtime services should be opt-in through explicit contracts;
future native-language/runtime integration points extend this foundation, including
managed-to-native calls and native-to-managed entry. The
[runtime architecture proposal](proposals/runtime-architecture.md) remains exploratory,
not a commitment to implement every participation model before managed execution.
Opting into a service still requires its ownership, safety and lifetime rules.

**Async foundation clarification (2026-10-11):** the runtime should own asynchrony,
with language implementations and eventually native languages/other runtimes able to
project and integrate its lifecycle. Raven callbacks are a transitional projection,
not the definition of suspension. The [design boundary](runtime-scheduling-design.md#runtime-owned-asynchrony-and-language-projections--2026-10-11)
keeps completion, scheduling and retained execution ownership separate. Continuation destination
and cross-mode wait policy remain the next bounded gates; foreign-runtime ABI and
suspension representation are not yet selected or implemented.

**Author-directed sequence (2026-10-11):** finish JIT qualification and integrate
that work into main (completed at `33648ce6`), then produce execution/HTTP and JSON
serialization benchmarks before moving from
the TaskQueue concept to schedulers, followed by
runtime-managed suspension. The intended suspension model lets the runtime handle
suspension points instead of requiring Raven-generated async state machines. This
is a direction, not implemented behavior or a selected stack representation. The author
also requires [faithful logical async stacks](runtime-scheduling-design.md#logical-async-stacks--author-direction-2026-10-11)
across suspension, retaining source/call relationships with compatible state-machine
diagnostics. This is an acceptance goal, not an implemented scheduler feature.
The [HTTP workbench](../benchmarks/jit-http/README.md) now passes 1,152 requests
across all three modes; initial timing is provisional due to background contention.
The [first benchmark runner](../benchmarks/framework/README.md) adds manually
registered Raven workers and JSON node serialization/deserialization batches.
Benchmark attributes now discover module functions through metadata and generate
typed registrations, with final-assembly verification and AOT execution coverage.
The JSON deserializer now uses discovery alongside manually registered serialization
to preserve setup outside measurement; both pass all three modes.
Independent testing and benchmarking data attributes now generate per-row cases:
inline scalar arguments and typed array providers, with filtering and all-mode
execution checks. Benchmark measurements are configurable; broader provider shapes
and lifecycle hooks remain gaps. Typed-object/collection JSON and internal operation
timing remain open.
The author also requests a [BenchmarkDotNet-like framework](benchmarking-framework.md),
combining external process orchestration with Raven workload measurement.
Experimental API/compiler compatibility may be broken where justified. Compare the
scheduler contract with .NET TaskScheduler/ThreadPool and the suspension mechanism
with compiler-generated async state machines; document benefits, costs and evidence
before claiming improvements. Evaluate GC roots, cancellation, cleanup, fairness,
I/O wakeups, debugging and interpreter/JIT/AOT parity. Green threads remain a design
candidate rather than an automatic consequence of this decision. See the existing
[scheduling design](runtime-scheduling-design.md) for inputs, not a replacement for
this sequence.

**Iteration evaluation progress (2026-10-11):** the runtime-library harness now
records discovery/build phases and repeatable process timings. The [local evaluation](experiments/jit-iteration/README.md)
compares existing debug-built and release-built AOT tools while checking HTTP handler
lifetime and Index/Range behavior. It recommends a bounded JIT shared-lowering probe,
followed by real test-framework execution. The [scalar JIT probe](experiments/jit-scalar/README.md)
now implements in-memory native invocation on macOS ARM64 with shared AOT lowering;
the [managed JIT probe](experiments/jit-managed/README.md) now runs the real 20-test
Index/Range framework assembly with filtering/fault parity. The
[async entry follow-up](experiments/jit-async/README.md) qualifies queued completion,
consecutive awaits, cancellation, pending-task and callback faults in all three modes.
[Timer completion/cancellation](experiments/jit-timers/README.md) is also qualified.
[HTTP client/server qualification](experiments/jit-http/README.md) now passes with
DNS/connect, accept/read/write, timeouts and fault cleanup. Windows x64 JIT also
passes its execution gate; production integration remains open beyond the explicit
development CLI path. A nine-run local
benchmark records median process times of 2.346s JIT, 2.962s interpreted and 9.00ms
prebuilt AOT for that same assembly; JIT includes compilation, AOT excludes its build.
This is test-run evidence, not HTTP throughput. The subsequent
[shared startup optimization](experiments/jit-managed/startup-optimization.md) reuses
immutable candidate indexes during validation and verification, with a balanced
before/after comparison for both JIT and interpreter. Continued interpreter improvement
remains part of the direction; sustained-workload and size-scaling evidence are still open.
A subsequent [retained-code CPU experiment](experiments/jit-cpu/README.md) now
checks repeated scalar invocations against AOT and the interpreter, separating
computation from metadata size. The bounded timer and HTTP milestones are now qualified
on macOS ARM64. Normal runtime/tooling integration and broader service qualification
remain the next functional work; these measurements do not make the private probe a
production execution mode.
The author also requests a later [equivalent HTTP client/server benchmark](native-execution-investigation.md#later-benchmarking-against-net-and-other-platforms)
against .NET, Go and JavaScript across each runtime's supported interpreted/JIT/AOT
modes. This remains planned; the current HTTP checks establish correctness only.
Memory views and suspension remain related design work.

**Module story closure (2026-10-10):** the author requests closing the namespace/module
authoring and metadata story so work can move on. The accepted model and implemented
boundary are [recorded here](declaration-modules.md). Keep Namespace as the string and
Module as the assembly-scoped descriptor; no aggregate module API or resolver redesign
is required. Interpreter/macOS ARM64 shared-namespace qualification passes at 61e687bd.
The final [Windows gate](https://github.com/marinasundstrom/neoCLR/actions/runs/38056974148)
passes at 61e687bd; artifact hashes have not been independently checked. This does
not qualify the later guest-function discovery changes or complete native introspection.

**Discovery progress:** the testing-framework follow-up establishes
the guest module-function metadata enumeration contract needed to discover TestAttribute
methods, reusing host discovery behavior and retaining manual registration. Native type
enumeration, general callable invocation, grouping, broader context discovery
and the imported function-reference emitter gap remain separately scoped follow-ups.
The runtime definition selector now validates exact assembly/module ownership and
preserves source function order without execution; its native bridge requires an
explicit module catalog. Four focused checks pass. Ownerless method/parameter and method-attribute recipes now pass focused backend
checks, including absent DeclaringType, fixed/named descriptions and excluded parameter
targets. Guest ModuleInfo.GetFunctions and method-level attribute queries now pass an
interpreted Raven consumer using the real TestAttribute, without invoking bodies.
Open generic signature snapshots remain explicitly unsupported. Explicit native
enumeration and method-attribute retention now pass a macOS ARM64 JIT/AOT consumer;
in-process callable registration remains open. These remaining
module-function discovery/reflection gaps, including ownerless parameter/return
attributes, are queued in [issue #27](https://github.com/marinasundstrom/neoCLR/issues/27).
This follow-up does not change the active milestone priority. The
[2026-10-12 execution baseline](experiments/guest-functions/execution-baseline/README.md)
passes explicit interpreter discovery and identifies `TypeDeclaringType` as the
first shared JIT/AOT prerequisite, before module-function enumeration.
[Retained declaring-type queries now pass](experiments/declaring-type/README.md)
a focused interpreter/JIT/AOT consumer on macOS ARM64. The subsequent [interface-to-object dispatch fix](experiments/guest-functions/interface-object-dispatch/README.md)
passes seven native Object regressions. The subsequent [schema-5 retention milestone](experiments/guest-functions/native-retention/README.md)
passes full discovery in interpreter, JIT and AOT, including negative checks for missing
catalog/function/attribute retention. The [CLI follow-up](experiments/guest-functions/native-retention/cli-validation.json)
qualifies `neoclr run --reflection-roots`, preserving explicit retention policy and
rejecting its use in interpreter mode. Windows native discovery remains unqualified. The guest-function
Windows run at 5e4cbd84 exposed an AOT shared-provider binding regression in nominal
attribute inspection; [the fix passes locally](experiments/native-attributes/shared-provider-validation.json)
with rebuilt libraries. Windows requalification remains pending.

**Author priority correction (2026-10-10):** after making guest discovery work, focus
on making the test framework useful through runner filtering before grouping.
The runner now implements exact ID and literal ordinal ID/name substring selection,
keeping manual registration and validating all registrations before filtering.
Author-directed dependency: close native Main(string[]) admission/host support so
filtering uses explicit entry arguments. [Native/interpreter qualification](../runtime/raven/tests/filtering-validation.json)
passes 32 executions and five discovery-signature rejection cases. Windows is pending. Grouping attributes remain later work;
expression parsing and case-insensitive substring matching are explicit future gaps.
Author follow-up: use this framework for new runtime-library behavior tests and add
focused tests alongside feature/fix work. Preserve compiler/backend coverage where
behavior is not observable in Raven; migrate existing tests as their areas are touched.
Author follow-up explicitly starts migration of existing program-form checks. The
[first five map ports](../runtime/raven/tests/collections/MapMaterialization.rvn) pass
within the 12-case native/interpreted collection suite and through `--filter Map`.
[Migration evidence](../runtime/raven/tests/map-migration-validation.json); retain the
old integration smoke consumer until Windows equivalence is qualified. Six subsequent
[queue/stack ports](../runtime/raven/tests/collections/QueueStack.rvn) cover growth,
reference identity, clear/reuse and snapshots; the expanded 18-test suite and existing
selectors pass both modes ([evidence](../runtime/raven/tests/queue-stack-migration-validation.json)).
Author-directed batch migration adds the remaining collection scenarios together,
with 46 tests split across three projects to stay within the current native metadata
encoder envelope. This packaging limit remains a separate gap, not a test API rule.
[Batch qualification](../runtime/raven/tests/collection-batch-validation.json): all 46
tests and existing selectors pass native macOS ARM64 and interpreted execution.
Windows qualification remains pending.
The next batch ports 12 public JSON DOM/document checks; all pass native macOS ARM64
and interpreted execution ([evidence](../runtime/raven/tests/json-dom-batch-validation.json)).
A further 12 JSON stream tests cover partial I/O, borrowed ownership, errors and
byte quotas; all pass native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/json-streams-batch-validation.json)). Independent
MemoryStream and reflection-mapping migration remain follow-ups.
Batch `memory-stream` adds 5 passing native/interpreted tests for MemoryStream seek, range, quota, overwrite and closed-state behavior
([evidence](../runtime/raven/tests/memory-stream-validation.json)).
Batch `string-construction` adds 5 tests for String character-sequence copying, graphemes and named arguments qualified in native macOS ARM64 and release-interpreted execution after fixing Char projection and String factory initialization
([evidence](../runtime/raven/tests/string-construction-validation.json)).
Batch `unicode-casing` adds 6 tests for Unicode casing expansions, contextual sigma and text preservation qualified in native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/unicode-casing-validation.json)).
Batch `int64-parsing` adds 5 tests for Int64 boundary parsing, lexical errors, overflow and formatting qualified in native macOS ARM64 and release-interpreted execution after implementing the missing native binding
([evidence](../runtime/raven/tests/int64-parsing-validation.json)).
Batch `primitive-parsing` adds 12 tests for primitive numeric and Boolean parsing values, format errors and overflow qualified in native macOS ARM64 and release-interpreted execution after fixing floating generic storage and parser bindings
([evidence](../runtime/raven/tests/primitive-parsing-validation.json)).
Batch `path-values` adds 5 tests for Path equality, hashing, display and colliding map behavior qualified in native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/path-values-validation.json)).
Batch `http-header-values` adds 5 tests for HTTP header repetition, lookup validation, snapshots and request metadata qualified in native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/http-header-values-validation.json)).
Batch `http-route-matching` adds 8 tests for HTTP route parsing, captures, decoding, numeric conversion and quotas qualified in native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/http-route-matching-validation.json)).
Batch `ip-address-values` adds 7 tests for public IPAddress parsing corpus, canonical formatting, equality and allocation churn qualified in native macOS ARM64 and release-interpreted execution
([evidence](../runtime/raven/tests/ip-address-values-validation.json)).
Batch `uri-resolution` adds 11 passing native/interpreted tests for URI resolution,
grammar, equality and byte quotas ([evidence](../runtime/raven/tests/uri-resolution-validation.json)).
This completes local migration of the ten selected batches: 69 added cases, 139 total
library tests. The Windows action at 898fc8a4 subsequently completed successfully
([run](https://github.com/marinasundstrom/neoCLR/actions/runs/38067140698));
its artifacts have not yet been independently audited. Gaps exposed by these tests were fixed rather than retained as native
admission exceptions. Final Object-slot/shift checks and Windows object emission pass;
Windows runtime qualification remains pending. A subsequent slice adds Map getter
indexers and MutableMap/HashMap setters, preserving safe Find lookup. Its Windows
[gate at ee21653d](https://github.com/marinasundstrom/neoCLR/actions/runs/38067619388)
now reports success; artifacts have not been independently audited. Later JsonObject
and civil-parser Windows runs are still in progress as of this review. JsonObject now also implements a read-only Map view with safe lookup, a faulting
string indexer and ordered snapshots. Its expanded 16-case JSON suite passes both
local modes ([evidence](../runtime/raven/tests/json-map-validation.json)), with four
isolated missing-field checks. Native serializer fallback admission and retained-type
conformance gaps exposed by the new tests are fixed; nominal reflection still needs
explicit retention. The subsequent civil parsing slice adds current/explicit-culture Parse on Date,
TimeOfDay and LocalDateTime; all eight cases pass both local execution modes
([evidence](../runtime/raven/tests/time-parsing-validation.json)). Native host-culture
lookup now has the same preference source as the interpreter and a parity gate.
Keep Windows qualification and duplicate-name/reference-order coverage visible as
validation work; do not redesign module semantics to address those checks.

**Author-directed testing slice (2026-10-10):** establish a Raven-written runtime-library
[test framework and runner](../runtime/raven/tests/README.md), with module-level
functions and an intended stable core contract. Explicit registration, assertions,
structured pass/fail/skip reports and continuation after assertion failure now pass
macOS ARM64 native/interpreter checks. The Windows runner action at 9db8eb4c succeeds; its artifacts have not yet been independently verified.
The author clarifies that this requires [general custom attributes](custom-attributes.md),
including AttributeUsage and applicable declaration kinds, not a test-only mechanism.
Host metadata authoring/introspection now preserves callable, field, property and
parameter annotations alongside types. Int32 enum fixed arguments and primitive named
arguments now round-trip through host metadata and native/CLI projection. Raven's
native importer now exposes these attributes and applies existing usage target and
multiplicity checks across assemblies. Native source annotation emission now passes
focused compiler round trips on the shared Raven integration line; the development Windows compiler pin now advances to Raven 9b04250e2. Runtime usage
declarations pass the [usage consumer](experiments/attribute-usage/README.md) on
macOS interpreter/AOT, including imported target rejection. Windows usage action
[38047067362](https://github.com/marinasundstrom/neoCLR/actions/runs/38047067362) succeeds at 4455880d; its artifact hashes are not independently checked.
External base identity, bounded fieldless authoring and constructor chaining now
pass host metadata and Raven interpreter/macOS AOT checks. TestAttribute supplies
constructor/named descriptions. The first [discovery adapter](../runtime/raven/tests/README.md)
uses host introspection over compiled native metadata and generates typed Raven
registrations; explicit registration remains supported. This is not guest in-process
module discovery or a compiler source-generator contract. Grouping with selective
execution is an author-requested follow-up; keep groups separate from IDs and descriptions.
Named field/property argument
snapshots now pass the expanded macOS interpreter/AOT consumer with source-built
libraries; the expanded [Windows action](https://github.com/marinasundstrom/neoCLR/actions/runs/38045805683) succeeds at 96ad94e2; its artifact hashes have not yet been independently checked. Explicit attribute-data retention now passes a standalone macOS ARM64
[public Raven consumer](experiments/native-attributes/README.md), using metadata-authored
annotations and no user constructor roots. The matching [Windows x64 gate](experiments/native-attributes/windows-validation.json)
passes at 6ae823c0, with available input hashes checked. Guest module scanning and general reflective callable adapters remain open. This supports the native/collection milestone; it does not
permanently reprioritize the rest of the platform roadmap.

**Author follow-up (2026-10-10):** after attributes support and the testing framework,
complete modules as part of the Introspection and RuntimeContext model. Build on
the existing ModuleInfo/AssemblyInfo contracts; audit module identity, ownership,
module-level function discovery and context traversal across interpreter and AOT.
Compare .NET metadata modules with Raven source modules explicitly before choosing
any additional API. This is the next sequenced task, not a claim of completed support;
see [the module follow-up](introspection-design.md#module-model-completion--author-direction-2026-10-10).
Host introspection now exposes logical modules directly through ModuleInfo and
assembly/context GetModules(), consolidating the temporary physical/declaration
facade split. Members and nested types retain canonical logical ownership; physical
token scopes remain explicit reader details. Raven's importer traverses every module. The [host migration evidence](experiments/declaration-modules/host-validation.json)
covers metadata contracts, a real compiled consumer and all three interpreter/macOS
AOT suites. The coordinated Windows action [38051487728](https://github.com/marinasundstrom/neoCLR/actions/runs/38051487728)
passes at 2e4d5b6d with Raven 1a0c4e627; its artifact hashes are not independently checked.
Guest traversal through RuntimeContext.Current.ExecutingAssembly now returns flat
logical modules in the interpreter; type/member/parameter ownership agrees and the
physical ModuleInfo.MetadataToken property is removed. AOT materializes type-to-module
snapshots for explicitly retained nominal definitions, including Module.Assembly
and AssemblyInfo.Name/FullName ownership inspection. Explicit schema-4 catalog roots
now support native AssemblyInfo.GetModules, including empty declarations, with a
missing-catalog fault. Native RuntimeContext.ExecutingAssembly now retains the lexical
caller assembly through direct query calls, including separate-library helpers and
callbacks, without requiring reflection roots. Context-wide discovery, type enumeration
and general callable invocation remain open. Guest module-function enumeration and
method-level attributes now pass the explicitly retained macOS ARM64 interpreter/JIT/AOT
consumer described above. See
[the guest consumer](experiments/guest-modules/README.md).

**Author clarifications (2026-10-10):** a module is a unit and namespace of members
within an assembly. Assemblies package one or more modules. Root namespaces and
submodule names are conventions based on qualified names, with **no hierarchy in
metadata**. Preserve the existing flat module table; do not synthesize parent modules.
The author supplies System.Runtime → System/System.Networking and Acme.CoffeeMaker →
Acme.CoffeeMaker/Acme.CoffeeMaker.Factories as examples. Compatibility with development
APIs is not a constraint. Namespace/module ownership should guide understandable
library distribution; new reference/ambiguity policies remain separate work.
See [the corrected direction](introspection-design.md#author-clarification-one-semantic-module-model-2026-10-10).

**Latest author clarification (2026-10-10):** the same module namespace may span
multiple assemblies. Imports/references to a module name combine matching declarations
across referenced assemblies, with .NET-style namespace and ambiguity semantics.
Dotted names follow namespace convention, without a native metadata hierarchy.
Assembly identity belongs to each declaration and to assembly-local contributions;
it does not make the namespace exclusive. The author further clarifies that modules
simply mean namespaces: keep .NET's naming/packaging flexibility and the existing
namespace projection and resolution baseline. Do not introduce a separate resolver
or module-loading model. The metadata module represents that same namespace;
assembly-scoped ModuleInfo records the assembly's contribution, not another module
concept. Prefer naming the assembly after its main/root module namespace, as a
convention rather than a compiler or metadata constraint. Keep NominalTypeInfo.Namespace
as the string name and Module as the assembly-scoped descriptor. Existing Raven
namespace lookup now passes the shared application/library types-and-functions
consumer on interpreter/macOS AOT with one import and no resolver changes. Duplicate
name/reference-order coverage and the introspection scoping needed for discovery
remain follow-up work; a separate aggregate descriptor is not a requirement. This clarifies the current module milestone rather than
reprioritizing unrelated capabilities. See the [current contract and implementation gap](declaration-modules.md).

## Active direction: usable native compilation (2026-10-09)

After Preview 13, the author selects native compilation as the next priority and
directs implementation in validated, individually committed slices. Make an
ordinary Raven project produce a standalone executable through a reproducible
toolchain workflow. Begin with macOS ARM64 console applications, then the existing
HTTP workload. Close required core/bootstrap gaps while tracking full bootstrap
completion independently. Preview 13's interpreter qualification remains unchanged.

Acceptance: explicit supported scope and unsupported-feature diagnostics, packaged
tool/dependency provenance, execution without an installed managed runtime,
interpreter/native behavior and fault parity, cleanup evidence, and scoped startup,
size, memory and sustained-request measurements as applicable to each consumer.
The first implementation slice should compose existing compiler/backend contracts;
it need not invent a new compiler API or claim general native coverage.

Next, qualify a bounded Windows x64 scalar/console port before extending GC and
HTTP services. This changes both OS and architecture relative to macOS ARM64;
existing Windows interpreter support does not qualify its native backend.
Follow with an experimental persistent native host (state, roots, callbacks,
faults and shutdown), then compatible-body hot reload with explicit code-generation
lifetimes. JIT is not a prerequisite for reload. API design and focused RavenDoc
repairs support these consumers; broad structural types, a wholesale RavenDoc
rewrite and JIT remain deferred until concrete needs justify them. Existing
structural experiments retain their branch restrictions.

**HTTP showcase clarification (2026-10-09):** next make both HttpClient and
HttpServer work consistently and predictably on macOS and Windows. Use one shared
HTTP/network contract and qualify both platforms with the same scenarios. The
[parity plan](native-http-parity.md) tracks the shared socket adapter, Windows
server host and native DNS/outbound-connect bindings. The first
[shared socket gate](native-network-parity-validation.json) passes all three
listener/accept/transfer consumers on macOS ARM64 and Windows x64 at `da8ca1f2`.
The subsequent [Windows HTTP project gate](windows-native-http-project-validation.json)
passes at `353188bf`: standalone greeting, fragmented input, duplicate-length
rejection, handler errors and callback faults, with interpreter parity on Windows
x64 and macOS ARM64. The subsequent [await-based client/server gates](native-http-await-validation.json)
pass at `fbd73677`: ten client cases and five server cases, including paired Raven
processes and faults, on Windows x64; corresponding macOS cases pass. The author also requests
native Windows ARM64 support. Track it as an additional target, with explicit
ABI, stack-probing, host/toolchain and native-execution gates; do not equate x64
emulation with ARM64 qualification. The author prioritized Windows HTTP project
integration first; that bounded gate now passes. The [serial reuse gate](native-http-reuse-validation.json) now passes on macOS ARM64
and Windows x64 at `15ebc8b5`, including completed task factories, pending/continuation
fault recovery and cancellation recovery on one heap/context. Next address explicit
retained state/root ownership and broader persistent-host contracts; the HTTP
host gate is deliberately stateless. The next
[private retained-session experiment](native-retained-sessions.md) now supplies
owned roots, fail-closed dispatch after faults and explicit shutdown admission.
[Lifecycle and generated-callback gates](native-retained-session-validation.json)
pass on macOS ARM64 and Windows x64 at `c77e2b07`, with matching fault output. An explicit opt-in callback bootstrap now transfers a strong handle into the session
without heap scanning; the [handoff gate](native-bootstrap-handoff-validation.json)
passes on macOS ARM64 and Windows x64 at `201f567b`. A [retained-state HTTP consumer](experiments/retained-http/README.md) now uses that
handoff for three serial exchanges with collection between calls and terminal-fault
shutdown. Its [qualification](native-retained-http-validation.json) passes three
matching scenarios on macOS ARM64 and Windows x64 (`84b6e425`). General async export completion and broader persistent-host contracts
remain open before reusable hosting or hot reload can be claimed.

**Author JSON direction (2026-10-09):** next make typed JSON serialization work in
native HTTP, including the introspection and reflection support it needs. The
[native JSON plan](native-json-plan.md) takes priority over further general hosting
and reload slices. Reuse the existing serializer and qualify typed client/server
payloads consistently on macOS ARM64 and Windows x64. Closed type-token storage and
identity and exact runtime TypeEquals, TypeName, TypeArgumentCount and TypeShape services
for primitive/closed nominal and vector token producers are implemented. A standalone native
consumer executes descriptor queries with interpreter parity. A build-time
source metadata catalogue now preserves
selected nominal declarations and closed property signatures through specialization
without rooting accessor bodies. Primitive boxing/unboxing and interface-valued arrays now advance the JSON
probe through checked vector creation/readback to property reflection. ObjectTypeHandle now maps source class
and primitive box identities; the public Raven GetType/typeof probe passes on macOS.
Explicit source-identity roots and checked parameterless construction now run
through the public Raven reflection wrapper on macOS, with the low-level constructor
gate qualified on macOS/Windows. Retained Int32/Boolean/String/reference vectors now
support creation, readback, length and semantic GetType identity; the public Raven
array consumer passes on macOS and the portable gate is qualified on macOS/Windows
([evidence](native-reflection-array-validation.json)). Explicit property metadata and
getter/setter policies now generate checked accessor adapters; focused macOS tests
cover access/value errors, real effects and user faults. Rooted property snapshots and vector element queries now execute through the public
Raven facades, and the flat typed JSON round trip passes on macOS. The author expands
the milestone to nested objects, arrays, typed lists/sequences and string-keyed maps;
the built-in list/sequence and string-keyed map mapper now passes macOS
native/interpreter round trips, including nested collections and preflight before
model code. Windows x64 native/interpreter collection qualification also passes at
`9a8f07a7`; [verified evidence](windows-native-json-collection-validation.json)
records the source inputs, downloaded artifacts and standalone dependency checks. The existing nested object, DOM and
scalar/object vector corpus now passes on Windows x64 with interpreter parity
([evidence](windows-native-json-validation.json)), alongside macOS ARM64.
Native GetGenericArgument now inspects retained closed collection type arguments,
with nested generic identity and invalid-index parity checks on macOS. Collection
reads construct ArrayList and ordinal HashMap using checked generic library adapters;
custom collections and non-string keys remain unsupported. See the
[collection design and bounds](native-json-plan.md#built-in-collection-mapping--development-2026-10-10).
Author-directed explicit JSON subtree support now passes in mixed typed models
(JsonValue/concrete node roots, properties and vectors) on macOS in both modes;
Object declarations do not infer nodes. Windows action
[37993972338](https://github.com/marinasundstrom/neoCLR/actions/runs/37993972338)
also passes at `891de0bb`; its artifacts have not yet been independently hash-verified.
The author defers converters and inheritance/union wire contracts until basic JSON
is ready; see the [follow-up](native-json-plan.md#converter-and-wire-contract-follow-up--2026-10-09).
Evaluate the Introspection/Reflection API alongside implementation before stability,
preserving its overall structure while keeping details revisable; see the
[pre-stable review](introspection-design.md#pre-stable-api-evaluation--2026-10-09).

**Author JSON API review (2026-10-11):** improve the API and consider features such
as converters. The bounded development slice adds JsonArray Sequence conformance,
indexing and snapshot iteration ([contract and evidence](json-dom-design.md#read-only-array-sequence-view-development-2026-10-11)).
The [converter assessment](json-converter-design.md) recommends options and typed
converters next within JSON, while retaining explicit decisions about callback effects,
nulls and nested dispatch. Converter support remains unimplemented; this one-off
review does not replace the active platform milestone.

**Author follow-up (2026-10-10):** once basic JSON serialization is good enough,
review the collection API and identify useful additions. A queue interface and
implementation are the first named candidate, not yet a settled API. Evaluate
general-purpose collection consumers, existing collection contracts and .NET
ergonomics/behavior before selecting additions. The author clarifies that TaskQueue
is not necessarily connected; collection additions do not require scheduler changes.
See the [Queue/Stack review](experiments/queue-stack/README.md): application-local
Option-slot storage probes preceded the public library and Windows qualification
recorded below; native element-reclamation instrumentation remains open. The
bounded basic JSON correctness/platform gate now passes on macOS ARM64 and Windows
x64; collection API work can proceed. The earlier interpreter/native JSON benchmark
and later framework-comparison direction remains recorded in the native JSON plan;
this new request makes the collection review the next author-selected topic. The
author subsequently adds sets; see the [set review and prototype](experiments/sets/README.md).

**Basic collection development:** Queue/ArrayQueue, Stack/ArrayStack and
Set/MutableSet/HashSet now pass the [public native/interpreter consumer](experiments/native-collections/).
Queue/Stack/Set also pass the [Windows x64 action](experiments/native-collections/windows-validation.json).
Map now supports iterable, deconstructable KeyValuePair values through a bounded native
positional-record storage contract. The [Windows x64 Map run](experiments/native-collections/windows-map-validation.json)
and [JSON regression](experiments/native-collections/windows-json-map-validation.json) now pass,
with source and artifact hashes checked. The author next requested [init accessor
support](init-accessors.md): metadata and compiler import/emission are implemented;
macOS and Windows x64 native/interpreter and cross-library rejection qualification
pass ([verified Windows evidence](experiments/init-accessors/windows-validation.json)). Full
native record semantics remain open.
The author-selected iterable/copy construction and LINQ ToMap slice now implements
HashMap constructors and pair/selector materialization, with macOS native/interpreter
positive and duplicate/disposal checks passing. [Verified Windows evidence](experiments/native-collections/windows-materialization-validation.json)
now covers those map overloads. Iterable constructors now also cover
ArrayList, ArrayQueue, ArrayStack and HashSet, with explicit ordering/copy contracts; see [the contract](collection-contracts.md#iterable-construction-and-tomap--development-2026-10-10).
Native for-loop iterator disposal is repaired in Raven 494dede84, using the same
scope-exit cleanup as use. The runtime .rvn source audit now applies selective
for/use conversions while preserving explicit long-lived ownership, advance-only
iteration and disposal before terminal faults; see the [audit](raven-conventions.md#runtime-source-audit).
[Verified Windows for-cleanup evidence](experiments/native-collections/windows-for-cleanup-validation.json)
now covers compiler 494dede84 and the iterable constructors at 4d34f35b; the later
[source simplification run](experiments/native-collections/windows-scoped-library-validation.json)
also passes at 0361085d with artifact and source hashes verified. A bounded AOT follow-up admits immutable
StringGraphemes Char[] snapshots; String(sequence) remains blocked by constructor
call admission, so its cleanup refactor remains deferred.
Future concurrent variants retain implementation-specific atomicity/iteration contracts; see the [contracts and concurrency boundaries](collection-contracts.md#basic-library-slice-and-future-concurrency--2026-10-10).

### Follow-up review areas — author direction, 2026-10-10

The author expands the post-JSON review list below. These are work areas to
investigate, not settled API designs or a new delivery order. Collections remain
the next selected review after basic JSON qualification; scheduling and suspension
continue to co-evolve with native foundations.

| Area | Bounded next investigation | Comparison and decisions to record |
|---|---|---|
| Runtime scheduling | Review runnable work, I/O wakeups, cancellation and host ownership against the existing [scheduler design](runtime-scheduling-design.md). | Compare .NET Task/ThreadPool responsibilities with runtime-owned execution; preserve interpreter/native lifecycle consistency. |
| Queue, Stack and sets | Review FIFO/LIFO and set interfaces and concrete implementations against [current collection contracts](collection-contracts.md) and actual consumers. | Compare .NET Queue/Stack/HashSet ergonomics, equality, duplicates, removal, empty results, iteration, growth and reference release; select useful additions without treating the broader [collection proposal](proposals/collections-api.md) as implemented. |
| Database-backed web sample | Select a provider and a small parameterized query/update scenario with a Dapper-like typed object mapper. | Evaluate Dapper/ADO.NET responsibilities, UTF-8 conversion, null/type mapping, connection/transaction lifetime and native macOS/Windows dependencies. Provider choice, mapping strategy and public contracts remain open. |
| Time Parse methods | Civil Date/TimeOfDay/LocalDateTime numeric parsers now pass eight local native/interpreted tests. Duration/TimeOffset add bounded invariant forms; Instant requires an explicit offset or Z. ZonedDateTime parsing validates the timestamp against an explicit named zone. | Current/explicit culture and typed Result errors are implemented; general patterns, non-Gregorian parsing and inline zone identifiers remain gaps. |
| Runtime suspension | Review safe suspension/resumption boundaries, retained roots and host-resource lifetime alongside scheduling. | Compare CLR suspension/GC coordination and cooperative execution approaches; distinguish runtime suspension from awaiting I/O and leave the stack/preemption model open. |

**Author clarification (2026-10-10):** explicitly include Parse methods for Date,
TimeOfDay and LocalDateTime, then audit Instant, Duration and TimeOffset rather than
leaving time parsing implicit. The initial audit found no Parse/TryParse methods.
The [civil parsing slice](date-time.md#development-parsing-2026-10-10) now implements
Parse(value) and Parse(value, culture), with a shared standard error union. Eight
framework cases pass both local modes, and native/interpreted host-preference output
matches. General formatting patterns, non-Gregorian parsing and inline zone identifiers
remain open. Duration and TimeOffset add invariant parsing with full tick-boundary
checks and the existing offset domain. Instant now composes invariant civil text
with an explicit offset or Z; it never guesses the host zone. Eight Duration/offset
cases and six Instant cases pass both local modes
([duration/offset evidence](../runtime/raven/tests/duration-offset-parsing-validation.json),
[instant evidence](../runtime/raven/tests/instant-parsing-validation.json)). No TryParse alias is added.
ZonedDateTime.Parse(value, zone) now validates the stated offset against pinned zone
rules; correct offsets select overlap candidates, while gaps/mismatches return
OffsetMismatch. Native lookup/offset/version queries share the interpreter's rule
kernel through optional native archive variants. Native MapLocal and machine-zone
discovery remain separate gaps. Windows validation includes the parsing suites and
host-culture parity probe.

**Author-directed encoding review (2026-10-10):** the
[review](design/encoding-api-review.md) is recorded after the time-zone slice. It
retains the established UTF-8/strict-ASCII baseline and compares role, completion,
error, quota and buffer contracts with System.Text. Eight provider/session checks have migrated into attributed tests; all pass
both local modes ([evidence](../runtime/raven/tests/encoding-validation.json)).
Eight further [stream adapter cases](../runtime/raven/tests/encoding-streams-validation.json)
cover writer completion/failure and selected readers in both local modes. Eight [boundary cases](../runtime/raven/tests/encoding-boundaries-validation.json)
also pass both local modes: Unicode reader boundaries, custom decoded output,
zero-progress transfers, failures and maximum input/write quotas. Direct decoder/drain
buffer-limit fixtures now add five passing native/interpreted cases
([evidence](../runtime/raven/tests/encoding-buffer-limits-validation.json)), with explicit
host budgets and a byte-write accounting fix exposed by the large inputs. Windows
qualification remains. Paired one-shot decoding and general writer completion follow
as proposals. This review does not
approve additional codecs or permanently reprioritize the roadmap. The author then
selected internal visibility for Utf8 and proposed InternalsVisibleToAttribute for
controlled implementation sharing. The [follow-up design](design/encoding-api-review.md#author-follow-up-internal-helpers-and-friend-assemblies-2026-10-10)
records the cross-assembly callers, .NET comparison and required compiler/runtime/AOT
access checks; neither runtime API change is complete yet. The general Raven
source/CLI friend-access fix is now on main at `52dc10011`, with 56 focused tests
passing independently of the native integration branch. Native compiler forwarding
and assembly-symbol exposure now pass the focused native attribute probe, including
friend/non-friend binding. Direct metadata and Raven-emitted consumers now execute
friend-only internal type/constructor/field/method access under the unsigned simple-ASCII
profile, returning 42. Compiler imports reuse cached semantic grants; resolved runtime
checks use a linked-snapshot index. The same Raven consumer now compiles, links and
returns 42 as native macOS ARM64 code; denied grants/private access produce no object
([regression fixtures](../tools/aot-poc/tests/fixtures/friend-assemblies/README.md)).
The source-runtime attribute now has its constructor, AssemblyName and repeatable
assembly usage policy; [real source-grant consumers](experiments/friend-runtime/README.md)
pass interpreted and native macOS ARM64 execution after all four libraries rebuild.
Full identity semantics and Windows qualification remain open.
The author subsequently clarified that grants should be sparse and mainly serve
internal tooling and testing assemblies. Runtime libraries should consume supported
public APIs; do not grant System.Data/System.Web internal access to complete the Utf8
migration. Preserve existing conversion contracts while migrating those consumers.
Public conversion migration now has paired one-shot Encoding.Decode with strict
UTF-8/ASCII results and a 65536-byte bound; custom policies must implement Decode.
System.Data and System.Web now use public codecs without grants; Utf8 is internal.
HTTP composes bounded conversions to retain larger buffered-body behavior.
Author-directed next work (2026-10-10): immediately after this encoding migration,
address HttpClient's configurable request timeout and Disposable/client-handler
ownership gaps, including use support and macOS/Windows cancellation/cleanup parity.
The [lifetime and timeout design note](design/http-client-lifetime.md) records the
.NET baseline, current timer/deadline constraints and required qualification.
The client and handler now implement Disposable with explicit ownership and cooperative
pending-request cancellation. The abstract handler and DelegatingHandler pipeline pass
ten functional cases and eight guard cases interpreted and on macOS ARM64 AOT;
[dispatch evidence](design/native-class-dispatch.md) records the bounded support.
Windows CI is configured but unexecuted. The private monotonic timer source and native
binding now support public Timer, serialized async callbacks and replaceable CancelAfter.
HttpClient.Timeout composes those layers with cancellation acknowledgement, a 100-second
default and explicit None disabling. The independent 15-second client/host budgets are
removed; five-second transport phase bounds remain gaps. See the
[focused timeout evidence](experiments/http-timeout/validation.json).
Author clarification (2026-10-11): build sensible Raven Timer and timed-cancellation
APIs rather than copying .NET mechanically. Those layers are implemented locally.
The subsequent string API direction keeps encoding explicit: public string operations
should describe text, not byte storage. GetUtf8ByteCount/SliceUtf8 remain migration
debt; replace their uses with appropriate encoding contracts before removing them,
without friend grants. ObsoleteAttribute now marks those two methods with warnings.
This does not displace the next HttpClient work.
Author-directed follow-up after HttpClient (2026-10-10): add compatible Index and
Range runtime types and connect Raven's existing indexing/slicing syntax to arrays,
strings and sequences. Compare .NET from-end indexing, half-open ranges, bounds,
copy/view behavior and sequence capabilities; preserve the established text-index
unit for strings rather than exposing UTF-8 storage through syntax. The first array
slice is implemented locally: Index/Range values, from-end read/write, half-open shallow
copies, Option-returning Get and Result-returning Slice. Nine framework cases and six
fault guards pass in interpreter and macOS AOT. The author clarified that consuming
APIs may fault; Index/Range describe positions. String Index/Range access and safe Get/Slice now pass locally with grapheme boundaries;
sequence Index/Range extensions, List contracts and ArrayList direct copying now pass
20 framework cases and 10 guards per mode in local interpreter/macOS AOT;
arbitrary Iterable indexing and borrowed views remain outside this slice. The author additionally
requested a memory-view design investigation after this slice (2026-10-11): compare
.NET Span/ReadOnlySpan/Memory/ReadOnlyMemory with alternatives enabled by neoCLR's
allocation/reference model, including lifetime, GC, async and CLI tradeoffs. This
is a research follow-up, not a commitment to four matching types. See the
[contract and evidence](design/index-range.md). Windows execution is pending.
Author-directed design follow-up (2026-10-15): review [generic bounds and consistent
Raven endpoints](design/generic-bounds.md), then integer iteration across .NET/neoCLR.
The syntax migration is integrated on Raven main and carried to `codex/neoclr`;
[native qualification](validation/range-endpoints/README.md) records working loops
and stored half-open bounds, with inclusive Range construction and range patterns
still requiring native lowering.
All known consumers are owned; compatibility is not a constraint. Structural work
is parked. Generic capability APIs remain design work; this direction does not
permanently change the active milestone.
The host metadata model and native manifests now preserve assembly annotations through
CLI/native snapshots and CLI reference projection. Loader checks validate constructor
shape, member-target rejection, limits and declared dependencies without execution.
Friend lookup performance is an author requirement: retain existing fast paths and cache repeated pair decisions.

A separate native gap observed during implementation: String.GetScalars exposes an
immutable UInt32 array that current native specialization rejects. This parser uses
UTF-8 byte input directly because its grammar is ASCII numeric. Broader immutable
scalar-array support still needs its own conformance and allocation tests.

No database provider, converter extension or green-thread implementation is claimed
by this list; time parsing is limited to the civil slice described above. The JSON benchmark/publication follow-up
remains recorded separately.

**Author clarification (2026-10-09):** once the native compilation foundations are
ready, let runtime suspension, runtime-owned scheduling and green threads co-evolve
with them. This is a cross-cutting design/implementation track alongside subsequent
native, HTTP and hosting slices, not work deferred until all AOT coverage or ports
are complete. Preserve shared lifecycle contracts across interpreter, AOT and
eventual JIT; current console/HTTP hosts, TaskQueue adapters, pthread stack guards
and TLS roots remain provisional. See the
[scheduling follow-up](runtime-scheduling-design.md#co-evolution-with-native-foundations--2026-10-09).
No stack model, preemption policy or public Scheduler API is selected yet.

Comparison: [.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
already offers project-level self-contained native deployment. The immediate benefit
here is making neoCLR's existing UTF-8/Raven execution accessible and reproducible;
costs include platform-specific linking/services, packaging and a bounded supported
subset. No performance superiority is established. Reuse the
[native execution investigation](native-execution-investigation.md) and
[native web evidence](../benchmarks/native-web/README.md). This author direction
supersedes older immediate-priority notes below; their completed evidence and open
technical gaps remain valid. No release date or version is selected.

**First implementation slice:** the source-checkout
[project-to-executable console workflow](native-poc-bundle.md#development-project-to-executable-workflow-2026-10-09)
now builds ordinary projects with a selected bundle and links a bounded macOS
ARM64 executable. [Focused evidence](experiments/native-project-build-validation.json)
covers UTF-8/interpolation, interpreter/native fault parity, executable-only
deployment, unsupported recursion rejection, existing-output preservation and
rejection of stale project output after a failed rebuild. The subsequent
[development kit](native-poc-bundle.md#relocatable-development-kit) packages the
matched backend/adapters and compiler/library bundle. Its
[extracted acceptance](experiments/native-build-kit-validation.json) passes outside
the checkout without Cargo, including standalone UTF-8 execution and pre-build
rejection of modified adapter/backend/compiler inputs. The opt-in
[HTTP project profile](native-poc-bundle.md#opt-in-http-project-profile-development)
now packages the existing server and bounded host. Its
[project/kit evidence](experiments/native-http-project-validation.json) covers
greeting, fragmented requests, duplicate-length rejection, handler errors and guest
callback faults with interpreter/native parity and standalone execution. The
console profile remains separate. Continue reviewing suspension/scheduler lifecycle
boundaries alongside native work. The first Windows slice now emits scalar/literal-
console x64 COFF through explicit target selection; cross-target checks and macOS
regressions pass. [Windows evidence](experiments/windows-aot-scalar-validation.json)
records the initial object-only boundary. The subsequent
[dedicated Windows run](windows-aot-execution-validation.json) passes all four
tests with no skips, including 32 MSVC-linked native/interpreter comparisons.
The [Raven Hello World Windows run](windows-raven-hello-validation.json) now passes
standalone PE/#Neo and NEOX execution with byte-exact interpreter output, alongside
the 32 scalar comparisons (five tests, no skips). The subsequent
[fresh-source Windows run](windows-fresh-raven-validation.json) builds pinned Raven
`71cafd35` with the current metadata writer and compiles Hello World on Windows;
both containers execute with interpreter parity (six tests, no skips). The
[expanded source gate](windows-raven-flow-validation.json) now passes calls,
branches, empty/repeated loops and exact UTF-8/NUL output in both containers
(seven tests, no skips). It adds the Boolean comparison-result lowering Raven
needs, with typed stack checks and Int32 signatures/locals. Next establish Windows
managed-console host allocation and stack-protection requirements before widening
managed services or project kits; account for suspension lifecycle boundaries in
that work. The [Windows host prerequisites](windows-native-host.md) now specify
these boundaries and include an independent guarded-heap adapter and focused
Windows workflow. [Heap evidence](windows-host-memory-validation.json) passes 12
allocation/release lifecycles and both guard boundaries. The
[Windows collector consumer](windows-collector-validation.json) now retains rooted
graphs, hands roots to published frames, reclaims cycles, rejects premature/foreign
teardown and passes exhaustion/cleanup plus existing collector contracts. The [Windows host stack gate](windows-stack-validation.json)
now rejects 128 KiB admission before allocation, rejects fibers and safely returns
from deep calls on 512 KiB/1 MiB stacks with collection and GC-frame cleanup.
The [generated integer-stack gate](windows-generated-stack-validation.json) now
passes Windows execution with inline page probing, final-frame bounds, small-stack
rejection, recursive diagnostic return, complete frame cleanup and context reuse.
The [generated array-heap gate](windows-generated-heap-validation.json) now passes
live local/argument retention, allocation churn, bounds/exhaustion faults, complete
reclamation and reuse. The author then explicitly directs continuing until Windows
native compilation works. A development Windows console project profile now builds
ordinary Raven projects through the guarded host. Its
[Windows project gate](windows-native-project-validation.json) passes five fresh
builds and standalone EXEs with exact interpreter parity for managed arrays/text,
interpolation, binary I/O and guest fault diagnostics. It also preserves existing
outputs and rejects stale output after a failed rebuild. This completes a usable
Windows synchronous console build path. Windows packaged project kits and
non-console services remain unsupported. Full
bootstrap, general hosting and reload remain open.

**Preview 13 published (2026-10-09):** after Raven's shared website publication,
the author requested the next neoCLR POC release for Windows and macOS, with a
final pass over urgent issues. Qualify Windows x64 and macOS ARM64 packages from
recorded revisions, including extracted consumers and editor integration. Keep
native-only core bootstrap limits and the ARM64 AOT scope explicit; this does not
make universal sample support or Windows x64 AOT a prerequisite. Preview 13 now
passes source, extracted-package and installed-editor qualification on the selected
platforms. [Preview 13](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.13) is published
with both bundles, the qualified source archive, evidence and checksums. See [release evidence](preview-13-validation.json) and
[bundle qualification](native-poc-bundle.md).

**Native String POC (2026-10-09):** unchanged production String and its native-only
source dependencies now pass interpreted and ARM64 execution, with static Unicode
services, GC-rooted intern ownership, array interface views and matching faults.
The release target is a bounded proof of concept, not resolution of every platform
limitation. Full core bootstrap, broad Object display and intern optimization remain
separate follow-up work. [Scope and evidence](experiments/native-core-bootstrap/README.md#production-string-poc-2026-10-09).

**Native core value prerequisite (2026-10-09):** selected native System.ValueType
classification now admits ordinary value declarations, with copied/mutated struct
parity in project run, interpreter and ARM64 native execution. Unchanged production
Option/Result initially reached missing native String.Concat support. Real UTF-8
fixture wrappers now pass both modes after an AOT module-ownership projection fix;
the four-source union/attribute build reaches imported class-base admission. Including
the production Attribute source now compiles the complete five-source subset and
passes union pattern/protocol execution in all three paths. [Evidence and next work](experiments/native-core-bootstrap/README.md#value-type-foundation-and-production-union-frontier-2026-10-09).
Generic String boxing/Object display now also preserves text and reference identity
across modes after correcting native reference-view lowering and excluding unboxed
values from Object dispatch candidates. [Evidence](experiments/native-core-bootstrap/README.md#generic-string-boxing-and-object-display-2026-10-09).
String-payload union display now passes Some/None/Ok/Error across modes with bounded
nonmatching Char-box tests. Int32 payloads also pass zero/negative/min/max and both
Result cases with the explicit native integer formatter; missing binding publishes no object.
[Integer display evidence](experiments/native-core-bootstrap/integer-display-validation.json). Matching Char boxes remain rejected; other primitive
payload display and full core bootstrap remain open.
[Display evidence](experiments/native-core-bootstrap/README.md#nonmatching-char-tests-and-string-union-display-2026-10-09).
Immediate constructed value receivers now lower through single-evaluation temporary
storage. The [escaping audit](experiments/native-core-bootstrap/README.md#constructed-receivers-and-escaping-audit-2026-10-09)
previously confirmed missing quote/backslash escaping in both modes when String.Replace
was absent. The private interpreter/native replacement service now passes focused
UTF-8 and fault checks. The source-owned String.Replace member, bridge/reference
and native fixture now expose it; production Option/Result escaping passes project,
interpreter and ARM64 checks. [Service contract](experiments/native-core-bootstrap/README.md#private-ordinal-replacement-service-2026-10-09).
The six production collection/equality interfaces needed by String now compile
unchanged against native-only metadata and pass imported generic dispatch in project,
interpreter and ARM64 execution. [Dependency contract evidence](experiments/native-core-bootstrap/README.md#production-string-dependency-contracts-2026-10-09).
The native fixture now exposes seven real scalar text-service facades; direct calls
pass both modes and source String resolves RuntimeServices. Those remaining text/vector
services, array/primitive dependencies and source String ownership are now qualified
by the production String POC above. Full core bootstrap remains open.
[Union source evidence](experiments/native-core-bootstrap/README.md#production-union-source-subset-2026-10-09).
[Text-service evidence](experiments/native-core-bootstrap/README.md#native-text-services-and-module-projection-2026-10-09).


**Release bootstrap prerequisite (2026-10-09):** Raven project mode now accepts
native-only core metadata through RavenNeoClrNativeCoreReference. A module consumer
passes project build/run, interpreter and ARM64 native execution with no CLI semantic
references; mixed configuration preserves existing output. See the
[project qualification](experiments/native-core-bootstrap/README.md#native-only-project-qualification-2026-10-09).
This closes project selection, not the release's full source-runtime bootstrap gate.
Next qualify native primitive/core completeness against the production library;
installed editor and extracted-package acceptance remain separate gates.

**Author-directed module foundation (2026-10-09):** implement logical modules as
declaration containers, retaining assembly deployment identity. The
[module contract](declaration-modules.md) records the format/API foundation. Raven
module syntax/imports and neoCLR editor/documentation terminology are implemented;
[consumer evidence](experiments/declaration-modules/validation.json) covers interpreted
and ARM64 native execution. Guest-runtime and access-policy boundaries remain open. This explicitly
advances the earlier exploration without adopting the entire proposal.

**Author-directed API organization (2026-10-09):** group date/time values, clocks,
calendars and timezone APIs under `System.Time`; rename the old `Time` value to
`TimeOfDay`. This is an implemented development identity migration, with unchanged
representation/arithmetic and a rebuild requirement, not a module-system milestone
or a claim of complete time-service AOT support. See the
[naming decision](date-time-design.md#systemtime-namespace-2026-10-09-development).

**IL semantics (author clarification, 2026-10-08):** use .NET/CLI instruction behavior
as the default across interpreter, AOT and eventual JIT. Document deliberate platform
divergences explicitly; cross-mode agreement alone does not establish correctness.
The [isinst correction](array-variance.md#type-tests-retain-cli-semantics-2026-10-08)
restores failed array tests to null while preserving deliberate mutable-array invariance.
This is a baseline requirement, not a claim that every instruction has been audited.

## Interface naming and async callback follow-up (2026-10-08)

Author direction: neoCLR interfaces must not use the .NET `I` prefix; Raven's neoCLR
target mappings must follow the runtime names. `AsyncStateMachine` and `TaskAwaiter`
now replace the source protocol names. The native class-interface callback slice
passes three real async consumers and focused GC/fault checks. The subsequent
`DrainEntryTasks` slice adds queue-only entry pumping; host-I/O async entry waiting
remains a runtime service gap. See the [sample assessment](experiments/aot-sample-assessment/README.md).
The portable Raven emitter now also preserves statement context for discarded awaits;
[native success/fault checks](../benchmarks/native-web/discard-await-validation.json)
cover the corrected source form. This is a compiler correction, not new scheduling
or metadata capability. The portable planner is not yet on Raven main.
A subsequent [reference-array slice](experiments/aot-console/record-arrays.md#default-null-reference-arrays-2026-10-08)
admits ordinary null-initialized class arrays. [Exact reference-array views](experiments/aot-console/record-arrays.md#exact-nominal-reference-array-views-2026-10-08)
now run the order sample through array-to-Iterable dispatch with matching interpreted/native output. The [byte-view prerequisite correction](experiments/aot-console/README.md#console-text-writers-and-nominal-byte-array-views-2026-10-08)
now preserves the exact backing specialization independently of discovery order, with
30 native/interpreter comparisons and closed-backing construction rejection. This
does not imply general array views or covariance. Following the [next-fix reassessment](experiments/aot-sample-assessment/README.md#reassessment-after-reference-array-views-2026-10-08),
[lexical path bindings](path.md#native-compilation-poc-2026-10-08) now pass the unchanged
Raven sample in all three modes. [Math target lookup](math.md#source-runtime-bootstrap-ownership-2026-10-08)
is corrected by removing competing bootstrap declarations. The [source-facade caller fix](introspection-design.md#source-built-executingassembly-correction--2026-10-08)
now closes both unexpected interpreter introspection failures; bootstrap remains a release gate. Author-requested [Math constants](math.md#assembly-level-constants-development-2026-10-08)
now use finite Double assembly-level metadata and compile-time inlining. Their interpreter
consumer and native/standalone runs now pass after [bounded Double lowering](experiments/aot-double/README.md).
Remaining floating instructions and Math services are explicit gaps. Bootstrap qualification
remains a release priority. [Native file output](file-output.md#native-compilation-development-2026-10-08)
and [input](file-input.md#native-compilation-development-2026-10-08) now pass the unchanged
combined file sample in interpreted, sanitized native and standalone modes.
Jagged arrays and RuntimeTypeHandle introspection remain excluded.
The wider audit also found the handwritten legacy union bridge's `IUnion`; it now
uses `UnionValue`, with regenerated fragments and executable union/time checks. This does not change the bootstrap release gate or promise
runtime suspension, JIT, reflection or stable callback ABI support.

## Bounded AOT investigation and first implementation (2026-10-07)

**Future execution investigation (author-directed 2026-10-07).** Investigate JIT
and especially native AOT while retaining the interpreter as a supported option.
Hot reload must be designed independently of execution mode. ARM64 is the primary
architecture direction for future native backends; this does not withdraw existing
host support. The [investigation](native-execution-investigation.md) records source
comparisons, runtime gaps and proposed AOT/reload experiments. Backend choice and
implementation scheduling remain open. The author identifies an AOT-compiled web app
as the motivating POC, with explicit direction to start simple: proposed scalar and
Hello World steps precede a minimal HTTP handler; hot reload follows separately.
The author also requests eventual benchmarking against .NET and other
languages/platforms; compare equivalent workloads once native consumers work,
without making a broad benchmark suite a prerequisite for the first simple proof.
**Bounded implementation follow-up:** the author then directs “Continue work on AOT”.
The [scalar ARM64 experiment](experiments/aot-scalar/README.md) implements native
wrapping integer arithmetic/direct calls through Cranelift and a C consumer, with
focused validation. This selects a bounded AOT slice on main; the native bootstrap/release
qualification priorities recorded below remain in place. The next committed slice
adds branches, locals, loops and stack joins with seven focused tests and 79 native/
interpreter comparisons on main. A following arithmetic-Fault slice adds checked
arithmetic, division/remainder and status propagation through compiled calls, with
a versioned experimental C ABI and nine tests/174 native comparisons. Execution
budgets, general managed services, the web demo, JIT and hot reload remain future work.
The next slice completes [native Hello World](experiments/aot-hello/README.md):
UTF-8 literal output with startup and console code linked into an executable that
runs independently of a shared managed framework/runtime.

**HTTP-driven dependency discovery (author clarification, 2026-10-08):** work toward
running the existing HTTP API to discover the native backend and runtime capabilities
it needs, including garbage collector integration if required. Bounded arenas and
small consumers are intermediate validation mechanisms, not constraints that the HTTP
API must fit or a final server memory policy. Collector choice, native root reporting
and reclamation remain open; pursue these when the real dependency chain requires them.

**Runtime architecture review (author direction, 2026-10-08):** reassess Function
objects, scheduling, GC roots and async integration against future runtime suspension
and green threads. The [review](runtime-scheduling-design.md#cross-runtime-reassessment--author-direction-2026-10-08)
keeps current native adapters provisional. No stack model or delivery milestone is
selected; continue the HttpServer POC while validating these boundaries. The author
reiterates runtime-owned scheduling of green threads as the future direction to
investigate; TaskQueue may be replaced by Scheduler, and current queue adapters do
not settle that public API or migration. The author further specifies cross-cutting
runtime services benefiting interpreter, AOT and eventual JIT, with shared lifecycle
contracts and backend-specific adapters.

**Future HTTP library candidate (author suggestion, 2026-10-08):** consider an
[HttpMethod abstraction](http-client-design.md#future-httpmethod-type--proposal-2026-10-08)
with standard and extension tokens. This began as a proposal; the author explicitly
requested implementation on 2026-10-11. The implemented contract and evidence are
[recorded separately](http-client-design.md#httpmethod-implementation--development-2026-10-11).

**Next-release teaser (author clarification, 2026-10-08):** native compilation is a
central source of platform value. Prioritize a reproducible standalone ARM64 HTTP demo,
repeated-request/cleanup evidence and an honest interpreter/native comparison for the
Native compilation page. Label the teaser WIP; a general Scheduler, JIT, green threads
and production qualification are not prerequisites. No release date/version is selected.

**Benchmarkable web app (author direction, 2026-10-08):** use the same checked-in
Raven workload for interpreted/native comparisons first, then equivalent apps on other
platforms. The [native-web workbench](../benchmarks/native-web/README.md) starts with
routing component measurements and admission of the existing HTTP server. Progress to
one persistent server process, correctness/cleanup tests and matched HTTP load before
publishing server comparisons. [Native compilation](../website/content/features/native-compilation/index.md)
has its own feature page and homepage entry. Any next-release POC remains explicitly
**work in progress**, with limitations and timing scope next to results. A socket-handle prerequisite now adds exact Int64/UInt64 erased transport, generic
specialization and equality. The isolated listener now runs in both modes with exact native service bindings and
explicit host-scope cleanup, including guest fault exits. [Stored native callbacks](experiments/aot-console/callbacks.md)
now retain receivers through collection and preserve fault traces in real Raven consumers.
Function arrays and the real ArrayList callback consumer now support growth, copies,
replacement and collection. [Reserved value arrays](experiments/aot-console/record-arrays.md)
now execute the real ArrayList<Result<int,string>> growth/copy/replacement path with nested
reference tracing. The full server originally reached the 128-type specialization budget; the narrower
HTTP result shape also exposes enum storage. [Native Int32 enum admission](experiments/aot-console/enum-values.md) now passes status/flags collection consumers; the nested result probe now runs in both modes after an explicit-core Void import fix and coordinated 64-lane native layout/GC admission. Coordinated 256-type/1,024-function/256-clone bounds now select the full server (208/570/213). Reference-array collections now pass growth/copy/replacement and GC tests in both modes. ObjectReferenceEquals now binds with null/alias/text-identity parity. StreamError.ToString now runs through a verified direct value-override projection. Int32 borrowed wrappers now support nested HTTP diagnostics. Private thread-affine strong host roots now retain native allocations with explicit release and entry-reset guards; focused sanitized kernel/compiled-entry checks pass. Private host dispatch now invokes retained fn<Void> callbacks between guest calls without resetting the heap, with GC/fault/rejection tests. The native accept kernel now passes loopback deferred-delivery, cancellation and cleanup tests. Exact CIL accept/result/cancel bindings now pass compiled completion/cancellation; full Raven Server admission reaches SocketReceive. Buffered native transfer kernels now pass snapshot/copy-back, cancellation, EOF and cleanup tests. Exact transfer bindings now pass a compiled callback-driven echo; full Server admission reaches SocketDeadlineAfter. Shared deadline services now pass kernel/compiled-echo checks; admission reaches the linked StringCompareOrdinal service. Ordinal comparison now passes 88 native/interpreter cases; admission reaches specialized RegisterTaskQueue. The [TaskQueue adapter](experiments/aot-console/task-queue.md) now preserves default/explicit-queue behavior in a real Raven consumer. Full Server admission reaches the recursive-call guard; diagnostics now identify the close/cancellation/callback cycle. The [opt-in stack guard](experiments/aot-console/native-stack.md) now passes direct/Function recursion, callback-root/fault/unwind and worker-boundary tests with final machine-frame bounds. No-result/Void entry adaptation now passes success/fault checks, and full Server passes compiler admission with the guard. Private host queue draining now passes real Raven success/fault consumers in both modes. The same Raven Server artifact now passes greeting, fragmented request, duplicate-length rejection and handler rejection in interpreter, sanitized native and standalone native modes. [HTTP evidence](../benchmarks/native-web/http-validation.json) includes cleanup and libSystem-only linkage. A 32-request persistent sample now passes all three modes. Three short alternating pairs measure 3.08 interpreter versus 20.09 native requests/s on Apple M1, with warmup and startup separated; [raw evidence/caveats](../benchmarks/native-web/persistent-http-validation.json) record different host/GC policies and unoptimized native CIL. A profile-driven native GC address index now measures 20.08 → 96.05 requests/s on the identical native object, with kernel/HTTP cleanup and sanitizer checks; [adapter comparison](../benchmarks/native-web/gc-index-validation.json). Interpreter lookup already uses managed identities, so no corresponding speedup is claimed. Next assess sample coverage and extend matched load/qualification; no external-platform ranking is claimed. Component benchmarks do not
qualify a server.

**Author clarification:** build the CIL-to-native, self-contained executable foundation
early, and extend it alongside platform evolution. **Hello World is the first executable
milestone**, followed by increasingly complex samples until HTTP Server. The author clarifies that the input is **neoCLR CIL**, produced by Raven
in its metadata format. The next slice now passes **Raven source → PE/#Neo metadata/IL
→ native ARM64 Hello World**, with standalone NEOX also tested; the original neoIL
consumer remains a backend test. The input is not ordinary .NET CIL. Required library/runtime support should be baked into
the final image. Scalar and Fault tests support this progression; they do not replace
the executable milestones. The author identifies trimming as a later step: establish
code/metadata roots and dynamic-use rules before removing dependencies. No trimming
is implemented or required for this first Hello World milestone. The author selects
types and members next, especially value types supporting union samples such as
Result and Some, to exercise payload access and control flow/branches. Inspect Raven
metadata/IL before selecting the first bounded representation and member slice.
The [first native value/member slice](experiments/aot-values/README.md) now compiles
Raven Counter and copied Int32/Boolean records, including constructors, fields,
accessors, borrowed receivers and branches. A follow-up slice adds nested
reference-free records with copied payloads and interior field borrows, validated by
the Raven Envelope sample. Ordinary output parameters now support a Raven member/
forwarding probe, with a native definite-assignment check and alias/Fault tests.
Byte tag storage and unchecked Int32/Byte conversions now pass Raven and numeric
boundary probes. Exact overloaded call/member resolution now passes a Raven consumer,
constructor and nominal output-overload tests. The [ordinary Raven union app](experiments/aot-union/README.md)
now constructs/matches Some and None natively under explicit closed-world selection,
with standalone deployment evidence. Generated formatting/boxing, general library
compilation and conditional output contracts remain unsupported. After this milestone,
the assistant recommends a Result-driven interactive integer reader: value-only Result
first, then native UTF-8 input/lifetimes and parsing. The value-only first step now runs
as local `ParseResult<int, byte>` with one closed shape per generic definition, reported
specializations and standalone evidence. `let ... else` and `if let` execute both paths;
plain positional `let` deconstruction is a recorded Raven native-emitter gap.
The [real library Result probe](experiments/aot-values/README.md#real-library-result-dependency-boundary-2026-10-07)
passes Raven/interpreter execution; its initial report recorded AOT's external-call rejection.
Closed-world inspection shares emission preparation and exposes this boundary.
An [explicit nongeneric value-library load set](experiments/aot-library/README.md) now
compiles separate Raven library/application artifacts into one standalone executable,
verifying original access scopes before projection. A follow-up now specializes one
closed generic value shape across that boundary, validated by Pair<int, byte>. The
runtime-owned System/Object validation context is now explicit and tested. The real
Result probe now runs as a standalone ARM64 executable: original interface conformance
is verified before metadata-only relationships are omitted from the direct-call projection.
Interface dispatch remains unsupported. The [nested input-outcome sample](experiments/aot-input/README.md)
now executes the actual Result<Option<byte>, ConsoleReadError> shape natively, with
metadata-only interface arguments kept outside executable specialization. Actual ReadByte
passes interpreter checks. Empty static member owners now compile, including a Raven
factory returning the nested result. Bounded primitive erased-value transport now compiles
pack/test/unpack, copied calls and output slots; mismatched unpack propagates RuntimeError.
Primitive static generic methods now specialize into bounded private bodies, with original
identities/origins reported and a Raven forwarding consumer validated. ReadByte's generic
helpers now compile from an explicitly supplied System seed with `--compile-system`;
the default remains validation only. The unchanged ReadByte probe now reaches its String-based
failure path. Explicit `fault` now lowers to terminal UserFault status in scalar/value
code, with first-fault propagation and no result publication. Immutable UTF-8 literals
now pass through value-profile locals, calls and output slots, with a Raven producer
validated in PE/NEOX. ReadByte reaches native service admission; an explicit
failure binding now captures shared code/message/trace diagnostics with explicit ABI v3
and a standalone Raven consumer. The author directs consistent interpreter/native faults
and accepts exit 1 for standalone unhandled faults; embedding hosts retain control.
[Shared host diagnostics](experiments/aot-fault-details/README.md) use a code-defined runtime
message catalog and a common 64-frame truncation contract. Interpreter CLI execution
faults now use the same presentation and exit convention; [native byte input](experiments/aot-console/README.md) now executes the ordinary Raven
Console.ReadByte wrapper, with byte/EOF/error and invalid-service fault evidence.
The author directs sustained Console support next (2026-10-08): value-profile
WriteLine now compiles its string/Boolean/empty-line wrappers alongside input,
with an interactive Raven consumer and broken-pipe fault parity. Invocation-owned
dynamic UTF-8 text now supports Int32/byte line output with bounded caller storage
and ABI v4 lifetime/exhaustion tests. Copied String-bearing records and generic
payloads now use aligned mixed-width lanes, with String defaults/pattern tests and
null-native-argument fault parity. An explicit bounded reference-arena profile now
compiles nongeneric classes and preserves aliasing/cycles until invocation reset,
with Raven class/Console evidence and interpreter null-fault parity. Exact nonvirtual
class callvirt now preserves its separate call-site null check. Packed byte arrays
now support zero initialization, aliasing, indexing, length and interior byte borrows,
with interpreter fault parity and a standalone Raven Console consumer. Character
WriteLine now preserves UTF-8 graphemes through statically linked Unicode validation,
with NUL defaults, invalid-input parity and a fresh Raven consumer. Signed-byte and
16-bit integer storage/conversions now feed ordinary numeric Console output, with
boundary and fresh Raven evidence. Explicit 64-bit formatting/native-width bindings
now cover the remaining integer Console output paths, including signed/unsigned
endpoints, with full-width storage and conversion parity. Invocation-region storage
does not provide general garbage collection. Multiple closed value shapes now
compile with separate layouts/methods and retained source identities: a fresh Raven
app combines Option<int/string/byte> and Result<int/string/input-outcome> with
Console.ReadByte. Bounds and original-scope validation remain enforced. Nongeneric
interface views/casts and the verified empty Object base now let all three standard
stream factories compile, with standalone Raven evidence. Constructed-class implicit
interface dispatch now compiles standard-input Read/Close and StreamError outcomes;
its measured graph extends private bounds to 64 types/sixteen value lanes. Explicit raw output/flush bindings and inhabited unit storage now compile ordinary
stdout/stderr streams with recoverable I/O outcomes and standalone parity. Text
reader/writer dependencies behind Write/ReadLine remain open. Closed generic classes
and inherited interface dispatch now pass nominal-shape and fault tests; the real
ArrayList<byte> dependency now compiles with checked byte reservations, growth and
interface aliasing. Unwritten reads and output validation preserve interpreter faults;
reserved-array element borrows remain rejected. String-valued erased transport and
static generic String helpers now preserve pointer width/lifetime and exact fault
parity. Explicit UTF-8 byte-count/slice bindings now preserve interpreter range/boundary
results and exact dynamic fault messages. UTF-8 encoding now returns immutable byte
value snapshots, preserving fixed-extent replacement and the bridge's ordinary copy
loop. Ordinary String instance bodies now run in a standalone Raven slice/count
consumer with Result patterns and exact interpreter fault parity. Measured selection
budgets expand to 256 functions/128 types/64 clones (Console.Write already selects
151/68/37 before array dispatch). Empty-record boxing now supports generated
EncoderState.Value/HasValue with distinct object identity, bounded lifetime and exact
fault sites; a fresh Raven Console consumer passes standalone validation. Verified
nominal byte-array interface dispatch now runs ordinary backing methods, completing
the Console.Write(string/int) and standard-error text-writer sample with exact
interpreter/native output and broken-pipe user-fault parity. Console.ReadLine now compiles its ordinary reader/decoder graph with strict UTF-8
decoding. Its measured 268-function/110-type/73-clone graph extends bounded selection
to 512 functions/128 clones and nested values to 32 lanes, with matching call and
object storage. Standalone line-input validation covers ten input cases and exact output-fault
parity. A second standalone session covers default/consecutive reads, typed range/closed
errors and non-owning wrapper close behavior. Object WriteLine now has bounded dispatch to verified class ToString overrides;
default/base display still depends on type metadata. Standalone object output
and exact output-fault parity pass. Constructor admission now rejects missing or
repeated base initialization rather than admitting a native/interpreter mismatch.
The next Console reader slice adds exact UTF-8 concatenation and ordinary static
String wrappers for ReadToEnd; nine standalone line/remainder cases and output-fault
parity are verified ([evidence](experiments/aot-console/read-to-end-validation.json)).
Shared Raven lookup fixes now qualify a fresh imported-Console compilation through
standalone execution, with no producer staging or reuse
([evidence](experiments/aot-console/raven-lookup-validation.json)). Shared Raven concat
conversion fixes also unblock integer/null interpolation in native CIL. String-only
interpolation runs standalone
([evidence](experiments/aot-console/interpolation-validation.json)). The next bounded
slice supports copied Int32 boxes and Object display through the existing formatter;
six endpoint/ordinary cases run standalone
([evidence](experiments/aot-console/boxed-int32-validation.json)). String Object views
now unblock mixed integer interpolation with identity, null/cast and output-fault
parity ([evidence](experiments/aot-console/string-object-views-validation.json)).
Fresh CharText identity also qualifies ordinary character Console output alongside
interpolation across seven grapheme/NUL cases
([evidence](experiments/aot-console/character-text-identity-validation.json)).
Verified closed String interface views now preserve casts and identity, including
inherited generic contracts; EquatableTo<string> round-trips run standalone
([evidence](experiments/aot-console/string-interface-views-validation.json)).
String content ceq is now compiled with exact UTF-8/null semantics, enabling ordinary
Raven equality and inequality without comparison allocations
([evidence](experiments/aot-console/string-equality-validation.json)). Verified String
interface implementations now compile through ordinary receiver projection, including
standalone EquatableTo<string>.Equals with Unicode/NUL/dynamic text and output-fault
parity ([evidence](experiments/aot-console/string-interface-equality-validation.json)).
Line-oriented Console input already has standalone coverage. A bounded repeated-input
consumer now combines Int32.Parse with Result/Option patterns and Console output across
11 input streams ([evidence](experiments/aot-console/parse-session-validation.json)). The
native parser preserves whole-text ASCII grammar and malformed-before-overflow precedence
without allocating. A bounded request-line/route consumer now runs standalone across
18 input streams, using an application union, Int32 IDs and newly bound ordinal String
predicates ([evidence](experiments/aot-console/request-line-validation.json)). It covers
only two GET routes, not a server or complete HTTP grammar. The real RoutePattern
consumer then exposed ArrayList<string> storage as its first native blocker. String-array
storage now supports that path: Parse/Match/GetInt32 runs standalone with exact output
and fault parity ([evidence](experiments/aot-console/route-pattern-validation.json)).
Eight routing outcomes and retained captures now pass repeated-request validation
([evidence](experiments/aot-console/route-lifetime-validation.json)). Native arena usage
grows to 117,507 bytes at 128 requests and exhausts a fixed 64 KiB budget; the interpreter
reclaims temporary objects during the same source workload. Private typed storage-layout
diagnostics now distinguish reference slots, conditional erased payloads and managed
borrows ([evidence](experiments/aot-console/route-trace-layout-validation.json)). Ordinary
native prologues now clear traceable local lanes and erased tags without changing guest
assignment rules ([validation](experiments/aot-console/route-root-seed-validation.json)).
Reachable IL-boundary plans now identify retained evaluation-stack roots, pending operands
and spill lanes ([evidence](experiments/aot-console/route-root-points-validation.json));
native adapter/dispatch internals are explicit coverage gaps. Opt-in stack probes now
emit the planned spills and expose initialized snapshots to a linked read-only diagnostic
callback ([validation](experiments/aot-console/route-root-probe-validation.json)). Opt-in
diagnostic frames now retain ancestor
snapshots and unlink on success/fault returns ([validation](experiments/aot-console/route-probe-frame-validation.json)).
Typed argument/local lane addresses are now published in those frames, with bounded-width
reads and seeded local roots ([validation](experiments/aot-console/route-root-storage-validation.json)).
Initialized constructor storage and successful call results now have phased diagnostic
publication ([validation](experiments/aot-console/route-transient-root-validation.json)).
Ordinary-call result handoff now uses seeded caller-owned pending storage observable at
callee removal ([validation](experiments/aot-console/route-result-handoff-validation.json)).
Native service/dispatch wrappers now publish typed argument copies in diagnostic frames
and unlink on all return paths without adding guest fault frames
([validation](experiments/aot-console/route-wrapper-root-validation.json)).
Fault-context message/frame-name slots now have bounded diagnostic enumeration during
unwinding and after host return; successful reentry retires the prior view
([validation](experiments/aot-console/route-fault-root-validation.json)).
An explicit `--native-gc` mode now connects the [bounded nonmoving collector](experiments/aot-console/native-gc.md)
to complete compiler boundaries. It recovers heap-borrow owners, relies on published
owning storage for stack borrows and never collects inside native allocation services.
The real routing workload completes 1,024 requests within 64 KiB, with retained captures
and output/fault parity ([validation](experiments/aot-console/route-native-gc-validation.json)).
Basic native GC is implemented for this bounded profile. A measured in-header worklist
now replaces repeated mark passes, reusing the interpreter's traversal approach;
[paired benchmarks](experiments/aot-console/native-gc-worklist-validation.json) show a
reverse-chain improvement without a material routing gain. Continue relevant performance
measurements and review improvements for both execution modes as the POC evolves. Conservative object candidates
can over-retain, and the current stress policy collects at every boundary. Next add precise
object maps and measured pressure scheduling, then continue the HTTP service dependencies;
full server execution remains unqualified. General host handles, concurrent collection
and borrowed String interface receivers remain unsupported.
The author adds a console-input sample after unions, exercising input and
parse outcomes; strings, native input services and their lifetime contracts are later
requirements, not implied by primitive value support.
The author also raises reference counting for initial native reference types.
[Compare it with the existing tracing contract](native-execution-investigation.md#reference-counting-as-an-early-native-experiment-2026-10-07);
this remains exploratory and does not select a new managed lifetime policy.

The author also proposes future metadata alongside native images for richer native
calling interfaces, requiring stable ABI conventions. [Options and costs](native-execution-investigation.md#metadata-beside-native-images-future-exploration-2026-10-07)
remain exploratory. The immediate Some/None sample milestone is complete within the
explicit closed-world profile; the interactive integer reader is the proposed next consumer. A read-only
AOT inspection command now exposes full declarations/calls/opcode inventories and actual
compiler admission; it does not trim or establish an interface ABI. An explicit
`--closed-world` follow-up selects the bounded direct-call closure and reports exclusions
to enable value-only union paths. This brings limited code selection forward; general
reflection-aware trimming and metadata retention policy remain later work.

## Existing native release gates

**Author clarification (2026-10-08):** the target release requires full Raven
compiler-target bootstrap through neoCLR metadata, consumed by neoCLR for interpreted
or native execution, with no .NET bridge in that target path. This tightens the earlier
permitted CLI-bootstrap staging below: a generated Core.dll or CLI reference projection
may support development evidence, but does not alone qualify the release target.
Audit and remove those remaining target dependencies and rerun the matched source-build,
application, project/editor and platform gates. Native runtime seed metadata remains a
separate runtime-service contract; do not confuse it with a .NET semantic bridge.
A [reduced native-core probe](experiments/native-core-bootstrap/README.md) now closes
one writer blocker: native core containers can reference their own local core types.
The explicit compiler API native-metadata mode now bypasses the .NET session and
compiles an integer consumer that returns 42 in interpreter and ARM64 native execution.
The explicit native core/library catalog API also passes a separate-library consumer
in both modes, including explicit `rvnc --native-core-reference` selection.
Project/editor integration and complete source-runtime core remain pending. This
is not full bootstrap qualification or a replacement for the application/editor gates.
Author-directed RavenDoc loader work now reuses the native catalog and existing full
renderer, with [focused reference coverage](experiments/native-core-bootstrap/ravendoc-validation.json).
The actual documentation bundle still requires migration and a public-API inventory check.
An [admission audit](experiments/native-core-bootstrap/ravendoc-bundle-audit.json)
confirms that the existing split bundle’s CLI core rejects on the native-only loader.
Produce the matching native core and rebuild those libraries before switching the website.
Compiler host tooling and compiler self-hosting are separate scope questions; this
clarification specifically concerns the compiler target and emitted execution path.
The HTTP native image already needs no installed .NET runtime, but that deployment
property is not proof that compiler-target bootstrapping is finished.

The release also requires updated website content, matching sample downloads and setup
instructions, and an editor/LSP workflow using native NeoCLR metadata;
see the [author-directed tooling gate](#author-directed-release-gate-editor-and-native-metadata-2026-10-05).
The author-requested initial metadata disassembler is implemented; broader integration
now follows the sample-driven POC priority below.
Working samples, including Tasks and `await` through the native compiler path, are
also required; runtime suspension and green threads are explicitly deferred.

Author reaffirmation (2026-10-07): bootstrapping remains the objective, with full native
NeoCLR support and an acceptable developer experience for the next release, including
the website. Documentation improvements support this gate; they do not replace native
library, build, execution and editor qualification. Current evidence below is partial.

Author release direction (2026-10-07): Windows is required for the next native metadata
release, alongside macOS. Runtime-only Windows CI does not qualify the compiler,
source-built libraries or editor. The manual `native-toolchain.yml` workflow builds a
matched Windows x64 candidate and runs extracted native samples; its execution and
installed VS Code acceptance are still pending. See [Windows qualification](windows-native-qualification.md).
Installation pages should give one short current path, with detailed troubleshooting
linked separately. Feature pages explain behavior and limits; history belongs in the
changelog. Raven must be accessible from the main navigation, including links to its
language website and playground.

## Stable execution baseline assessment (author direction, 2026-10-08)

The author requests running samples in interpreted/native modes to prioritize release
work, then clarifies that a stable baseline does **not** require all samples in all
modes. Keep the no-bridge compiler-target bootstrap gate above; do not silently expand
AOT scope to universal API parity. The [104-case executed survey](experiments/aot-sample-assessment/README.md)
records 68 emitted artifacts, 12 matched native successes and 56 first AOT rejections;
primitive arrays account for 39. The interpreter has 63 successful zero exits, three
intentional exit/fault controls and two introspection failures. Four focused callback/
queue/Result consumers and the HTTP baseline pass separately. The author subsequently
directs fixing easy and urgent native issues: assess bounded primitive arrays next,
then rerun selected consumers rather than treating all 104 as release blockers. Bounded scalar arrays now pass 77 mode/type comparisons and eleven GC kernel tests; of ten selected follow-ups, `library-arrays` now runs in both modes and nine reveal further metadata, primitive-wrapper, callback or service boundaries. [Follow-up evidence](experiments/aot-sample-assessment/scalar-array-followup.json). The next bounded fix preserves Boolean/Int64/UInt64 primitive receivers and admits wide ordering; Boolean and generic-collection samples now run in both modes, with 56 comparison and four mutation checks. [Primitive follow-up](experiments/aot-sample-assessment/primitive-member-followup.json) records the remaining calendar arithmetic boundary.

The author proposes future AOT reflection using IL-free sidecar metadata, converging
with metadata-based interop. [Design questions](native-execution-investigation.md#il-free-metadata-for-aot-reflection-and-interop-author-proposal-2026-10-08)
separate inspection from retained native invocation, ABI/pairing and trimming. This is
not implemented and is not a new prerequisite for the initial stable AOT baseline.

## Source-built System bootstrap frontier (2026-10-07)

**The native source-owned orders gate passes.** The 197-input aggregate library is
compiled separately; unchanged application-order-collections imports its native metadata,
emits and executes with exact checked-in output and exit 0. Explicit imported Object
selection unifies semantic roots without weakening inheritance checks. The runtime uses
the finalized retained seed and explicit source-root artifact. This meets the broad
native application gate under the permitted bootstrap dependencies, not complete API
coverage or a bootstrap-free/production-packaged release.

The 174-input **System.Runtime candidate also builds independently** of Data,
Networking and Web; unchanged orders imports it and executes with exact output/exit 0.
[Runtime split evidence](experiments/extended-cli-metadata/runtime-split-2026-10-07.md).
The first separate Data/Networking attempts expose two bounded gaps: Data's internal
array-reflection helpers cross the new boundary; metadata override validation cannot
select an imported native Object identity. The primitive flags-marker import blocker
is fixed. [Commands and diagnostics](experiments/extended-cli-metadata/optional-library-frontier-2026-10-07.md).
Explicit imported Object authoring now passes C# contracts and API/Raven consumer
execution (42). Networking advances to a System.Value encoding dependency.
[Imported-root evidence](experiments/extended-cli-metadata/imported-object-authoring-2026-10-07.md).
Imported Value ownership now passes source-free parse/type-test/unpack execution
(42), with 165 metadata groups passing. Networking reaches imported virtual Object
method calls. [Value evidence](experiments/extended-cli-metadata/imported-value-2026-10-07.md).
Imported Object slot references now execute ToString/GetHashCode/Equals overrides
through an object receiver (42), with 165 metadata groups passing.
[Slot evidence](experiments/extended-cli-metadata/imported-object-slots-2026-10-07.md).
Separate **System.Networking now compiles and executes** the unchanged cancellation
consumer against System.Runtime (exact stdout, exit 0). The CheckedStorage failure was
an omitted explicit `--bootstrap-intrinsics` audit option; the subsequent boxing gap
now validates the complete selected external Object contract.
[Networking gate](experiments/extended-cli-metadata/separate-networking-2026-10-07.md).
Separate **System.Data now executes JSON object mapping** using the public Runtime
ArrayReflection boundary. Nested models, setters, managed/jagged arrays, mutation,
reference identity and invalid-input checks pass against emitted references only.
[Data gate](experiments/extended-cli-metadata/separate-data-2026-10-07.md).
Separate Web now compiles and executes through a supported NetworkDeadline contract
owned by Networking. Source-free deadline, headers, base-address, JSON-client and route
consumers pass, along with the loopback header-cancellation check. HTTP retains one
15-second exchange budget. [Evidence](experiments/extended-cli-metadata/separate-web-2026-10-07.md).
Raven e141006f3 fixes the intermittent `System.Void` encoding failure at type binding:
exact bootstrap Void lookup now honors the selected native unit owner. Two deterministic
regressions and six fresh builds pass. [Evidence](experiments/extended-cli-metadata/unit-bootstrap-2026-10-07.md).
The general async unit-return correction is handed to the author-designated Raven release
task; its integrated compiler revision is af47cb0a5, with native async qualification still
separate. Native project import/run now carries the exact source-built Object root via
RavenNeoClrObjectLibrary (Raven 0f85f53b8); the unchanged HTTP headers project executes
and invalid ownership preserves the prior output. [Evidence](experiments/extended-cli-metadata/native-project-root-2026-10-07.md).
Native project-reference builds and prebuilt workspace imports now pass a four-project
execution diamond, including generic object mutation and failure preservation.
[Graph evidence](experiments/extended-cli-metadata/native-project-graph-2026-10-07.md).
The checked-in native System.Runtime project now builds the 175-source foundation;
unchanged orders executes against its output after explicit retained-seed finalization.
[Runtime project evidence](experiments/extended-cli-metadata/native-runtime-project-2026-10-07.md).
Data, Networking and Web now have checked-in native projects. A staged build produces
all four libraries, finalizes the retained seed against the exact Runtime output and
records a hashed bundle manifest. Five native consumers and the project HTTP consumer
pass against those artifacts. [Bundle evidence](experiments/extended-cli-metadata/native-class-library-bundle-2026-10-07.md).
The staged bundle also supplies relocatable project configuration. Workspace symbol
loading and ordinary HTTP project execution pass after relocation, with native owners
and missing/conflicting-input rejection checked. [Configuration evidence](experiments/extended-cli-metadata/native-bundle-configuration-2026-10-07.md).
Installed-editor acceptance now covers this split, including native optional-library
navigation, configuration recovery and orders/Tasks execution. Hover latency is
measured separately; the reported persistent delay remains open.
[Editor evidence](experiments/extended-cli-metadata/native-split-editor-2026-10-07.md).
The bundle now also stages and hashes each assembly's generated XML/Markdown help;
relocation verifies native IPAddress documentation. This transports existing comments,
not complete API coverage: Data's generated XML currently contains no member entries.
[Documentation packaging](experiments/extended-cli-metadata/native-bundle-documentation-2026-10-07.md).
The native POC packager now accepts the split bundle and records its explicit Object
root, runtime seed and module list. Extracted verification uses those same selections;
relative sample projects and SDK settings avoid development checkout paths.
[Distribution qualification](experiments/extended-cli-metadata/native-split-distribution-2026-10-07.md).
Standalone bootstrap preparation now generates Core and retained-seed inputs from
checked-in sources without previous Numbers/Http or Runtime artifacts; the class-library
builder validates its manifest before compilation.
[Preparation evidence](experiments/extended-cli-metadata/native-bootstrap-preparation-2026-10-07.md).
Author reprioritization (2026-10-07): defer the native RavenDoc provider/model migration
and assembly-label correction. A possible RavenDoc rewrite in Raven is future exploration,
not a current implementation commitment. These changes are not gates for this release.
Keep the existing reference's bridge provenance explicit; do not relabel CoreProbe as
production ownership. Preserve the [future provider direction](experiments/extended-cli-metadata/native-documentation-provider-2026-10-07.md).
Independent source checkouts now rebuild the runtime, SDK, bootstrap inputs and four
libraries; the extracted collection/Tasks/JSON/HTTP gates pass after explicit Apple SDK
selection and serial build recovery. All 26 installed-package editor checks also pass.
[Clean-source evidence](experiments/extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md).
Next: exact release-candidate/platform qualification and remaining installation/editor
issues. Prioritize demonstrated release failures and useful shipped behavior;
assess Platform boundary work against that scope rather than expanding documentation
architecture. Existing API-help coverage limitations remain visible.
The bundle alone is not an SDK release.
The API site remains one namespace/type reference across assemblies; declaring-assembly
provenance belongs on pages. Native multi-input RavenDoc migration must preserve that
shape, as recorded in [API maintenance](../api-docs/README.md#one-class-library-reference-across-assemblies).

Author clarification (2026-10-07): follow the executable split with real library and
Platform integration projects whose outputs form the shipped artifacts, rather than
leaving the graph in audit source lists. Future RavenDoc work needs an explicit assembly
bundle and target-independent, commit-pinned declaration source links, including exact
GitHub file/line locations. See [project gate](experiments/extended-cli-metadata/package-boundaries-2026-10-07.md#real-project-graph-author-direction-2026-10-07)
and [documentation acceptance](../api-docs/README.md#planned-bundles-and-declaration-source-links).
These are recorded requirements; project/bundle/source-link implementation remains open.
Author merge direction (2026-10-07): once bootstrap is ready, merge
`codex/native-system-bootstrap` into neoCLR `main` and continue development there.
Unqualified project-graph/editor/shipping artifacts keep this
merge gate open; the instruction authorizes merging when qualified, not prematurely.

Author-directed foundation rule (2026-10-07): System.Runtime owns well-defined
fundamentals; higher-level packages depend on it, never the reverse. Exact Runtime
membership remains subject to dependency review. .NET is a comparison point, not a
required package layout; document benefits and costs of different boundaries.
The author's platform-package direction is recorded separately from assembly ownership:
[candidate distribution boundaries](experiments/extended-cli-metadata/package-boundaries-2026-10-07.md).
No new package resolver or native plugin mechanism is a prerequisite for these gates. Keep the existing POC sample/editor gates as ownership moves.
[Commands and evidence](experiments/extended-cli-metadata/source-owned-orders-2026-10-07.md).

### Earlier imported source-root frontier

Native System.Value now imports with its declared semantic identity. The unchanged
orders consumer gets through binding and reaches **System.Collections.ArrayList<T>
emission validation**. Investigate imported source Object ownership for generic classes
before relaxing any capability checks. Full-artifact verification/control execution
from the retained-catalog slice remains valid; broad application execution is still open.
[Value import evidence](experiments/extended-cli-metadata/value-import-2026-10-07.md).

### Earlier erased-value importer frontier

All 197 aggregate inputs emit, and the finalized retained seed now names the source
owner explicitly. The combined load set verifies 2,433 IL functions and runs an
API-authored control (42). The next gate is **ordinary Raven native consumption**:
unchanged `application-order-collections` fails on native System.Value's classification
as a nonexistent Raven SpecialType. Fix that importer fact before expanding execution
or splitting core/Data/Networking/Web. Full class-library API coverage is not claimed.
[Catalog evidence](experiments/extended-cli-metadata/retained-catalog-2026-10-07.md).

### Earlier retained dependency frontier

The selected source/imported System.Void now uses native unit storage, including
callback results and generic arguments. API-authored execution and separately compiled
Raven NativeMemory consumers pass. All 197 aggregate inputs emit; full admission now
reaches a **retained seed dependency edge to the source owner**, after the DNS signature
contract. Resolve the seed/source dependency catalog before splitting and consuming core.
[Canonical unit evidence](experiments/extended-cli-metadata/canonical-unit-2026-10-07.md).

### Earlier source-unit callback frontier

Separately scoped WriteLine value/no-result declarations now execute under their
own admitted registry contracts. Full-System admission advances to **source-owned
unit in native callback signatures** (DnsLookup), with no new compiler or format
change. Resolve that selected-unit representation next, then resume core admission
and the candidate optional-library split.
[Evidence](experiments/extended-cli-metadata/scoped-service-results-2026-10-07.md).

### Earlier retained-service frontier

The full-source audit now removes the competing seed Object; all 197 inputs emit.
Runtime admission advances past Array backing to **retained/source WriteLine result
ABI** (Void value versus no-result). Reconcile service ownership/contracts next.
The candidate Runtime/Data/Networking/Web inventory has no local core-to-optional
edges; build and consume those assemblies in dependency order after core admission.
[Prioritized gates and evidence](experiments/extended-cli-metadata/library-boundaries-2026-10-07.md).

### Earlier aggregate-emission frontier

All 197 diagnostic System inputs now emit a native PE. Required schema 4 admits
larger library envelopes while retaining earlier-profile bounds. Full-artifact runtime
admission next rejects **nominal array backing storage**; resolve that actual source
shape before claiming bootstrap. The author also suggested optional System.Data,
System.Networking and System.Web assemblies: map dependency/service ownership and
keep the aggregate compile as coverage, not a proposed monolithic distribution.
[Evidence and packaging direction](experiments/extended-cli-metadata/expanded-library-2026-10-07.md).

### Earlier String frontier

Intrinsic String now retains and executes its selected source Object relationship.
The linked PE constructor gate returns 42; the 197-input audit advances to the
**binary library payload limit**, with no published output. Next inspect payload
size and coordinated writer/reader budgets before expanding them.
[Evidence](experiments/extended-cli-metadata/string-root-2026-10-07.md).

### Earlier array frontier

Array backing validation now admits the explicitly selected fieldless source Object
root. Linked PE execution verifies allocation and mutation through an alias, returning
42. The 197-input audit advances to **System.String -> System.Object base-category
validation**. Next align intrinsic String reference storage with that source-root
relationship. Full System still publishes no artifact.
[Evidence](experiments/extended-cli-metadata/array-root-2026-10-07.md).

### Earlier erased-value frontier

Source-owned System.Value now has the runtime erased-carrier representation; the
real environment payload type-test/unpack gate executes successfully. The 197-input
System audit advances to **native array backing-storage validation**. Next trace
that contract while preserving managed arrays and canonical source ownership.
Full System still publishes no artifact; erased helpers remain explicit seed inputs.
[Evidence](experiments/extended-cli-metadata/source-value-2026-10-07.md).

### Earlier construction frontier

Native reflection construction now works with source Object/RuntimeTypeHandle;
constructor execution, distinct identity and access/missing-constructor checks pass.
The 197-input audit excludes the seed-only GetType extension and reaches
**System.Value ownership at Environment.GetCurrentDirectory's generic call**.
Next reconcile the source/imported Value intrinsic contract without relaxing stack
validation. Full System still publishes no artifact.
[Evidence](experiments/extended-cli-metadata/reflection-construction-2026-10-07.md).

### Earlier Object handle frontier

Source Object now uses an explicit native handle service facade. A source-owned
Object/RuntimeTypeHandle consumer verifies and executes identity/hash checks. The
196-input audit advances to **ReflectionConstruct dependency-contract resolution**.
Next resolve that remaining bootstrap signature through the native adapter boundary;
full System still publishes no artifact.
[Evidence](experiments/extended-cli-metadata/object-handles-2026-10-07.md).

### Earlier closed-family frontier

Closed families now preserve source Object as their base. Protected initialization
and virtual dispatch execute through the resulting three-level hierarchy. The
195-input audit advances to **ObjectTypeHandle dependency-contract resolution**;
next align this intrinsic with source Object/RuntimeTypeHandle ownership through
explicit bootstrap contracts. No full System artifact is published.
[Evidence](experiments/extended-cli-metadata/closed-object-root-2026-10-07.md).

### Earlier unit ownership frontier

Early unit resolution now rejects wrong-assembly lookup fallbacks. Source-owned Void
remains canonical through generic interface signatures, including when unit is needed
before source declarations exist. The native unit-interface consumer and NativeMemory
success/fault gates pass, with the ordinary-bootstrap control preserved. The 195-input
System audit now reaches **direct-base constructor call validation**. Next identify
the caller/target relationship and repair it at the owning layer; no full System
artifact is published. [Evidence](experiments/extended-cli-metadata/unit-owner-2026-10-07.md).

### Earlier unit-reference frontier


Explicit source/native System.Void ownership now preserves inhabited unit parameters,
no-result calls and PTR VOID signatures. The separately compiled NativeMemory consumer
runs with source-owned Void; double-free, overflow and rejection gates pass, as does
the ordinary-bootstrap control. The 195-input full-System audit clears binding and
pointer admission but stops during encoding on a remaining **bootstrap System.Void
reference**. Next trace and migrate that reference through semantic unit ownership;
do not reintroduce a competing seed copy. No full System artifact is published.
[Source-unit evidence](experiments/extended-cli-metadata/source-unit-2026-10-07.md).

### Earlier source attribute frontier


Source Attribute and UnionAttribute now preserve local inheritance, and native unions
reuse the source marker constructor. Canonical metadata inspection, separate consumer
execution (exit 42), embedded-marker control and invalid-marker rejection pass.
The 195-input System audit next rejects **NativeMemory.Alloc's pointer to source Void**;
binding is clean and no full-System artifact is published. Next reconcile that signature
with the existing pointer-to-CLI-void encoding and configured unit ownership.
[Source-attribute evidence](experiments/extended-cli-metadata/source-attributes-2026-10-07.md).

### Earlier bootstrap attribute frontier


Native FlagsAttribute and MethodImpl(InternalCall) validation now uses the explicit
primitive-bootstrap identity, independently of source Object ownership. The source-root
regression and wrong-owner rejection pass; ordinary-bootstrap NativeMemory still runs.
The 194-input System audit next rejects **UnionAttribute**, whose System.Attribute base
is external bootstrap metadata. Next establish explicit Attribute ownership/inheritance
for the source library and its metadata representation. Full System still emits no output.
[Core-attribute evidence](experiments/extended-cli-metadata/core-attributes-source-root-2026-10-07.md).

### Earlier generic-root frontier

Generic reference classes can now inherit the source Object root through an explicit
Raven target capability and the metadata builder/definition path. Generic storage,
base construction and inherited dispatch execute in focused native tests. The System
audit gets past Array<T> and next stops on enum attribute ownership. Next reconcile
FlagsAttribute selection with source/core declaration ownership, then retry emission.
No full System artifact is emitted; ordinary imported-root consumers are still unsupported.
[Generic-root evidence](experiments/extended-cli-metadata/generic-object-root-2026-10-07.md).

### Earlier NativeMemory frontier

Source NativeMemory now compiles and is consumed through native metadata. Both Alloc
forms and Free execute; double-free and checked-size overflow fault, and unsupported
pointer signatures publish no output. The full-owned-handle audit clears **binding
across 194 inputs**, then rejects **Array<T>**, the first generic reference type with a
source-defined Object base. Next extend that shared ownership/metadata relationship
rather than patch individual Array APIs. Full System still emits no assembly.
[NativeMemory evidence and limits](experiments/extended-cli-metadata/source-native-memory-2026-10-07.md).

### Earlier prerequisites

NativeAllocation runtime prerequisites now execute through native containers: exact
allocation/free/checked-size services share the established pointer heap and limits.
Five focused service tests and 17 pointer regressions pass. Bounded unmanaged pointer
signature authoring, reading/import and introspection now pass 162 C# groups; an
API-authored allocation/free assembly executes with exit 42. Next add Raven pointer
mapping, then connect the unchanged NativeMemory sources. The four source binding errors remain; no full-System progress
is claimed from runtime-only support. [Runtime contract](heap-and-pointers.md#source-nativeallocation-services-2026-10-07).

Explicit source/native terminal-function ownership now clears the six let-else and two
HTTP Task return errors. The audit has **four binding errors across 192 inputs**, all
NativeAllocation imports/member uses. Local and separately compiled Fail control-flow
consumers execute both branches correctly; invalid owners publish no output. Next resolve
NativeAllocation at its owning native instruction/service layer, then re-run the full
compile to expose any encoding/linking frontier. Full System still emits no assembly.
[Terminal-flow evidence](experiments/extended-cli-metadata/source-failure-flow-2026-10-07.md).

### Earlier frontier — 2026-10-06

The source System.Fail helper now compiles and its native consumer reports the expected
UserFault without returning. Full-owned-handle binding is down to **12 diagnostics across
192 inputs**. Next make terminal-call recognition use an explicit source/native owner
contract instead of the legacy core identity; then resolve NativeAllocation. Six let-else
and two HTTP Task return errors still need isolation. No full System assembly is emitted.
[Failure gate and remaining boundary](experiments/extended-cli-metadata/source-failure-2026-10-06.md).

Source Console now compiles and executes through native metadata with its own explicit
bootstrap profile: no Console declaration in the primitive core or retained seed.
UTF-8 input/output, EOF, overloads and non-owning wrappers pass an artifact-only consumer.
The full-owned-handle binding audit is down to **14 diagnostics across 190 inputs**.
Next address RuntimeFailure/NativeAllocation ownership, let-else termination and HTTP
Task return binding. Full System still fails binding; its later encoding/linking gate
has not been reached. [Console evidence](experiments/extended-cli-metadata/source-console-2026-10-06.md).

The source native integer declarations now compile separately with explicit ownership,
and an artifact-only consumer executes signed/unsigned CompareTo calls. The corresponding
metadata categories, Raven mappings and runtime widening services are connected. The
full-owned-handle audit is now **30 diagnostics across 188 inputs**, down from 35.
Next complete Console service adapters and their seed compatibility, then re-assess
remaining binding errors. [Native integer gate](experiments/extended-cli-metadata/source-native-integers-2026-10-06.md).

The calendar/time-zone group now compiles separately and its native consumer executes
DST gaps/overlaps, offsets and optional local/zoned union conversions. Native constructor
union import and managed mapping-array results close the exposed integration gaps.
The latest full-System audit has **35 diagnostics across 186 inputs**, down from 48;
Console/native-width service inputs and remaining binding failures are next.
[Calendar evidence](experiments/extended-cli-metadata/source-calendar-2026-10-06.md).

The author requested continuation toward full bootstrap after Preview 12. A fresh
inventory confirms that Storage's existing adapters and six omitted contracts compile;
release omission does not mean missing compiler support. The first full-source crash
was a PE-only reflection-loader assumption, now fixed on an isolated Raven integration
branch with 41 focused tests and unchanged released library compilation controls.
Union ToString now completes the real source Object signature on demand; declaration
order regressions pass and the full build reaches diagnostics instead of crashing.
The existing source RuntimeTypeHandle contract also works once explicitly selected in
the audit. Next complete missing runtime-service inputs, reduce residual binding failures,
and finish the ownership catalog before replacing the CLI primitive reference.
The Math family now compiles and executes all 20 functions through a separate native
consumer; its 15 missing services are resolved and full-source diagnostics drop to 59.
[Gate and bootstrap import limitation](experiments/extended-cli-metadata/source-math-2026-10-06.md).
Callable nullable metadata now survives native encoding, introspection and Raven import.
The independently compiled GC null-call consumer executes, including KeepAlive(null),
with unchanged runtime semantics. Reference/array/generic symbol controls, 17 focused
.NET checks and seven existing native consumers pass. Continue reducing the remaining
full-System service/binding failures and completing ownership; context/field nullable
annotations remain a bounded follow-up rather than reopening this GC gate.
[Annotation direction](design/callable-nullability.md).
[GC gate](experiments/extended-cli-metadata/source-heap-2026-10-06.md).
The Environment family now compiles and executes all three APIs through a native
artifact-only consumer. Managed argument arrays preserve independent snapshots and obey
allocation limits. The full-owned-handle inventory is now 48 diagnostics (184 inputs).
Next address Console/native-width service ownership and remaining Calendar services;
retain public overloads while closing their underlying contracts.
[Environment gate](experiments/extended-cli-metadata/source-environment-2026-10-06.md).
Full System still does not compile. [Frontier and evidence](experiments/extended-cli-metadata/native-bootstrap-frontier-2026-10-06.md).

## Native source release direction (2026-10-05)

The author explicitly reaffirmed that native metadata replaces the CLI translation
bridge and that APIs must execute correctly. Do not expand the legacy bridge to clear
its snapshot audit as the next native-release task. The selected libraries now rebuild
directly from 115 cumulative and 19 HTTP source/adaptor files; 15 consumers compile,
13 non-network executions and both live HTTP/JSON rounds pass using those new assemblies.
[Native source gate and explicit bootstrap limits](experiments/extended-cli-metadata/native-source-release-2026-10-05.md).
The local extracted bundle now passes five project compilations, collections/Tasks/JSON
execution, live HTTP/JSON checks and 19 installed VSIX checks.
[Bundle procedure](native-poc-bundle.md) and
[qualification evidence](experiments/extended-cli-metadata/native-bundle-2026-10-05.md).
Preview 12 is selected for macOS arm64. Both retained bootstrap inputs now reproduce
byte-for-byte from recorded sources. Local candidate qualification passes 1,784 runtime
tests, extracted native consumers and the website gate. Hosted canonical, OS boundary
and minimum-Rust validation pass. The work is integrated into main; Preview 12 and
the website are published. [Publication evidence](preview-12-publication.json). [Candidate evidence](preview-12-validation.json).
The primitive CLI core and retained seed remain explicit temporary bootstrap inputs;
complete replacement and full System source ownership remain open.

## Native VS Code POC accepted (2026-10-05)

The author requested continued work until VS Code acceptance. Real VS Code 1.140.0
on macOS arm64 now passes the bounded native editor gate: native artifact-only import,
completion/hover, read-only declaration navigation, reference replacement with unsaved
buffers, missing dependency diagnostics/recovery, and build/run through the same
project configuration. Unchanged collections and Tasks/await samples execute against
the recorded source-built library artifacts; a .NET editor control remains green.
[Acceptance evidence and reproduction](experiments/extended-cli-metadata/native-vscode-acceptance-2026-10-05.md).

This supersedes the editor wiring/qualification next steps in the historical slices
below. The matched local bundle and installation acceptance are now recorded above.
Bootstrap provenance, main integration and Preview 12 publication are recorded. Initial
project-open failures still use existing logs; workspace-external file watchers,
full decompilation, general native project/package dependency builds and other OS
qualification remain beyond this POC. Full class-library source bootstrap remains a
separate roadmap gate. Preview 12 publishes the bounded macOS arm64 native POC.

Native API documentation now also passes real editor acceptance: XML/Markdown hover
and completion prose, sidecar-only refresh and member-level fallback.
[Documentation evidence](experiments/extended-cli-metadata/native-ide-docs-2026-10-05.md).
The author explicitly includes website/API rendering through RavenDoc in the next
release gate, alongside matched package installation.

The matched SDK/installed VSIX now passes 19 checks on macOS arm64, including native
Option/Result API help. RavenDoc union-case summaries, all 1,803 generated pages and
18 website tests pass. Bootstrap reproduction and the local runtime/dependency bundle are now qualified;
Preview 12 publication followed the passing candidate CI gate.
[Release gate and remaining work](experiments/extended-cli-metadata/native-release-gate-2026-10-05.md).

## Shared host snapshots prepare editor integration (2026-10-05)

Raven's native driver now uses a reusable explicit reference catalog. Core identity,
semantic references and emission bindings come from matching snapshots; reloading a
replaced library exposes new members while old compilations retain old symbols.
Native and .NET execution controls pass. This is a prerequisite, not LSP completion.
Next connect evaluated native project configuration to this catalog, then qualify
semantic refresh, imported-member navigation and editor build/run with the same inputs.
[Scope and evidence](experiments/extended-cli-metadata/native-editor-catalog-2026-10-05.md).

## Inheritance completes the original sample gate (2026-10-05)

The unchanged inheritance sample now compiles and executes on both targets with
stdout `7` then `42`, exit 0 and empty stderr. Native abstract slots, exact local
overrides, direct base calls and inherited interface conformance use existing CLI
flags/native runtime contracts. Collections and interface controls still execute.
All ten original POC samples now have execution evidence across the linked slices;
this is not a new full-library or release claim. Next qualify the native metadata
language-server/VS Code workflow and matching setup/download instructions.
[Evidence and limits](experiments/extended-cli-metadata/native-inheritance-2026-10-05.md).

## Abstract class metadata prerequisite (2026-10-05)

Ordinary abstract classes now retain CLI Abstract independently of static and closed
families through definitions, builders, native readers and introspection. Concrete
subclasses execute constructor chaining and inherited-field reads on both runtimes.
The unchanged inheritance sample remains blocked: ordinary virtual/abstract method
slots, Raven admission and dispatch still need coordinated completion. The sample
count remains nine of ten. [Focused evidence](experiments/extended-cli-metadata/abstract-classes-2026-10-05.md).

## Ad-hoc error composition (author-directed, 2026-10-13)

The [configuration-loader slice](experiments/adhoc-error-unions/README.md) supports
propagating either existing error into `Result<T, E1 | E2>` using temporary nominal
carriers. It follows Raven's .NET carrier approach and needs no shared error base
or converter-only aggregation. Native structural union identity and values remain
future work; nominal domain errors remain preferable for many public APIs.
This bounded author-directed integration does not change the active performance
priority. Validation, compiler branches and replacement limits are linked above.

## Union reference-producer regression closed (2026-10-05)

Raven's lexical union case repair is integrated independently on main and the native
line. Cold semantic queries and emission agree even with legacy bootstrap declarations;
190 main checks and 196 integration checks pass. Plain/generic union factory libraries
and consumers execute on both targets. The current native-line compiler regenerates
the website CLI reference byte-for-byte identically to the checked-in snapshot.
[Evidence and reproduction](experiments/extended-cli-metadata/union-lexical-cases-2026-10-05.md).
Resume the remaining inheritance sample, then native editor/LSP release qualification.

## Website release preparation (2026-10-05)

Homepage and Raven integration content now distinguish published bridge behavior from
native development. The API snapshot is regenerated using Raven main `08f34891b` and
current bridge sources. The Option/None failure recorded on integration `9a4f74884`
is resolved by the lexical case repair above.
See [the reference producer record](../api-docs/README.md#release-preparation-snapshot-2026-10-05).
Matching downloads, release-specific instructions, editor evidence and manual publication
remain pending; a successful local website build does not release the platform.

## Ordinary .NET field-return regression repaired (2026-10-05)

The separate .NET field-return candidate reproduces on Raven main and is repaired on
that shared line, preserving receiver order, object identity and early-return behavior.
The integrated fix branch has been deleted. This closes a compiler quality issue found
through native HTTP work; it does not advance the native sample count beyond nine of ten.
[Validation and release follow-up](experiments/extended-cli-metadata/dotnet-field-return-2026-10-05.md).
The next sample gate remains `application-inheritance`, followed by the native editor/LSP
release workflow. Full System completion remains outside the POC gate.

## Value construction clears the ninth sample (2026-10-05)

The unchanged `application-types` sample now compiles and executes with expected class
identity and struct-copy behavior. Raven initializes value auto-properties through their
owned backing fields, retaining the existing constructor verifier. The dual-target
source-value probe validates initialization and copies on .NET and NeoCLR.

Nine of ten original samples now have native compilation and runtime evidence. Next
unblock `application-inheritance`: abstract base declarations, virtual/override slots and
interface dispatch through that hierarchy. Keep the separately recorded ordinary .NET
field-return candidate and editor/LSP release acceptance visible.
[Value construction evidence](experiments/extended-cli-metadata/native-value-properties-2026-10-05.md).

## Async entry completion and remaining sample gate (2026-10-05)

Native Task<unit>/Task<int> entries now drain registered work before observing the result;
arguments, integer exit status, cancellation and unresolved-task faults are verified.
Both unchanged async Main samples execute. Eight of the ten original POC samples now
compile, and all eleven non-network controls pass (including five focused regressions).
The HTTP pair retains its recorded localhost execution evidence.

Next unblock `application-types` (Point constructor receiver validation), then
`application-inheritance` (native declaration capability), with focused regressions and
ordinary .NET controls. The separately recorded .NET field-return candidate still needs
main reproduction and isolated repair. Keep editor/LSP release checks visible after the
sample gate; full System completion is not a prerequisite.
[Entry contract and inventory](experiments/extended-cli-metadata/native-async-entry-2026-10-05.md).

## Native async state machines execute (2026-10-05)

Top-level nongeneric async functions now emit through Raven's existing heap lowering.
A native driver/runtime regression verifies completed/pending awaits, hoisted local
preservation and cancellation with exact output. The unchanged cancellation sample also
executes. Immutable hoisted closure captures now preserve Promise identity; both async
samples reach the entry completion blocker. Nongeneric class async methods now preserve
receiver identity and private access through metadata nesting. Both unchanged HTTP samples
now compile and execute together over localhost, including JSON error responses and
GET/POST mapping. Async Result propagation success/error paths have a focused execution
regression. Next complete async entry semantics for the remaining two async samples.
This does not introduce runtime suspension or green threads.
[Async scope](experiments/extended-cli-metadata/native-async-emission-2026-10-05.md) and
[native HTTP execution evidence](experiments/extended-cli-metadata/native-http-execution-2026-10-05.md).

## POC sample gate and next blocker (2026-10-05)

Three of ten selected unchanged samples compile through the native driver and execute
with exact output: order collections, interfaces and JSON object mapping. Five samples
share native Task async-identity failures; constructor-receiver validation and inheritance
account for the other two. [Inventory, evidence and reproduction](experiments/extended-cli-metadata/poc-sample-inventory-2026-10-05.md).
Next connect native Task/builder identities to existing heap async lowering and validate
the async/HTTP samples. Source-root replacement is not a prerequisite for these retained-
seed POC cases. Full System completeness does not gate the POC.

## Author scope clarification: end-to-end POC (2026-10-05)

The author clarified that the goal is an end-to-end NeoCLR POC, not solving every
platform feature for release. **Samples must compile.** Keep the existing editor and
execution objectives visible, but do not make complete System or exhaustive feature
coverage prerequisites for the POC. After the bounded root driver slice, inventory the
samples through ordinary compiler commands and prioritize shared compilation/execution
blockers by how many samples they unlock. Do not broaden scope merely to complete APIs.

The source-root library now compiles through `rvnc neoclr --source-object-root` and
executes through `neoclr --object-root` with an explicit seed. The caller remains neoIL;
Raven imported-root consumers are still pending. [Reproduction](experiments/extended-cli-metadata/source-object-driver-2026-10-05.md).

## Native metadata disassembly is available (2026-10-05)

`neoclr disassemble` inspects PE/#Neo, NEOX and format-5 JSON without dependencies or
execution. Declaration facts and indexed instruction operands are preserved in a
diagnostic listing; reassembly is not supported. Six focused disassembler checks and
nine existing CLI/source-emission checks pass. [Usage and limits](il-inspection.md).
Resume the source-root driver/consumer and constructed-local-base work below; native
Tasks/await samples and the shared CLI/editor configuration remain release requirements.

## Raven source Object root executes (2026-10-05)

Raven `246e8a5db` emits a source Object root and derived override; NeoCLR loads the PE
under explicit host selection and executes it. 47 focused compiler and 15 runtime checks
pass. [Evidence and remaining limits](experiments/extended-cli-metadata/source-object-emission-2026-10-05.md).
The author-requested initial metadata disassembler is now implemented. Resume
source-root driver/consumer wiring and constructed local-base support. Production System,
VS Code/LSP and native Tasks/await samples remain release gates.

## API-authored local Object overrides execute (2026-10-05)

A derived class emitted by the metadata API now constructs through its protected root
constructor and dispatches all three overrides through the selected root. Native readers
and introspection preserve exact Equals identity and reused-slot flags; 158 C# groups and
14 runtime identity tests pass. [Evidence and reproduction](experiments/extended-cli-metadata/owned-object-overrides-2026-10-05.md).
The next compiler slice must explicitly admit the baseless source root and its concrete
virtual declarations, map special Object to that source owner, and wire the metadata
adapter. Keep emission guarded until these contracts and the executable probe agree.
VS Code/LSP and working native Tasks/await samples remain release gates.

## API-authored root boxing executes (2026-10-05)

`AssemblyBuilder.ObjectType` now selects the owned root for boxing/value-test results.
An API-authored BoxedDisplay function executes in NeoCLR without a legacy System binding;
incomplete roots and mixed bootstrap return signatures reject. All 157 C# groups pass.
This closes an emitter prerequisite exposed during the Raven adapter review. Local
Object overrides, compiler declaration capabilities and selected-root driver wiring
remain next; production source Object and editor/Tasks-await release gates remain open.
See [root-boxing evidence](experiments/extended-cli-metadata/object-root-boxing-2026-10-05.md).

## API-authored Object slots execute (2026-10-05)

The metadata API now declares Object's three concrete virtual new slots and preserves
those facts through native readers, introspection and CLI reference projection. An
API-produced PE loads through explicit host root selection and executes all three slots
plus boxed Int32 display. 157 C# metadata groups and 13 runtime identity tests pass.
Next connect root signatures/boxing/overrides to Raven emission and dependency selection,
then the shared CLI/VS Code project gate. Production source Object is not yet emitted.
See [reproduction and scope](experiments/extended-cli-metadata/object-root-slots-2026-10-05.md).

## Object root declaration authoring passes (2026-10-05)

The metadata API now authors an explicit baseless canonical Object through definitions
or builders, preserving CLI Object signatures in the reference projection. All 156 C#
metadata groups pass. This is declaration support only: native virtual slots, boxing
root selection, imported root selection and compiler/driver wiring remain before the
production seed can be replaced. The editor gate includes VS Code and must share that
same compiler configuration; no language-server integration is claimed by this slice.

## Source Object producer binding passes (2026-10-05)

Raven `e748b089f` [selects the source Object semantic root](experiments/extended-cli-metadata/source-object-binding-2026-10-05.md)
before member signatures. Named/keyword types, implicit bases and overrides agree;
75 focused compiler regressions and the native no-publication probe pass. Both emitters
still reject this opt-in configuration. Next implement metadata definition/builder root
authoring, then imported root selection and driver/runtime wiring. The production root
has not replaced the retained seed, and full-System compilation remains incomplete.

## Explicit runtime Object-root selection executes (2026-10-05)

The [host selection gate](experiments/extended-cli-metadata/explicit-object-root-2026-10-05.md)
now loads a separately authored root and executes boxed Object slots and rootless
overrides. Exact identity/slot validation and seed dependency/access checks preserve
the existing lookalike rejection. Selection is transient host configuration, with no
format extension. 86 focused regressions pass. Next connect source-root ownership to
Raven binding and metadata authoring, then ordinary CLI loading; source Object and
full-System compilation are still incomplete.

## Object root ownership contract isolated (2026-10-05)

The [root investigation](experiments/extended-cli-metadata/object-root-ownership-2026-10-05.md)
locks seed-root dispatch and rejects implicit ownership by an external type's name.
A proposed assembly-name relaxation broke an existing lookalike rejection and was
reverted. Next implement explicit root selection across bootstrap configuration,
binding, metadata authoring and runtime admission as one contract. Source Object and
full-System compilation remain incomplete; no seed replacement is enabled by this slice.

## Object service prerequisite executes (2026-10-05)

The [Object service gate](experiments/extended-cli-metadata/object-services-2026-10-05.md)
executes source adapters for reference equality, base equality and identity hashing.
A runtime linking fix preserves separate assemblies' InternalCall declaration identities.
The source Object probe now has no missing-service errors; root identity, override and
generic conversion errors remain. Next implement explicit canonical Object ownership
across binding, metadata boxing/slot authoring and runtime dispatch. Object is not a
scalar primitive, and its seed root must remain until that replacement is executable.

## Source-owned runtime handle executes (2026-10-05)

The [source handle gate](experiments/extended-cli-metadata/source-handle-ownership-2026-10-05.md)
adds RuntimeTypeHandle to the combined library (140 production sources plus nine native
adapters), removes its competing seed declaration and executes separate JSON/Tasks
consumers through native import. Source Object ownership is the next core blocker,
followed by missing service families. This supersedes the handle blocker below;
full-System bootstrap and native async remain open.

## Combined source library and array dispatch (2026-10-05)

The [post-HTTP assessment](experiments/extended-cli-metadata/post-http-compilation-2026-10-05.md)
compiles 139 production sources plus nine adapters into one library; two JSON consumers
and the Tasks consumer execute against that artifact. Array/Object dispatch now retains
the Array<T> element argument. The full 166-source attempt still fails before publication.
Next reconcile source Object/RuntimeTypeHandle ownership and complete core native service
contracts, then the time-zone family. Native async and full .NET library parity remain open.

## Native source HTTP gate passes (2026-10-05)

The [HTTP execution gate](experiments/extended-cli-metadata/source-http-2026-10-05.md)
passes all 25 commands: the complete HTTP source group emits, seven artifact-only
consumers execute, and cancellation, client/server status and eleven upload cases pass.
This supersedes the historical storage/network next-step notes below. Callback HTTP
is supported in this development integration; native async emission remains open.
Next reassess full-System source/bootstrap ownership and generic Array/Object dispatch
before expanding the combined library. Full-System compilation, dual-target library
parity and the post-bootstrap benchmark/release gate are not complete.

Native descriptor prerequisites now include closed interface authoring/linking and
source static extensions with generic type tokens; see [focused evidence](experiments/extended-cli-metadata/introspection-prerequisites-2026-10-05.md).
Source descriptor Object overrides also execute across separate native assemblies;
see [slot authoring and dispatch evidence](experiments/extended-cli-metadata/reference-object-overrides-2026-10-05.md).
The unchanged BindingFlags source also compiles and executes through an artifact-only
consumer; [flags evidence](experiments/extended-cli-metadata/flags-enums-2026-10-05.md).
The [production JSON mapping gate](experiments/extended-cli-metadata/source-json-mapping-2026-10-05.md)
now passes: unchanged source library, separate artifact-only consumers, nested objects,
Boolean/string/int properties, arrays, shared mutation and validation before side effects.
The earlier prerequisite notes below describe the path to this checkpoint.

## Cumulative library including Tasks executes (2026-10-05)

The [metadata capacity fix and executable gate](experiments/extended-cli-metadata/cumulative-library-type-budget-2026-10-05.md)
now compile 111 production System sources plus four native adapters into one library.
Both JSON consumers and the existing Tasks/Concurrency consumer execute against that
artifact. The source lookup and 256-type authoring blockers below are resolved. Next
complete storage and DNS/socket service contracts, then remaining core source ownership;
full-System and .NET source-library parity are still open.

## Source primitive lookup resolved (2026-10-05)

The [source-member fix and execution gate](experiments/extended-cli-metadata/source-primitive-members-2026-10-05.md)
now compile the cumulative 109-file library and execute both JSON consumers against
that single artifact. Adding Tasks/Concurrency passes binding and reaches the metadata
writer's 256-type cap; reconcile that bounded authoring/reader limit next, then the
storage/network service families. Full-System source ownership is still incomplete.

## Post-JSON compilation reassessment (2026-10-05)

The [fresh compilation audit](experiments/extended-cli-metadata/system-post-json-assessment-2026-10-05.md)
keeps the 75-source baseline compiling, but isolates a source/import member-lookup
discrepancy: String.SliceUtf8 fails alongside source String and succeeds through its
native artifact. This blocks the cumulative encoding/streams/JSON library. Fix that
first, then complete the storage and DNS/socket service-contract families (7 and 21
missing names respectively). Tasks compile against emitted libraries. The complete
166-file System attempt still fails binding and publishes no assembly; source/bootstrap
core ownership remains a later prerequisite. These findings supersede older frontier
counts and do not reopen the completed separate-library JSON execution gate.

## Active priority: full-System capability batches (2026-10-04)

The author directed a strategic assessment of blockers that unlock the most library
compilation, replacing API-by-API progression. Follow the
[full-System strategy](experiments/extended-cli-metadata/system-compilation-strategy.md)
and [fresh audit](experiments/extended-cli-metadata/system-compilation-audit-2026-10-04.json).
Earlier chronological next-step notes below are historical where they conflict.

Current checkpoint: the Number numeric-family story is complete, including all ten
source implementations and separate generic-library/consumer execution. The cumulative
70-source numeric library compiles in the fresh [post-Number audit](experiments/extended-cli-metadata/system-post-number-audit-2026-10-04.json).
Earlier Number/Self/numeric prerequisite notes below are historical.

The source-order binding defect is now fixed: provisional conversions no longer enter
Raven's cache during declaration binding. Empty-first and reversed 70-source builds
pass; the 76-source combination with Tasks/Concurrency also compiles. Artifact-only
numeric, generic Number, broad application and task consumers execute. Raven 459856a71
and independently validated main f0c3b75a0 each pass 139 focused .NET tests.
[Execution and regression evidence](experiments/extended-cli-metadata/source-order-conversions-2026-10-04.json).

The common text-service prerequisite now executes through native libraries: Unicode
scalar classification, grapheme/vector operations, Unicode casing and UTF-8 slicing.
The expanded seed preserves the numeric gate. Explicit interface properties and pattern
if-expressions pass native emission. The text prerequisite evidence below predates the
completed canonical text ownership gate described next. [Text evidence](experiments/extended-cli-metadata/text-services-native-2026-10-04.json).

See [explicit-property integration](experiments/extended-cli-metadata/explicit-properties-2026-10-04.md).

Source-owned String now executes instance operations and collection-interface dispatch
through a separate native consumer, with an explicit retained seed and ownership manifest.
See [the String integration](experiments/extended-cli-metadata/source-string-2026-10-04.md).
Source-owned Char and String now execute together, including literal/pattern/equality
lowering, interfaces and the real Sequence<char> String constructor. Separate native
consumers pass the existing grapheme, comparison, slice and construction samples.
See [text gate and evidence](experiments/extended-cli-metadata/source-text-2026-10-04.md).
Imported nullability and wider source groups remain open.

The [post-text audit and encoding gate](experiments/extended-cli-metadata/source-encoding-2026-10-04.md)
now prove seven unchanged encoding sources in a separate native library, incremental
UTF-8/ASCII execution, and preserved native text/.NET emission controls. The 75-source
baseline and 81-source task combination compile. Memory streams are already part of
that baseline. The lowered required-result/early-return wrapper is now fixed: five unchanged text-stream
sources compile as another native assembly and execute over source-built MemoryStream.
[Stream gate and evidence](experiments/extended-cli-metadata/source-text-streams-2026-10-04.md).
The [class-hierarchy foundation](experiments/extended-cli-metadata/class-hierarchy-foundation-2026-10-04.md)
now proves runtime inheritance across three binary assemblies and materializes bounded
local native base relationships in reader/introspection views. Definition/builder authoring now preserves local nongeneric bases and direct constructor
calls, including inherited field layout. C# tests execute the generated CLI/native PE.
Raven's ordinary driver now compiles the same local inheritance/mutation consumer for
.NET and neoCLR, and both execute with return 42. The general constructor binding fix
is integrated into Raven main at `4f95db536` with the other validated independent fixes.
The runtime now checks protected constructor family access and closed class direct-family
ownership. Protected constructor authoring and reader/facade round trips also pass,
including generated native PE execution. Closed-family metadata and Raven admission
now pass the [paired direct and separate-library gate](experiments/extended-cli-metadata/closed-family-2026-10-04.md).
Native symbols retain closure/direct children and actual bases; emitters author bounded
reference conversion contracts from symbol facts. External base declarations and
virtual hierarchy contracts remain open. Shared local-assignment and conditional
propagation now unlock the [five-source JSON library and separate native DOM consumer](experiments/extended-cli-metadata/conditional-propagation-json-2026-10-04.md).
The consumer returns 42, including number parsing, duplicate rejection and alias
mutation. General lowering fixes are integrated into Raven main at `e1df355a2`.
The [internal document codec gate](experiments/extended-cli-metadata/source-json-codec-2026-10-04.md)
now also executes Unicode round trips, alias mutation and invalid-input/cycle rejection
through unchanged source bodies. The public serializer/object mapper subsequently gained an
explicit introspection dependency and runtime-service catalog through the
[handle-first dependency sequence](experiments/extended-cli-metadata/introspection-native-next-2026-10-04.md).
The [native typeof boundary](experiments/extended-cli-metadata/native-typeof-2026-10-04.md)
now executes generic and external nominal tokens through a separately compiled test
provider backed by the real TypeName service. That earlier gate did not yet cover production descriptors/member reflection
or public JSON object mapping; the new mapping checkpoint above now does.
The [handle/reflection service gate](experiments/extended-cli-metadata/native-handle-reflection-2026-10-04.md)
also proves identity, generic arguments, object-type lookup and real parameterless
construction across native assemblies. That gate left descriptor-returning service
ownership as its next dependency boundary. The metadata API now
[authors bodyless runtime-service functions](experiments/extended-cli-metadata/internal-call-authoring-2026-10-05.md),
including output-owned result types. Raven now admits explicit internal declarations
and executes the [source-owned handle-service gate](experiments/extended-cli-metadata/source-internal-calls-2026-10-05.md).
The first [source-owned module descriptor factory](experiments/extended-cli-metadata/source-module-descriptors-2026-10-05.md)
also executes with assembly/module-scoped provider resolution. TypeInfo/member factories,
production descriptor compilation and owned snapshot arrays now execute in the JSON gate. The author reaffirmed on 2026-10-05 that reflection
and JSON object mapping are the current proof point before expanding into sockets and
HTTP. Reuse the internal-call/type-ownership foundation there, while validating network
error, resource-lifetime and async contracts separately.
.NET JSON-library parity remains unproven.

The focused JSON DocumentReader/JsonValue source group also requires the standalone
ReflectionError union in its explicit source dependencies; it does not yet require
runtime introspection services.

Next reassess the cumulative source-library inventory after the native JSON mapping gate,
then expand toward the existing socket/HTTP sample with explicit service ownership. Expand remaining service families and
callback/storage/generic support from demonstrated failures. See the strategy's
[current reassessment](experiments/extended-cli-metadata/system-compilation-strategy.md#current-reassessment-after-number-2026-10-04)
for bounded gates and ordering.

The older full 166-source attempt stops in binding; its 141 distinct missing runtime
service members measure the selected bootstrap's coverage, not missing runtime
implementations. The audit adds compilation evidence only. Preserve existing native
execution gates and ordinary .NET controls; full rebuilt-library dual-target parity
remains open. No backend rewrite, format fork, speculative async prerequisite or
performance claim follows from this assessment.


Fixed-offset milestone (2026-10-04): unchanged TimeOffset joins the 48-source --offsets
gate. Separate native consumers execute signed/fractional tick round trips, offset
limits and civil boundaries; expanded native acceptance and paired Duration pass.
Raven bc30fb0f2 admits imported value-type static properties via explicit capability;
17 focused C# controls pass. Next bounded work: explicit time-zone bootstrap services
(database version, lookup, system zone, offset and local mapping). Source inventory
rejects those missing services before output; cascading errors remain unassessed.
Full .NET calendar/offset parity, array-element addresses and legacy snapshot refresh
remain open. No metadata/runtime/bootstrap change in this slice.
[Evidence](experiments/extended-cli-metadata/offset-source-2026-10-04.json).


Clock milestone (2026-10-04): the 47-source --clock gate builds source-owned Instant,
Clock and SystemClock, then executes separate clock/overflow/interface consumers.
Expanded native application/calendar/globalization and paired Duration pass; two runtime
clock tests pass. Explicit UnixTimeTicks core/seed binding is required. Next bounded
library work: fixed offsets, then time-zone service contracts. Full .NET clock/calendar
parity and array-element receiver addresses remain open. Legacy translated snapshot
refresh fails in its union-reference build ('None' not in scope); keep that maintenance
blocker explicit without changing unverified hashes.
[Evidence](experiments/extended-cli-metadata/clock-source-2026-10-04.json).


Globalization milestone (2026-10-04): the unchanged larger sample now executes through
native library imports with explicit --instructions 1000000. Default limits are unchanged.
Expanded --calendar acceptance, Hebrew formatting, broad native application, paired
Duration and six CLI tests pass. Next bounded source-library work: Clock/SystemClock
dependency ownership; then Instant/TimeOffset/time-zone contracts. Their absence in the
current 43-source subset is confirmed by compiler inventory. Full .NET calendar parity
and array-element receiver addresses remain open. No compiler/metadata change.
[Evidence](experiments/extended-cli-metadata/globalization-budget-2026-10-04.json).


Date/calendar milestone (2026-10-04): terminal runtime-call control flow now closes
let-else failure paths (Raven 5c60425db). The --calendar gate builds 43 unchanged sources;
separate Date formatting/arithmetic, generic collections and broad native application
consumers execute. Paired Duration and 60 focused .NET checks pass. Unchanged Hebrew
date formatting also matches expected output. The larger globalization consumer compiles
but reaches the CLI instruction budget; next bounded work is explicit execution-budget
configuration for that native multi-assembly case. Full .NET calendar coverage and
array-element receiver addresses remain open. No metadata/runtime implementation change.
[Evidence](experiments/extended-cli-metadata/calendar-source-2026-10-04.json).


Nested propagation milestone (2026-10-04): eager binary local initializers now lower
in evaluation order (Raven cb0f48bd7; main backport 7db0f3dfe). Native reference-only
Result checks, broad application acceptance and paired Duration pass; focused .NET
tests pass (12 integration, 11 main). Next bounded task: Date.ToString's metadata
verifier rejection, local loaded before store on some path. Date compilation publishes
no artifact and execution remains open. This slice does not generalize arbitrary
expression propagation or change metadata/runtime contracts.
[Evidence](experiments/extended-cli-metadata/nested-propagation-2026-10-04.json).


Discard propagation milestone (2026-10-04): Raven a99152da0 normalizes discarded
propagation in shared lowering and emits portable discard statements. The independently
compiled native Result consumer checks once-only evaluation and early failure returns;
expanded native application acceptance and paired Duration pass. Eighteen focused
.NET checks pass; the general lowering fix is independently isolated at 8e0f88eda
with six tests, then backported to main as 22539952c with the same six checks passing.
Unchanged Date/calendar sources move to an unlowered nested propagation
expression; a minimal `Read()? + 1` reproduces rejection before publication.
Next bounded task: normalize nested expression propagation while preserving evaluation
order and early returns. Array-element receiver addresses and full Date execution remain
open. No metadata/runtime/bootstrap change.
[Evidence](experiments/extended-cli-metadata/discard-propagation-2026-10-04.json).


Grapheme character milestone (2026-10-04): explicit Char signature import, canonical
CLI encoding and native intrinsic owners allow separate-library character calls and
String indexing to execute. Raven `2bd39805c` preserves symbol-only emission for core
Char. Broad native acceptance, paired Duration, seven consumers and 130 C# groups pass.
Unchanged Date sources now pass binding; next bounded task is discarded propagated
values in the portable emitter. Array-element receiver addressing is a separate recorded
gap with a failure-before-publication test. No VM/format change.
[Evidence](experiments/extended-cli-metadata/characters-2026-10-04.json).

Calendar bootstrap services (2026-10-04): explicit core/seed bindings now execute host
culture lookup, local-time conversion and grapheme-count String.Length. The existing
value-array-to-reference-array adapter preserves nominal array ownership; Unicode,
component/fraction/storage and range-fault checks pass alongside native broad application
and paired Duration acceptance. No compiler/runtime implementation change. The unchanged
Date dependency inventory now reports only string-indexing binding errors. Next bounded
work remains the grapheme Char signature/indexer contract across import, emission and
runtime linking; the earlier scalar-indexing wording was incorrect, not a new direction.
[Evidence](experiments/extended-cli-metadata/calendar-services-2026-10-04.json).

Duration foundation (2026-10-04): unchanged Duration, ComparableTo and EquatableTo
compile into separately consumed libraries on both targets. The same value-contract
consumer returns 42 on .NET and NeoCLR; native ArrayList<Duration> and broad application
acceptance also pass. No compiler/metadata/runtime implementation change. The Date
source inventory now identifies binding prerequisites: string indexing, SystemCultureName
and UnixTimeToLocal. Next bounded work is the explicit native string-index/character
contract needed by culture selection/formatting, then the two executable service adapters.
Do not infer later emission/runtime completeness from this binding inventory.
[Evidence](experiments/extended-cli-metadata/duration-foundation-2026-10-04.json).

Value-result receiver milestone (2026-10-04): Raven `9d06edf80` preserves getter/call
copy semantics while native bootstrap imports admit exact Int64 receiver storage.
A separately compiled ArrayList<long> consumer verifies copy/indexer behavior, evaluation
order, signed extrema and mutable struct receiver semantics. Broad native acceptance,
seven consumers, 129 metadata groups and 29 focused C#/.NET checks pass. The unchanged
full generic-collections sample now rejects only for absent Date. Next bounded work is
inventorying Date/globalization source dependencies before selecting a coherent library
expansion; do not substitute a Date stub. Source primitive ownership remains open.
[Evidence](experiments/extended-cli-metadata/value-receivers-2026-10-04.json).

Integer bootstrap acceptance (2026-10-04): unchanged library-integers now executes
with exact output using explicit Int32.Equals/ToString seed contracts. No compiler,
metadata or runtime code change; the expanded native broad gate passes. Source-built
Int32 and boxed virtual dispatch remain open. The next unchanged generic-collections
sample rejects at binding for absent Int64.CompareTo and Date; these are missing
selected-library contracts, not demonstrated .NET regressions.
[Evidence](experiments/extended-cli-metadata/integers-2026-10-04.json).

Full comparer sample execution (2026-10-04): Raven `1fb1bbd45` lowers bound signed
Int32/Int64 ranges behind the explicit native capability and uses argument addresses for
value-parameter receivers. Metadata now exposes LoadArgumentAddress/Ldarga and decodes
CLI Object signatures through the explicit core identity. Unchanged library-comparers
compiles and runs with exact output against the separately built native source library.
129 metadata groups, native address/binding execution checks, 55 focused .NET tests,
seven native consumers and the expanded broad gate pass. No runtime/format change.
Next bounded work is reassessing the remaining unchanged numeric/collection samples
against the expanded library; broader guest introspection and full primitive source
ownership remain open. [Evidence](experiments/extended-cli-metadata/ranges-comparers-2026-10-04.json).

Native source comparer milestone (2026-10-04): unchanged StringComparer.rvn now
compiles into the independently consumed native class library. The explicit comparer
bootstrap supplies primitive String/Int32 members and runtime services; metadata imports
validate primitive receiver modes and the Object.GetHashCode virtual slot. Raven
`7b3928239` admits that bounded slot and bound static binary operators. The focused
consumer verifies ordinal/folded equality and hashing, Unicode scalar ordering, map
replacement and extreme Int32.CompareTo. Expanded broad native gate, seven consumers,
128 metadata groups, dedicated native binding checks and 34 C#/.NET tests pass.
Next bounded blocker: integer-range loop lowering in unchanged library-comparers.
No runtime/format change or full comparer-sample completion is claimed.
[Evidence](experiments/extended-cli-metadata/source-comparers-2026-10-04.json).

Native primitive capture prerequisite (2026-10-04): Raven `1b15715bf` admits immutable
Int32/Int64/Boolean/Byte local captures through the existing closure frames. A separately
compiled FunctionEqualityComparer/HashMap consumer executes with a captured divisor,
escaped callbacks and distinct per-iteration native values. Mixed-width arithmetic now
honors the bound operator's promoted operand types. Broad native acceptance, seven
consumers and 37 focused .NET/C# checks pass. No metadata/runtime or bootstrap change.
Next: explicit StringComparer runtime-service and primitive CompareTo ownership for the
unchanged comparer sample; arbitrary structs/mutable/parameter/receiver captures remain
outside this profile. [Evidence](experiments/extended-cli-metadata/primitive-captures-2026-10-04.json).

Native query acceptance (2026-10-04): unchanged library-query-basics and
library-query-names compile and run with checked-in expected output against the
separately compiled native source library. An instrumented iterator consumer confirms
lazy construction, early/exhaustion disposal, repeated enumeration and idempotent
explicit disposal through imported Filter/Map/Take implementations. The expanded broad
application gate passes without compiler/runtime changes. A seven-sample assessment
records remaining library-surface gaps: numeric/comparer APIs and guest introspection
are absent from this selected library. Next bounded expansion is numeric/comparer source
ownership and primitive contracts; guest introspection remains a separate, larger step.
These are binding diagnostics, not evidence of new .NET regressions or full-library
completion. [Evidence](experiments/extended-cli-metadata/query-acceptance-2026-10-04.json).

Native reference captures (2026-10-03): Raven `623cbc1d8` lowers lambdas capturing
immutable reference locals into fresh private frames using existing instance callback
bindings. Unchanged library-list-filters now compiles, verifies and executes with exact
output against the separately built native class library. Escaped callbacks, independent
factory instances and shared object mutation pass; mutable bindings reject before output.
The broad application, seven native consumers and 31 focused .NET tests pass. No runtime,
metadata format or bootstrap configuration change. Mutable/value/parameter/receiver
captures remain unsupported; general lexical shared storage and full-library coverage
remain open. [Evidence](experiments/extended-cli-metadata/reference-captures-2026-10-03.json).

Receiver-bound callbacks (2026-10-03): Raven `a53b6412a` and the metadata writer now
support owned nongeneric nonvirtual reference-instance callback targets. The runtime's
existing instance binding preserves shared mutable receiver identity, confirmed by CLR
and NeoCLR execution and a source method-group consumer of separately built ArrayList.Find.
All 128 metadata groups, 31 focused .NET tests, seven native consumers and the expanded
broad source-library gate pass. This is a captured-lambda prerequisite, not completion:
next work remains closure-frame lowering for unchanged library-list-filters, including
shared mutable captures and correct lexical lifetimes. No runtime/format-version change.
[Evidence](experiments/extended-cli-metadata/instance-callbacks-2026-10-03.json).

Native array callbacks (2026-10-03): Raven `35464aabf` preserves imported no-result
callback contracts separately from inhabited source unit and emits receivers through the
configured nominal array backing. The metadata library admits nested vector signatures
using existing CLI SZARRAY/native ArrayRef encodings; the runtime accepts exact nominal
backing casts without copying storage. Unchanged library-array-callbacks and a nonempty
nested-array mutation consumer now pass alongside the broad source-library application.
Validation: 127 C# metadata groups, 27 focused .NET tests, 19 runtime tests and seven native
consumers pass. Next bounded native gap: captured callbacks in library-list-filters.
[Evidence](experiments/extended-cli-metadata/native-array-callbacks-2026-10-03.json).

Native end-to-end sample expansion (2026-10-03): Raven `3367f3200` fixes vector
receivers for projected inherited interface accessors, exposed by Array<int>.Count in
unchanged collection-capabilities. Existing reference conversion uses semantic symbols
and target capability checks; nominal Array<T>, metadata and runtime remain unchanged.
The source-library driver gate now includes a Count/mutation regression and three
unchanged samples (Option, propagation, collection capabilities), all verified/executed
with exact output/status alongside the broad order-collections application. Seven native
consumers and 13 focused .NET tests pass. Next bounded native task: no-result callback
import/emission for unchanged library-array-callbacks. Captured functions and missing
numeric/date source coverage remain separate gaps; full-library completion is not claimed.
[Evidence](experiments/extended-cli-metadata/native-array-receiver-2026-10-03.json).

Native catalog validation (2026-10-03): Raven `42503ec23` converts conflicts between
native snapshots and the explicit primitive bootstrap into RAVT003, including failures
while constructing the introspection catalog. A C# regression reproduces the previous
uncaught exception and now verifies both reference orders and unchanged output bytes/
position on failed emission. Seven native consumers pass. No Runtime Contract, format,
runtime or ordinary .NET behavior changes; prior broad-application evidence is reused.
[Evidence](experiments/extended-cli-metadata/native-catalog-validation-2026-10-03.json).

Native type materialization cleanup (2026-10-03): Raven `777499170` constructs ordinary
type and union/case/companion symbols from canonical introspection views and removes
obsolete raw-signature mapping helpers. The separately compiled source-union library
and unchanged application-order-collections pass, as do all seven native consumers.
Runtime Contract/bootstrap configuration, metadata format and ordinary .NET paths are
unchanged. The native gate is evidence for this cleanup, not full dual-target completion.
[Evidence](experiments/extended-cli-metadata/native-type-materialization-2026-10-03.json).

Native type facts (2026-10-03): Raven `003b9a38d` consumes declaration and generic
parameter facts through the retained type view. The host facade now supplies declared
parameter Name with explicit CLI limitations. All 127 metadata groups and seven native
consumers pass; ordinary .NET is unchanged. Remaining definition use is concentrated in
root declaration materialization and union transport, requiring separate review.
[Evidence](experiments/extended-cli-metadata/native-type-facade-2026-10-03.json).

Native callable cleanup (2026-10-03): Raven `015e6f66d` constructs method, constructor
and module-function symbols from introspection views, preserving declaration order and
canonical member identity. All seven native consumers pass. Remaining definition use
is in type/union construction and explicit legacy bindings, not callable-symbol creation.
[Evidence](experiments/extended-cli-metadata/native-callable-facade-2026-10-03.json).

Native importer cleanup (2026-10-03): Raven `12b545bc1` consumes field/property
introspection views directly and reuses canonical accessor symbols. Seven native consumers
pass before/after. The remaining legacy references have explicit seed/probe callers;
none were removed speculatively. Continue reviewing remaining declaration construction
against existing facade facts, preserving ordinary .NET behavior.
[Evidence](experiments/extended-cli-metadata/native-member-facade-2026-10-03.json).

Array-lowering boundary (2026-10-03): Raven `19a3e84c0` confines vector expansion to
portable body planning; ordinary .NET retains its established loop emitter. The 86
focused checks, native broad application and native labeled-loop case pass. Loop capture
now matches main's known failure rather than acquiring a different integration result;
correct lexical closure lifetimes remain independent work. Next target cleanup: inventory
native-versus-legacy reference consumers before removing any bridge branch.
[Evidence](experiments/extended-cli-metadata/portable-array-boundary-2026-10-03.json).

Compiler simplification implementation (2026-10-03): Raven `c2a66d82a` removes the
second production .NET body emitter while retaining the native planner and metadata
library. All 94 selected checks pass before/after, and the native broad application
executes against its separate library. Constraint expectation reconciliation is committed
separately; closure lifetime/shared array lowering remains the next unresolved compiler
audit. Further importer and legacy-bridge cleanup is still pending, not completed.
[Evidence](experiments/extended-cli-metadata/dotnet-emitter-simplification-2026-10-03.json).

Target-boundary audit (2026-10-03): retain the NeoCLR metadata library and native
symbol/backend boundaries. Raven audit commit `10f5f0089` records 71 further passing
.NET checks on both lines and one shared constraint round-trip failure. Reconcile that
baseline discrepancy and evaluate simplifying the second production .NET body path
before expanding adapters. These are bounded review recommendations; no backend has
been removed or replaced. See the [conversation record](development-timeline.md).

Direction reassessment (2026-10-03, author-directed): pause new adapter/compiler work
and re-establish ordinary Raven/.NET behavior against main before selecting another
implementation slice. The proposed CLR array adapter below is suspended. Current main
and integration both pass 36 focused array/iteration checks and an ordinary .NET
array/IList mutation/LINQ executable. The failing bootstrap configuration asserts custom
NeoCLR interfaces for CLR vectors; it is not evidence that ordinary .NET arrays regressed.
Keep native acceptance, ordinary .NET regression checks and cross-runtime source-library
compatibility evidence distinct. This reassessment does not claim the earlier paired
source-library gate complete or permanently remove it from scope.
[Audit](experiments/extended-cli-metadata/dotnet-main-audit-2026-10-03.json).

Paired .NET unit milestone (2026-10-03): explicit source-void/unit mapping removes invalid
CLR generic storage. Separate union, ArrayList, HashMap and query consumers run successfully
on .NET; 22 unit-contract tests pass on both Raven branches. Native broad execution remains
passing. The .NET broad consumer now compiles and executes its expected prefix before
custom array-interface execution terminates abnormally. Next bounded task: an explicit
.NET array adapter conversion, with unsupported configurations rejected before emission.
[Evidence](experiments/extended-cli-metadata/dotnet-unit-execution-2026-10-03.json).

Paired .NET assessment (2026-10-03): executable storage/failure adapters pass. The shared
safe override-return strengthening fix is independently validated on Raven's main-based
fix branch. Native broad execution remains green. The unchanged .NET source library emits,
but consumer compilation fails loading Propagatable instantiated with CLR System.Void.
Next bounded task: explicit source-unit/CLR-value projection and rejection of invalid void
storage before publishing an assembly. Then prove .NET array/interface adaptation and the
unchanged broad consumer. [Evidence](experiments/extended-cli-metadata/dotnet-source-assessment-2026-10-03.json).

Broad native gate update (2026-10-03): unchanged application-order-collections compiles
against the separately emitted source library and executes with exact checked-in output
and exit 0, including collection mutation, callback/query composition, union propagation,
array iteration and shared object identity. Reproduce with the bootstrap acceptance script
`--application`; [evidence](experiments/extended-cli-metadata/broad-native-execution-2026-10-03.json).
Next bounded gate: executable .NET source-library adapters and the paired unchanged sample;
full-runtime compilation and broader native categories remain subsequent work.

Nominal-array execution update (2026-10-03): the separate native consumer now mutates
vector storage through MutableSequence<int>, observes the same storage through its array
reference, and executes Filter/ToList through source Array<T>/ArrayIterator<T> (42).
[Evidence](experiments/extended-cli-metadata/nominal-array-execution-2026-10-03.json).
Next: resume unchanged application-order-collections with this ownership manifest and
close its runtime-service binding gaps. Full dual-target library/application execution
remains open.

Author clarification (2026-10-03): retain nominal Array<T> backing and defer structural
array decisions. Source Array/iterator and separate consumer compilation succeed; the
next bounded task is identity-aware vector-to-nominal interface backing/runtime dispatch.
[Assessment](experiments/extended-cli-metadata/nominal-array-assessment-2026-10-03.json).

Query gate update (2026-10-03): unchanged complete query sources now compile and import
into an executing native consumer (OfType/Filter/Map/ToList/Single, boxed values and shared
reference identity). [Evidence](experiments/extended-cli-metadata/query-import-2026-10-03.json).
The unchanged broad application next fails at array extension lookup (Order[].Filter);
configured iteration identity/conversion is the next bounded task. The full dual-target gate remains open.

Query integration update (2026-10-03): native extension declaration/import now executes
a separate generic extension library and consumer. The unchanged full query library
advances to OfType object-to-generic conversion, which still rejects before publication.
[Assessment](experiments/extended-cli-metadata/query-source-assessment-2026-10-03.json).
The broad application and full dual-target source-library gate remain open.

Source-library gate update (2026-10-03): unchanged HashMap and comparer sources now
compile into a separately imported native library and execute collisions, growth,
replacement, callback dispatch and shared object mutation. No backend change was needed.
[Evidence](experiments/extended-cli-metadata/hashmap-import-2026-10-03.json).
Query composition, the broad application and executable .NET class-library adapters remain open.

Author priority clarification (2026-10-03): finish Raven targeting neoCLR and executable
end-to-end compilation. CLI tables, signatures and instructions remain the baseline;
new semantics and broader format redesign are subsequent experiments, not prerequisites
for the current gate. Make only metadata changes required by demonstrated integration
failures. The current PE/#Neo execution transport remains transitional; retaining it for
this gate does not establish ordinary CLI metadata or IL as authoritative at runtime.
See [scope clarification](design/dual-target-compilation.md#integration-priority-clarification-2026-10-03).

Union continuation (2026-10-03): bounded type custom-attribute authoring, native/CLI
round trips and catalog-based introspection now provide the metadata foundation.
Raven union publication and imported semantic reconstruction remain pending; see the
[foundation scope](design/dual-target-compilation.md#union-attribute-foundation-2026-10-03).

Author-approved implementation gate (2026-10-03): [dual-target driver and source-library plan](design/dual-target-compilation.md).
Paired ordinary-driver Hello and separate generic library/consumer cases now execute on
both targets; native references load directly with explicit --core-reference. Next the broad
collections sample, retaining only the explicit primitive bootstrap/runtime seed.

Author clarification (2026-10-02): importer and emitter must communicate through Raven
symbols, without emission reusing loader objects. Raven compiler contracts and the
metadata library body-generator API are independent boundaries. The first bounded
slice now emits primitive/method-generic/vector assembly-level-function references from
symbols and artifact values (seven consumers execute; 107 metadata C# groups pass).
Public top-level root-class identities, including unconstrained generics, now also
author directly from symbols; other type profiles retain their existing conversion
metadata path. Namespace-function signatures now also carry root-class constructions and vectors
through symbol-only reference authoring. Public nonvirtual root-class methods and constructors now also reconstruct from
symbols, including owner generic parameters. Public nongeneric root-class fields now also author from symbol storage and explicit
layout ordinals. Nongeneric interface identities, direct relationships and abstract dispatch now also
author from symbols. The library now exposes its own IILGenerator, consumed only behind Raven’s NeoCLR
adapter. The audit identifies translated CLI bindings, unsupported profiles, host setup
and lazy symbol materialization as remaining reader dependencies. The generator now also owns body-authoring internals; builder methods forward while
writer-side validation remains unchanged. Static native containers now also reconstruct as declaration owners from symbols,
with value signatures still excluding static classes. The audit found native concrete implementations do not carry CLI-projection final/virtual
flags. Native callable reader fallback is now removed; incomplete contracts diagnose.
Native type/field fallbacks and the emitter resolver are now removed too; all seven
consumers execute. Native host bindings now use the registered compiler reference's
captured identity/digest without a separately supplied reader definition. Legacy CLI
bindings remain explicit. Reader-backed semantic materialization remains supported.
The generic consumer assessment exposed closed generic field storage rejection. That
bounded gap is now fixed: a separate BoxStorage library holds Box<int> and Box<int>[];
all seven consumers execute (42), and 108 metadata groups plus CLR/both-container
field execution pass. Fields on generic declaring owners and generic interface imports now also pass:
reader relationships retain owner arguments, emitted references are symbol-authored,
and generic inherited dispatch executes for primitive and nominal payloads. All seven
Raven consumers and 108 metadata groups pass, with .NET and both-container execution.
See [generic field/interface evidence](experiments/extended-cli-metadata/generic-fields-interfaces-2026-10-02.json).
Author follow-up proposes a pure metadata resolution/view layer to unify local and
external inspection. The initial C# MetadataLoadContext and assembly/module/nominal facade are implemented;
Raven uses them instead of its private dependency resolver. 109 metadata test groups pass;
[all seven consumers execute](experiments/extended-cli-metadata/introspection-context-2026-10-02.json).
Constructed type and field views now project signatures for Raven; canonical view-to-symbol
mapping preserves array identity. Method/parameter views now also project open method scopes; Raven no longer performs
recursive generic-signature projection. Declared property/direct interface views now also
substitute owner arguments for Raven (2026-10-03); [109 C# groups and seven runtime consumers pass](experiments/extended-cli-metadata/introspection-properties-interfaces-2026-10-03.json). Bounded transitive interface traversal now also lives in the facade. Generic method inspection now also supports canonical constructions on open/closed owners.
Constructor/member classification and accessibility metadata now live in the facade;
paired driver execution passes. External generic interface implementation/inheritance now
works across separately compiled contracts, implementation and consumer assemblies,
including diamond traversal and dispatch on both targets. See [driver evidence](experiments/extended-cli-metadata/dual-driver-external-2026-10-03.json).
A checked-in ownership manifest and primitive-only bootstrap now support separately
compiled iteration/collection contracts on both targets. Next extend ownership to the
retained seed and ArrayList dependencies; also implement the author-requested native Self
signature path through the metadata APIs. Ordinary native driver opt-in now enables the
existing checked-storage intrinsic, with separately compiled generic helper consumers
executing 42 and rejecting uninitialized reads. The unchanged Option/Propagatable native
interface out-parameter blocker is now resolved: metadata views preserve modes and
three-assembly ref/out dispatch executes on both targets. Unchanged Propagatable emits;
Option now reaches source-union/declaration emission rejection. The metadata foundation
now admits inline generic value payloads and direct native value snapshots; tag/payload
copy execution and imported constructors return 42 on CLR and neoCLR. Raven imports
these as structs. Ordinary top-level source structs now compile and execute on both targets with generic
inline payloads, default constructors and independent copies. Native emission selects an
explicit value declaration capability. Native nested class/value snapshots and Raven symbol ownership now also preserve
same-named case scopes; direct nested constructor imports execute on both runtimes.
See [nested case evidence](experiments/extended-cli-metadata/nested-cases-2026-10-03.json).
Ordinary nested source class/value declaration emission now also executes on both
targets, with scoped payload identity and copy checks; see
[nested source evidence](experiments/extended-cli-metadata/source-nested-2026-10-03.json).
Byte discriminator storage, conversions and signatures now execute on both targets;
see [byte evidence](experiments/extended-cli-metadata/byte-discriminator-2026-10-03.json).
Generated union declaration discovery now retains complete case/member contracts and
physical generic case owners. Native preflight reaches synthesized ToString overrides;
unchanged Option now passes value-type interface admission and reaches the same override
blocker. Metadata-generated constrained interface dispatch already executes with mutation
and independent copies on both runtimes; compiler declarations/concrete calls also pass.
See [value-interface evidence](experiments/extended-cli-metadata/value-interfaces-2026-10-03.json). Generated case constructors and payload getters now also pass shared body planning,
including generic payloads; see [core-body evidence](experiments/extended-cli-metadata/union-core-bodies-2026-10-03.json).
Bounded value ToString override declarations now preserve and execute the Object slot on CLR;
native writing now validates the explicit retained System slot, retains override names/flags,
and executes imported direct calls plus boxed ordinary/generic Object dispatch. See
[native override evidence](experiments/extended-cli-metadata/native-value-overrides-2026-10-03.json). See
[override declaration evidence](experiments/extended-cli-metadata/value-override-authoring-2026-10-03.json).
Raven now admits the bounded override and executes local ordinary/generic value calls through
explicit --runtime-seed binding. Source unions and unchanged Option reach generated ToString
conversion lowering; [driver evidence](experiments/extended-cli-metadata/raven-value-overrides-2026-10-03.json)
records the existing CLR return-nullability mismatch separately.
Union/case metadata, local synthesized display and separately compiled plain/generic
union matching now execute on both targets. Native imports retain parameter names and
physical case scopes; external value operands are authored from symbols. The unchanged Option/Result and iteration sources now build into a separate native
library and execute in a reference-only consumer with the bounded executable seed.
See [source union evidence](experiments/extended-cli-metadata/source-unions-2026-10-03.json).
The .NET adapter and broader collection/application gates remain open. See
[separate union evidence](experiments/extended-cli-metadata/union-separate-execution-2026-10-03.json). See
[union declaration evidence](experiments/extended-cli-metadata/union-declarations-2026-10-03.json). See
[union payload evidence](experiments/extended-cli-metadata/union-payloads-2026-10-03.json) and
[source value driver evidence](experiments/extended-cli-metadata/source-values-2026-10-03.json). ArrayList now has explicit Fail/callback bootstrap bindings and passes source-included
execution with alias mutation, independent copies, iteration and Find. Separate library
import and execution now also pass after function-signature materialization and metadata-only
callback views. See [separate ArrayList evidence](experiments/extended-cli-metadata/arraylist-import-2026-10-03.json). See the dual-target tracker for evidence. The first Self contract-only slice now preserves Self
through native readers/writers and scoped introspection; Raven and typed dispatch authoring
remain pending. Configured unit generic union payloads now bind and execute (42); ordinary CLI void
remains rejected without the unit contract. The broad source-library/application gate remains open.
See [closed generic field evidence](experiments/extended-cli-metadata/closed-generic-fields-2026-10-02.json).
See [native host binding evidence](experiments/extended-cli-metadata/native-host-bindings-2026-10-02.json).
See [native reference boundary evidence](experiments/extended-cli-metadata/no-native-reference-fallback-2026-10-02.json). See [callable boundary evidence](experiments/extended-cli-metadata/no-native-callable-fallback-2026-10-02.json). See [static-owner evidence](experiments/extended-cli-metadata/symbol-only-static-owners-2026-10-02.json).
See [body-engine evidence](experiments/extended-cli-metadata/body-engine-2026-10-02.json). See [generator evidence](experiments/extended-cli-metadata/body-generator-2026-10-02.json). See [interface evidence](experiments/extended-cli-metadata/symbol-only-interfaces-2026-10-02.json). See [field evidence](experiments/extended-cli-metadata/symbol-only-fields-2026-10-02.json). See [member evidence](experiments/extended-cli-metadata/symbol-only-members-2026-10-02.json).
See [nominal call evidence](experiments/extended-cli-metadata/symbol-only-nominal-calls-2026-10-02.json). See [type-reference evidence](experiments/extended-cli-metadata/symbol-only-types-2026-10-02.json). See [validation and remaining scope](experiments/extended-cli-metadata/symbol-only-functions-2026-10-02.json). See [direction and current gaps](design/extended-cli-metadata.md#independent-compiler-and-library-generation-boundaries-2026-10-02).

Author-directed priority (2026-10-02): proceed with direct native metadata import into
Raven, developing the reader/writer library toward its existing builders → definitions
→ metadata → PE architecture. Primitive assembly-level functions now materialize directly into the existing definitions
(103 metadata groups pass). Raven now binds these native functions directly with
exact dependency identity and compilation-owned symbols; 67 .NET regressions pass.
Native calls now import and execute across that boundary, including a Raven-produced
library read directly and consumed by Raven (42). Fieldless nongeneric static classes
and primitive static methods now also load directly into Raven; Boolean/Int32 overload
calls execute in neoCLR (42). Fieldless instance classes now also support direct
constructor/member import, allocation, local aliases and calls (42). Primitive fields
now load into definitions/symbols, and a stateful class consumer returns 42 through its
constructor and instance method. Direct public primitive imported field loads/stores
now execute too, including alias writes and direct constructor/field-read consumers (42).
Local nominal class parameter/result signatures now also load directly, including
factories, namespace/static/instance identity functions and nominal constructor arguments;
the Raven consumer executes (42). Local nominal fields now also load and emit directly; a Raven consumer replaces an
object field and mutates the replacement without changing the original (42).
Cross-dependency class signatures now resolve through explicit exact-identity metadata
resolvers; Raven-produced payload/holder libraries and their consumer execute (42).
All five native consumers pass. Primitive and nominal class arrays now also load directly
in method/constructor/field signatures, including external element types; cross-library
array aliases and element replacement execute (42). Non-indexed properties now also load with canonical accessors and execute across native
libraries (42), including static getters and nominal/vector setters. Private setter and
read-only writes diagnose. Indexed properties now also load directly, including overloads; cross-library indexed
reads/writes execute (42). Nongeneric interfaces and local inheritance/implementations now also load; inherited
method/property dispatch executes across native assemblies (42). All six consumers pass.
Interface-valued fields, constructor parameters and arrays now also work across native
libraries, preserving aliases and dispatch after replacement (42). The early-return diagnostic gap is now independently reproduced and fixed on .NET
(106 focused tests); the native negative case diagnoses before emission and all six
consumers still execute (42). The metadata reader/import API now preserves static generic methods/functions,
including parameter vectors; CLR and native execution pass (42, both containers).
Raven now imports those generic symbols through shared inference and constructed-method
substitution; all seven native consumers execute (42), including generic forwarding and
array aliases. Unconstrained native generic root classes now also import with owner-scoped
parameters and shared constructed-type/member substitution. Box<int>/Box<Item> construction,
methods, properties and vector aliases execute (42); all seven consumers and 105 metadata
groups pass. Local closed generic signatures now also retain immutable definition/argument
identity; Raven consumes Box<int> factory/identity calls (42). Scoped local constructions
Box<T> and their vectors now also import and execute with preserved method/owner parameter
identity; all seven consumers still return 42. External constructions now also resolve
through exact dependencies; a three-assembly generic consumer executes (42), with 106 C#
metadata groups passing. Qualified assembly-level-function lookup now includes native ownerless
methods; inferred/explicit calls execute, invalid explicit arguments diagnose and .NET
controls pass. See [qualified lookup evidence](experiments/extended-cli-metadata/native-qualified-functions-2026-10-02.json).
Constraints remain pending before full System loading. See [external generic evidence](experiments/extended-cli-metadata/native-external-generics-2026-10-02.json). See [open signature evidence](experiments/extended-cli-metadata/native-open-signatures-2026-10-02.json). See [closed signature evidence](experiments/extended-cli-metadata/native-closed-signatures-2026-10-02.json). See [generic owner evidence](experiments/extended-cli-metadata/native-generic-owners-2026-10-02.json).
See [generic import evidence](experiments/extended-cli-metadata/native-generic-symbols-2026-10-02.json). See [return diagnostic evidence](experiments/extended-cli-metadata/native-return-diagnostics-2026-10-02.json). See [storage evidence](experiments/extended-cli-metadata/native-interface-storage-2026-10-02.json).
See [interface evidence](experiments/extended-cli-metadata/native-interfaces-2026-10-02.json). Setter-only indexed assignments now also compile and execute (42);
source reads still require a getter. See [setter-only evidence](experiments/extended-cli-metadata/native-writeonly-indexers-2026-10-02.json). See [indexer evidence](experiments/extended-cli-metadata/native-indexers-2026-10-02.json). See [property evidence](experiments/extended-cli-metadata/native-properties-2026-10-02.json). See [array evidence](experiments/extended-cli-metadata/native-array-signatures-2026-10-02.json). See [external signature evidence](experiments/extended-cli-metadata/native-external-signatures-2026-10-02.json). See [nominal field evidence](experiments/extended-cli-metadata/native-nominal-fields-2026-10-02.json) and [nominal signature evidence](experiments/extended-cli-metadata/native-nominal-signatures-2026-10-02.json).
Native reading/import now precede further source-library/bootstrap expansion. CLI projections remain existing
controls, not the new integration route. See the [implementation alignment](design/extended-cli-metadata.md#direct-native-semantic-import-implementation-alignment-2026-10-02).

Development checkpoint (2026-10-02): the metadata feature branch integrates the
structural Function runtime and direct Raven Function emission. The unchanged collections
application now compiles to native PE/#Neo, verifies and executes with exact expected
output against the explicitly bound translated System library. All 95 metadata contract
groups pass. Unchanged ArrayList plus its source interfaces now compile to native PE
and execute growth, copy independence, iteration, callback searches and Option results,
with expected negative-capacity/index faults. Explicit assembly-level-function dependency
binding and configured array-shape length support complete this bounded source slice.
Translated System remains required; full library source emission and native semantic
symbol loading remain open. Unchanged callback comparer implementations now also execute
through Function fields and interface dispatch (42). HashMap and its source dependencies
now also compile and execute collision/growth,
update, key-snapshot, inherited-interface and Option lookup checks (42). Reference-payload consumers now also verify shared object identity and replacement
independence (42). The broad application with source collections exposes a mixed
source/seed iteration identity boundary, still present when query operators are included
as source. Next establish a coherent source bootstrap core/iteration contract before
extending query emission; closure environments and remaining System units remain open.
Author-directed integration stays on the feature branches;
no main-branch feature merge or roadmap milestone completion is implied.
See [bundle-hashed evidence](experiments/extended-cli-metadata/collections-end-to-end-2026-10-02.json)
and [integration contracts](raven-cli-bridge.md#unchanged-collections-end-to-end-acceptance-2026-10-02-development).

**Updated 2026-10-02.** This is the authoritative default for work priorities,
milestone sequencing and scope. Explicit author directions take precedence.

## Current work

**Imported constructor checkpoint (2026-10-02):** value constructors and imported
class/value constructors preserve CLI/native identity and construction safety. Raven
constructs imported generic values and executes against a separate native library (42).
All 84 metadata groups, 29 focused compiler tests and five native controls pass. The
unchanged collections sample still rejects Option<Order>(None) because None is a nested
CLI type. Next implement nested definitions/references and exact declaring-type identity
in the metadata/import contracts, then return to native System dependencies.

**Propagation guard checkpoint (2026-10-02):** the metadata API emits literal terminal
failure using the existing native fault instruction, with C# success/failure execution
on CLR and neoCLR. Raven explicitly marks generated invalid-carrier guards; its .NET
adapter retains null-throw behavior, while native emission terminates with a diagnostic.
All 83 metadata checks and 14 focused Raven tests pass. The unchanged collections sample
now passes the guard and reaches imported carrier construction from None; CLI control
still emits 7168 bytes. Next implement imported value/union carrier constructors and
continue into native System dependencies; no full application execution is claimed.
[Sample evidence](experiments/extended-cli-metadata/collections-after-terminal-guard-2026-10-02.json).

**Author-directed naming correction (2026-10-02):** the terminal namespace action is
`System.Fail(message)`; `Fault` remains the result. Update the matching compiler,
reference and library bundle together. This correction precedes, and does not replace,
the next propagation terminal-failure emission slice. See [the contract](system-fault.md).

**Raven value-receiver checkpoint (2026-10-02):** imported value methods now consume
managed local/ref receiver addresses through explicit shared emission capabilities.
A separate native-library Raven consumer verifies and executes mutation and generic out
calls (42). The unchanged collections sample advances to a lowered throw guard; CLI
control still emits 7168 bytes. Next define terminal failure handling for compiler-generated
invalid propagation carriers before following remaining union/System dependencies.
[Native control](experiments/extended-cli-metadata/raven-value-receiver-validation-2026-10-02.json),
[collections](experiments/extended-cli-metadata/collections-after-value-receivers-2026-10-02.json).

**Value-receiver metadata checkpoint (2026-10-02):** owned/authored and imported value
instance methods now preserve initialized managed receivers through CLI/native emission
and reference projection. Separate generic value/out consumers execute on both runtimes
(42); value constructors and constrained dispatch remain rejected. Next connect Raven's
imported value-receiver admission and address emission to this validated contract, then
rerun the unchanged collections sample.

**Raven ref/out checkpoint (2026-10-02):** shared .NET/neoCLR emission now preserves
ref/out signatures, local addresses, indirect access and uninitialized output locals.
Five native controls pass, including source forwarding/mutation (42); 64 focused C#
tests pass. The unchanged collections sample now rejects imported value-receiver
`TryGetOutput(out Order)` invocation admission, with CLI control still 7168 bytes.
Next implement exact imported value-receiver/member contracts, then follow the same
sample through propagation failure/Unit and native System identity requirements.
[Controls](experiments/extended-cli-metadata/raven-ref-out-validation-2026-10-02.json),
[collections](experiments/extended-cli-metadata/collections-after-raven-ref-out-2026-10-02.json).

**Sample reassessment (2026-10-02):** four small native execution controls pass; the
31-case inventory has four library emit/verify successes, eighteen emission rejections
and nine binding failures. All twelve selected applications emit their CLI controls but
none reaches native execution. Next connect ref/out locals and calls through Raven's
portable emission contract, then follow the unchanged collections sample into imported
value receivers and exact System dependencies. Whole-library compilation requires an
implementation seed; its 700 binding diagnostics are not 700 independent codegen gaps.
[Assessment and evidence](experiments/extended-cli-metadata/readiness-assessment-2026-10-02.md).

**Out-call checkpoint (2026-10-02):** explicit output contracts now map CLI Param Out
flags to existing native out_parameters metadata. Generic forwarding, imported calls
and native CLI projection preserve them; producer validation requires assignment on
every normal return. Metadata library/consumer binaries execute on CLR and neoCLR (42).
Raven admission and imported value receivers remain the next integration work; the
unchanged collections application has not advanced yet.
[Contract and validation](design/extended-cli-metadata.md#output-parameter-contracts-2026-10-02).

**Byref-call checkpoint (2026-10-02):** writable managed-reference parameters now
survive CLI/native emission, generic substitution, imports and CLI projection. An
initialized local can be passed and mutated through a separate assembly; metadata C#
checks execute on CLR and native verify/run returns 42. Out assignment contracts and
imported value receivers remain the next propagation gaps; Raven admission and the
unchanged collections sample are not yet advanced by this metadata-only slice.
[Contract and checks](design/extended-cli-metadata.md#writable-ref-parameters-2026-10-02).

**Managed-local checkpoint (2026-10-02):** metadata builders now emit typed ldobj/stobj
for owned local addresses, including generic copies, with definite-assignment checks.
The same C# producer executes on CLR and loads/verifies/runs as a native binary (42).
This is the first bounded part of propagation support; byref parameter signatures,
out-call contracts and imported value receivers remain open. The unchanged collections
sample is still blocked at native out-local admission. See
[the contract and validation](design/extended-cli-metadata.md#typed-local-object-operations-2026-10-02).

**Shared-lowering checkpoint (2026-10-02):** fix the concrete-case construction
exception masked by semantic-model fallback. The apparent propagation rejection is
now resolved at shared lowering; unchanged collections native emission reaches
synthesized uninitialized out-local admission. The CLI control still emits 7168 bytes.
The general Raven fix is isolated as `dcc77ef5f` on the main-based compiler-fixes branch,
with 25/25 focused tests passing on both compiler lines. Next build explicit managed
local-address/byref-signature/value-receiver support for propagation, retaining CLR/CIL
semantics and capability checks. Union/protocol execution and native System identity
mapping are not complete. [Evidence](experiments/extended-cli-metadata/collections-after-shared-union-lowering.json).

**Imported-member checkpoint (2026-10-02):** imported values retain their signature
category, and constructed interface/final virtual class calls now use explicit imported
member contracts. Separate metadata and Raven consumers execute on neoCLR (42); the
metadata case also executes on CLR. The unchanged collections sample now passes
TryAdd/Add admission and stops at `BoundPropagateExpression` (`?` in PendingOrder).
Next investigate propagation/union lowering, retaining the original sample and explicit
dependency contracts. This is not proof that the full collections application or native
System identity mapping is complete.
[Dispatch evidence](experiments/extended-cli-metadata/imported-interface-validation.json),
[unchanged sample](experiments/extended-cli-metadata/collections-after-interface-dispatch.json).

**Active author-directed priority (2026-10-01):** the author considers the metadata API
architecture good enough for now. Resume the unchanged Raven/native end-to-end case;
defer broad API migration, naming cleanup and encoding/PE extraction unless a concrete
integration blocker requires them. This supersedes the earlier instruction to prioritize
full definition unification. Preserve the recorded architecture as direction, not a gate.

Historical validation against metadata `e13c8634` and Raven `80edf8fbe` confirms the unchanged
collections sample binds and emits its CLI control (7168 bytes), but native emission
still rejects PendingOrder's Option<Order> signature. The immediate work is preserving
imported value categories through admission, signatures, native dependency loading and
projection, then advancing the original sample to its next observed blocker.
[Fresh evidence](experiments/extended-cli-metadata/collections-after-definition-migration.json).
Do not widen reference-type admission to make value unions pass. Remaining instance/
generic-owner imports, union operations and translated-System identities must be
validated incrementally; their completion is not claimed by the current probe.

**Layering (author clarification, 2026-10-01):** builders → definitions → metadata → PE,
with readers reversing the encoding/container boundaries. Current combined encoding/PE
packaging and immutable reader snapshots still need separation/materialization. This is
an architectural target, not a new binary format. Direct generic types now share names
and constraint storage with builders; constrained calls execute on both runtimes (42).

**Builder direction (author clarification, 2026-10-01):** retain Cecil-like definitions,
with generation builders inspired by Reflection.Emit, without replacement/drop-in claims.
Both layers share one graph. Property declarations/accessor associations now participate
in that graph; existing Add-style builder APIs remain available.

**Definition migration checkpoint (2026-10-01):** authored assembly/type/field construction
now shares declaration objects with builder facades and retains existing CLI/native
encoding. Manual struct execution returns 42 on both runtimes; Raven cross-assembly
probes remain green. Canonical method declarations and authored entry/function views now follow; direct
full instruction editing, remaining generic migration and reader materialization
remain open. Method definitions now own instruction/local/label storage, with existing
helpers using that same body and cached read-only local/label views. Interface relationships
now have append-only authored definitions; inherited dispatch executes on both runtimes.
Direct nongeneric interface and abstract-method declarations now dispatch on CLR/neoCLR
(42), while relationship registration still uses builder helpers.
Direct root-class constructors and instance methods now execute readonly initialization
and object calls on CLR and neoCLR (42).
Direct static type methods now attach through TypeDefinition.Methods and execute through
the manual assembly-function call chain.
Direct assembly-function construction and helper-call execution are now covered as well. The author reaffirmed CLI/CIL as the baseline to extend or modify, and continued
authorization for end-to-end work. [Contract](../api-docs/experimental-metadata.md#authored-definitions-first-migration-slice).

**Author-directed architecture correction (2026-10-01):** definitions must be the
canonical editable assembly graph, usable directly; builders must be optional facades
that mutate the same definitions, and writers must consume definitions. The current
immutable-snapshot/separate-builder graph does not meet this requirement. After finishing
the in-progress value payload/address slice (633a4a4a), prioritize this refactor before
more builder-only capability expansion. The first acceptance case is direct definition
construction of the author's struct/field example plus preserved builder compatibility.
[Plan and current gap](../api-docs/experimental-metadata.md#definition-first-model-author-direction-2026-10-01)
and [conversation](development-timeline.md#2026-10-01-definitions-are-the-editable-metadata-model).
The author subsequently endorsed Mono.Cecil alignment where it matters; follow the
[sourced alignment map](../api-docs/experimental-metadata.md#cecil-alignment-reference-reviewed-2026-10-01).
This changes the immediate implementation sequence, not the Raven/native end-to-end goal.

**Value-type category prerequisite (2026-10-01):** the metadata producer now emits
owned nongeneric/generic value definitions with primitive fields and preserves CLI
VALUETYPE categories through native binary projection. Default values, field reads,
generic forwarding and arrays verify/run on CLR and neoCLR (42). Raven admission and
the collections Option<Order> gate remain unchanged. Generic payload storage and addressed local mutation now pass (633a4a4a); explicit
imported value-type/category mapping and union emission remain open. The newer
definition-model direction above takes priority; do not represent value unions as classes.
[Evidence](experiments/extended-cli-metadata/value-type-validation.json).

**Nominal method imports (2026-10-01):** separately emitted Raven binaries now execute
static factory/reader calls with Box<consumer Order>, payload alias mutation and nominal
overload matching (42). Dependency-local reference signatures are imported through the
metadata API; value-type unions and imported instance/generic-owner methods remain open.
The unchanged collections sample still stops at Option<Order>; preserve its value-type
category rather than widening reference admission. [Probe](experiments/extended-cli-metadata/nominal-method-validation.json)
and [current gate](experiments/extended-cli-metadata/collections-after-nominal-methods.json).

**Imported generic arguments (2026-10-01):** Raven/native execution now covers
consumer-owned nominal arguments, imported constructions and caller method/owner
parameter forwarding through external unconstrained static generic methods (42).
This closes an instantiation gap; the unchanged collections gate remains the
Option<Order> signature and downstream member/native-identity requirements.
[Evidence](experiments/extended-cli-metadata/generic-library-validation.json).

**Raven external-signature checkpoint (2026-10-01):** the target adapter now consumes
registered external reference types and constructions. Separate Raven-produced binaries
verify/run (42), including nullable locals, interface arrays and generic forwarding.
The unchanged collections gate advances past Register admission to PendingOrder's
Option<Order> return signature. Imported value/union contracts, constructors/member calls
and translated-System native identity mapping remain open; direct sample execution is
not complete. [Probe](experiments/extended-cli-metadata/external-signature-validation.json)
and [current gate](experiments/extended-cli-metadata/collections-after-external-signatures.json).

**External signature prerequisite (2026-10-01):** the independent metadata API now
emits external reference classes/interfaces and generic constructions, preserving CLI
TypeRef/TypeSpec shape and native dependency scope. An API-produced Box<consumer Order>
case loads/verifies/executes as binary assemblies (42), including generic forwarding.
This does not yet advance the unchanged collections sample: next connect these types
to imported member signatures and translated-System native identity mapping in Raven.
[Evidence](experiments/extended-cli-metadata/imported-type-validation.json).

**Primary acceptance reaffirmed (2026-10-01):** use unchanged existing samples to
prove Raven → neoCLR assembly → runtime loading/execution. API work and focused probes
support this gate. After imported generic-method support, order-collections still binds
and emits the CLI control but direct emission writes no image: Register's imported
MutableMap<int, Order>/ArrayList<Order> signatures remain unsupported. Next address
external nominal/generic type and member identities against the translated System
library, then advance through this sample's constructor/callback/union requirements.
[Fresh evidence](experiments/extended-cli-metadata/collections-after-generic-imports.json).

**Author reaffirmation (2026-10-01):** .NET metadata is the baseline, with explicit
neoCLR extensions such as assembly-level functions. The Cecil-like API is the primary
abstraction for inspecting, modifying and creating assemblies. Preserve familiar
metadata shape/behavior; current immutable snapshots, editable producer graphs and
bounded imports are incremental coverage, not the final loaded read–edit–write model.
[Conversation](development-timeline.md#2026-10-01-net-metadata-baseline-and-a-complete-cecil-like-lifecycle).

**Imported generic-method checkpoint (2026-10-01):** separate Raven library and
application binaries now execute unconstrained static generic methods with concrete
primitive/vector arguments (42), including arity overloads and alias mutation. This
completes a bounded method-instantiation prerequisite; imported nominal identities,
generic declaring types and constrained dependencies remain open before collection
imports. [Evidence](experiments/extended-cli-metadata/generic-library-validation.json).

**Imported vector checkpoint (2026-10-01):** direct Raven emission now supports
static primitive-vector calls across separately emitted library/application assemblies.
The neoCLR-profile binary pair verifies and returns 42, including overload selection
and shared array mutation. This establishes a bounded external signature path;
nominal/generic collection imports remain the next gap, and native symbol loading
remains deferred. [Evidence](experiments/extended-cli-metadata/vector-library-validation.json).

**Collections binding checkpoint (2026-10-01):** the imported carrier-constructor
bug is fixed in Raven binding and integrated into local main (`46491585e`). The
unchanged order-collections sample now imports, verifies and runs with exact output;
assembled binary App and System also verify/run. This is legacy-bridge/assembler
control evidence, not direct Raven metadata emission or a fresh System source build.
Direct emission now reaches the imported generic Register signature boundary instead
of the old profile gate. Keep imported collection contracts/member references as the
next backend acceptance step. [Evidence](experiments/extended-cli-metadata/order-collections-binding-validation.json).

**Native profile checkpoint (2026-10-01):** after independently proven general Raven
fixes reached local main and a bounded .NET refactor-parity audit, direct emission now
accepts the real neoCLR semantic profile with a validated CLI declaration core.
Hello World via another function, Unit entry, array iteration and owned interface
dispatch verify and run as native binaries without host core references. Core-name
and version mismatches reject before output. This completes the bounded profile gate,
not the implementation seed or native metadata importer. Next prepare implementation
bootstrap dependencies and generic collection/imported-member contracts; keep the
known imported-carrier and loop-capture issues explicit when expanding samples.
[Evidence and scope](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md#native-profile-gate-and-refactor-parity-follow-up).

**Post-dispatch assessment (2026-10-01):** the author requested a broader library/app
readiness check. Owned nongeneric interface dispatch now executes through the direct
binary pipeline. All twelve selected existing samples bind and emit ordinary CLI with
the real target snapshot, but direct emission still rejects the neoCLR target profile.
Three unchanged applications run through the current legacy bridge; order-collections
exposes invalid Option constructor selection in emitted CLI before import. The next
proposed integration milestone is the native target profile and implementation seed,
then generic collection contracts/imported members and broad union/callback emission.
The whole-library attempt with an application-only snapshot is not a supported bootstrap.
See the [assessment, evidence and acceptance gates](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md).

**Interface declaration checkpoint (2026-10-01):** unchanged Comparer<T> and
EqualityComparer<T> now emit ordinary CLI/native interface contracts and load/verify
in both file orders. This does not yet prove interface dispatch. Disposable and Iterator<T> now join that evidence with abstract properties and
nongeneric inherited contracts. Iterable<T> and interface-valued reference/default
flow now execute too. Owned nongeneric implementations and interface method/property dispatch now execute
through two classes and reference arrays in both orders (42). The author requests
reassessment against existing library/application samples next; generic dispatch and
inheritance remain deferred. [Evidence](experiments/extended-cli-metadata/interface-library-runtime-validation.json).

**Whole-source checkpoint (2026-10-01):** complete, unchanged
System.Globalization.Language now executes on .NET and neoCLR in both file orders
(und/sv/he; result 42), using shared static-property accessor calls. No new opcode or
runtime format is needed. Comparer<T>/EqualityComparer<T> bind but await interface
emission; ArrayList needs native library dependencies. [Current evidence](experiments/extended-cli-metadata/whole-library-runtime-validation.json).
The author reaffirmed .NET behavior/instruction semantics as the baseline; unsupported
features are coverage limits unless a divergence is explicitly chosen.

**Constraint checkpoint (2026-10-01):** owned nominal bounds and class/struct/new
requirements now pass Raven → independent metadata API → native binary loading and
execution alongside .NET. This is a bounded integration, not complete class-library
emission. Per the author's request, pause expansion to assess the state and select
subsequent work from concrete consumer gaps. [Assessment and evidence](experiments/extended-cli-metadata/state-assessment-2026-10-01.md).

**Latest author-directed acceptance (2026-10-01):** compile real Raven class-library
parts before broader metadata loading. The original integer Math Min/Max/Sign
functions now emit with native namespace identity and execute 11 boundary cases on
.NET and binary neoCLR in both source orders. This is selected-source coverage under
the host-core bootstrap, not a full System build. Whole Math/UnicodeScalar/GC still
require native dependencies. Next use the [order-collections application](experiments/raven-target/samples/application-order-collections.rvn)
to drive constructor/property, object, generic and delegate contracts incrementally.
The unchanged Order declaration now emits with canonical properties/backing fields,
accessors and its explicit constructor. A separate Main constructs it and checks
Boolean/Int32 boundaries; both source orders verify/run on .NET and binary neoCLR,
returning 42. The independent API supplies fields, constructors and property associations;
Raven reuses compiler-synthesized accessor bodies through the shared instruction plan.
Both the metadata API and Raven now support owned nominal locals; mutation through
an alias is observed through the original object on both runtimes. Ordinary instance
calls also execute, including private helpers and ordered argument side effects.
Private mutable primitive storage now emits fields without property/accessor rows,
with qualified reads and mutation validated on both runtimes. Computed getters and
implemented block/arrow accessors now execute too, including private setters and
`field` backing storage with preserved property metadata. Explicit root constructors
now accept expression bodies with overload/argument-order execution coverage. Default
constructors and primitive field/property initializers now share the compiler initialization
plan and execute on both targets. Owned nominal parameters/results now flow through
shared backend type mapping; factories, self-return, constructor arguments, overloads
and alias mutation verify/run on .NET and binary neoCLR in both file orders. The current
nominal-signature/default-constructor/primitive-initialization slices are complete. Explicit
mutable primitive fields also preserve visibility, initialization and alias mutation
through binary execution. Mutable owned nominal fields and private storage now preserve
class identity, initializers and alias mutation in both source orders on both runtimes.
Owned nominal properties now preserve auto/computed/explicit accessor associations,
private setters and backing storage through the same pipeline. A shared binder fix
also refreshes provisional storage initializers without replacing field identity, now
including private stored properties. Explicit root `base()` uses the existing initialization
contract after semantic validation; side-effecting initializers and both body forms execute
on both runtimes. Readonly instance storage now preserves CLI/native flags and rejects
ordinary direct/managed-address writes in verification and execution; Raven consumes it
for private `val` storage and stored `val` properties. The connected vector milestone
is now implemented: metadata declarations now preserve CLI SZARRAY/native ArrayRef for
primitive and owned-class arrays. API-produced binaries now allocate, index, mutate
and measure arrays on neoCLR, including aliasing and bounds faults. Raven now emits
array literals, storage, calls, indexing and Length on both targets; Order-array alias
mutation and nested/labeled iteration return 42 in both source orders. This bounded
array emission milestone is complete: [evidence](experiments/extended-cli-metadata/array-runtime-validation.json).
The next connected milestone is indexed properties: metadata producer/projection
support now preserves index parameters and overloaded accessor associations.
API-produced binaries verify and execute indexed accessors on neoCLR (42); shared
Raven now emits indexed accessor calls and associations; overloads, multiple indices
and read-only getters execute on both targets in both source orders (42). A concrete
Order collection now also proves nominal/array indices, object aliases, evaluation
order and indexed bounds faults. The author directs generics as the next connected
milestone. Generic function/static-method declarations now preserve named parameters,
CLI GenericParam/MVAR and native method parameter identities; generic locals and arrays
round-trip. Instantiated calls and Raven consumption follow. Broader field/constructor shapes and
generic/delegate/union contracts remain subsequent work. The full consumer still has 49 binding errors
from native collection/LINQ/union dependencies under the host-only bootstrap.
[Evidence](raven-cli-bridge.md#namespaced-functions-and-real-math-source--2026-10-01).

**Compatibility baseline reaffirmed (2026-10-01):** preserve ordinary .NET/CLI metadata
and instruction encodings, with explicit neoCLR extensions. The current native #Neo
execution payload plus CLI reference projection is a working bridge, not yet full
extended-CLI executable compatibility. Keep that gap visible while adding ordinary
compiler coverage; do not treat this bridge as a permanent replacement format.
[Design boundary](design/extended-cli-metadata.md#compatibility-baseline-reaffirmed--2026-10-01).

**Author-directed metadata exploration (2026-09-30).** Begin extended CLI metadata
on `codex/extended-cli-metadata`, based on main, with later Raven integration. The
author confirms the Cecil-style metadata API remains an independent project, consumed
by Raven's neoCLR target through compiler-owned loader/emitter adapters; integration
does not move the library into the compiler. [Project ownership](design/extended-cli-metadata.md#independent-metadata-project-raven-target-consumer-2026-09-30).
**Latest sequencing clarification:** establish a working metadata format and its APIs,
then integrate the refactored compiler, then add improvements such as structural types.
The direct native producer/load test below is the bounded acceptance baseline;
ordinary compiler-required metadata coverage is next.

**Architecture follow-up (2026-10-01):** after reviewing Reflection dependencies, the
author directed a staged codegen refactor. The first slice now feeds both bounded
backends compiler-lowered bodies, including implicit value returns. The first callable reference table now shares symbol identity resolution with typed
backend handles. Shared source callable plans separate native source validation from
builder creation. Shared public/internal nongeneric static-type plans now drive backend type
builders, including partial declarations coalesced by symbol identity with every part
validated. Primitive signatures/locals now share logical value/no-result types and
backend-owned mappers; broader type/field references and full declaration traversal remain next,
and the first paired body capability now supports initialized Int32 locals/assignments.
Comparisons (including negated forms), if/else and lowered loops with break/continue now execute on both runtimes;
Int32/Boolean/no-result signatures now flow through both backends, native imports and
reference projections, with Boolean local initialization/assignment, equality and short-circuit &&/|| and discarded primitive call results. Int64 signatures/locals and signed Int32↔Int64
conversions and signed unary +/−/~ now share that path; broader conversions and general type/field references remain next;
String signatures, locals, literals and imported text helpers now share the path,
with computed console output on both runtimes. The independent metadata API also supports typed argument stores with caller
isolation; ordinary Raven source parameters remain immutable. Backend-owned instruction/built-in-type capability profiles now admit shared bodies
before native builder allocation, with restricted-profile tests validating
selective admission. Signed Int32/Int64 division, remainder, bitwise AND/OR/XOR and signed shifts now execute on both backends. Logical assembly-function/static-method/static-type categories now share those
profiles. Public/internal static type visibility is also preserved across both targets.
The independent metadata API now preserves public/internal/private static method
visibility, with direct binary runtime enforcement. Raven now emits these methods
through shared visibility admission and preserves access in compiler references.
Block and expression bodies now share compiler lowering and execute on both targets.
Eager Boolean AND/OR/XOR now share that path with exact Boolean metadata/runtime
operands; mixed Boolean/integer inputs remain invalid.
Primitive value-producing if/else now shares typed branch joins on both backends;
value blocks also permit initialized locals, assignments, calls and internal if/loop
control flow before the result, with returns/outgoing jumps rejected during planning.
The metadata API now supports public/internal ownerless functions with native access
enforcement; Raven now preserves explicit public/internal and default internal access.
General object/field and broader metadata-category contracts and
metadata loading remain future slices. See [integration scope and validation](raven-cli-bridge.md#compiler-lowered-native-bodies--2026-10-01).

**Shared compiler pipeline:** Raven's explicit native backend now participates in
`Compilation.Emit`; rvnc and API wrappers share compiler setup and target validation.
The author directs reuse of common .NET/neoCLR lowering with backend abstractions for
builder differences. The shared linear-body model now feeds .NET/native method-builder adapters and
executes the same Int32 and Unit Hello/helper compilations on both runtimes, including
assembly functions, explicit/implicit returns and an empty Unit entry. Supported callable
signatures and declaration-builder contracts are now shared too; concrete adapters preserve
CLI type-method and native assembly-function ownership. Type/signature builders, generics and
general member bodies remain subsequent boundaries. The author asks to avoid large workarounds
and prioritize the Hello/helper end-to-end case. Shared metadata loading is deferred;
the optional System driver has exposed a host/projection type collision and explicitly
rejects that call. Its direct API case still passes; driver import is not yet reliable. [Integration boundary](raven-cli-bridge.md#shared-emission-pipeline--2026-09-30).

**Latest acceptance:** Raven's opt-in emitter now writes binary PE/#Neo assemblies;
neoCLR decodes their native metadata directly. Hello World, an entry-point call to
Greet, both source-file orders and the two-library chain all verify/run successfully.
Schema-1 JSON containers remain readable. The separate Cecil-style API still supplies
CLI reference projections to Raven's existing .NET semantic provider. The author
identifies class-library compilation and Raven symbol loading as the next consumer,
with existing JSON-to-assembly translation as a bootstrap. The complete current System
JSON now translates to a standalone binary native envelope, verifies all 641 IL functions
and runs both Hello cases and a generic dependency chain. This has no CLI reference
projection. Next: compile the runtime class library from Raven source and compare
against these translated artifacts; broad symbol import, a native symbol provider
and production registration remain unimplemented.
[Binary profile](design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30)
and [measured phases](experiments/extended-cli-metadata/binary-loading.md).

**Author-directed consumer exploration:** Exercise translation of the existing Raven
CLI bridge's test/sample output before expanding direct compiler emission. A proper
neoil assembler producing native assemblies is a future producer. The Raven collection
library profile exceeds the current binary transport bounds; retain that explicit gap
while isolating application translations with matching JSON System. Floating-point
operand tests additionally expose the need for full UInt64 bit-pattern encoding.
**Follow-up implemented:** standalone schema 3 adds that encoding and bounded larger
library capacity; FloatingMath, OptionPositional and ValueCopy now run against binary
Raven System. The neoil binary producer and direct native compiler/symbol import remain
next work. [Profile and validation](design/extended-cli-metadata.md#library-execution-profile-3--2026-09-30).
[Experiment and reproducible checks](experiments/extended-cli-metadata/raven-sample-translation.md).

**Direct assembler checkpoint:** `assemble --format neox` now emits schema-3 native
assemblies directly from neoil, with verification before output and JSON kept as the
default. The full Raven collection System and three sample applications pass independent
.NET metadata comparisons and native execution. Direct Raven class-library source emission
and native compiler symbol import remain next. [Producer evidence](experiments/extended-cli-metadata/direct-assembly.md).

**Author-requested performance evidence:** A release comparison finds the full Raven
System assembly 63% smaller and about 20% lower decode time than current pretty JSON,
but no meaningful complete-startup/run improvement (about 3.45 seconds). Linking and
admission dominate; direct-typed JSON is a faster diagnostic alternative to both current
decoders. The author explicitly accepts smaller files as an improvement and defers
runtime optimization to the future. Retain this baseline and continue metadata/compiler
integration; the benchmark does not reprioritize runtime optimization.
[Method and limits](experiments/extended-cli-metadata/json-vs-assembly-benchmark.md).

**Raven continuation:** The author selects continued Raven work after the size benchmark.
The native compiler adapter now emits Unit-returning helpers and static library methods,
reimports their CLI no-result projections and runs a separate application against the
binary library. This removes the artificial Int32 return from side-effecting helpers;
entry points remain Int32. Namespaced static classes now preserve identity through
compiler reimport and native calls, including same-name types in different namespaces
and both source orders. Wider signatures, native symbols and actual runtime class-library
source compilation remain next. [Compiler contract](raven-cli-bridge.md#unit-returning-native-helpers-and-library-methods--2026-09-30).

**Native compiler command:** An opt-in `rvnc neoclr` command now compiles source files
and native writer-produced PE references directly to PE/#Neo, including assembly-owned
functions. The author confirms symbol loading from translated System/System.Runtime is
also required. Prioritize that loader acceptance alongside native emission; host primitive
binding is only a bootstrap, not evidence of translated System symbol loading. Broader
native declaration reading/projection or a native symbol provider is still required.
[Command and loader boundary](raven-cli-bridge.md#opt-in-native-compiler-command--2026-09-30).

**First translated System symbol slice:** An explicit static Int32 callable view now
adapts selected native declarations to Raven's existing semantic importer. Math.Min binds
to that view and executes against the translated System through both API and rvnc paths.
This is not full core import: primitive binding remains hosted, and generic/instance/field/
property contracts remain next. The author supports adapting the existing importer first,
then a separate provider as needed, and removing Reflection.Emit from .NET in the future.
A common instruction/operand model and later ILProcessor-like editing are recorded future
metadata API directions. [Scope and architecture](raven-cli-bridge.md#translated-system-callable-import--2026-09-30).

**Native entry contract:** Raven now emits parameterless Unit Main as a no-result
native entry, including global/static ownership and helper calls, and neoCLR exits zero.
The metadata reader/writer reuse format 5's existing no-result encoding. This removes
an artificial compiler restriction while keeping metadata support and target integration
as the main objective. Full System import and richer callable/body coverage remain next.
[Entry contract and evidence](raven-cli-bridge.md#unit-entry-points--2026-09-30).

**Compiler-facing opcode surface:** The independent metadata API now exposes bounded
Emit overloads for typed opcode/operand construction, and Raven's native emitter uses
them. Convenience helpers share the same path. This advances target code generation;
it does not claim full opcode coverage or ILProcessor-style editing. Broader metadata,
System symbol import and native source coverage remain the objective.
[Contract](raven-cli-bridge.md#opcode-based-metadata-emission--2026-09-30).

**Earlier metadata checkpoints (historical progression):** The PE reader now recovers
owned global/type callable declarations and the writer's static Int32 signature subset;
21 C# contract groups cover ownership, unsupported signatures, decoding limits and
explicit MemberRef resolution for the emitted nominal static method subset. General
signatures, broader member contracts and body import remain readiness gaps. The author
now selects **Refactor Raven for platform targets** (activity
`01a0f154-2448-7df3-8536-c837097b46c2`) as the integration consumer: attempt a bounded
compiler import/emission/runtime case once this subset suffices, then drive remaining
metadata support from that case rather than waiting for exhaustive coverage. Stage 1
now passes: Raven binds a small source program and an API-produced PE dependency;
an opt-in adapter consumes public operations and emits native bytes through the separate
library, which neoCLR verifies and executes to 42 ([evidence](experiments/extended-cli-metadata/raven-compiler-validation.json)).
The frontend still uses the .NET metadata provider; native provider/production emitter
composition are next. The case exposed shared operation and parameter-import fixes,
validated by 87 focused Raven tests and integrated into local Raven main. See the
[inspected compiler boundaries and next case](design/extended-cli-metadata.md#refactored-compiler-as-the-next-end-to-end-consumer-2026-09-30). Structural types remain a future
design requirement (arrays, tuples, Function types, unions/intersections and synthesized
members), not a prerequisite for that first compiler integration. The
[design and staged acceptance plan](design/extended-cli-metadata.md) is exploratory;
an isolated codec/inspector now validates experimental framing and structural
signatures, host-catalog nominal resolution and synthesized-member references with
30 focused tests. Differently numbered fixture references resolve to equal structural
type/member keys. A bounded PE32 #Neo probe preserves conventional metadata for
.NET/Cecil inspection; Cecil rewriting strips the extension. An experimental marker/digest and explicit expected-input profile now reject
stripped/changed metadata in 11 recognition cases. The first .NET reader/writer library now covers envelope framing, with four shared
fixtures and 49 cross-reader rejection cases. Its structural signature codec now passes
14 cross-reader vectors and 103 rejection cases. .NET reference tables and explicit-catalog
structural identity pass 95 shared vectors (69 rejections). .NET synthesized-member
contracts now pass 49 shared vectors (30 rejections). A typed .NET reference-profile
Read/Create/Write facade passes 36 shared vectors (29 rejections). Bounded .NET PE32
artifact recognition/extraction passes 50 shared cases (44 rejections). Following the
author’s Cecil suggestion, the [provisional API direction](design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30)
places assembly/module/reference/definition objects above these codecs. An initial
read-only model now reads real assembly/module/TypeDef declarations, with generic/nested
Unicode and snapshot-scoped reference consumer checks. AssemblyRef identity and explicit
host resolution now pass 11 standalone C# contract tests. The author selects the
Cecil-like model as the primary compiler abstraction for metadata/PE manipulation,
with adaptations as needed. Physical TypeRef resolution and a controlled static-Int32
PE writer now support an end-to-end producer baseline. API-produced application and
library PEs execute in neoCLR through the existing CLI bridge and matching native
library, returning 42. Broader IL/signature coverage, general rewriting, Raven codegen
integration and direct native #Neo loading remain pending. The author also requires
functions outside types: the model now exposes assembly-owned functions and emits
native format-5 assemblies directly, with a cross-assembly function test returning 42
in neoCLR without the CLI bridge ([evidence](experiments/extended-cli-metadata/native-validation.json)). Production NEOX loading and structural runtime support remain unimplemented. This scopes the requested exploration
without promoting all proposals or merging the structural runtime experiment.
The author additionally requires eventual reader/writer support on both .NET and
neoCLR. The author further clarifies that the .NET API should support later Raven compiler
integration, with a potential Raven port beneath the pending Metadata Introspection API;
[readiness criteria and open choices](design/extended-cli-metadata.md#net-api-direction-and-potential-raven-port-2026-09-30)
keep this distinct from completed integration. The [cross-platform library plan](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr)
separates shared format/conformance contracts, .NET tooling, native support and an
actual neoCLR guest library. Concrete consumers are Raven’s symbol loader/code
generation and neoCLR assembly loading into Introspection/assembly emission; these
full library consumers remain planned; the .NET framing foundation is implemented.

**Immediate focus — Raven neoCLR target support (author-selected 2026-09-30).**
After the bounded main backport below, put the structural Function experiment on
hold and work in Raven to improve neoCLR target support. The Function runtime
branch remains isolated; its proposals are open, not completed on main. Specific
acceptance now includes integrating target-gated Self in Raven and neoCLR while
keeping structural types on feature branches in both repositories. This author
direction supersedes the earlier restriction on Raven main integration; a native
metadata layer and replacement backend remain future work. General compiler fixes
still require independent validation.
The earlier Web API direction below remains recorded for later resumption.

**Author-directed library backport (2026-09-30).** Bring Raven callback function
syntax and lazy OfType filtering to main independently of the structural Function
runtime branch. This bounded library change keeps the existing callable and
introspection models. It is complete; the subsequent author direction above selects
the next focus. See the
[query contract and evidence](raven-query-api.md#runtime-type-filtering--2026-09-30-backport).

**Author-directed release — Preview 11 (2026-09-27).**
The current bounded surface is qualified for release with macOS arm64 Raven tools
and a Windows x64 native-runtime ZIP. See [release notes](preview-11-release-notes.md)
and [qualification evidence](preview-11-validation.json). Generic async, shared Task.Run,
text/number foundations, nested JSON and routing samples are included. Windows
Raven SDK/bridge qualification remains separate. The full Linux suite passes; a
validator-only exit-code correction has independent archive-smoke evidence.
This closes release preparation without adding optional Web API capabilities.

**Preceding direction — minimal Web API (author-selected 2026-09-27).** Focus on
serving a useful Web API, nested JSON serialization/deserialization, and a
route parser used within an existing HttpServer handler, including named and typed
parameters. This later author direction defers the earlier separate WebApplication
project and Minimal API infrastructure.
A rudimentary SQL interface with a SQLite provider is an optional follow-on based
on the existing proposal, not a prerequisite or approval of its complete surface.
The [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api) owns
application acceptance; the [bounded plan](web-api-plan.md) proposes the sequence
and records design choices still to validate. The first nested typed JSON slice is
implemented in development with [consumer and HTTP evidence](experiments/json-object-mapping/nested-validation.json).
JSON now matches the 1,024-byte HTTP body budget, with [boundary and case evidence](experiments/json-object-mapping/payload-validation.json).
Typed arrays now support collection payloads; see [collection evidence](experiments/json-object-mapping/collection-validation.json).
The author's route/union refinement selects the next bounded routing slice: direct
RoutePattern/RouteMatch parsing and optional application union dispatch are now
implemented in development; see [the case](experiments/http-routing/README.md).
A [generated attribute-driven mapper experiment](experiments/route-union-mapper/README.md)
validates schemas and reuses compiled patterns. The author subsequently selects
runtime attribute reflection with cached startup mapping. Member/parameter
[attribute data](attribute-introspection.md) is now implemented in development;
the [runtime mapper case](experiments/runtime-route-mapper/README.md) now validates route schemas at startup and retains patterns, conversion bindings and case/carrier constructors for request handling. Source generation remains a future alternative. The mapper ships as tested Preview 11
sample source; it is not an installed SDK mapper API.
Enum, Uuid and Option JSON mapping remain requested and pending.
This explicitly supersedes the previous general useful-library priority; M2–M6
remain candidates. Preview 10's completed POC stays closed.

### Completed POC and preceding library direction

The bounded HTTP POC is [complete on macOS arm64](http-capabilities.md#finish-this-poc),
including independent peers, known-length uploads and the extracted-package check.
Feature scope is frozen; do not automatically start another HTTP feature.

**The POC is released and done for now:**
[Preview 10](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10),
published 2026-09-27. Option, Result
and TaskOutcome use standard Raven unions, and the native reflection boundary uses
the matching Option layout. See [release evidence](preview-10-validation.json).

The [toolchain/release tracker](tracking/toolchain-release.md) owns residual compiler
observations and delivery maintenance. Hosted CI passed all six split jobs at
`1404454e` (slowest 8.88 minutes); later migration corrections passed the focused
checks requested by the author and final extracted-package checks. The released POC feature
scope remains frozen; the new Web API direction is a separate increment. Thematic consolidation is complete.

**Preceding direction: useful library APIs**, selected by the author after release.
The author-selected comparer slice is implemented in development: equality/hash and
ordering policies, callback adapters, StringComparer.Ordinal and a HashMap policy
constructor. See the [library/data sequence and evidence](tracking/library-data.md#active-direction--useful-api-gaps).
The author-selected explicit String comparison modes and matching ignore-case policy
are implemented in development. The [String/System.Text foundation review](design/text-abstraction.md#systemtext-foundation-review--2026-09-27)
is complete: keep the UTF-8/grapheme model, but establish scalar, position, ownership
and conversion-progress contracts before dependent API expansion. The author clarifies
that the immediate System.Text scope is encoding/decoding foundations and possibly
a small builder, with Swift as the closer model for text API shape.
The [library tracker](tracking/library-data.md#string-design-review-before-further-expansion)
owns the boundary experiment, [paired text API sketch](design/text-abstraction.md#consumer-api-sketch-identical-behavior-two-vocabularies)
and the shared Encoding/Decoder integration in both stream adapters. Development
constructors accept encoding selection with UTF-8 defaults and strict ASCII, backed
by focused consumer checks. The [bounded report construction evaluation](design/text-abstraction.md#bounded-report-construction-evaluation--2026-09-27)
passes its contracts but does not justify promoting the managed builder: ordinary
concatenation is faster for the tested report sizes. Keep public builder promotion
deferred under that earlier evaluation. On 2026-10-08 the author explicitly selected
a related [StringBuilder and String.Join side quest](design/string-building.md), with
fluent append, byte-limit faults and shared interpreter/native materialization. This
bounded development API work revisits the deferral without replacing the native HTTP
priority or endorsing the earlier prototype. The [Encoder acceptance/drain evaluation](design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27)
led to the [public Encoder integration](design/text-abstraction.md#public-encoder-integration-development--2026-09-27):
independent factories, bounded progress and explicit StreamWriter.Finish are implemented
in development with matching API artifacts and focused consumers. The bounded
UTF-8/strict-ASCII encoding foundation is complete. Broader codecs, general writer
completion and further text capabilities need separately scoped consumer work. General
scalar/range APIs and the broader portfolio are not prerequisites. Experimental
types and proposed names are not adopted System APIs.
The subsequent author-selected [casing and Int64 report slices](design/text-casing-integer.md)
are implemented in development: Unicode 17 full default String casing, typed strict
Int64 parsing, decimal formatting and bounds properties. Focused consumer/native/
metadata checks and matching API artifacts cover this bounded work. No new broader
milestone is selected by completing these slices.
The author next selected [Number and concrete primitive parsing](design/numeric-contracts.md).
The bounded development slice is implemented with focused consumer/native/metadata
evidence; all numeric parsers use NumberParseError. A parsing interface is on hold. The author also
identified [general interface capabilities](tracking/runtime-language.md#interfaces-as-a-platform-capability)
as important direction: static members, default bodies and member access control.
Number is a first consumer, not a permanent numeric-only interface model.
Native metadata integration now separately compiles and executes the unchanged
Single/Double Number implementations through explicit scalar ownership; see the
[checkpoint](experiments/extended-cli-metadata/system-compilation-strategy.md#source-floating-primitive-ownership-2026-10-04).
The subsequent cumulative source-library rebuild executes all ten numeric types with
seed ownership removed. The metadata generator now executes concrete static constrained interface calls on
both targets. Owned nongeneric method interface bounds now round-trip and import into
Raven symbols; external nongeneric method bounds now also retain scoped identity and
execute in a separate-contract metadata fixture. Open method-parameter constrained calls
now execute for owned interface targets, including native Self substitution. External
static and constructed instance call targets now execute through Raven. The ordinary
driver rebuilds all ten numeric source types, compiles a separate generic algorithms
library, and runs a source-free consumer of every Number member and generic forwarding
with exit 42. See the completed generic Number gate in the system compilation strategy.
The next bounded interface slice adds application defaults and public/private static
helpers through Raven, with [focused evidence](experiments/interface-helpers/README.md).
The [interface limitations table](tracking/runtime-language.md#interface-limitations--development-checkpoint-2026-09-27)
separates native support from Raven import gaps. A subsequent bounded
[explicit class implementation slice](experiments/explicit-interface-implementations/README.md)
adds ordinary methods on application classes/interfaces. Accessors, generic/value-type
import, private instance helpers and broader static/default/accessibility cases remain open.
The author also selected [Result/Task Main integration](experiments/entry-results/README.md),
with direct async Main adoption in relevant samples after focused validation.
This is a bounded startup integration, not a new broader milestone.
The author next selected [Task.Run with shared captured objects](tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
as the canonical work-submission API, with runtime-selected execution. Development
now has native shared captures, typed/completion-only overloads and task unwrapping.
The [original Task.Run compiler integration failures](experiments/task-run/compiler-gaps/README.md)
are corrected: short-name lookup and block-lambda returns use independently tested
Raven fixes integrated into both branches. Direct completion-only await
now passes with the target compiler unit-result fix. Mutable-local
sharing in ordinary async methods is corrected by the independently tested Raven
closure fix. Generic-method capture metadata is repaired in Raven.
[Generic async application import](experiments/task-run/README.md#generic-async-application-import)
now handles the constructed state machine, shared closure and generic holder, with
forced suspension, identity and cancellation checks. Closed ordinary static helpers
and their generated generic types follow normal Raven metadata.
Async methods inside generic classes have a separately reproduced Raven arity bug.
Thread's future public role is open.
M2–M6 remain candidate applications; no complete File Catalog
or additional HTTP feature is required by the comparer slice.

The author next selected [constructor discovery and Reflection execution](reflection-members.md):
argument-based activation, invocation and field access, with Result failures and a
separate website feature page. This is bounded library/runtime work, not selection of
a new application milestone.

The author selected a bounded `System.Runtime.GC` API after Reflection: execution-local
object counters, explicit full collection and lifetime use. See the
[GC contract](runtime-gc.md); generation/byte accounting and collector tuning remain
outside this one-off author-directed slice.

The author selected [value tuple support](tuples.md) on 2026-09-28, using
`System.Tuple` as the neoCLR name corresponding to .NET `System.ValueTuple`.
This is a bounded author-directed addition; it does not replace the Web API direction.
The [runtime tracker](tracking/runtime-language.md#value-tuples--2026-09-28) owns status.
On 2026-10-14 the author directed replacing this nominal representation with
[structural tuples](structural-tuples.md). Native tuple signature roundtrips and
substitution are implemented, followed by opt-in closed tuple values and positional
access in the interpreter, resolved host identities and ordered element descriptors.
Guest ordered element introspection is available through `TupleTypeInfo`, and
native C# signature authoring, opt-in snapshots and callable imports preserve ordered
tuple shapes with CLI rejection. C# construction/element operations execute nested
tuples across imported calls with copy semantics;
Raven source-local scalar/nested tuple lowering executes named/positional reads
and copies (42). Native Rust signatures and host descriptors now preserve names
without changing structural equality. C# authoring/import and Raven emission retain
native names, and explicit Raven tuple imports execute across libraries (42). Guest
name queries now preserve per-signature names with positional defaults. Raven local tuple mutation now preserves copies through nested writeback (42). Native structural binding now removes nominal carrier lookup, including nine-element execution (42). Native projects now enable all three structural forms together, with a separate-library sample returning 42 and default interpreter feature admission (2026-10-15). Fresh Runtime, Data and Networking library builds pass; Web still exposes structural-union conversions in Result error propagation/case construction: its alternatives are nominal value unions. Author direction includes value and reference alternatives. Interpreter boxed value-alternative admission, host authoring/reads and compiler exact conversions are validated. Direct structural ref/out slots are validated (44 runtime checks and source consumer 42). Structural callback mapping now clears the delegate-resolution failure; all four source libraries generate in the v3 bundle. Direct callback execution returns 42. Bounded generic value-wrapper and callback paths are now validated (53 runtime checks); broader generic forms remain incomplete. The v3 consumer now resolves metadata, executes HTTP-header checks and preserves prior output on an invalid configuration. Generic wrapper/callback substitutions are covered by focused tests. The fresh-bundle awaited Task<Result<Response, HttpError | JsonError>> consumer also passes completed/queued success and both imported error alternatives (42), with 54 structural tests; see `docs/experiments/value-unions/task-results.json`. Direct Task<Response | HttpError | JsonError> and Task<Response<int> | HttpError | JsonError> consumers now pass completed/queued/async-forwarded alternatives and reference identity (42), with 21 compiler and 55 runtime checks; see `docs/experiments/value-unions/direct-task-results.json`. Scoped nominal constituents now preserve method/type parameters through native metadata and runtime substitution; the Response<T> generic-method consumer returns 42 and 58 runtime checks pass (`docs/experiments/value-unions/open-generic-results.json`). Bare parameter operands and symbolic constraint coverage remain incomplete. Author direction (2026-10-15): prioritize correct AOT structural data representation, with JIT awareness and efficiency as first-class requirements; interpreter success is not backend completion. Union-local closure capture, whole-union boxing and broader endpoint/backend coverage remain open. Nonlocal mutation,
native layout/backends and removal of legacy runtime tuple declarations
remain staged follow-ups on the experimental branches.

## Theme trackers

Each work item has one status owner. Use that tracker for its status, remaining
scope, next evidence and linked design. Cross-theme dependencies link to their
owner; they do not create a second checklist.

| Theme | Status owner | Scope |
| --- | --- | --- |
| HTTP and networking | [Client/server capability tracker](http-capabilities.md) | HTTP POC finish line, client/server matrix, transport limits and future protocol work |
| Runtime and language | [Runtime/language tracker](tracking/runtime-language.md) | Values, references, GC, async execution and deferred type-system/runtime experiments |
| Library and data | [Library/data tracker](tracking/library-data.md) | Text, collections, files/streams, time, introspection, JSON and general resource contracts |
| Toolchain and delivery | [Toolchain/release tracker](tracking/toolchain-release.md) | Raven integration, diagnostic debt, packaging, CI, API documentation and publication gates |

These are maintenance ownership boundaries, not an instruction to develop four
features in parallel. HTTP owns the application package acceptance decision;
tooling owns packaging procedure and general release checks.

## Milestones

The author now selects the Web API increment after M1. M2–M6 remain candidate
products; their earlier ordering is not approved API scope.

| Milestone | Sample product | Status / scope owner |
| --- | --- | --- |
| M1 — Communicate | Hello Service + Hello Client, now the typed HTTP/JSON exchange | Released in Preview 10; done for now |
| Web API increment | Nested/collection JSON + typed route-parser case | Active direction; [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api); JSON and direct routing slices implemented; generated union mapper experiment validated |
| M2 — Work with data | File Catalog | Candidate; library/data tracker |
| M3 — Handle waiting and failure | Download Queue | Candidate; HTTP, library and runtime contracts must be selected together |
| M4 — Human time and presentation | Activity Report | Candidate; library/data tracker |
| M5 — Explain programs | Assembly Explorer | Candidate; library/data tracker, with tooling integration |
| M6 — Across hosts | Portable Sample Pack | Candidate; toolchain/release tracker |

The detailed earlier product sketches and comparisons are preserved in the
[dated roadmap](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance).
An implemented supporting API does not by itself complete a milestone.

## Working rules

- Select a bounded task from the active milestone and keep its tracker and evidence
  current. A one-off author request does not silently reprioritize all future work.
- Prioritize features over optional optimization. Investigate runtime cost when it
  blocks a selected feature, materially impairs supported use, or fails a release
  criterion. Preserve defects and measurements without making indefinite profiling
  a prerequisite for unrelated work.
- At most one small companion task may accompany a larger feature. It must finish
  independently without introducing a new language, ABI, culture or lifetime contract.
- Run only validation needed for the change, including performance tests when relevant.
  Avoid routine website builds for unrelated work; keep content and API snapshots
  current. Run the full suite only when the change or uncertainty requires it. This reflects the author’s 2026-09-27 validation correction.
- Reuse the relevant .NET/CLR comparisons in linked design notes; deepen research
  when a contract changes. Follow [design research](design-research.md). Proposals
  are exploratory inputs, not specifications or implementation commitments.
- General Raven fixes must be independently validated on a main-based feature
  branch before neoCLR integration. Keep target policies isolated; follow AGENTS.md.
- Distinguish implemented source, tested development artifacts, packaged evidence
  and released behavior. Public APIs require matching reference documentation.
  Runtime release and website publication remain separate operations.

## Maintaining tracking

Put current status and next actions in the owning theme tracker. Put contracts,
alternatives and primary-source comparisons in design notes, commands/results in
experiment records, implemented changes in the changelog, and significant direction
exchanges in the development timeline. Do not append a new “current priority” section
to an older roadmap. Update the existing status row and preserve prior decisions as
history when they matter. Add a new theme only when existing ownership cannot fit it.

The 2026-09-26 issue inventory has been assigned across the theme trackers. It is a
dated inventory, not a fresh GitHub status check. Original issue details and all nine
Raven issue assessments remain in the [archived triage](history/planning-20260927/issue-fix-roadmap.md).
No issues were closed or implementations revalidated by this documentation change.

## Historical entry points

Older links are retained below and lead to dated records. Their former “current” or
“next” headings do not override the current work section above.

<a id="current-checkpoint--finish-this-http-poc-2026-09-26"></a>

- [Current checkpoint — finish this HTTP POC (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-checkpoint--finish-this-http-poc-2026-09-26)

<a id="current-priority--next-release-features-2026-09-26"></a>

- [Current priority — next-release features (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-priority--next-release-features-2026-09-26)

<a id="implemented-checkpoint--http-json-clients-2026-09-26"></a>

- [Implemented checkpoint — HTTP JSON clients (2026-09-26)](history/planning-20260927/platform-roadmap.md#implemented-checkpoint--http-json-clients-2026-09-26)

<a id="issue-driven-priorities--2026-09-26"></a>

- [Issue-driven priorities — 2026-09-26](history/planning-20260927/platform-roadmap.md#issue-driven-priorities--2026-09-26)

<a id="active-direction--sockets-for-a-web-application-2026-09-24"></a>

- [Active direction — sockets for a web application, 2026-09-24](history/planning-20260927/platform-roadmap.md#active-direction--sockets-for-a-web-application-2026-09-24)

<a id="http-client-pipeline-checkpoint--2026-09-24"></a>

- [HTTP client pipeline checkpoint — 2026-09-24](history/planning-20260927/platform-roadmap.md#http-client-pipeline-checkpoint--2026-09-24)

<a id="networking-and-web-release-target--discussion-2026-09-25"></a>

- [Networking and web release target — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#networking-and-web-release-target--discussion-2026-09-25)

<a id="scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24"></a>

- [Scheduling checkpoint before the public socket bridge — 2026-09-24](history/planning-20260927/platform-roadmap.md#scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24)

<a id="author-directed-interface-naming--2026-09-25"></a>

- [Author-directed interface naming — 2026-09-25](history/planning-20260927/platform-roadmap.md#author-directed-interface-naming--2026-09-25)

<a id="authority-and-use"></a>

- [Authority and use](history/planning-20260927/platform-roadmap.md#authority-and-use)

<a id="how-the-plans-fit-together"></a>

- [How the plans fit together](history/planning-20260927/platform-roadmap.md#how-the-plans-fit-together)

<a id="milestones-at-a-glance"></a>

- [Milestones at a glance](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance)

<a id="release-checkpoint--async-and-tasks"></a>

- [Release checkpoint — Async and Tasks](history/planning-20260927/platform-roadmap.md#release-checkpoint--async-and-tasks)

<a id="next-release-validation-efficiency"></a>

- [Next-release validation efficiency](history/planning-20260927/platform-roadmap.md#next-release-validation-efficiency)

<a id="post-release-concurrency-direction--2026-09-23"></a>

- [Post-release concurrency direction — 2026-09-23](history/planning-20260927/platform-roadmap.md#post-release-concurrency-direction--2026-09-23)

<a id="future-networking-and-web-namespaces--consideration-2026-09-23"></a>

- [Future networking and web namespaces — consideration, 2026-09-23](history/planning-20260927/platform-roadmap.md#future-networking-and-web-namespaces--consideration-2026-09-23)

<a id="minimal-http-application-dependencies--consideration-2026-09-24"></a>

- [Minimal HTTP application dependencies — consideration, 2026-09-24](history/planning-20260927/platform-roadmap.md#minimal-http-application-dependencies--consideration-2026-09-24)

<a id="progressive-delivery-before-networking--revised-2026-09-23"></a>

- [Progressive delivery before networking — revised 2026-09-23](history/planning-20260927/platform-roadmap.md#progressive-delivery-before-networking--revised-2026-09-23)

<a id="m1--communicate-hello-service-and-hello-client"></a>

- [M1 — Communicate: Hello Service and Hello Client](history/planning-20260927/platform-roadmap.md#m1--communicate-hello-service-and-hello-client)

<a id="m2--work-with-data-file-catalog"></a>

- [M2 — Work with data: File Catalog](history/planning-20260927/platform-roadmap.md#m2--work-with-data-file-catalog)

<a id="m3--handle-real-waiting-and-failure-download-queue"></a>

- [M3 — Handle real waiting and failure: Download Queue](history/planning-20260927/platform-roadmap.md#m3--handle-real-waiting-and-failure-download-queue)

<a id="m4--human-time-and-presentation-activity-report"></a>

- [M4 — Human time and presentation: Activity Report](history/planning-20260927/platform-roadmap.md#m4--human-time-and-presentation-activity-report)

<a id="m5--explain-programs-assembly-explorer"></a>

- [M5 — Explain programs: Assembly Explorer](history/planning-20260927/platform-roadmap.md#m5--explain-programs-assembly-explorer)

<a id="m6--across-hosts-portable-sample-pack"></a>

- [M6 — Across hosts: Portable Sample Pack](history/planning-20260927/platform-roadmap.md#m6--across-hosts-portable-sample-pack)

<a id="research-products-alongside-the-milestones"></a>

- [Research products alongside the milestones](history/planning-20260927/platform-roadmap.md#research-products-alongside-the-milestones)

<a id="active-progress--2026-09-23"></a>

- [Active progress — 2026-09-23](history/planning-20260927/platform-roadmap.md#active-progress--2026-09-23)

<a id="working-rules-and-immediate-next-step"></a>

- [Working rules and immediate next step](history/planning-20260927/platform-roadmap.md#working-rules-and-immediate-next-step)

<a id="later-exploration-generic-math-interfaces--2026-09-24"></a>

- [Later exploration: generic math interfaces — 2026-09-24](history/planning-20260927/platform-roadmap.md#later-exploration-generic-math-interfaces--2026-09-24)

<a id="http-status-names-and-pattern-contracts--discussion-2026-09-25"></a>

- [HTTP status names and pattern contracts — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#http-status-names-and-pattern-contracts--discussion-2026-09-25)

## Author-directed calendar slice — 2026-09-27

The author requested useful date rendering with Gregorian/Hebrew calendars, Swedish and Hebrew cultures, invariant presentation and system discovery. The [provisional implementation and evidence](calendar-globalization.md) complete this bounded library slice without permanently reprioritizing later milestones. Unified localization stays separate, with shared interfaces and interchangeable providers/sources still to design.

Author follow-up, 2026-09-27: the next slice is the Time API and time-zone handling. DateTime and globalization have separate feature pages. This records sequencing, not implemented zone support.

### Time and named zones — implemented development slice, 2026-09-27

The selected [Time/zone slice](time-zones.md) adds TimeOffset, ZonedDateTime, named IANA
rules and explicit Unique/Ambiguous/Skipped mapping. DateTime is a nominal
parenthesized union of LocalDateTime and ZonedDateTime following the author's
clarification. Time wraps, civil addition carries the date, and elapsed Instant
addition checks overflow. The bounded 1900–2099 named-zone range and pinned 2025b
database are provisional. Broader rules/providers, parsing and scheduling remain open.

### Feature branch organization (2026-09-30)

At the author's request, structural Function work continues on
`codex/structural-types` with this Self-integrated main as its base. The former
`feature/function-types` name is retired after validation and synchronization.
The branch organization does not enable structural types on main.

Read-only callable import checkpoint (2026-09-30): Raven emission now consumes a
metadata definition snapshot without the producer builder graph. C# and Raven native
consumers return 42; [integration scope](raven-cli-bridge.md#read-only-call-imports--2026-09-30)
still excludes native symbol loading and production target registration.

Compiler adapter checkpoint (2026-09-30): the Raven consumer now calls an optional
compiler-owned emitter API with explicit contracts and diagnostics. Native verification
still returns 42; [integration scope](raven-cli-bridge.md#compiler-owned-native-adapter-checkpoint--2026-09-30)
retains the .NET input bootstrap and leaves native loading/target composition pending.

Multi-file adapter checkpoint (2026-09-30): cross-file function calls now execute
in both source-tree orders, with correct later-file diagnostics and no partial output
on validation failure. The native provider and production composition are still pending.

Native-input checkpoint (2026-09-30): a bounded native declaration reader now feeds
a temporary reference-only PE into Raven's existing semantic provider. The original
native dependency runs with all three emitted application variants to 42. See
[scope and replacement plan](raven-cli-bridge.md#native-dependency-input-through-a-reference-projection--2026-09-30);
a native symbol provider and production registration remain pending.

Raven-to-Raven dependency checkpoint (2026-09-30): the end-to-end producer is now a
Raven library, with a separately compiled Raven application consuming its native
metadata through the temporary reference projection. Overload/helper calls execute
to 42 and dependency failure cases are checked. Native symbol loading and wider
source/visibility support remain staged follow-up work.

Transitive runtime checkpoint (2026-09-30): three Raven-compiled native assemblies
load/verify/execute as a dependency chain to 42, in either supplied module order.
The native reader exposes direct exact references; missing and wrong-revision
transitive dependencies fail runtime verification. Runtime acceptance remains required;
this checkpoint is superseded by the direct container gate below; a native compiler symbol provider remains open.

Direct runtime metadata gate (2026-09-30): the same API-produced PE/#Neo library
files now feed Raven reference binding and neoCLR runtime loading. The two-library
chain and single/multi-file applications verify/run to 42. The runtime rejects missing
or altered recognition data and unsupported required execution schemas. This initial
profile carries format-5 JSON inside section 256/schema 1; it does not yet remove text
parsing or claim faster loading. Next evaluate binary native encoding and separate
load/link/verify timings, while staging the native compiler symbol provider. See the
[implementation and limits](design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).

Author-selected first acceptance programs (2026-09-30): Hello World directly in
the entry point, then through an entry-point call to another function. Both now
compile to PE/#Neo, load/verify/run in neoCLR, print exactly one line and exit zero.
The bounded Console string-literal bridge uses an explicit compiler reference
contract; general string signatures and no-result source entry points remain staged.

Binary-loading checkpoint (2026-09-30): execution schema 2 now carries bounded CBOR
and decodes directly into the runtime module model without a JSON intermediate.
Schema 1 remains readable; the experimental Raven PE emitter selects schema 2.
Hello World/function-call and the two-library cases pass. Native indexed tables, a
compiler-native symbol provider and broader signatures remain staged work. Compare
[measured loading phases](experiments/extended-cli-metadata/binary-loading.md) before
inferring performance gains; metadata loading and execution are separate costs.

Class-library consumer direction (author clarification, 2026-09-30): compile the
neoCLR runtime class library and load its symbols into Raven. Reuse present .NET-like
metadata through an explicit temporary projection while allowing native contracts
to diverge. The author proposes translating existing JSON into neoCLR assemblies as
a bootstrap; next test a bounded real class-library slice through that translation
and Raven reference loading. General translation is not implemented by the current
static-Int32 writer. Preserve unsupported information by rejecting it, then grow
coverage from the actual library instead of encoding permanent .NET restrictions.

Generic producer follow-through (2026-10-01, author-directed): unconstrained owned
function/static-method instantiations now execute through ordinary CLI MethodSpec and
native generic call arguments. C# producer tests cover forwarding, typed vectors and
nominal identity; API binary verify/run returns 42. Raven source integration now passes
the Order generic consumer in both source orders and on both targets. Generic types,
constraints, imported generic symbols and generic instance methods remain deferred.

Expanded generic acceptance now covers inferred/explicit calls, recursive forwarding,
multiple type parameters and overloads, generic array creation/iteration, conditional
values and object identity. Both source orders return 42 on .NET/native; five binding-valid
unsupported forms reject with source diagnostics and no output. [Recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes the tested runtime and consumer hashes. This is a bounded milestone; native
generic types, constraints, instance methods and imported generic symbols are still open.

Next generic receiver slice (2026-10-01): ordinary owned class instance methods now
preserve generic call signatures and receiver identity through API-produced CLI/native
binaries (42). Raven adapter integration and broader receiver consumers follow; generic
types/constraints and native symbol loading remain distinct next boundaries.

Expanded receiver acceptance: no-result generic copy/reverse methods, recursive
instance calls, receiver/argument evaluation order and separate receiver state execute
on both targets (42). [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes. Runtime checks also preserve generic receiver
and argument roots during forced collection; API validation rejects wrong/missing
receivers and generic constructor projections.

Generic-storage follow-through (2026-10-01): typed default producer support now passes
binary CLI/native execution. Local address/initobj/default helpers preserve method
parameter scope and definite assignment. Raven default(T) and generic clearing are
the next bounded integration, before generic owner/constraint and import work.

Typed default integration now passes Raven-to-.NET/neoCLR execution, including generic
numeric clearing and null-reference faults after clearing object vectors. The producer
API also preserves branch offsets around initialization and rejects invalid local-address
provenance. This completes the bounded receiver/default milestone; generic owners,
constraints, imported generics and full class-library compilation remain open.

Static generic owner producer slice (2026-10-01): independent declaring-type and method
parameter scopes now emit ordinary CLI VAR/MVAR, constructed TypeSpec/MemberRef and
MethodSpec calls. Native open/constructed owner records already support this behavior;
C# producer binaries verify/run 42 without a runtime change. Reference projection
retains type/method arities. This follows the existing [generic metadata contract](generic-metadata.md)
rather than flattening owner parameters into method parameters, which would lose type
identity. Generic instance type layouts and fields remain the next larger boundary;
Raven source integration follows this API slice.

Raven static generic owner integration now verifies/runs 42 on both targets and in
both source orders. Explicit capabilities preserve independent owner/method arguments;
the .NET resolver also fixes source method calls that previously left their declaring
type open. Tested matching producer: `0da5a3b0`, receiver runtime `6a7a0dd2` or later
on `codex/extended-cli-metadata`; Raven `codex/metadata-consumer`. No Runtime Contract
configuration change. Generic owner arrays/defaults and alias mutation pass; generic
instance layouts/fields, constraints, imported owners and full class-library compilation
remain open. See [binary evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).

Static generic owner milestone complete: multiple/reordered owner arguments and
method-to-owner forwarding execute across targets; native reader scope/name checks
reject malformed declarations. Next bounded emission work remains generic instance
layouts and fields before constraints/imports/full class-library acceptance.

Generic instance class producer slice (2026-10-01): AddGenericClass and constructed
class signatures now support instance constructors, owner-typed fields and calls on
exact constructed receivers. CLI uses standard GENERICINST/VAR/member references;
native uses the existing generic record/reference contract, preserving field layout
and type identity. API-produced binaries verify/run 42 without runtime changes.
Raven integration follows. External constructed fields, generic properties, constraints
and full class-library emission remain open. This reuses the [generic metadata design](generic-metadata.md)
rather than creating a second generic storage representation.

Generic instance storage integration now passes: Raven shares generic class value,
constructor and field-body planning; native emission binds owner and method arguments
independently. Primitive, Order and nested Box<Box<int>> consumers verify/run 42 on
.NET/neoCLR in both source orders. Matching producer: `62bf5931` or later; runtime
`6a7a0dd2` or later on `codex/extended-cli-metadata`, Raven `codex/metadata-consumer`.
No Runtime Contract configuration change. Generic properties and external constructed
field references are explicit remaining bridge limits, followed by constraints/imports
and broader class-library acceptance. [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes; this does not establish full-library emission.

Generic property producer slice (2026-10-01): generic static/instance property values
and index parameters now preserve VAR scope and accessor associations through standard
CLI Property/MethodSemantics and existing native Constructed owners. Reader validation
requires exact canonical open owner identity/arguments. API reflection execution and
native binary verification/run (42) pass; no native schema or Runtime Contract change.
Raven accessor integration follows; external constructed fields and constraints/imports
remain separate work. This extends the existing [generic metadata contract](generic-metadata.md).

Generic property/indexer integration is now verified through Raven: setter/getter calls,
Order alias mutation and independent generic key/value parameters run 42 on .NET and
neoCLR in both source orders. Matching producer/reader: `dbe03b1a` or later on
`codex/extended-cli-metadata`; Raven `codex/metadata-consumer`; receiver runtime
`6a7a0dd2` or later. No Runtime Contract change. Fifteen focused C# tests pass;
[recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes consumer/runtime hashes. External constructed fields, constraints and generic
imports remain open, along with broader class-library acceptance.

Constructed-field producer slice: typed field references now bind generic owners and
validate exact receiver/value types before emitting standard CLI Field MemberRefs or
existing native field operations. No schema/Runtime Contract change. Raven integration
follows; the author directs generic type constraints immediately after this slice.
Constraints must preserve CLR/native semantics rather than translating unlike flags.

Author-directed type-constraint slice (2026-10-01): owned nongeneric class bounds now
use CLI GenericParamConstraint and existing native TypeBound, preserving meaning rather
than mapping CLR flags to native notvoid/notreference. Producer/reference projection and
binary runtime verification/execution pass; invalid concrete native arguments reject.
Raven integration follows. The author requests the other constraint categories next;
class/struct/new/nullability and interface/dependent bounds require distinct contracts.
This reuses the existing [generic contract research](generic-metadata.md).

Typed boxing continuation (2026-10-03): the metadata IL generator now exposes Box and
raw Emit(Box, signature), producing standard CLI and existing native boxing instructions.
Raven's target-owned capability lowers value/generic-to-object conversions without
changing binding; .NET retains its existing backend. The normal native command executes a generic box
smoke case with the explicit core/retained-System seed. C# API tests separately observe
value dispatch, primitive display and reference identity; the smoke case alone does not
prove these semantics. Union preflight now stops at generated value-payload addressing;
Object virtual calls/formatting and union metadata preservation remain open. Runtime
Contract settings and format versions are unchanged. No implicit reference fallback.

An independent .NET arrow-method bug exposed by this regression work is fixed by
consuming the existing bound return block, retaining implicit generic boxing. Ten tests
pass both on the integration line (Raven 697a093d7) and the main-based fixes branch
(c96305e50); main is unchanged. No new language/Runtime Contract rule is introduced.

See [typed-boxing evidence](experiments/extended-cli-metadata/typed-boxing-2026-10-03.json)
for commands, source/artifact hashes and cross-repository validation (Raven 558462967).

Owned field-address continuation (2026-10-03): metadata IILGenerator now supports
LoadFieldAddress/Emit(Ldflda) for owned mutable definitions and constructed field references.
Raven admits a separate FieldAddress capability carrying only IFieldSymbol; its adapter
resolves owned field handles. Getter/method receivers nested in value fields use the
actual storage; no spill-copy workaround is introduced. Imported/readonly field addresses
remain explicit limits. Ordinary .NET stays on its existing backend. API and ordinary
compiler-command cases execute nested generic mutation through object aliases (42) on both
targets; 120 C# metadata groups and ten focused Raven tests pass. No runtime/format or
Runtime Contract configuration change. Unchanged Option and the source-union controls
now reject at `<RavenFormatUnionValue>` BoundBinaryExpression (null comparison), with no
native output. Null/reference operations, remaining formatting and union/case metadata
preservation are next; full union execution is not claimed.

[Field-address evidence](experiments/extended-cli-metadata/owned-field-addresses-2026-10-03.json)
records commands, artifact/source hashes and Raven c951e33c0.

Reference-test continuation (2026-10-03): the metadata IL generator now supports IsNull,
IsInstance and String checked casts. CLI uses standard ldnull/ceq, isinst and castclass;
native uses existing ref.isnull/isinst/castclass, with no runtime or format change. Raven
has explicit null-test/discard-type-test capabilities; non-user-defined null comparisons
reuse bound operator facts. Overloaded equality is not replaced. .NET retains its general
backend and Runtime Contract settings are unchanged. API and normal driver executions
return 42 on both targets; 121 metadata groups and focused capability tests pass. Source
union preflight now stops at Object.ToString dispatch inside the formatting helper, with
no native output. Wider patterns, core virtual dispatch, remaining formatting and native
union/case metadata remain open.

[Reference-test evidence](experiments/extended-cli-metadata/reference-tests-2026-10-03.json)
records commands/hashes and Raven 5bb8f4dfc.

Development update (2026-10-03): core Object.ToString dispatch now executes for boxed
values through the metadata API and ordinary Raven commands. Union preflight next
requires synthesized null literal emission; native union execution is still pending.

Typed null continuation (2026-10-03): Raven ad3d8a71f emits contextual reference nulls
through the existing metadata LoadDefault operation. Return/local/argument driver cases
execute on both targets; 11 focused tests pass. Plain/generic unions reach the native
union/case metadata gate. Unchanged Option next rejects its TryGetOutput case pattern.
No metadata/runtime change or union execution is claimed. See
[null evidence](experiments/extended-cli-metadata/typed-null-2026-10-03.json).

Option body continuation (2026-10-03): Raven a99cd3c3e reuses existing case-pattern
lowering in conditional branches and Boolean values, and emits RuntimeUnitContract's
inhabited value as a nominal default. Native configured unit out/value arguments
execute (42); .NET configured ValueTuple storage also executes. Existing CLI/native
instructions and runtime mappings are unchanged. Unchanged Option and its dependencies
now pass source-body preflight and reach the union/case metadata publication guard.
Next implement general custom-attribute authoring/reading and introspection, retaining
Raven's .NET UnionAttribute, case-name/ordinal and generic companion contracts through
the existing runtime custom_attributes representation. No union-only wire format or
unmarked-struct fallback is planned. Encoding and native union execution remain unproved.
[Case/unit evidence](experiments/extended-cli-metadata/union-case-unit-2026-10-03.json).


### 2026-10-04 capability-batch progress

The [System strategy](experiments/extended-cli-metadata/system-compilation-strategy.md)
unit subgate now executes unchanged MemoryStream from a separate native library,
including Result<unit, E>, shared interface identity and byte mutation. Generic-unit
storage/calls and 38 focused .NET controls pass. Erased Value/service ABI work remains
next. A cumulative single PE also exposes the existing 1 MiB envelope bound; separate
libraries remain supported. This is not full-System completion.

### Source storage execution (2026-10-05)

The remaining eleven production storage sources now compile against the cumulative
115-input library using explicit native service adapters. The unchanged storage-poc
sample executes from emitted references: creation, UTF-8 reads, seek/re-read,
listing, child lookup and error cases. An additional FileText consumer checks bounded
writes/reads and preservation of file contents on rejected writes. This closes the
seven missing storage service declarations from the post-JSON audit.

Raven's portable emitter preserves empty-stack context through value-block wrappers;
the metadata writer admits external value types in module-function references using
existing encoding. Neither change alters Runtime Contract configuration, ordinary
.NET semantics or primitive/source ownership. The next high-unlock family remains
DNS/socket service contracts and their imported Error identities, followed by full
source/bootstrap ownership. No complete-System or dual-target gate is claimed.
See `docs/experiments/extended-cli-metadata/source-storage-2026-10-05.md` for exact
artifacts, commands, source hashes and validation.

### Source DNS/socket execution (2026-10-05)

Six production DNS/IPAddress/socket sources now compile into a separate native library
with explicit service adapters. The unchanged network-cancellation sample executes
against that artifact and the cumulative source library: localhost DNS, pre/pending
cancellation, loopback accept/connect, transfer buffers and resource reuse all pass.
Raven admits converted temporary value receivers and immutable by-value parameter
captures through its existing adapters; 59 focused .NET checks pass. There is no new
Runtime Contract switch, instruction encoding or nominal delegate representation.

The apparent imported Error-to-Error diagnostics disappear once service declarations
are present; no union identity workaround was added. The full networking/web source
group now reaches an unlowered BoundPropagateExpression. Prioritize Result propagation
in HTTP next, then reassess full-source/bootstrap ownership. This is native execution
evidence, not .NET class-library or full-System completion. See
`docs/experiments/extended-cli-metadata/source-network-2026-10-05.md`.

### Author-directed release gate: .NET comparison after bootstrapping (2026-10-05)

After the entire runtime class library compiles, benchmark representative neoCLR
programs against equivalent .NET programs before releasing this integration. Publish
the methodology, reproducible sources/commands, measured results and limitations on
the website with the release. This is a future release requirement, not current
performance evidence or a diversion from completing bootstrapping.

Compare observable workloads and outputs, accounting explicitly for text-model and
other intentional semantic differences. Record compiler/runtime revisions, release
build configurations, hardware/OS and dependency artifacts. Separate startup/assembly
loading from warmed execution; report warmup/JIT policy, repeated measurements and
variation. Include memory/allocation measurements where comparable, and distinguish
host/native resources from managed allocations. Retain raw data and correctness checks;
choose representative computation, collections/text/JSON and I/O workloads after the
full-library gate rather than selecting only favorable cases. Do not infer runtime
speedups from assembly size alone. Website publication remains a release action.

### HTTP condition propagation (2026-10-05)

Shared Raven lowering now handles nested propagation in HTTP's short-circuit conditions.
A native artifact-only consumer executes skip/success/error paths for AND/OR, including
one-time side effects. Integration validation passes 21 propagation tests and 65 focused
shared-emitter/runtime-contract tests. No runtime, metadata format or ownership change
is needed. Full HTTP source compilation advances to callback emission admission; that
is the next bounded task before HTTP execution. Full-System bootstrapping remains open.
Evidence: `docs/experiments/extended-cli-metadata/condition-propagation-2026-10-05.md`.

### Author-directed release gate: editor and native metadata (2026-10-05)

The author added a working editor experience to the release objective: language-server
support consuming and emitting NeoCLR metadata, including VS Code. This is a release requirement alongside
class-library bootstrapping and the recorded .NET comparison benchmark gate. It does not
replace the current core ownership prerequisite work. Implementation is not claimed yet.

The editor/LSP should use Raven's existing semantic pipeline and native metadata importer.
Editor build/run commands should use the ordinary compiler's NeoCLR emitter and dependency
catalog; do not build a separate metadata writer inside the language server. Preserve the
.NET target's existing editor behavior and the independent importer/emitter boundaries.

Acceptance should cover an ordinary NeoCLR project referencing source-built System and a
separately compiled library with its sources absent: diagnostics, completion, hover and
symbol/declaration navigation expose imported signatures and members; reference updates
invalidate affected semantic state; missing or conflicting dependencies produce useful
project diagnostics. Build from that editor/project configuration emits native assemblies
that the runtime loads and executes. CLI and editor must agree on target, bootstrap,
reference identities and artifact revisions, with no silent CLI metadata projection.
Run a focused .NET editor regression alongside the native integration case.

The author also suggested “perhaps also having the disassembler.” Keep a bounded read-only
command-line disassembler as a release candidate to assess: assembly identity/dependencies,
type/member signatures and flags, tokens and decoded instruction bodies/branch targets,
using the existing metadata reader. Source reconstruction/decompilation is a separate
scope. Decide whether the diagnostic benefit and implementation cost justify inclusion;
no implementation, release commitment or editor integration is asserted for it yet.

### Author-directed release gate: samples, Tasks and await (2026-10-05)

The author requires release samples to execute successfully, explicitly including Tasks
and `await`. Existing Tasks library/callback consumer evidence and older translated
async experiments do not complete the native compiler/emitter gate. After source-root
ownership wiring, assess the checked-in samples through ordinary native compiler
commands and prioritize async lowering/emission gaps that block them. Preserve the
working .NET path and use the current runtime execution model; do not make runtime
suspension or green threads prerequisites. Those are future exploration, not this release.

Record a concrete sample inventory with expected output and termination. Native coverage
must include async entry points, result-bearing and completion-only await, composition
and the actual scheduling/error/cancellation behavior exercised by those samples.
Distinguish an already-completed task from work that completes later when validating
continuations. CLI and editor build/run must use the same target artifacts and settings.
Compared with .NET, share Raven's established async semantics/lowering where applicable
and adapt representation at the target boundary; do not introduce new scheduling promises.


#### Initial native semantic editor gate (2026-10-05)

Explicit native `.rvnproj` references now reach the language server through an optional
metadata provider. A separately emitted library supplies completion, hover and
missing-member diagnostics over real stdio; twelve ordinary project checks pass.
[Evidence and reproduction](experiments/extended-cli-metadata/native-project-editor-2026-10-05.md).
This advances the author-directed editor gate only. Next: native reference refresh
without CLI substitution, declaration navigation, source-built System/async ownership
configuration, then shared native project build/run and VS Code acceptance. None of
those remaining gates is implied by the small library fixture.


### Native arithmetic sample follow-up (2026-10-08)

Wide division/remainder now pass 72 interpreted/native cases with width-correct fault
checks. `library-calendar` joins the passing sample baseline; time-contracts/time-zones
still expose other boundaries. [Evidence](experiments/aot-sample-assessment/division-followup.json).
Continue bounded fixes without making universal sample parity a release requirement.


### Declaring-library documentation follow-up (2026-10-09)

Author-directed RavenDoc migration now renders Runtime/Data/Networking/Web metadata
after assembly-member documentation fixes. The 1,632-page audit demonstrates real
owner identities, not coverage parity or native-only bootstrap. Preserve the full
reference while comparing public members, documentation and routes.
[Evidence and remaining gates](experiments/native-library-documentation-audit/README.md).


The author clarified that the updated neoCLR website belongs to the upcoming release.
Native documentation must use canonical assembly-member IDs, retain XML help and
sidebar entries, resolve Object/member links, omit nonexistent CLR containers, and
work at normal API URLs. The local integrated preview exercises these; production
coverage and no-bridge qualification remain open. A fresh bundle closes the old
bundle's missing Math constants; an intermittent HttpContext metadata validation
failure seen on the first build still needs investigation. See the documentation audit's integration procedure.

Native JSON evidence update (2026-10-09): the public
[document corpus](experiments/native-json/README.md#representative-document-checks)
now validates nested object and DOM round trips on macOS in both modes. This also covers typed scalar/object vectors. Lists/sequences, maps, jagged native
arrays remain open. Windows project qualification passes at `91911ce9`
([evidence](windows-native-json-validation.json)).

**Migration gap closure (2026-10-10):** missing Int64 native parsing and Char/String
factory projection are fixed and their native exclusions removed. Raven long/String
constant-pattern emission is also fixed; restored payload patterns pass the five
MemoryStream and twelve JSON DOM cases in both local modes. Primitive parsing is also fixed;
Object equality/hash dispatch remains implementation work before the ten-batch
migration can be called complete. Windows execution of these fixes remains pending.

**Timer layer (2026-10-11):** public Timer accepts synchronous and `Task<()>` callbacks,
with fixed-delay non-overlapping scheduling across restart, stop and disposal. Ten
interpreter/macOS ARM64 scenarios pass; Windows CI is configured, execution pending.
See [the contract](design/http-client-lifetime.md#public-timer-and-asynchronous-callbacks-2026-10-11).
Author direction is to continue through timed cancellation and HttpClient.Timeout
before returning to Index/Range.

**Timed cancellation layer (2026-10-11):** CancelAfter now composes Timer with source
lifetime, replacing countdowns and allowing explicit None disabling. HttpClient.Timeout
is next; the native host watchdog and old transport exchange budget must be removed
from that request-timeout path before claiming its configured semantics.

**HTTP timeout layer (2026-10-11):** HttpClient.Timeout now composes per-request timed
cancellation with handler cleanup, a 100-second default, explicit disabled state,
frozen configuration and typed timeout outcomes. Nine handler scenarios, six guards,
a native 16-second request and two real-network cases pass locally in the applicable
interpreter/macOS modes ([evidence](experiments/http-timeout/validation.json),
[network](experiments/http-timeout/network-validation.json)). Windows remains pending.
The independent 15-second client/host limits are removed; shorter transport phase
bounds remain tracked gaps. Author follow-up is HttpMethod with predefined values,
parsing and custom construction before returning to Index/Range.

**HTTP method values (2026-10-11):** HttpMethod supplies predefined tokens, a custom
constructor, typed Parse and .NET-style case-insensitive value equality. HttpRequest
now exposes the value and Create overloads; custom wire spelling survives client and
server processing. Eight framework cases and six wire/guard cases pass in each local
mode ([values](../runtime/raven/tests/http-method-values/validation.json),
[wire](experiments/http-method/validation.json)). CONNECT tunnelling remains unsupported.
Windows CI is configured, execution pending. The requested timeout/HttpMethod increment
is complete locally; Index/Range remains the next author-directed feature.


### Author-directed error composition follow-up (2026-10-13)

The bounded API review following ad-hoc error propagation selects HTTP JSON client
helpers: they combine independent HttpError and JsonError streams without adding a
new domain. Their development signatures now expose those alternatives directly;
nominal domain errors and application boundaries remain. The style guide and tested
examples describe when this improves readability. Raven main fixes inline generic
union binding; the native branch carries it and fixes runtime-await residual-return
completion. See [review decisions](raven-error-api.md#review-existing-api-aggregation)
and [focused validation](experiments/http-json-client/README.md). This author-directed
slice does not change milestone sequencing or claim native structural union support.


### Author-directed generic composition experiment (2026-10-14)

The isolated composition branches now cover the author's nested generic union
examples semantically and execute closed generic reference bounds in runtime tests.
Full generic metadata/compiler/storage integration remains open. This experiment
does not change active milestone sequencing or enable compositions on main.
[Scope and focused evidence](experiments/structural-compositions/README.md#generic-composition-first-slice--2026-10-14).

Follow-through: closed generic reference bounds now cross the native metadata
adapter and Raven call emitter, with eight passing interpreter consumers. Generic
argument composition/storage remains open; see the [call-integration checkpoint](experiments/structural-compositions/README.md#closed-generic-composition-call-integration--2026-10-14).

The next bounded runtime slice stores closed reference compositions as arguments of
reference-class wrappers, including nested payloads across GC. Host/compiler wrapper
transport and owned calls remain open; this is not Task execution.
[Checkpoint](experiments/structural-compositions/README.md#runtime-composition-argument-storage--2026-10-14).

Runtime wrapper constructors/nonvirtual methods now pass focused execution and
substituted-body checks. The author also explicitly requires union constraints,
with intersections especially useful for inputs and constraints. Union disjunction
and intersection conjunction must remain distinct; native constraint integration
is pending. This does not change milestone sequencing.
[Checkpoint and direction](experiments/structural-compositions/README.md#runtime-wrapper-constructormethod-calls--2026-10-14).

Wrapper signatures now cross the experimental metadata and Raven imported-call
paths, with twelve executable host/Raven consumers. Native constraints, open
parameters, source storage and real Task integration remain pending. Intersection
use-site guidance does not restrict return types.
[Checkpoint](experiments/structural-compositions/README.md#wrapped-composition-metadata-and-raven-calls--2026-10-14).


The isolated composition interpreter now admits checked reference-array payloads
and substituted generic array fields, a prerequisite for Task's ArrayList-backed
result slot. Compiler/host array transport and actual Task execution remain open.
[Runtime array checkpoint](experiments/structural-compositions/README.md#runtime-reference-array-storage--2026-10-14).


The interpreter now admits composition arguments in generic interface identities,
allowing the real Task/Promise/ArrayList construction path for the nested union
example. Completion remains blocked by substituted managed-local initialization;
interface dispatch and compiler transport remain separate work.
[Task construction checkpoint](experiments/structural-compositions/README.md#generic-interface-composition-arguments-and-task-construction--2026-10-14).


The composition interpreter now admits exact, immediate managed-local default
initialization after substitution, removing the observed ArrayList/Task completion
blocker while retaining general byref gates.
[Initialization checkpoint](experiments/structural-compositions/README.md#immediate-compound-local-initialization--2026-10-14).


The real Task/Promise/ArrayList interpreter consumer now completes and reads each
alternative of Task<Response | HttpError | JsonError>, preserving payload identity,
first-completion semantics and GC retention. Pending/cancelled reads and out-of-union
payloads remain rejected. Raven source/await integration is still a separate step.
[Task delivery evidence](experiments/structural-compositions/README.md#real-task-union-completion-and-result-delivery--2026-10-14).


Explicit imported-result locals now preserve native composition signatures in the
host builder; Raven source-local/await integration remains pending. Structural types
are intended as fully introspectable runtime types: tuples (later), unions,
intersections and functions, with arrays retaining Array<T> backing.
[Result-local evidence and type model](experiments/structural-compositions/README.md#imported-composition-result-locals--2026-10-14).

The companion compiler now executes imported-result source locals/copies; fourteen
paired consumers pass. Await/state-machine composition storage remains the next slice.

The author-directed composition experiment now integrates Raven await lowering with
imported Task union contracts and native reference-class state storage. See
[await delivery](experiments/structural-compositions/README.md#raven-composition-await-delivery--2026-10-14)
for scope and validation; this does not change mainline milestone priorities.

The experiment's shared reflection relationship core now preserves composition
assignment guarantees and nominal-only strict ancestry; guest descriptors remain
open. [Bounded relationship slice](experiments/structural-compositions/README.md#composition-relationship-core--2026-10-14).

Resolved host composition identity keys now preserve constituents and generic
arguments without nominal definition tokens. Guest descriptors remain pending.
[Identity checkpoint](experiments/structural-compositions/README.md#resolved-composition-identities--2026-10-14).

The experiment now exposes recursive host composition descriptors, including
constituents within closed generic arguments; guest providers remain gated.
[Host descriptor checkpoint](experiments/structural-compositions/README.md#host-composition-descriptors--2026-10-14).

Author correction reserves TypeInfo.IsUnion for structural identity and separates
nominal detection as NominalTypeInfo.IsUnionDeclaration. Existing nominal consumers
are migrated; broader nominal-helper grouping and guest structural providers remain
follow-up work. [Classification correction](experiments/structural-compositions/README.md#author-correction-structural-isunion--2026-10-14).


Guest UnionTypeInfo and IntersectionTypeInfo now expose direct constituent snapshots
in the experimental interpreter. This supersedes the guest-provider gates in the
preceding checkpoints. Member/hierarchy queries, source structural typeof emission
and backend support remain open. [Descriptor probe](experiments/composition-type-info/README.md).

2026-10-14 author-directed experiment: companion Raven now emits opt-in source union signatures to standalone NEOX, with native Task result delivery and reflection validated. See [source union evidence](experiments/source-structural-unions/README.md). This does not enable published/default builds or JIT/AOT.

2026-10-14 author-requested Raven intersection slice complete: direct source signatures, trait dispatch, reflection and cross-library consumers pass; [evidence](experiments/source-structural-intersections/README.md). Production/open-generic/backend support remains future work.

2026-10-14 author-directed constraint work: [native constraint checks and companion compiler semantics](experiments/composition-constraints/README.md) implemented as a bounded experimental slice. Native Raven emission/reflection remain explicit follow-up work.

2026-10-14 native metadata constraint follow-up: host inspection now preserves closed
structural method bounds and exposes their complete type views. Native authoring,
Raven emission/import, class constraint materialization and guest constraint reflection
remain pending; [focused evidence](experiments/composition-constraints/README.md).

2026-10-14 bounded native method constraint authoring is implemented with local closed
reference bounds, checked instantiation and interpreter result 42. Compiler wiring,
external/open bounds and guest constraint reflection remain pending; see the
[constraint evidence](experiments/composition-constraints/README.md).

2026-10-14 native structural method constraints now pass Raven emission and separate
library consumption, including a closed generic bound and interpreter result 42.
Open bounds/forwarding, class constraints, generic-body traits and broader backends
remain experimental gaps; [evidence](experiments/composition-constraints/README.md).

2026-10-14 constrained method wrappers now forward parameters when their closed
bounds imply the callee’s requirements. Native and Raven library-consumer execution
return 42; [evidence](experiments/composition-constraints/README.md). Open operands,
owner-parameter forwarding and generic-body traits remain separate gaps.

2026-10-14 bounded generic-body trait use: structural method bounds now authorize
owned nongeneric interface calls; the Raven intersection body returns 42. Union
alternatives must all guarantee the requested trait. [Evidence](experiments/composition-constraints/README.md).
Broader generic/target/backend support remains outside this experimental slice.

2026-10-14 imported nongeneric trait targets now pass structural-guarantee admission:
separate Raven library/consumer execution returns 42 using local intersection bounds
that inherit imported interfaces. External bound authoring remains pending.
[Evidence](experiments/composition-constraints/README.md).

2026-10-14 closed constructed imported trait targets now pass native structural proof
and Raven execution (42). Exact type arguments are required; weaker unions reject.
See the [constraint evidence](experiments/composition-constraints/README.md).

2026-10-14 external union-bound follow-up: native method authoring and Raven's
existing opt-in mapping now preserve imported closed reference alternatives,
including constructed interfaces. A three-assembly consumer returns 42; shape-only,
mismatched and partial-union arguments reject. External intersection-bound authoring
and open operands remain gaps. See [evidence](experiments/composition-constraints/README.md#external-union-constraint-alternatives--2026-10-14-development).

2026-10-14 external intersection-bound follow-up: imported closed interfaces and
open reference classes now support method intersection authoring and resolved native
inspection. Raven retains nominal conformance, resolves selected dependencies in any
input order, and emits closed generic interface conjunctions. The three-assembly
consumer returns 42, including an imported open-class bound. Host checks retain
closed-hierarchy admission and reject CLR-final/value or unresolved/mismatched bounds.
[Evidence](experiments/composition-constraints/README.md#resolved-external-intersection-bounds--2026-10-14-development).


**Author-directed range follow-up (2026-10-15):** signed int/long native range
predicates now have a bounded [qualification record](validation/range-patterns/README.md).
Checked inclusive Range conversion, safe iteration and generic capability design
remain subsequent work; this does not reprioritize the platform milestones.


The range correctness follow-up now includes [checked inclusive conversion](validation/checked-range/README.md)
and [signed integer domain termination](validation/range-iteration/README.md), with
main-first compiler work carried separately to the native line. The author's
[approved capability direction](design/slicing-capabilities.md) separates Result-based
safe slicing, optional faulting indexers, bounds and explicit enumerable progressions.
System.Sliceable<TSlice> now provides the nominal safe slicing capability, with
Array<T>/ArrayList<T> implementations and faulting range indexers sharing Range
validation; see [native qualification](validation/sliceable/README.md). Generic
Interval/progression APIs and other numeric-domain stepping remain unimplemented.


**Variance follow-up (author direction, 2026-10-15):** investigate nominal interface
variance and safe result-value conversion beyond .NET's reference-only model,
using Sliceable<int[]> to Sliceable<Iterable<int>> as an acceptance case. Retain
restrictions that protect writable state and validate nested parameter positions.
The [research and staged plan](design/interface-variance.md) now has validated
runtime metadata and bounded interpreter reference conversions, with a
[retained consumer](validation/interface-variance/README.md). Native specialization now retains verified interface/slot identities; the separately compiled Raven consumer passes interpreter, AOT and managed JIT. SDK annotation authoring, inspection, CLI/native projection and snapshot round trips
are supported. Raven annotation transport and native dispatch qualification are validated; guest generic-parameter introspection and eleven audited library annotations are qualified. Closed indexed and sequence-returning generic argument queries now pass interpreter/AOT/JIT against a fresh class-library bundle; open-parameter sequence/variance inspection remains qualified in the interpreter. Native source catalogues also retain method generic constraints and distinguish constructions while preserving declaration identity; this build-time retention does not close guest discovery. Compiler-owned descriptive handles now preserve open-definition/parameter identity through selection and native lowering without executable layouts; source-backed name/count/argument/token/shape query wiring is validated. Shared assembly/module discovery recipes now preserve generic and nongeneric definitions with exact module ownership checks; explicit schema-6 native discovery binding now lowers assembly/module recipes with identity-preserving snapshot tokens. Display-name/namespace queries now bind against source identities; static-owner tokens are now admitted without allowing instance storage. The retained Raven consumer now passes interpreter/AOT/JIT for assembly/module discovery, parameter variance and identity, namespace filtering and concrete closed arguments with explicit type-catalog retention. Open generic member/relationship reflection remains an explicit gap. String variance now adapts arguments/results through existing identity-preserving Object/interface representations; interpreter handles remain heap-budgeted and native pointer adaptation avoids payload copies. SDK variance now reuses ordinary explicit imported-base ancestry for local class arguments; focused native metadata/CLR checks and a separately compiled Raven interpreter/AOT/JIT consumer now pass, including identity, mutation and rejected reverse/unrelated conversions. Native Object-root reference identity now survives private renaming; interface-valued producer/consumer widening to Object is qualified in interpreter/AOT/JIT. Array-valued arguments are also qualified in producer/consumer Object views with preserved identity and mutation, without element boxing. Closed array-backing conformance now survives native specialization, qualifying array-to-Iterable arguments in producer/consumer calls with identity and iteration across all three modes. Imported class arguments now also inherit interface views through explicitly registered dependency base chains; SDK rejection/CLR checks and the separately compiled Raven interpreter/AOT/JIT consumer pass with identity and mutation checks. Generic dependency base contracts now substitute scoped arguments through transitive ancestry for reference/variance checks, with SDK and emitted CLR validation. Local native generic base-signature materialization and open/closed host introspection now pass focused tests, including inherited interface enumeration. External constructed bases now also pass host reader/introspection checks with exact dependency identity and substituted interface views. SDK authoring of generic classes over ordinary nongeneric bases now passes CLR/interpreter/AOT/JIT with inherited storage and cleanup checks. Raven source emission of generic classes over local nongeneric bases is now enabled with retained consumer evidence in docs/validation/generic-class-base/raven; separately compiled generic derived owners are also admitted with local/imported execution evidence and invariant conversion rejections; SDK owned constructed generic bases now pass CLR/interpreter/AOT/JIT with reordered arguments, transitive storage, value/reference payloads and inherited interface dispatch; Raven owned constructed-base emission now retains exact constructor arguments and schedules original base definitions; imported constructed chains now retain exact ancestry and complete inherited field offsets; recursive source base arguments and external generic-base derivation now pass retained interpreter/AOT/JIT consumers, including protected initialization; native scalar constant/arithmetic base arguments are now admitted with receiver-protecting validation; helper calls and control flow in base expressions remain backend gaps. .NET behavior is the default, not a reason to retain implementation restrictions. The current author-directed scope is minimal
reference-only interface variance. The author explicitly postponed both Sliceable
and Result variance on 2026-10-16; neither is an acceptance gate for this bounded
feature. Result-value widening and Sliceable adapters are deferred. Read-only producer/comparison interfaces opt into variance; mutable contracts, Result and Sliceable remain invariant. This does not resume
structural-type work or change unrelated milestone priorities.

**Recursive generic base authoring (2026-10-16):** SDK deferred base binding now
passes CLR/interpreter/AOT/JIT self-reference storage checks and rejects actual
inheritance cycles. See [retained evidence](validation/constructed-class-base/README.md).
Raven also emits recursive local and external base arguments; the retained
source consumer exercises both generic and nongeneric recursive owners.

**External generic base authoring (2026-10-16):** SDK construction, initialization,
inherited storage and nominal interface dispatch now pass CLR/interpreter/AOT/JIT.
See [the qualification record](validation/external-constructed-class-base/README.md).
Raven source classes over external generic bases, including recursive arguments
and protected initialization, also pass a separate-provider interpreter/AOT/JIT
consumer. Generic class virtual slots and broader constrained dependency contracts
remain open; structural work stays parked.

**Generic virtual slots (2026-10-16):** concrete generic reference-class SDK
slots/overrides and native closed dispatch are qualified in CLR/interpreter/AOT/JIT,
including reordered generic base parameters and direct base calls. See
[retained evidence](validation/generic-class-virtual/README.md). Imported generic virtual contracts and direct base calls also pass the separate-provider [fixture](validation/external-constructed-class-base/README.md). Raven emission/import now passes local and separate-provider [native consumers](validation/generic-class-virtual/raven/README.md), including direct base calls and value/reference dispatch. Abstract generic classes and broader constrained dependency contracts remain open.
Structural work remains parked.

**Abstract generic bases (2026-10-16):** SDK authoring and native execution now pass
CLR/interpreter/AOT/JIT, including inherited abstract slots and rejection checks.
See [qualification](validation/generic-class-virtual/README.md). Raven local/imported abstract contracts now pass the [native consumers](validation/generic-class-virtual/raven/README.md), with capability/CLR tests and exact generic arguments. Broader constrained dependency contracts remain open; structural work stays parked.

**Constrained specialization prerequisite (2026-10-16):** native AOT/JIT now
validate reference/value/default-constructor requirements before specializing closed
record/class constructions. The [retained consumer](validation/special-type-constraints/README.md)
covers inherited and nested constructions; negative tests cover invalid arguments.
Nongeneric nominal class bounds now also validate before specialization. Raven dependency import/use of the existing reference-class special/nominal constraint profile is qualified by the [separate-provider fixture](validation/imported-special-constraints/raven/README.md). Constrained methods, constructed/interface bounds and external source-bound authoring remain open. This does not resume structural types or broaden the variance scope.


**Author-directed function-kind groundwork (2026-10-16, feature branch only):**
`func` is intended to constrain structural function types for ordinary library
function lists/signals, without event metadata. Native runtime/host SDK groundwork
is isolated on `codex/function-kind-constraint`. Raven now separates structural
function symbols from nominal delegates and emits `where T: func` for native
generic classes, including separate-provider import. See the [consumer evidence](validation/function-kind-constraint/README.md).
The host SDK now retains method FunctionType constraints through authoring, snapshots, introspection and imports, with argument/proof validation and fail-closed CLI export. Native AOT/JIT method specialization now validates function-kind requirements, preserving source declarations. Raven method emission/import and separate-provider forwarding now return 42 in interpreter/AOT/JIT; signature bounds remain open. Invocation decisions are explicitly
postponed. This is not a resumption of the broader structural roadmap. See
[direction](proposals/structural-types.md#20-generic-constraints-on-function-types).


**Author-directed provisional invocation primitive (2026-10-16, feature branch):**
Following function-category constraints, the author requested usable dynamic
function invocation without freezing the API. The runtime primitive and sample-local
Raven facade use checked boxed arguments and existing typed calls, qualified in
interpreter/JIT/AOT. See [evidence and .NET comparison](validation/dynamic-function-invocation/README.md).
This bounded work does not resume general structural types or add event policy.

The function follow-up also qualifies native structural `typeof` tokens and
Object.GetType recovery from selected function bindings. The Raven consumer checks
exact type identity and dynamically forwards a function argument/result before
invocation. Detailed native method-snapshot/generic reflection remains separate.


The bounded function follow-up also qualifies Raven source-level type patterns,
including generic `() -> T` tests and capture/invocation. The shared parser fix was
validated on Raven main using standard .NET delegates, then carried deliberately to
the native feature branch; the retained invocation consumer covers all three native
execution backends. No broader structural milestone or new variance rule is implied.


The same retained consumer now compares source-level function patterns with
TypeInfo.IsInstanceOfType, including closed generic signatures and negative cases.
This is qualification of the existing reflection relationship contract, not a new
function variance or dynamic invocation API.


Author-directed public facade (2026-10-16): provisional FunctionReflection now lives
in System.Runtime.Reflection with a Result<Object?, ReflectionError> Invoke. Input
validation is shared across interpreter/JIT/AOT, with target faults still propagating.
The consumer imports the fresh five-library bundle and the API reference is refreshed.
Generic overloads remain future work; this does not resume the wider structural plan.


### Function work integrated (2026-10-16)

By explicit author direction, neoCLR main fast-forwarded from `971fbceec` to
`8aa37b938`, and Raven codex/neoclr from `9302f2e4c` to `e4acca9cf`.
All validated function-kind metadata/constraints, structural function symbols,
function patterns, introspection and provisional Result-returning FunctionReflection
commits are preserved. The general pattern parser fix is already on Raven main
(`0c8cda8dc`); Raven main and the neoCLR line were not merged wholesale.

The completed function-kind branches were deleted locally. Raven's older
codex/consistent-ranges branch was also deleted after confirming it is an ancestor
of main. Parked remote structural experiments are untouched. This is local integration,
not a push, release or website publication. Earlier branch-only checkpoints describe
historical state; current support is on the integration branches above.

Runtime Contract configuration and behavior are unchanged by these fast-forwards.
Retained interpreter/JIT/AOT success and target-fault evidence and the fresh five-library
bundle remain applicable because the code trees are identical. FunctionReflection
remains provisional; generic invocation overloads, signature bounds, ref/out and
broader value marshalling remain open. Other structural work remains parked.

Integration checks passed: 62 focused Raven syntax/semantic/runtime tests, the host
SDK function-kind authoring/import/introspection checks, the reference fingerprint
check and four native API snapshot checks. No code or public signature changed
during integration, so the retained interpreter/JIT/AOT consumers were not rerun.


### Author-directed compound operators (2026-10-16)

After function integration, the author requested C# 14-style compound overloads in
Raven main and the neoCLR compiler line. The initial supported-token slice now uses
in-place instance calls on both targets, with explicit SDK managed element-address
authoring. [Validation and boundaries](validation/compound-operators/README.md)
include .NET and interpreter/JIT/AOT evidence, with nominal struct arrays remaining
interpreter-only. Remaining checked/extension/interface/virtual coverage is
follow-up work. This bounded language improvement does not reprioritize unrelated
milestones or resume broader structural work.

Remainder and shift follow-up: Raven main `b5e6bb0d3`, native carry `57c567509`,
101/104 focused tests respectively, and the expanded interpreter/JIT/AOT consumer
pass. See the linked qualification record. The author clarified that NeoCLR
operator work is intended to support the System.Numerics interfaces; nominal
capability contracts and their generic dispatch remain the next layer, not an
implicit consequence of supporting an operator spelling.

Instance increment slice: Raven main `94f133cab`, native `a28fe52ed`, with 231/233
focused tests and native interpreter/JIT/AOT success. Prefix and postfix selection
follow .NET conventions; checked and declaration-modifier support remain open.

Truth/logical slice: Raven `7039b3e23`/`198c22bea`, 107/109 focused tests and
interpreter/JIT/AOT success. Both targets use paired truth methods and conditional
bitwise calls, following .NET conventions; direct logical declarations require
migration. Checked forms and broader interface/virtual operator coverage remain open.

The ordinary operator matrix now passes on .NET 11 and native interpreter/JIT/AOT
with Raven `75607f005`/`8a82d185b`. Parser regressions exposed by composition are
fixed and retained in 87 focused tests per line. The qualification record separates
this result from still-open checked contexts/forms, wider declaration modifiers,
System.Numerics conformance and native imported-provider coverage.


Checked declaration foundation: Raven `29a458e27`/`cb83fb9dd`, with 98 focused
tests on each line and all 14 checked metadata identities executed on .NET 11 and
native interpreter/JIT/AOT. This closes declaration/export and symbol-import support,
not checked expression semantics. Next bounded work is checked contexts, overload
selection/fallback and built-in overflow handling; the qualification record above
lists limits. No runtime API or metadata-container change, and no structural-work
resumption, is involved.


Checked arithmetic prerequisite (2026-10-16): the SDK now authors all six checked
integer add/subtract/multiply operations with standard CLI and existing native
encoding. Shared JIT/AOT lowering now preserves 32-bit, 64-bit and native widths
and reports overflow without writing the result. See the [qualification matrix](validation/checked-arithmetic/README.md)
and SDK API reference. No Runtime Contract or library API change is required.
Raven checked contexts, overload selection/fallback and checked numeric conversions
remain the next compiler/SDK work; structural work stays parked.


Checked conversion authoring (2026-10-16): the SDK now exposes all 20 standard
checked integer conversion instructions, preserving signedness and destination
width with existing CLI/native encodings. The [.NET/interpreter matrix](validation/checked-conversions/README.md)
covers boundary values and floating-point truncation/overflow. JIT/AOT still reject
these operations; native lowering is the next prerequisite before Raven checked
contexts and narrowing can be qualified. No Runtime Contract configuration, native
encoding version or compiler behavior changes in this SDK slice.


Native checked conversion follow-up (2026-10-16): all 20 conversions now use shared
JIT/AOT lowering with exact integer range checks, floating truncation and ordered
bounds checks, and existing ArithmeticOverflow fault propagation. The [native matrix](validation/checked-conversions/README.md#native-jitaot-lowering-follow-up)
qualifies macOS ARM64; Windows profile restrictions are unchanged. This supersedes
the backend rejection noted above. Raven maps checked contexts and conversion selection to these operations;
no Runtime Contract, encoding or library change is required.


## Checked expression qualification (2026-10-16)

Raven main `935ab4af7` introduces checked/unchecked parenthesized expressions and
checked binary operator lookup. Its deliberate native carry `57fec6ec7` maps integer binary
`+`, `-`, `*` to existing signed/unsigned overflow instructions. Native checked
operator overloads are ordinary method calls. No Runtime Contract setting,
container version, temporary carrier or runtime-library API is added. The normal
.NET target emits standard checked IL; native overflow remains a terminal
ArithmeticOverflow fault rather than a catchable .NET OverflowException.

The nearest context wins, and contexts do not flow into nested function bodies or
called methods. Checked binary candidates are resolved first, with ordinary
fallback only if none applies; ambiguity remains an error. The compiler owns this
lookup; the SDK/runtime own checked instruction validation and execution.

Broader constant-expression coverage, nullable/pointer conversions and mutations,
lifted arithmetic and checked binary expression trees remain gaps.
Unsupported checked operations receive RAV0373 rather than silently executing
unchecked. Numeric casts and signed unary negation are supported. Existing native
primitive admission limits are unchanged.
Broader structural work remains parked.

See [the reproducible compiler qualification](validation/compound-operators/README.md#checked-expression-follow-up)
for matched compiler revisions, bundle, source and backend evidence. The next
bounded compiler slice is block contexts, followed by compile-time overflow
evaluation. Checked mutation qualification is recorded below.


### Checked unary negation follow-up (2026-10-16)

Raven main `307f3b55c` adds signed integer unary negation in checked expressions and
checked unary overload lookup with ordinary fallback. The native carry emits a
width-correct zero and existing checked subtraction after evaluating the operand
once, including ordinary small-integer promotion. Floating-point negation retains
its existing behavior. No Runtime Contract switch, new opcode, metadata version or
class-library API is introduced; this is compiler integration of existing runtime
operations. The .NET compiler also maps checked unary expression trees to
`Expression.NegateChecked` and exposes `IUnaryOperation.IsChecked`.

The [operator qualification](validation/compound-operators/README.md#checked-unary-follow-up)
records the matching builds, tests, standalone consumer and minimum-value overflow
checks. Unary negation, numeric conversions, checked mutations and blocks are supported;
checked binary expression trees and broader constant-expression coverage remain gaps.
Broader structural work stays parked.

The native unary carry is `371363a67`; all positive and unary-overflow backend
checks pass. The author's exception-free recovery question is recorded in the
[recoverable arithmetic proposal](design/checked-arithmetic.md). This is design
work alongside compiler completion, not an implemented API or a permanent roadmap
reprioritization.


Recoverable arithmetic follow-up (2026-10-16): the author confirmed the same
safe-method/faulting-operation distinction used by slicing and indexing. A
[source-local addition prototype](validation/recoverable-arithmetic/README.md)
checks this policy using existing nominal Result and the existing safe Int32.Divide
API. Public recoverable arithmetic API adoption and performance qualification remain pending;
this does not unblock or reprioritize broader structural work.


## Minimal numerics capabilities

System.Numerics provides AdditionOperators, SubtractionOperators, MultiplyOperators
and DivisionOperators using native Self. Number composes them with ordering and
identities. The [qualification](validation/numerics-interfaces/README.md) covers ten
primitive implementers, a Complex demo, source-local and imported constrained calls,
and interpreter/JIT/AOT execution. Public API references include the four interfaces.

Mixed operand/result contracts, additional identity interfaces, constructed interface
method bounds and production Complex semantics remain separately scoped. This
author-directed capability work does not reprioritize the active milestone or resume
parked structural types.


Checked conversion qualification: Raven main `0c272f423` supports range-checked
numeric casts and checked explicit conversion pairs. The native adapter uses
existing checked instructions, and imported checked conversion symbols retain
conversion classification. [Evidence](validation/compound-operators/README.md#checked-conversions)
covers .NET, interpreter/JIT/AOT, separate-provider selection and narrowing,
unsigned-source and floating overflow faults. Recoverable arithmetic remains separate.


**Checked mutation compiler slice (2026-10-16):** Raven main `9dd7ea722` implements
checked compound assignments and increment/decrement, checked overload selection,
small-integer write-back and single evaluation of mutation locations. The native
carry `a72b1b24c` passes interpreter/JIT/AOT success and overflow probes using
existing checked instructions and terminal faults. See
[qualification](validation/compound-operators/README.md#checked-mutation).
Checked blocks and bounded numeric constant evaluation are now qualified below; broader
structural work remains parked. This bounded task does not reprioritize the roadmap.


**Checked blocks and constant evaluation (2026-10-16):** Raven main `3fa828516`
and native `48b0b6f75` add lexical checked/unchecked blocks. Main `489cc12a4`
adds compile-time overflow diagnostics for built-in fixed-width addition,
subtraction, multiplication, negation and numeric casts, plus typed const local/field
dependencies. Explicit unchecked integer constants wrap. No runtime API, metadata
or instruction change is required. Broader constant-expression coverage (including
native-width/enum arithmetic) remains open. Existing nullable/pointer and checked binary
expression-tree gaps remain; structural work stays parked. This author-directed
compiler task does not change milestone priorities. See [qualification](validation/compound-operators/README.md#checked-blocks-and-constant-evaluation).

Native Int32 class constants now emit storage-free metadata and support imported
reads; wider payload types remain a gap. Native optional parameters remain rejected.

The constant evaluator's validated native carry is `9ea79f0e2`: 175 focused tests
pass on each compiler line, the local-constant consumer returns 42 on .NET and
interpreter/JIT/AOT, and both targets reject the four overflow cases before emission.

### Constant and readonly qualification (2026-10-16)

General Raven constant operator/default/lowering slices are committed on main and
carried deliberately to codex/neoclr. Native Int32 class literal metadata uses
neoCLR `e453a7bbf` and Raven `1b00893b3`; source and separate-provider consumers pass
interpreter/JIT/AOT. Numeric constant materialization is main `e44a7ca96` / native
`d194b4bda`, with 150 focused tests on each compiler line and native execution.
The [qualification record](validation/compound-operators/README.md#additional-constant-operators-and-native-class-constants)
retains revisions, commands and hashes. Parameter-default arithmetic works on .NET;
native optional declarations remain explicitly rejected until metadata/import support.

The author-directed readonly work now includes Raven native source emission and
separate-provider consumers, qualified on interpreter/JIT/sanitized AOT. Canonical
`.cctor` bodies, strict/relaxed timing and per-closed-type initialization use the
[implemented runtime protocol](design/readonly-fields.md); constants do not substitute
for readonly storage. Guest field/initializer introspection and public static-field
reflection are covered by [the focused consumer suite](validation/field-reflection/README.md).
Native roots distinguish descriptive field/constructor metadata from field execution.
Wider literal payloads, native-width/enum constant expression coverage and general
closed-generic reflection execution remain separate gaps. Structural work stays parked;
these author-directed slices do not reorder the platform milestones.


### Author-directed concurrent GC investigation — 2026-10-18

Following Serve admission diagnostics, the author requested continuing through a
concurrent collector implementation, informed by alternatives beyond .NET. The
[comparison and safety/measurement gate](design/concurrent-gc-evaluation.md) supersede
the earlier provisional ordering for this investigation, without selecting a collector
or reprioritizing unrelated milestones. Implementation, three-mode lifetime parity,
worker-overlap proof and sustained Serve/Go qualification remain open.


The subsequent author direction requires [separately includable memory-management
providers](design/memory-management-providers.md), with activation of an included,
compatible implementation. This includes non-GC possibilities. The initial broader provider-boundary proposal is deferred by the narrower
author direction below; it is not a prerequisite for the proof of concept.


The author narrowed the first implementation to **GC selection only**, then requested
continued concurrent-GC investigation. Development now exposes `--gc mark-sweep` and
execution-local Rust selection, with native backend validation. Only the existing
collector is available; broader provider composition and concurrent collection remain
open. See [selection semantics](runtime-gc.md#explicit-collector-selection-development).

The isolated [snapshot proof of concept](experiments/concurrent-gc-snapshot/README.md)
now demonstrates progress with an outstanding worker and reproduces conditional-owner
retention across repeated cycles. It remains excluded from production. UBSan/bounds
passes the counterexample; ThreadSanitizer is unavailable on this host. No concurrent
collector or new sustained Serve rate is qualified. Keep subsequent experiments small
and use phase costs to choose useful off-thread work.


[Sampled phase evidence](../benchmarks/http-server/serve/gc-phases-20261018/README.md)
now identifies graph tracing as about 60% of sampled Serve collector CPU in two
sampling intervals. This motivates testing worker handoff/lifetime infrastructure
before reusing the walker on isolated data; no production concurrency or capacity
gain is established by these diagnostics.
