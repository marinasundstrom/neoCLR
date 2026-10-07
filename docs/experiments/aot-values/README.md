# AOT value types and members

**2026-10-07 — bounded native implementation on main.** Raven Counter and a second
copy/member sample now compile through **Raven → neoCLR metadata/IL → ARM64 native
executable**. Both run without a shared managed framework or runtime. Some/None and
Result unions remain the next representation step, followed by a console-input sample.
The earlier [producer inventory](inventory.json) is historical: it records the
pre-implementation rejection and the broader union requirements.

## Samples and reproduction

[Counter](counter.rvn) constructs a value, mutates it through a member and branches
on its result. [Copies](copies.rvn) adds an Int32/Boolean record, assignment, copied
parameters/results, selection through branches, default initialization and a loop.
Each exits zero only when its assertions pass. [Native evidence](native-validation.json)
records the exact compiler/runtime inputs, source/artifact hashes and commands.

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

The output directory must be new. This runs both samples in the interpreter, compiles
their PE/#Neo bodies to native objects, links the C startup adapter, checks for no
object imports and only macOS `libSystem` executable linkage, then runs each copied
executable alone with an empty environment. Select a compatible Apple SDK as in the
[Hello World instructions](../aot-hello/README.md). Raven's existing primitive bootstrap
is a build-time dependency; native code never executes CLI projection bodies.

The checked-in [Counter](Counter.pe) and [Copies](Copies.pe) artifacts come from the
same pinned producer and corresponding sources. Rust tests use them without invoking
Raven on each run. Fixture hashes and fresh run hashes are recorded separately;
byte-identical producer rebuilds are not asserted.

## Implemented value profile

A module containing type declarations selects the new flat-value profile. It admits
up to 32 local nongeneric value Record types, each with zero to eight Int32/Boolean
fields, and up to 128 uniquely named functions/members. Reference types, nested record
fields, generics, inheritance, interfaces and explicit layout are rejected. Properties
use their emitted accessor methods; they do not introduce separate native storage.

Supported members have copied Int32/Boolean/record parameters and results or inhabited
Void/no-result returns. Instance methods require a borrowed by-reference receiver.
Constructors use the same member call machinery and publish a value only after success.
The source module's ordinary metadata, member-access, initialization and lifetime
verification still runs before native emission. References cannot be stored in fields,
locals, ordinary parameters or results in this profile, so they cannot escape a frame.
Readonly receivers, out parameters, virtual dispatch, overloaded names and external
library/service calls remain unsupported here, including console calls in value-bearing
modules. The original scalar/literal-console profile remains available for Hello World.

The native layout is private to this experiment:

- Each Int32 or Boolean field uses a four-byte lane, in metadata field order. Empty
  records use an unobservable dummy lane. This is not the public platform layout;
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
sentinel. Ten negative inputs cover classes, reference/nested fields, packing, invalid
receivers, field indices, dead unsupported instructions, escaping receiver values,
call-identity mismatches and uninitialized locals. Existing scalar/Hello tests remain.

The [.NET struct baseline](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/struct)
copies values on assignment, parameter passing and return, and distinguishes default
initialization from constructor execution (reviewed 2026-10-07). The observable copy
behavior is the target here, with the existing neoCLR interpreter as the executable
oracle. An aggregate represented only by a reused pointer would be simpler but would
introduce unintended aliasing; snapshots plus explicit borrows avoid that. Flattened
primitive lanes simplify this bounded backend at the cost of more arguments and a
provisional layout; they are not a performance improvement claim or a settled ABI.
The original native decoder/verifier and common Fault lowering remain shared.

## Next steps toward union-based console input

The author proposes console input once unions work: represent available input,
end-of-input/failure and parse success/error, then branch over the results. That will
exercise input services and reference-containing strings as well as union control flow.
It does not follow automatically from primitive flat-record support.

The inspected Choice/Some/None producer emits nested Record payloads, a tag, out-parameter
pattern extraction and branches, plus supporting attribute classes and generated
members using boxing, virtual calls and strings. Counter has one type/five methods;
Choice has six types/twenty methods. The union probe requires explicit native Runtime
references. Its initial host-bootstrap attempt rejected generated ToString with
`Native emission does not support union body ToString: optional/expanded arguments.`
The explicit native dependency configuration succeeds in the interpreter; no Raven fix
or native union execution is claimed by this slice.

Next add nested value storage and the required member/out-parameter semantics, retaining
nominal identities and actual field contracts rather than recognizing union source names.
Exercise both Some/None branches, then Result success/error. Resolve generated-member
and library dependencies explicitly. Reference-bearing payloads and console input follow
with defined native lifetime and service contracts. Trimming and general heap management
remain separate later work; unsupported union bodies are not silently discarded.
