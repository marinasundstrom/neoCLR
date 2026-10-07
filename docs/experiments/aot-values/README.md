# AOT value types and members

**2026-10-07 — bounded native implementation on main.** Raven Counter, copy/member
and nested-record samples now compile through **Raven → neoCLR metadata/IL → ARM64
native executable**. They run without a shared managed framework or runtime. The
[ordinary Some/None union app](../aot-union/README.md) now runs under explicit closed-world
selection. A local generic Result consumer and pattern-binding app now run too;
console input and the actual library Result dependency contract remain later steps.
The earlier [producer inventory](inventory.json) is historical: it records the
pre-implementation rejection and the broader union requirements.

## Samples and reproduction

[Counter](counter.rvn) constructs a value, mutates it through a member and branches
on its result. [Copies](copies.rvn) adds an Int32/Boolean record, assignment, copied
parameters/results, selection through branches, default initialization and a loop.
Each exits zero only when its assertions pass. [Native evidence](native-validation.json)
records the first flat-value slice. [Nested](nested.rvn) adds a record payload between
scalar fields, nested constructors, property copies, branching and default initialization.
[Nested evidence](nested-validation.json) records its producer and native deployment checks.
Both reports include exact compiler/runtime inputs, source/artifact hashes and commands.

Build the AOT tool and use the pinned native-enabled Raven compiler from the
[clean bootstrap qualification](../extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md):

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/aot-values/verify_native.py \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-values-run
```

The output directory must be new. This runs all six samples in the interpreter, compiles
their PE/#Neo bodies to native objects, links the C startup adapter, checks for no
object imports and only macOS `libSystem` executable linkage, then runs each copied
executable alone with an empty environment. Select a compatible Apple SDK as in the
[Hello World instructions](../aot-hello/README.md). Raven's existing primitive bootstrap
is a build-time dependency; native code never executes CLI projection bodies.

The checked-in [Counter](Counter.pe), [Copies](Copies.pe) [Nested](Nested.pe) and [Outputs](Outputs.pe) artifacts
come from the same pinned producer and corresponding sources. Rust tests use them without invoking
Raven on each run. Fixture hashes and fresh run hashes are recorded separately;
byte-identical producer rebuilds are not asserted.

## Implemented value profile

A module containing type declarations selects the value profile. It admits
up to 32 local nongeneric value Record types, each with zero to eight fields containing
Int32, Byte, Boolean or other local records, and up to 128 functions/members.
Each complete inline layout is limited to eight primitive/dummy lanes. Recursive inline
layouts (including mutually recursive unused types), unresolved field types and layouts
exceeding that bound are rejected before emission. Reference values, inheritance, interfaces and explicit layout remain unsupported.
Explicit closed-world mode additionally supports the bounded type specialization below;
empty abstract/sealed lexical companions retain metadata identity but cannot be used
as values or executable member owners. Properties
use their emitted accessor methods; they do not introduce separate native storage.

Supported members have copied Int32/Byte/Boolean/record parameters and results or inhabited
Void/no-result returns. Instance methods require a borrowed by-reference receiver.
Constructors use the same member call machinery and publish a value only after success.
The source module's ordinary metadata, member-access, initialization and lifetime
verification still runs before native emission. References cannot be stored in fields,
locals, ordinary parameters or results in this profile, so they cannot escape a frame.
Explicit `out` parameters may borrow Int32, Byte, Boolean or local-record storage.
Readonly receivers, conditional `out(true)` parameters, general `ref` parameters,
virtual dispatch and external
library/service calls remain unsupported here, including console calls in value-bearing
modules. Explicit [value-library load sets](../aot-library/README.md) now extend direct
calls to supplied, verified dependencies. The original scalar/literal-console profile
remains available for Hello World.

The native layout is private to this experiment:

- Each Int32, Byte or Boolean field uses a four-byte lane, in metadata field order. Empty
  records use an unobservable dummy lane. Nested records are recursively flattened,
  including empty-record dummy lanes, with field offsets computed from full child widths.
  This is not the public platform layout;
  `sizeof`, packing, native pointer access and foreign struct ABI operations are rejected.
- Arguments and locals occupy frame slots. A value load copies field values into SSA
  operands; stores, parameters, returns and branch joins preserve those snapshots.
  A copied value never becomes an alias merely because another call mutates its source.
- `ldloca`/`ldarga` and field addresses borrow storage. Member calls receive the
  receiver's address, while ordinary record parameters receive copied lanes. `ldobj`,
  `stobj`, `initobj`, raw record construction and value/byref field operations preserve
  the interpreter's distinctions. `default` zeroes fields without executing a constructor.
- Private calls flatten copied records into primitive arguments and write results to
  caller-owned storage. The existing `neoclr_entry_v2(Int32, Int32*) -> status` remains
  the only export. Root signatures are static `() -> Int32` or `(Int32) -> Int32`.
- No-result member calls push nothing; inhabited Void has its own stack kind. Boolean
  comparisons produce Boolean values, and branch joins require matching types as well
  as matching stack height. Checked arithmetic and Fault statuses reuse scalar lowering.
- By default all declared bodies/opcodes are admitted; unsupported dead instructions are rejected.
  Supported unreachable instructions may be omitted. Default admission does not select
  a call closure; explicit closed-world selection is documented below. Recursive
  calls are rejected. Limits are 8,192 instructions, 1,024 locals and 32 explicit
  parameters per function, with at most 64 KiB of explicitly allocated frame slots
  including alignment allowance. This does not bound backend spills or total native
  stack use and does not implement interpreter instruction budgets/cancellation.

No heap allocation, reference counting, tracing collector or runtime library is needed
for these reference-free value samples. This is not a claim that all value types avoid
memory management: values containing managed references need lifetime/root handling.

## Validation and design comparison

Focused tests cover both Raven fixtures in PE/#Neo and standalone NEOX, plus interpreter
comparisons for record snapshots across mutating calls, typed record stack joins,
argument/result copies, value field updates, interior references, self-copy, and zero/eight
field records crossing the private call boundary. Scalar host input reaches value
functions at Int32 extremes. Constructor/member arithmetic Faults preserve the output
sentinel. Ten negative inputs cover classes, reference fields and recursive inline storage, packing, invalid
receivers, field indices, dead unsupported instructions, escaping receiver values,
call-identity mismatches and uninitialized locals. Three additional rejection cases
cover mutual inline cycles, unresolved fields and oversized flattened storage. Native
tests cover the nested Raven fixture in both containers and three-level field addresses,
whole-payload replacement, deep alias preservation, empty nested records and clearing
a payload without overwriting adjacent fields. Twenty-eight value-profile tests pass; the earlier
fifteen scalar/Hello test results remain applicable.

The [.NET struct baseline](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/struct)
copies values on assignment, parameter passing and return, and distinguishes default
initialization from constructor execution (reviewed 2026-10-07). The observable copy
behavior is the target here, with the existing neoCLR interpreter as the executable
oracle. An aggregate represented only by a reused pointer would be simpler but would
introduce unintended aliasing; snapshots plus explicit borrows avoid that. Flattened
primitive lanes simplify this bounded backend at the cost of more arguments and a
provisional layout; they are not a performance improvement claim or a settled ABI.
The original native decoder/verifier and common Fault lowering remain shared.

Nested storage extends the same .NET-style value-copy contract and interpreter oracle;
it adds no new source or metadata convention. Keeping nested payloads inline avoids
allocation and preserves deep field borrows, but copies every primitive lane and keeps
a strict size bound. Pointer-backed payloads would require lifetime and aliasing rules
that this slice deliberately does not introduce. Layout is still private: no foreign ABI,
layout-query, GC, reflection or performance guarantee is inferred from these tests.

## Ordinary output parameters (2026-10-07)

[Outputs](outputs.rvn) exercises Raven output parameters through a borrowed instance
receiver, a forwarding function and both success/miss branches. Its ordinary `out`
contract writes the payload on either result, matching the inventoried Raven union
extractor. [Evidence](outputs-validation.json) records the fresh producer/interpreter/native
pipeline and executable-only deployment. Tests also use PE/#Neo and standalone NEOX.
This is an output-member probe, not a handwritten substitute for a Raven union.

The backend passes output addresses through the private native ABI. Every admitted
output callee must prove a whole-slot assignment (`stobj`, `initobj` or forwarding to
another admitted output) on every normal return path. Reads, field borrows and member
calls on that output require prior assignment. Branch joins intersect assignment facts;
ambiguous output-address identities at joins and partial construction before a whole-slot
write are conservatively rejected. Borrowed parameters cannot be rebound or escape.
The ordinary verifier still checks caller initialization, types and member contracts.
Its callee-output enforcement is dynamic in the interpreter, so AOT adds this explicit
proof instead of assuming that metadata flags validate a body. No native runtime check
or managed allocation is introduced. Faults propagate the existing status; output writes
performed before a Fault are not rolled back, while the exported result stays untouched.

This reuses the [.NET/CLI output-contract comparison](../../design/extended-cli-metadata.md#output-parameter-contracts-2026-10-02):
BYREF identifies storage, Out declares a parameter mode, and body assignment must be
established separately. The native proof is stricter than the interpreter for some valid
programs (partial initialization and ambiguous borrow joins). That bounded acceptance
cost avoids introducing native initialization tracking and its runtime overhead. No
performance improvement is claimed. Conditional `out(true)` remains unsupported because
its return-sensitive assignment proof is different from ordinary `out`.

The pinned Raven producer reports `RAV0269` for the sample's uninitialized direct
forwarding form. The checked-in Raven probe explicitly initializes before forwarding;
the IL test forwards uninitialized output storage without that workaround. This is a
producer limitation in this exact bundle, not a neoCLR output rule; related general Raven
forwarding work is recorded in [library validation](../../raven-library-port-validation.md).
No compiler or bridge encoding changes are included here.

Eight new native comparisons cover the Raven fixture, uninitialized/initialized outputs
on both branches, two outputs aliasing the same nested payload, and an output callee
Fault. Six rejection cases cover missing writes, premature reads, conditional/general
reference contracts, invalid output indices and escaping output references. General
Some/None execution still needs
attribute classes, generated boxing/string/virtual methods and library dependencies.

## Byte tags and conversions (2026-10-07)

[Tags](tags.rvn) compiles Raven Byte fields, getters/setters, construction, default
initialization and tag-dependent branches to native code. The [Tags fixture](Tags.pe)
and [deployment evidence](tags-validation.json) use the same pinned producer. This is
a primitive tag probe, not a replacement implementation of Raven unions. Their emitted
Byte tag and `conv.i4` operations motivated this slice; full unions remain unsupported.

Byte retains its storage identity (including distinct `Byte&` borrows), while loads,
parameters and call results use Int32 evaluation-stack values. Every Byte storage write
truncates to the low eight bits: locals, arguments, fields, raw record construction,
indirect stores and function results. Aggregate normalization follows nested field
layouts. `conv.u1` truncates but leaves an Int32 stack value; `conv.i4` is admitted only
for the existing Int32 evaluation category. Boolean, record and address operands are
rejected. Other narrow/wide numeric types, checked conversions and floating-point
conversions are outside this slice. The scalar-only profile is unchanged.

The [.NET 10 OpCodes.Conv_U1 contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.conv_u1?view=net-10.0)
(reviewed 2026-10-07) distinguishes unsigned-byte conversion from the Int32 stack slot.
This follows the same observable behavior, with neoCLR's interpreter as the executable
oracle. Treating Byte as unrestricted Int32 storage would mishandle overflow; using
physical one-byte native slots would instead require new layout and call rules. The
bounded backend keeps four-byte private lanes and masks writes, trading compactness
and potential extra operations for reuse of the tested aggregate ABI. This is not a
public layout choice or performance claim. No Raven/compiler bridge changes are needed.

Native checks cover the Raven sample in both metadata containers plus eight runtime
inputs (-1, 0, 128, 255, 256, 511 and Int32 extremes) across storage, call, output and
conversion boundaries. An arithmetic check after `conv.u1` proves the stack result is
not itself Byte storage. Three negative inputs reject Boolean conversion, unsupported
checked narrowing and treating `Byte&` as `Int32&`. All twenty-six value tests pass;
unchanged scalar evidence remains in the earlier report.

## Overloaded members and native symbols (2026-10-07)

[Overloads](overloads.rvn) exercises Raven instance methods with Int32 and Boolean
parameter overloads. The [fixture](Overloads.pe) and [evidence](overloads-validation.json)
record compilation through native metadata and execution without a managed runtime.
Calls resolve by exact name, owner, instance/static mode and ordered parameter types.
A supplied definition identity must also match, including its module/revision/row;
a bad identity never falls back to a similarly named method. Symbolic calls must have
one matching candidate. Return types are not overload selectors. The existing native
loader/verifier remains responsible for metadata-definition validity and accessibility.
Generic specialization, virtual dispatch and cross-module compilation remain unsupported.
Root selection is still a name-only CLI argument: duplicate root names are rejected,
even when only one overload has a supported entry signature. Use a uniquely named wrapper.

This applies the existing [member identity contract](../../member-identities.md) to AOT,
following the .NET/CLI distinction between metadata call identity and native linkage
(the [metadata baseline](../../design/extended-cli-metadata.md) remains applicable).
Matching only names loses overload identity; generating signature strings as semantic
identities would duplicate metadata rules and risk conflating nominal payload types.
The bounded candidate search uses exact existing metadata contracts. Its cost is at
most 128 candidates and intentionally lacks source-language conversion-based selection.

The author asks whether function names need mangling. Private native symbols already
use `neoclr_value_<function-index>`, so methods with identical names do not collide in
this single object. The public `neoclr_entry_v2` export is unchanged. These local symbols
are neither stable metadata identities nor a separate-compilation ABI. A future mangling
contract should account for module/type identity, method signatures and generic arguments,
with versioning and collision rules; its format and compatibility policy remain open.
No stable mangling scheme or separate-object linking is implemented by this slice.

Tests cover the Raven fixture in PE/#Neo and NEOX, symbolic calls without definition
IDs, overloaded constructors, and output overloads for two nominal payload types with
the same field layout. Five negative inputs reject wrong definition IDs, owner, parameter
types or instance mode, and ambiguous roots. Twenty-six value tests pass; the earlier
scalar evidence remains applicable. Full generated Raven unions still need attribute,
boxing/string/virtual member and library support; unsupported bodies are not trimmed.

## Next steps toward union-based console input

The author proposes console input once unions work: represent available input,
end-of-input/failure and parse success/error, then branch over the results. That will
exercise input services and reference-containing strings as well as union control flow.
It does not follow automatically from reference-free record support.

The inspected Choice/Some/None producer emits nested Record payloads, a tag, out-parameter
pattern extraction and branches, plus supporting attribute classes and generated
members using boxing, virtual calls and strings. Counter has one type/five methods;
Choice has six types/twenty methods. The union probe requires explicit native Runtime
references. Its initial host-bootstrap attempt rejected generated ToString with
`Native emission does not support union body ToString: optional/expanded arguments.`
The explicit native dependency configuration succeeds in the interpreter. The subsequent
[union app milestone](../aot-union/README.md) now executes selected value-only paths
natively; formatting/boxing members remain excluded in explicit closed-world mode.

Nested value storage, ordinary output parameters and both Some/None branches now work.
The [post-milestone reassessment](../aot-union/README.md#reassessment-next-sample) recommends
Result success/error followed by an interactive integer reader. Resolve generated-member
and library dependencies explicitly. Reference-bearing payloads and console input follow
with defined native lifetime and service contracts. General trimming and heap management
remain later work; explicit closed-world mode reports every excluded declaration.

## Read-only compiler inspection

`neoclr-aot-poc --inspect Choice.pe @entry` prints deterministic JSON describing all
input types, function signatures/definition IDs, call operands and opcode counts. It
also runs real compiler admission with console capability disabled, discarding any
object bytes. It never executes native code, links, writes an output image, trims input
or changes accepted programs. Exit zero means inspection succeeded; inspect
`admission.accepted` for compilation status. Corrupt containers fail the command.
The first compiler error is not an exhaustive diagnostic list; the full inventory
shows other features and dependencies that remain to be addressed. Add `--closed-world`
after the root to inspect explicit selection/specialization; see the
[library dependency boundary](#real-library-result-dependency-boundary-2026-10-07).

[Choice.pe](Choice.pe) is the original pinned producer artifact from [inventory.json](inventory.json).
It remains rejected: six types/twenty methods include attribute classes, boxing,
string formatting and virtual calls as well as the supported value/tag/member paths.
The inspection report is build tooling, not the author's future native-interface metadata
sidecar. It has no exported-address map, stable ABI or loader contract.

## Explicit closed-world selection

`neoclr-aot-poc --closed-world Choice.pe @entry choice.o > selection.json` opts into
bounded direct-call selection. Normal compilation still checks all declarations.
This is a limited form of code trimming brought forward to run value-only union paths;
it is not the future general trimmer. The report lists selected/excluded functions and
types with original identities and row mappings. No source-name recognition is used.

Selection includes every direct call/constructor in each selected body, even calls
in unreachable instructions, and recursively retains signature/local/field storage
and lexical owner types. All selected instructions still pass ordinary AOT admission
and verification. External calls, virtual dispatch, reference-bearing signatures and
unsupported generic shapes fail; the selected profile retains its existing bounds and recursion rejection.
Input inventories are bounded to 4,096 functions and 1,024 types. No reflection, metadata
execution, callbacks or dynamic roots are supported; this mode must not be used for an
application requiring them. Merely ignoring unsupported functions is not its contract.

The compiler creates a private canonical verification projection. It preserves executable
names, signatures, visibility, layouts and bodies, relocates definition references, and
removes custom attributes and property descriptors from that internal projection.
Origins retain field readonly/access facts; source assembly membership is checked, while
external assembly bindings and omitted property tokens are removed. Accessor bodies enter through calls. The original native metadata artifact is
unchanged; no rewritten deployment metadata or stable ABI is produced. A required missing
callee/owner, invalid supplied identity, unsupported selected body or private-field access
fails before object creation. Input outside the selected closure is inventoried, not fully
verified for execution. Constructor initialization and output-assignment checks still run.

Compared with [.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
(reviewed 2026-10-07), which requires trimming and restricts dynamic loading, this is a much
smaller explicit direct-call contract. It lets ordinary Raven union value paths run without
first implementing their unused formatting/boxing services. The cost is a closed execution
surface and omitted reflection metadata; it cannot promise general library compatibility.
Full metadata retention policy, export roots and dynamic-use annotations remain future work.


## Generic Result and pattern bindings (2026-10-07)

[Result app](result-app.rvn) defines ordinary `ParseResult<T, E>` and instantiates
`ParseResult<int, byte>`. It constructs both cases, returns/copies the carrier, binds
Ok with `if let` and Error with `let ... else`, and checks 42 and -7. Different generic
argument types exercise substitution of both parameter positions and Byte conversion.
[Pattern app](pattern-app.rvn) separately exercises matching and non-matching paths for
`let Choice.Some(value) = item else { ... }` and `if let Choice.Some(value) = item`.
The generic app imports `ParseResult.*` for unqualified case patterns. These spellings
were compiled with the pinned native producer; do not infer all pattern syntax works.

Use `verify_union.py` with the compiler/bundle/runtime arguments in the
[union reproduction](../aot-union/README.md), adding `--sample result-app` or
`--sample pattern-app` and a fresh output directory. The checked-in
[Result evidence](result-validation.json) and [pattern evidence](pattern-validation.json)
record source/tool hashes, interpreter execution, native selection, object imports,
OS-only dynamic linkage and execution alone in an empty directory/environment.
[ResultApp.pe](ResultApp.pe) and [PatternApp.pe](PatternApp.pe) are the tested producer
fixtures. Rust tests execute each in PE/#Neo and re-encoded NEOX form.

**Plain `let` deconstruction is still a producer gap.** The
[probe](pattern-deconstruction.rvn) uses `let (first, second) = Pair()` with an ordinary
value-type `Deconstruct` member. Raven accepts the language form but its pinned native
emitter rejects `BoundAssignmentStatement (BoundPatternAssignmentExpression)` with
NEOMETA001, before neoCLR IL or an executable exists. The pattern reproduction checks
this expected rejection and absence of an output artifact. A union-case `let` binding
without `else` is not the same language construct; use `let ... else` for refutable
union matching. Fixing native positional deconstruction belongs in Raven's shared
compiler line and requires a new pinned native bundle; this slice changes no Raven code.

Under `--closed-world`, the compiler specializes selected type-generic members and
storage before ordinary selection/verification. It binds calls against the original
closed nominal signatures, substitutes type parameters, and retains distinct case
identities even when their native lane widths match. The JSON report records original
definitions and chosen arguments. The original metadata is unchanged. Only **one closed
instantiation per local type definition** is allowed in one executable; the existing
32-type/128-function and layout limits still apply. Generic methods, constraints, open
arguments, reference payloads, generic lexical owners and multiple instantiations are
rejected. Empty static lexical companions are retained solely for access/identity facts.
No public generic ABI, runtime generic dictionaries or new managed services are introduced.

Compared with [.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
(primary-source comparison reused from the investigation), value-type specialization
moves layout and call decisions to build time; .NET supports multiple closed instantiations
and documents their code-size cost. This experiment deliberately accepts one shape per
local definition, allowing existing private symbols/metadata identities to remain unique.
The benefit is a small verifiable foundation; the cost is rejection of ordinary programs
using, for example, both `Result<int, byte>` and `Result<int, bool>`. Multi-instantiation
identity/mangling, actual library dependencies and general trimming remain future work.
The next consumer is still the interactive integer reader, beginning with those library
contracts and native UTF-8 input/lifetimes rather than a network server.


## Real library Result dependency boundary (2026-10-07)

[Library consumer](library-result-app.rvn) uses `System.Result<int, byte>` from the
pinned native `System.Runtime.dll`, with the same success/error assertions and pattern
bindings as the local generic sample. **Raven native emission and interpreter execution
pass; native compilation remains unsupported.** There is no replacement carrier or
Result-specific backend intrinsic. [Evidence](library-result-validation.json) records
producer/runtime/AOT hashes, exact commands, the emitted application, both inspection and
actual compilation rejection, and the relevant library declarations. The
[fixture](LibraryResultApp.pe) drives the focused inspection regression test.

Reproduce using the same `verify_union.py` compiler/bundle/runtime arguments as above,
with `--sample library-result-app` and a fresh output directory. This mode deliberately
expects interpreter success followed by dependency rejection and verifies no object is
created. It does **not** claim a standalone executable was produced.

Read-only admission now supports the same explicit selection mode as emission:

```sh
neoclr-aot-poc --inspect ResultApp.pe @entry --closed-world
neoclr-aot-poc --inspect LibraryResultApp.pe @entry --closed-world
```

The first accepts the local generic sample and includes its selection/specialization
report; the second reports an external `TryGetValue` call with its exact owner, generic
arguments and output payload signature. `admissionMode` identifies whole-module or
closed-world admission. `admission.phase` distinguishes a selection failure from a
selected-body compilation failure. `selection` is null when preparation failed, and is
retained when preparation succeeded but body admission failed. The original declaration
and opcode inventory always covers the whole source artifact. Inspection exits zero
when it produced a report, even when `admission.accepted` is false. Input decoding and
CLI errors still fail the command. It creates no object and executes no generated code;
it is not a native metadata sidecar. Default inspection remains whole-module admission.

### Next bounded linking work

The pinned library contains 334 types and 1,959 functions. The app's external signatures
refer to the Result carrier and distinct generic Ok/Error case types; the case types also
need their static lexical companion. The carrier implements
`Propagatable<Result<T, E>, T, E>`. These are real metadata dependencies even though this
consumer makes direct calls rather than using interface dispatch. The current one-source-
assembly selector and reference-free record profile cannot simply accept this load set.

The next implementation should first prove an explicit application-plus-value-library
load set with exact module/revision/definition binding, assembly-aware private/internal
access checks, and a reported direct-call closure. Then address generic interface
contracts needed by the actual Result without silently deleting `implements` metadata.
Add negative probes for wrong revisions, missing dependencies, duplicate definitions
and cross-assembly access before extending the application to native input.

This follows the existing [.NET Native AOT comparison](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
and [metadata identity baseline](../../design/extended-cli-metadata.md): closed-world
compilation must resolve required library code at build time, while source language
patterns lower to calls and branches independently of native linkage. Prefer reusing
neoCLR's binding/identity semantics over concatenating declarations or recognizing
`System.Result` by name. Reuse costs an explicit load-set/projection contract; ad-hoc
concatenation risks repairing bad identities or weakening access checks. This is a
provisional next-slice design, not implemented library AOT or a new public ABI. General
trimming, multiple generic instantiations and input/lifetime services remain later work.


The [subsequent explicit value-library slice](../aot-library/README.md) now implements
application-plus-library compilation with original-scope verification and standalone
Raven evidence, including a follow-up with one closed generic value shape per definition. Generic runtime-library/interface support remains open;
this does not change the recorded rejection of the System.Result probe.
