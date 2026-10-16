# Raven operator qualification (2026-10-16)

Development support, paired with Raven main `7819e2fea` and its deliberate carry to
`codex/neoclr` at `164ebf81f`; metadata SDK element-address support is neoCLR `1b4561f3b`.
Broader structural work remains parked. Nothing here declares a release.

Raven compound overloads are `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`,
`<<=`, `>>=` and `>>>=`. Binary `>>>` is also supported.
A declaration such as `func +=(amount: int)` is a public instance method with one
by-value parameter and unit result. Applicable operators mutate writable variables;
ordinary properties/indexers keep binary-plus-setter fallback. Assignments retain
Raven's unit result. No Runtime Contract flag, carrier type or runtime opcode is
introduced. CLI metadata uses standard special-name methods; native metadata uses
ordinary instance method signatures and calls. This follows the
[C# 14 in-place operator model](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/proposals/csharp-14.0/user-defined-compound-assignment)
while preserving Raven assignment-result semantics.

## Evidence

- Raven main: 70 focused operator, conversion and reference-field tests pass on
  .NET 11, including an emitted-library consumer, ambiguity, class alias identity,
  struct locals/array elements, field/index side effects, and binary/property fallback.
- Native compiler line: 62 focused compound/operator/instance-plan/reference-field tests pass.
- Native `Main.rvn`: interpreter, macOS ARM64 AOT and managed JIT all return 42.
  AOT host uses undefined-behavior/bounds sanitizers. Covers class alias identity,
  explicit-over-binary precedence, struct-local mutation and a reference-array index
  evaluated once. See `results.json` for exact commands and artifact hashes.
- Native `ValueArrays.rvn`: interpreter returns 42; AOT explicitly rejects allocation
  of the nominal struct array. See `value-array-results.json`. This is an existing
  backend limitation, not a copy-and-writeback substitute or successful AOT claim.
- Metadata SDK `--array-address-integration`: CLI and native interpreter checked
  element-address/store/read return 42; malformed element types are rejected.

The matched bundle is `target/function-reflection-library-validated-20261016`.
The script records the compiler, metadata SDK, runtime, code generator, bundle and
source fingerprints for the supported full-backend case.

## Reproduce

Build Raven's native compiler against the matching SDK, then run from neoCLR:

```sh
python3 docs/validation/compound-operators/run.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --output target/compound-operators-check
```

Add `--value-arrays` with a separate output directory to exercise the interpreter
success and expected AOT rejection. That profile does not attempt JIT execution.
The scripts use the existing native host from `interface-variance` and do not
install or publish a bundle.

## Remaining boundaries

Checked forms, compound extension/interface/virtual declarations, instance
declaration modifiers are follow-up work. Native imported-library compound calls and other architectures are not
claimed by this fixture. The SDK is a host API documented in the
[manual reference](../../../api-docs/experimental-metadata.md#managed-array-element-addresses-development-2026-10-16);
no Raven class-library API or reference snapshot changed.

## Remainder/shift follow-up

Shared Raven main `b5e6bb0d3` and deliberate native carry `57c567509` add remainder
and shift assignments plus zero-filling binary `>>>`. Main passes 101 focused
tests; the native compiler line passes 104. The expanded `Main.rvn` passes
interpreter, macOS ARM64 AOT with sanitizers, and managed JIT, all returning 42.
It exercises the four additional explicit compound methods, int64 shift counts,
zero fill and masking of negative/width-sized int32 counts.
`shift-results.json` records this run; `results.json` retains the initial slice.
The same validated bundle is used. No runtime change or new opcode is required.

The .NET target follows .NET metadata conventions. NeoCLR uses those names for
this slice too; purposeful nominal capability contracts remain separate. The
author intends this operator work to support System.Numerics interfaces on
NeoCLR next. Implementing a plus operator alone does not make a type numeric.

## Instance increment follow-up

Raven main `94f133cab`, native carry `a28fe52ed`: parameterless `func ++()` /
`func --()` mutate writable variables. Prefix returns the updated value; discarded
postfix prefers the instance method, while consumed postfix uses the static
overload. Property/indexer access remains static-plus-setter. Standard CLI names
are `op_IncrementAssignment` and `op_DecrementAssignment`; native dispatch uses
ordinary calls. No Runtime Contract switch or runtime opcode is added.

231 focused main tests and 233 native-line tests pass. `increment-results.json`
records interpreter/JIT/AOT return 42 with class/struct prefix values, discarded
postfix mutation and consumed-postfix fallback. Existing array-allocation limits
remain unchanged.

## Truth and short-circuit follow-up

Raven main `7039b3e23`, native carry `198c22bea`: paired static `true`/`false`
use standard metadata names. `if`/`while` use truth testing; `&&`/`and` and
`||`/`or` call `&`/`|` only when the left truth test requires the right operand.
Direct logical overload declarations are rejected and must migrate to the truth
pair plus bitwise methods. No new Runtime Contract setting or native opcode.

107 focused main tests and 109 native-line tests pass. `truth-results.json`
records interpreter/JIT/AOT return 42, including evaluation counts and the case
where both truth tests return false. The source consumer remains an ordinary
nominal class example; this does not resume structural-type work.

## Full ordinary-operator matrix

Parser fixes on Raven main `75607f005` and native `8a82d185b` resolve logical-not
declarations versus bang macros, member access on parenthesized bitwise results
versus casts, and relational/logical chains versus generic arguments. Both lines
pass 87 focused parser/operator tests. Native intersection parsing is preserved.

The current `Main.rvn` exercises arithmetic `+ - * / %`, bitwise `& | ^`, shifts
`<< >> >>>`, unary `+ - ~ !`, all six comparisons, static and instance `++`/`--`,
implicit/explicit conversions, compound instance methods and truth-based logical
operations. The same source returns 42 on ordinary .NET 11 and on native
interpreter/macOS ARM64 JIT/AOT. Native AOT uses the existing sanitizer host.
See `matrix-dotnet-results.json` and `matrix-native-results.json` for commands
and fingerprints. Older result files describe the source revisions they tested.

The native command above remains valid. For ordinary .NET:

```sh
python3 docs/validation/compound-operators/run-dotnet.py \
  --compiler ../raven-ranges/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll \
  --core ../raven-ranges/src/Raven.Core/bin/Debug/net11.0/Raven.Core.dll \
  --output target/operator-matrix-dotnet-check
```

The helper copies the matching Raven.Core dependency beside the executable;
without it, direct `dotnet` execution fails to resolve Raven.Core.

This completes the tested ordinary operator spelling matrix, not every operator
form: checked contexts/overloads, compound virtual/interface/extension declarations,
System.Numerics interface conformance/generic dispatch, and native imported-provider
coverage remain distinct work. No platform capability is inferred from spelling.


## Checked declaration and metadata foundation

Raven main `29a458e27`, native carry `cb83fb9dd`: `func checked` declares the
standard checked arithmetic, explicit conversion, compound-assignment and
increment/decrement methods. Each requires an ordinary counterpart with matching
parameter/ref and return types. Source/import symbol identities are distinct;
checked explicit conversions import as conversions. Ordinary Raven expressions
continue to select ordinary methods. This follows .NET metadata conventions and
requires no runtime opcode, SDK change or Runtime Contract switch.

Both compiler lines pass 98 focused tests and the required syntax-generator build.
The tests invoke all 14 checked method identities, check invalid/unpaired forms,
partial generic declarations and imported metadata, and exercise a C# consumer of
checked arithmetic/explicit conversions. The repository's Roslyn 4.12 dependency
predates C# 14, so that consumer does not qualify instance operator syntax.
Operator arrow bodies also use the normal body lowering path, fixing invalid IL
for unit block expressions discovered by this matrix.

[CheckedDeclarations.rvn](CheckedDeclarations.rvn) directly calls the metadata
names to force each checked method to execute. It returns 42 on .NET 11 and native
interpreter/macOS ARM64 JIT/AOT, with the existing sanitizer host for AOT. These
calls qualify method representation and execution, **not** Raven checked lookup or
primitive overflow behavior. Ordinary arithmetic and conversion selection are
checked separately in the same sample. Exact commands and hashes are retained in
`checked-declarations-dotnet-results.json` and
`checked-declarations-native-results.json`.

Reproduce using the earlier native/.NET commands with the additional argument:

```sh
--source docs/validation/compound-operators/CheckedDeclarations.rvn
```

Use a fresh output directory. The native bundle remains
`target/function-reflection-library-validated-20261016`.

Checked contexts, overload selection, conversions and bounded constant evaluation
are qualified below. A checked declaration does not itself change arithmetic inside
its body. Broader interface/virtual/extension compound declarations remain separate.
Structural work stays parked.

## Checked expression follow-up

Raven main `935ab4af7` adds `checked(...)` / `unchecked(...)`, lexical context
precedence, integer binary overflow checking and checked binary `+ - * /` lookup.
The deliberate native carry `57fec6ec7` uses the six existing signed/unsigned arithmetic
instructions from SDK `1876fb72b` or later. No Runtime Contract setting, native
container change or new class-library API is involved. Ordinary expressions keep
their behavior. A checked method declaration does not implicitly check its body.

Both compiler lines pass 73 focused tests, covering runtime overflow, nested
contexts, nested-function boundaries, source/imported operator selection,
inapplicable-candidate fallback, ambiguity preservation, diagnostics and ordinary
operator/parser/operation regressions. Required syntax-generator builds pass.

`CheckedExpressions.rvn` exercises checked operator preference and fallback,
unchecked nesting, successful signed/unsigned primitive arithmetic and wrapping.
It returns 42 on .NET 11 and native interpreter/macOS ARM64 managed JIT/AOT.
`checked-expressions-dotnet-results.json` and `checked-expressions-native-results.json`
retain commands and artifact hashes. Use the existing .NET/native commands with
`--source docs/validation/compound-operators/CheckedExpressions.rvn` to reproduce.

`run-checked-overflow.py` generates six small Raven consumers for signed/unsigned
addition, subtraction and multiplication. Each faults in the interpreter, JIT and
AOT; native hosts check ArithmeticOverflow status/code 2 and an untouched result
slot. AOT uses the existing undefined-behavior/bounds sanitizer host. Exact source,
commands and fingerprints are in `checked-expressions-overflow-results.json`.

```sh
python3 docs/validation/compound-operators/run-checked-overflow.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --output target/checked-expression-overflow-check
```

Use a fresh output directory. Runtime/SDK baseline is neoCLR `1f1e99af3`.
The bundle remains unchanged; the updated compiler
is built against the current metadata SDK. .NET uses OverflowException; native
execution uses a terminal ArithmeticOverflow fault. This is the existing platform
fault contract, not a new checked-context exception model.

Remaining work includes lifted/pointer arithmetic and checked binary expression
trees. RAV0373 diagnoses unimplemented checked operations instead of silently
ignoring context. Later sections qualify conversions, unary/mutation operations,
blocks and bounded constant evaluation.
The current native primitive admission profile is unchanged. Structural work stays
parked. Imported operator selection is covered in compiler tests; this fixture
does not add native separate-provider qualification.

## Checked unary follow-up

Raven main `307f3b55c` and deliberate native carry `371363a67` add checked
signed unary negation and checked unary overload selection with ordinary fallback.
Each compiler line passes 73 focused tests, including floating negative zero,
small-integer promotion, nested unchecked behavior and .NET NegateChecked trees.
The expanded CheckedExpressions consumer returns 42 on .NET 11 and native
interpreter, managed JIT and AOT on macOS ARM64. The checked-unary-*-results.json
files retain exact commands and hashes. Earlier binary results describe the earlier
sample revision; these files qualify the expanded source.

Run the existing positive consumer command with CheckedExpressions.rvn; pass
`--unary` to run-checked-overflow.py for separate int/long minimum-negation cases.
Both cases fault on all three native backends; JIT/AOT preserve the result slot.
Runtime/SDK baseline `1f1e99af3` and bundle
`target/function-reflection-library-validated-20261016` are unchanged. No runtime
opcode or Runtime Contract switch is added. Unary negation is no longer a compiler
gap; checked binary expression trees and broader constant-expression coverage
remain follow-up work.

Native faults are terminal, not recoverable exceptions. See the separate
[recoverable arithmetic proposal](../../design/checked-arithmetic.md).


## Checked conversions

CheckedConversions.rvn checks narrowing, unsigned widening, floating truncation,
nested unchecked contexts and ordinary/checked user-defined conversion selection.
The shared .NET runner and native runner use the same source. The native overflow
runner's --conversions mode checks signed narrowing, an unsigned-source boundary
and floating-to-integer overflow. Overflow is terminal ArithmeticOverflow on
NeoCLR, not an exception or Result; native fault hosts also check the result slot
is untouched.

The compiler retains checked context through bound rewriting and exposes
IConversionOperation.IsChecked. Conversion lookup replaces matching ordinary
explicit declarations with checked declarations before selecting the best candidate;
unpaired ordinary and implicit conversions still participate. .NET uses conv.ovf
instructions; the native adapter maps the same intent to existing SDK instructions.
No metadata format or Runtime Contract switch changes. Nullable/pointer checked
casts and checked user-defined expression trees remain diagnosed. Block and
compile-time overflow qualification follows below.


Validation records: [ordinary .NET](checked-conversions-dotnet-results.json),
[native source-local](checked-conversions-native-results.json),
[native imported provider](checked-conversions-import-results.json), and
[native overflow cases](checked-conversions-overflow-results.json). Main revision
`0c272f423` passes 106 focused tests and the 421-test overload-resolution suite.
The native carry also passes the 106 focused tests. Native consumers use the
`numerics-public-libraries-20261016` bundle and retain exact tool/source hashes.
The source-local result predates only the imported-symbol classification correction;
the imported consumer and overflow cases use the corrected adapter.

To reproduce imported conversions, compile CheckedConversionProvider.rvn as a
Library project with AssemblyName CheckedConversionProvider and the bundle's
NeoCLR.ClassLibrary.props. Run run.py with CheckedConversionConsumer.rvn and
--reference <CheckedConversionProvider.dll>. To reproduce fault checks, use
run-checked-overflow.py --conversions with the same compiler and bundle.

The completed native compiler carry is `c290f2536`. All three fault cases pass on
interpreter/JIT/AOT; ordinary .NET and both positive native consumers return 42.

## Checked mutation

`CheckedMutation.rvn` is shared by .NET and native runs. It exercises ordinary and
checked instance compound operators, prefix results, discarded postfix, byte
write-back and an array receiver/index with observable side effects. The array is
initialized inside the method because native static nonconstant initializers remain
unsupported. The compiler fix captures locations before read/modify/write and keeps
checked operator/conversion intent through both emitters. Native statement lowering
also unwraps checked expressions before dispatching assignments and discarded blocks.
No runtime opcode, metadata format or Runtime Contract switch changes.

Raven main `9dd7ea722` passes 159 focused mutation/conversion/operator tests and the
421-test overload-resolution suite. The native carry passes the same 159 focused
tests. NeoCLR overflow remains terminal ArithmeticOverflow, while .NET uses
OverflowException. Built-in overflow prevents write-back; checked lookup does not
make user-defined bodies transactional. Nullable/pointer checked mutations and
mutation expression trees remain explicit gaps.

Reproduce the positive consumer with `run-dotnet.py` or `run.py`, selecting
`--source docs/validation/compound-operators/CheckedMutation.rvn`. Run native fault
checks with `run-checked-overflow.py --mutations`; it covers int compound-add
and byte increment overflow, including untouched native result slots.
Use the matched `target/numerics-public-libraries-20261016` bundle.

Completed native carry: `a72b1b24c`. The shared consumer returns 42 on .NET 11,
interpreter, managed JIT and sanitized AOT. Both native overflow probes pass on
all backends. Exact tool/source/bundle hashes and command results are retained in
[.NET results](checked-mutation-dotnet-results.json),
[native results](checked-mutation-native-results.json) and
[overflow results](checked-mutation-overflow-results.json).

## Checked blocks and constant evaluation

Raven main `3fa828516` and native carry `48b0b6f75` add checked/unchecked statement
blocks. They reuse lexical context, ordinary block scope and control flow; nested
functions and member bodies start fresh. Both generator builds and 71 focused
block/expression/mutation tests pass. `CheckedBlocks.rvn` returns 42 on .NET 11,
interpreter, managed JIT and sanitized AOT. `CheckedBlockOverflow.rvn` faults with
ArithmeticOverflow on all native backends and preserves native result slots.

[.NET block results](checked-block-dotnet-results.json),
[native block results](checked-block-native-results.json), and
[block overflow results](checked-block-overflow-results.json) retain tool/source hashes.
Reproduce with `run-dotnet.py` or `run.py --source <source>`; use `--overflow` with
the native overflow source. The native bundle is `numerics-public-libraries-20261016`.

Raven main `489cc12a4` evaluates resolved built-in constant fixed-width addition,
subtraction, multiplication, negation and numeric casts. Default constant context
is checked; explicit unchecked integer constants wrap. Const locals/fields retain
typed values and dependencies. User-defined operators are never executed by the
compiler. Decimal overflow does not wrap. These rules mirror .NET; no runtime
metadata, opcode or Runtime Contract configuration changes are needed.

Native-width and enum constant arithmetic are not expanded. Out-of-range unchecked
floating conversions remain backend-dependent and are not folded. Checked binary
expression trees and existing nullable/pointer gaps remain separate.

`ConstantArithmetic.rvn` checks local dependencies, narrowing and wrapping.
Int32 native class constants are qualified in the following section; this fixture
retains its local-constant coverage.
`ConstantOverflow.rvn` checks addition, narrowing, multiplication and negation
rejection. `run-constant-diagnostics.py` requires four RAV0374 diagnostics and no
emitted assembly on either target. Pass `--dotnet-compiler`, `--native-compiler`,
`--core`, `--bundle` and a fresh `--output` directory. Positive consumers use the
existing .NET/native runners with `--source ConstantArithmetic.rvn`.

Validated native carry: `9ea79f0e2`. Both compiler lines pass 175 focused tests;
main also passes the 421-test overload-resolution suite. Both generator builds pass.
The shared local-constant consumer returns 42 on .NET 11, interpreter, managed JIT
and sanitized AOT; both compilers reject all four overflow cases without an assembly.
[.NET constant results](constant-arithmetic-dotnet-results.json),
[native constant results](constant-arithmetic-native-results.json), and
[compile-time rejection results](constant-diagnostics-results.json) retain exact
commands and source/tool/bundle fingerprints. Wider class-constant payloads remain a native gap; Int32 support is qualified below.


## Additional constant operators and native class constants

Raven main `84bdf63bf` / native `c59392f96` add division/remainder, bitwise operations,
complement and masked shifts to typed constant evaluation. Both lines pass 80 focused
tests. Main `49ca35b30` / native shared-binder carry `5f9a9c4f3` extend arithmetic
parameter defaults (77 focused tests per line, plus 421 overload tests on main).
Native callable admission still rejects optional declarations, so this is .NET
support, not native default-parameter metadata support.

Main `e44a7ca96` / native `d194b4bda` materialize numeric binary constant results
at lowering time. ConstantDivision.rvn verifies unchecked minimum/-1 division and
minimum%-1 remainder without runtime faults. Both lines pass 150 focused tests;
the native consumer returns 42 on interpreter/JIT/AOT.

NeoCLR `e453a7bbf` adds storage-free Int32 literal-field metadata and SDK roundtrip/
reference projection. Raven native `1b00893b3` admits declarations and source/imported
reads. ClassConstants.rvn and ClassConstantsConsumer.rvn both return 42 on
interpreter/JIT/AOT. Emit a Library project named ClassConstantsProvider containing
the Constants declaration, then pass its DLL to `run.py --reference` for the consumer.
Rebuild both hosts with `cargo build -p neoclr -p neoclr-codegen`; an older host
rejects literal_fields. The numerics-public-libraries-20261016 bundle remains unchanged.
Other class constant payload types and guest field-reflection enumeration remain gaps.

Evidence: [operators](constant-operators-native-results.json),
[constant division](constant-division-native-results.json),
[source class constants](class-constants-native-results.json), and
[imported class constants](class-constants-import-results.json).
The SDK `--class-constants` checks CLI/native/reference roundtrips, metadata values,
visibility and storage exclusion; runtime `literal_field_tests` checks persistence
and malformed duplicate rejection. API snapshot fingerprints pass unchanged.

## Readonly field qualification and remaining admission limits

ReadonlyFields.rvn returns 42 on .NET with both instance and static readonly storage.
ReadonlyInstance.rvn returns 42 on native interpreter/JIT/AOT with constructor-initialized
private val storage and a public getter. Native static field initializers remain
rejected. They require type initialization, distinct from literal metadata support.
See [the implementation requirements](../../design/readonly-fields.md).

`run-constant-admission.py --compiler <native rvnc> --bundle <bundle> --output <fresh directory>`
checks that arithmetic optional parameters and static readonly initializers are
rejected explicitly with NEOMETA001 and no emitted assembly. These are recorded
limitations, not successful feature execution. [Admission results](constant-admission-results.json),
[.NET readonly results](readonly-fields-dotnet-results.json), and
[native instance-readonly results](readonly-instance-native-results.json).

Remaining constant work includes native-width/enum expression coverage, wider native
class-literal payload types and native optional-parameter metadata/import. Structural
work remains parked. Static readonly initialization is the next author-directed
runtime/compiler feature, with once-only execution, ordering, reentrancy, failure,
closed-generic storage and publication semantics still to implement.

Ordinary .NET evidence also includes [arithmetic defaults](constant-defaults-dotnet-results.json)
and [materialized division](constant-division-dotnet-results.json), each returning 42.
