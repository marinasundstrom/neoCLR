# AOT value types and members

**2026-10-07 — bounded native implementation on main.** Raven Counter, copy/member
and nested-record samples now compile through **Raven → neoCLR metadata/IL → ARM64
native executable**. They run without a shared managed framework or runtime. Some/None and
Result unions remain the next representation step, followed by a console-input sample.
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
exceeding that bound are rejected before emission. Reference types, generics, inheritance,
interfaces and explicit layout remain unsupported. Properties
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
modules. The original scalar/literal-console profile remains available for Hello World.

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
- All declared bodies/opcodes are admitted; unsupported dead instructions are rejected.
  Supported unreachable instructions may be omitted. There is no trimmer. Recursive
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
a payload without overwriting adjacent fields. Twenty-one value-profile tests pass; the earlier
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
checked narrowing and treating `Byte&` as `Int32&`. All twenty-one value tests pass;
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
types or instance mode, and ambiguous roots. Twenty-one value tests pass; the earlier
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
The explicit native dependency configuration succeeds in the interpreter; no Raven fix
or native union execution is claimed by this slice.

Nested value storage and ordinary output parameters are now implemented. Next admit
the remaining generated-member/dependency contracts, retaining nominal identities and actual field contracts rather than recognizing union source names.
Exercise both Some/None branches, then Result success/error. Resolve generated-member
and library dependencies explicitly. Reference-bearing payloads and console input follow
with defined native lifetime and service contracts. Trimming and general heap management
remain separate later work; unsupported union bodies are not silently discarded.
