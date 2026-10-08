# neoCLR platform roadmap

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
endpoints, with full-width storage and conversion parity. Interface
views/dispatch and broader stream support behind Write/ReadLine remain next; this region is
not a replacement for general native collection. Multiple closed value shapes now
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
ArrayList<byte> dependency next requires checked backing reservations. The author adds a console-input sample after unions, exercising input and
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
slice now emits primitive/method-generic/vector namespace-function references from
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
→ metadata → PE architecture. Primitive namespace functions now materialize directly into the existing definitions
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
metadata groups passing. Qualified namespace-function lookup now includes native ownerless
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
with expected negative-capacity/index faults. Explicit namespace-function dependency
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
deferred; do not start a builder optimization project. The [Encoder acceptance/drain evaluation](design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27)
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
