# Ordinary struct equality and hashing

Author-directed class/struct soundness work, 2026-10-20. This is development
qualification, not a release-wide gate or exhaustive support for every field shape.

## Contract and .NET comparison

Ordinary nominal structs need default Object.Equals/GetHashCode behavior even when
they declare no overrides. Equality requires the same closed runtime type and equal
instance fields; empty structs of the same type compare equal. Null or a different
type compares false. Field equality uses each field's Object equality contract;
reference identity is correct only when that field's class has no override. Explicit
struct overrides take precedence, while same-name ordinary methods do not replace a
virtual slot. Static fields do not participate. These rules follow
[ValueType.Equals](https://learn.microsoft.com/dotnet/api/system.valuetype.equals).

Hashes combine field hashes and agree for equal values. Neither exact CLR hash
numbers nor persistent hash identities are promised. Custom equality still needs a
matching custom hash override. String/Char fields retain neoCLR's documented
canonical Unicode equality; .NET's ordinary String equality is ordinal.

The author explicitly confirms that EquatableTo remains user-implemented. This work
adds neither that interface nor ordinary struct operators. Records keep compiler-
generated members and ordinary class/struct representations. No object layout,
native calling convention, CLI contract mapping or Raven compiler changes are needed.
System.Value remains an erased payload carrier, not a new spelling of ValueType.

## Execution implementation

`src/value_defaults.rs` constructs runtime-owned method bodies. Interpreter Object
dispatch uses them only after checking explicit overrides. Native preparation
materializes them in a private execution projection before closed specialization
and selection, retaining existing call/rooting machinery. Original source metadata
and access checks remain authoritative; reflection catalogues use the original
source snapshot. A nonserialized runtime-only slot marker identifies the helpers;
a source method with a similar name cannot acquire fallback privileges. The interpreter
grants that trusted marker access only to its own instance fields; ordinary callers
retain the existing field access checks. Their names
are collision-checked and do not add declared source members or public APIs.

Direct integer and Boolean field comparisons/hashes avoid additional managed field
boxes. Wider integer hashes include both halves. Native bitwise admission now accepts
matching Int32, Int64 and native-integer operands, consistent with the emitter's
existing width support. Other fields use Object dispatch and can require temporary
boxes. The interpreter constructs execution bodies and copies values; this is not a
zero-allocation implementation or a throughput claim. More optimized field-specific
paths can follow measurement without changing equality semantics.

Borrowed views, primitive definitions and enum definitions do not receive these
ordinary-struct bodies. Other field shapes remain bounded by each engine's existing
boxing/dispatch support; in particular native floating-point/enum boxed adapters
remain gaps. Unsupported shapes must not silently fall back to object identity.

## Validation

`tests/value_defaults.rs` covers nested values, reference overrides, null/empty
values, exact nominal identity, floating field equality/hash in the interpreter,
integer/Boolean fields with a two-object heap limit, and private helper identity.
The heap limit is a focused storage check, not an allocation or performance benchmark.
`tests/object_equality.rs` retains existing primitive/override/identity coverage and
updates the obsolete rejection expectations for ordinary structs and hiding methods.

The native executable test covers separate boxes, both integer widths, equality,
hashing and unequal fields. `Program.rvn` exercises direct inherited calls, Object
calls, empty/nested/generic structs, canonical string fields and custom class-field
equality. `check.py` runs the same compiled consumer in interpreter/JIT/AOT and
records artifact hashes. Matching compiler: Raven `8da3a45f8` on `codex/neoclr`,
local bundle label `4c051e332-dirty-record-policy`; source class libraries are
unchanged from the HttpVersion qualification.

Six focused default tests and six existing field-access regressions pass.
The source consumer passes interpreter, JIT and AOT; `validation.json` records the
artifacts and results. `metadata-validation.json` confirms ten selected fallback
helpers and no such members in the source reflection catalogue. The native default
and existing Object identity/override/null regression each pass. The existing broad
Object suite passed 38 tests with one obsolete rejection expectation; after updating
that expectation, the focused hiding-method test passes. This is not a repeat of the
full suite after every change.

macOS local executables may require ad-hoc signing to avoid the previously recorded pre-harness
dyld stall; that host delay is not an execution test result.

Focused commands (use the repository's configured macOS SDK/toolchain when required):

```sh
cargo test -p neoclr --features native-metadata --test value_defaults --test field_access
cargo test -p neoclr-codegen --test console ordinary_struct_defaults_execute_in_native_code
python3 docs/experiments/struct-defaults/check.py \
  target/struct-default-matched-bundle/bundle target/struct-default-matched-app \
  --output docs/experiments/struct-defaults/validation.json
```
