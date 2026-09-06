# Unions, enums, and the next fundamental milestone

Status: proposed direction for the next implementation slices. The interpreter
still special-cases Option and Result. The [generic metadata foundation](generic-metadata.md)
is now implemented; general union definitions and generic execution remain pending.
Reflection is not required for this work.

## Terminology

Use **union** for a tagged choice whose cases may carry typed payloads. Option and
Result are unions. Use **enum** for named constants with an integer underlying type,
including flag sets. Keeping these terms separate avoids implying that Result's
error payload is merely an integer. An untagged overlapping native layout is a
separate explicit-layout capability, not what union means here.

## Minimum metadata model

A union definition needs a type identity, an ordered generic parameter table, and
a case table. Each case has a unique numeric tag, an optional tooling name, and an
ordered list of typed payload fields. Payload fields have indices and optional names.
Generic parameter references use indices into their declaring definition. A closed
type reference consists of the definition identity and its ordered type arguments.
Names are mappings for authoring/tooling and need not match higher-level source
identifiers; execution resolves identities, indices, and tags.

Case tags should be explicit stable unsigned 32-bit values. Source order must not
silently change their meaning. A compiler may assign tags, but the resulting metadata
must record them. Names and tags are unique within a union; tags need not be dense.
Generic arity, parameter indices, payload types, and duplicate tags are validated
before execution. Open types cannot be instantiated as runtime values.

Proposed library definitions, shown as a schema rather than new assembler syntax:

| Definition | Type parameters | Tag | Case | Payload fields |
| --- | --- | --- | --- | --- |
| System.Option | 0: T | 0 | None | none |
| System.Option | 0: T | 1 | Some | 0: value of T |
| System.Result | 0: T, 1: TError | 0 | Ok | 0: value of T |
| System.Result | 0: T, 1: TError | 1 | Err | 0: error of TError |

Retain the current Ok/Err vocabulary initially. TError is an ordinary type parameter;
it need not inherit from a universal Error class. System.Error can remain the basic
library error representation while structured library error types become possible.
None carries no payload. Some<Void> and Ok<Void> each carry one actual Void value:
zero-sized storage does not erase the distinction between a case with a Void field
and a case with no fields.

## Interpreter values and instructions

Start with a union value containing its closed type identity, its active case tag,
and the active case's typed payload values. This has value-copy semantics, like a
record. Pointer fields copy addresses and lifetime information; copying a union does
not allocate pointees, acquire ownership, or choose a memory-management policy.

Provide three fundamental operations: construct a selected case from its payload
fields; test whether a value has a selected case; extract a selected payload field
from a matching case. Case operands identify the closed union type and numeric tag;
payload operands additionally use a field index. Assembly names may resolve to these
operands. No runtime string lookup or reflective API is needed.

Construction consumes exactly the declared payload values after generic substitution
and produces one union value. Case testing produces Boolean. Extraction checks the
active case and field index; a mismatch is a Fault, not a recoverable Error. Programs
must test the case before extracting when it is not already established. An error
case is an ordinary value and does not transfer control or unwind frames.

The current some/none/ok/err/is.case/ldcase instructions are bootstrap scaffolding.
Migrate their implementations to the general case mechanism; convenient spellings
can lower to it. Legacy metadata needs an explicit version/migration decision when
hard-coded Type::Option/Result and the global Case enum are replaced. Do not silently
reinterpret existing serialized modules with a changed schema.

## Native representation comes separately

Do not infer that an all-zero byte pattern constructs a union, even when tag zero
names a case. Its payload might not have a valid zero value. Native heap allocation
and case construction stay separate operations. No null sentinel or spare-bit
optimization is needed for the interpreter milestone.

A future baseline native representation can use an explicit discriminant plus storage
large/aligned enough for the largest payload. Before implementing it, specify padding,
initialization of inactive bytes, nested layouts, and copying of stored pointers.
String and other values without native layouts prevent blindly assigning such a layout
today. Native union ABI compatibility, representation optimizations, and native enum
marshaling are separate work; the interpreter representation is not a promised ABI.

## Implementation order

The unaligned-access slice is complete. Next:

1. Completed: generic-parameter and constructed-type references with arity validation
   and substitution for fields. Generic execution remains separate.
2. Add user-defined union case metadata and the three case operations, with tests
   for nested Option/Result, Void payloads, wrong-case access, and value copying.
3. Define Option and Result in the platform-written System library, then remove
   runtime recognition of those names wherever the general mechanism suffices.
4. Add integer-backed enums as a distinct metadata form. Set the underlying integer
   width and named constants explicitly; decide flags and unnamed bit-pattern rules
   in that slice instead of inheriting union payload behavior.

Continue verifier work around stack joins, initialization, and case access as these
operations settle. Broad reflection, inheritance/dispatch, generic constraints and
variance, managed ownership wrappers, and runtime async are not prerequisites for
this milestone. Array storage contracts also remain separate.
