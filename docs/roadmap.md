# Direction and migration

The platform name is undecided; neoCLR names the runtime only. The existing code
is a small semantic testbed. It is not a commitment to Rust for every component,
JSON for distribution, or the exact instruction extensions used here.

## Strategy review: implementation paused

The user requested a strategy review after the marker-attribute slice. The milestones
below retain earlier direction; they are not authorization to resume runtime work.
The [construction, mutation, and initialization proposal](construction-and-initialization.md)
recommends settling addressed access and initialization contracts, developing verifier
and member-reference foundations, then introducing a small high-level compiler as
another producer of the same metadata/IL. Library code can migrate from handwritten
IL incrementally; compiler self-hosting is not required. This sequence is proposed,
not an accepted change to executable semantics. The [addressed-access proposal](addressed-access.md)
recommends a basic control-flow verifier as the first executable slice when work resumes,
followed by explicit receiver/reference capabilities with lifetime and access checks.

## Immediate focus: heap allocation and pointers

The initial subset implements native allocation/free, layout, casts, byte offsets,
field addresses, and indirect loads/stores. Construction remains separate from
storage. See [heap and pointers](heap-and-pointers.md) for current checks and limits.
Native integers/address conversions are also implemented. Next pointer capabilities
include direct foreign memory access, broader P/Invoke marshalling, argument/local addresses, explicit field offsets, and broader ABI controls. Sequential
record packing and minimum-size reservations are implemented. Frame-local byte allocation
(`localloc`) and typed/block memory initialization and copying are implemented. Checked numeric conversions are now implemented. Integer and floating-point
layouts, arithmetic, and indirect access are now available, along with a first
[native interop subset](native-interop.md). Diagnostic side tracking must not
become a compulsory ownership policy for the platform.

Defer reference counting, GC, automatic destruction, lifetime-aware wrappers, and
allocator/collector integration. These are recorded in [memory layers](memory-model.md)
and [allocation proposals](allocation-encoding.md). The current Ref arena is
scaffolding, not a prerequisite ownership policy for the pointer layer.

## Next focus: library-defined unions

Prioritize execution and metadata fundamentals for Option<T> and Result<T,TError>
before reflection or broader object-model features. A union is an ordinary carrier
of one of a fixed set of variant types; reserve enum for integer-backed constants.
Follow a .NET 11-inspired attribute/member convention instead of introducing a
union type category or dedicated instructions. See [the convention](unions-and-enums.md).

The [generic metadata foundation and closed record values](generic-metadata.md)
are implemented, including field access, value copying, and static/instance members
on generic types. Closed generic native layouts, allocation, and typed memory access
are also implemented for supported field types. Marker custom attributes on types
and methods are implemented. Next comes the typed access/storage needed for ordinary
library carriers, with further attribute capabilities as required.
Then migrate System Option/Result from their bootstrap implementation. This work
must preserve explicit allocation and avoid requiring null or boxing for absence.

## Later milestones

1. `brfalse`, label-based `switch` tables, and equality/ordered comparison branches
   are implemented. Conditional branches
   support Boolean, integer, pointer, and prototype Ref operands. Compact constant and slot aliases are implemented; short branches remain pending.
   Add a control-flow verifier: typed stack states at joins, definite local
   initialization, valid return paths, and maximum stack calculation.
2. Extend type/generic metadata and implement arrays when their storage contracts
   are ready; see [arrays and pointers](arrays-and-pointers.md).
3. Implement a CLI-based binary reader/writer for the supported subset, preserving
   standard table/heap/token and opcode encodings where semantics permit. Define
   versioned extensions only for required deviations; see [format direction](format-direction.md).
4. Extend the platform-written System library and bootstrap linker into general
   loadable modules, structured error definitions, and generic unions. Keep familiar
   namespaces while defining contracts around Option and Result.
5. Introduce interfaces without naming prefixes, explicit dispatch metadata, and
   a modest collections library.
6. Build .NET metadata/IL inspection and translation for a supported subset, with
   actionable diagnostics for semantic differences.
7. Revisit explicit lifetime operations and generic ownership abstractions such as
   counted Ref<T>. Decide library versus VM support then, keeping optional memory
   management independent of the low-level VM. Evaluate native interop, concurrency,
   and runtime async against the required contracts.

The assembler must grow toward full platform expressiveness, with .NET ilasm as
the capability baseline; see [assembler design](assembler-design.md).

## Migration principles

Use [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
metadata and instruction concepts as the baseline vocabulary. Preserve familiar
namespaces, signatures where meaning permits, and ordinary arithmetic/control-flow
structure. Each incompatible behavior needs an explicit mapping or diagnostic.

Prefer a source recompile path first. Imported .NET class instances generally need
an explicit identity/ownership representation to retain aliasing and lifetime; converting them to frame-owned
copies silently would change programs. Imported structs can often remain owned
values, but boxing, reflection, interface dispatch, and layout still need work.

Translate void-return calls with their changed stack effects: neoCLR produces a
real Void value. Insert an explicit discard when adapting a CIL caller that
expects no result. Introduce `Option` for APIs whose nullable values mean absence;
do not rewrite null tests mechanically when null is a deliberate reference state.

Exception-heavy APIs need signature and control-flow adaptation to `Result`,
including explicit resource cleanup. A translation tool must diagnose unsupported
handlers rather than discard them. Faults cannot stand in for ordinary recoverable
exceptions without changing the contract. A future external .NET bridge may catch
host exceptions at that boundary and return structured Errors, but guest exception
semantics should not leak in.

Array covariance, byrefs, unsafe pointer arithmetic, reflection, dynamic code,
finalizers, disposal, async state machines, and unchecked integer overflow all
require deliberate treatment. Ordinary `add`/`sub`/`mul` already retain wrapping semantics; checked `.ovf`
operations terminate with Faults rather than throw. Surface required changes early
through a compatibility report rather than promise binary execution.

No importer, source compiler, bridge, binary metadata writer, or compatibility
analyzer is implemented yet. A useful migration success criterion is a small real
library recompiling with localized, explained changes and equivalent observable
behavior in the supported subset.
