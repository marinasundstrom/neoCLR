# Native console-input prerequisites (2026-10-07)

The [outcomes sample](outcomes.rvn) compiles Raven to native neoCLR metadata/IL and
then a standalone macOS ARM64 executable. It uses the actual runtime library's
`Result<Option<byte>, ConsoleReadError>`: bytes 0, 128 and 255, EOF, Unavailable and
ReadFailed. Nested `if let` and `let ... else` distinguish the cases. This sample
constructs fixed outcomes; it does **not** read stdin.

[Validation evidence](validation.json) records the pinned compiler, runtime library,
source/artifact hashes, interpreter execution, matching AOT inspection/emission reports,
no object imports and execution with only the executable present and an empty environment.
The executable links only macOS libSystem. `Outcomes.pe` and `ReadByte.pe` are the
producer artifacts, not native executables.

## Metadata-only generic relationships

Option and Result implement different closed forms of Propagatable. Option also uses
Void as its error argument. The previous AOT load-set path tried to specialize these
metadata-only interface shapes as executable values and rejected the nested result.
Now the fully verified original relationships are removed from the private projection
**before** executable specialization. The report substitutes the owner's actual generic
arguments into each original relationship, preserving its constructed types, source
owner identity and metadata-only Void. Interface rows do not consume native shape limits.

Original runtime conformance verification still precedes this step. Executable generic
values still have the one-shape-per-definition limit; executable Void payloads, interface
storage/dispatch, explicit implementation mappings and selected generic methods remain
unsupported. This is bounded code selection, not general trimming or a metadata sidecar ABI.
Twelve focused load-set tests pass, including the new distinct-interface-shapes/Void
regression and existing invalid-conformance, generic-value and access-control checks.

## The actual read boundary

[read-byte.rvn](read-byte.rvn) calls the existing `Console.ReadByte()`. The pinned
interpreter returns exit 0 for EOF or byte 42 and exit 2 for another byte (tested with
`x`). The original report records rejection of the static Console owner. With the subsequent
static-owner slice, inspection/emission now reaches the erased `Value` local and rejects
it explicitly. That report retains the Value boundary; the current script asserts the later generic-method boundary and absence of an object.

Removing that first restriction alone will not supply native input. The existing
[console contract](../../console-io.md) implements the public method in Raven; its
runtime service transports Byte, Void or Int32 status through erased `System.Value`.
The wrapper calls generic value-test/unpack services. These are further native backend
requirements, not permission to replace the public method by name with a host intrinsic.

The bounded primitive erased-value slice below now passes; generic helper calls and
an explicit native input capability/service contract remain. Service tests must distinguish EOF,
zero/high bytes, unavailable capability and I/O failure, including interrupted reads.
An input-driven ASCII integer parser can consume bytes without allocating strings;
UTF-8 line decoding and text ownership remain subsequent work.

## Comparison and provisional direction

This reuses the platform's existing input contract rather than adding a new public API.
.NET's [Stream.ReadByte](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.readbyte?view=netframework-4.8.1)
returns an Int32 with -1 for EOF. Rust's [Read](https://doc.rust-lang.org/std/io/trait.Read.html)
separates I/O errors from counts, with zero indicating EOF for a nonempty buffer
(primary API documentation reviewed 2026-10-07). neoCLR's existing nested union makes
byte/absence/failure explicit in the type; its cost here is nested layout and generic
specialization. No speed or allocation advantage over those platforms is claimed.

A byte service needs no retained guest buffer: it returns a copied scalar. Starting
there postpones the choice among bounded owned text buffers, tracing and reference
counting without selecting any of them. Compiling the ordinary wrapper preserves its
status-to-union semantics, but requires more backend work than a hard-coded Console
intrinsic. A versioned, explicitly enabled host service linked into the executable is
preferred provisionally; its ABI, binding validation and failure behavior are still to
be implemented and tested. General .NET-style stream objects and native managed text
would address broader scenarios but are unnecessary for the first byte-input consumer.

## Reproduce

Set SDKROOT to the matching Xcode SDK on macOS ARM64, then run:

```sh
python3 docs/experiments/aot-input/verify.py \
  --compiler /absolute/path/to/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --reuse-read-byte \
  --bundle /absolute/path/to/neoclr-native-poc \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-input-outcomes
```

The output directory must not exist. Producer fixtures use the same pinned bundle as
the [library Result experiment](../aot-library/README.md); no shared framework/runtime
is needed to execute the resulting native outcomes app.


## Static member owners (2026-10-07)

The [static factory consumer](static-outcomes.rvn) now compiles ordinary static methods
returning the nested input-result shape. [Updated evidence](static-validation.json)
repeats both native consumers and records ReadByte's erased-Value boundary. No Console
method is replaced by a special intrinsic.

A metadata-only owner must be a nongeneric, empty, abstract sealed reference record,
without an explicit base, enum information, packing, minimum size or generic constraints.
The owner contributes nominal method identity and access checks; it contributes no
receiver or native object allocation. Static methods remain ordinary compiled bodies.
Local/parameter/result storage, fields and instance receivers still cannot use that type.
The same predicate is applied by specialization and value admission. Empty lexical union
companions continue to work. Static state, generic static owners and reference allocation
are outside this profile.

This follows C#'s [static-class distinction](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/classes-and-structs/static-classes-and-static-class-members)
(primary documentation reviewed 2026-10-07): member grouping does not require an object
instance. This slice adds backend coverage of existing Raven/native metadata behavior,
not a language or runtime contract. Unlike general .NET static-class support, this AOT
profile admits only empty owners and does not add type-initialization or static-storage
machinery. Preserving owner identities costs a metadata row but avoids flattening names
or weakening library visibility. General managed-reference support would be unnecessary
for these calls and remains separate.

Focused tests cover whole-module/closed-world execution, library calls with and without
generic value specialization, and rejection of storage, field-bearing/non-static owners
and private/internal access. The current consumer's library wrappers still need erased
value operations, generic runtime helper calls and service binding before native input
can execute; passing the static owner boundary does not imply those features work.


The static-owner validation encountered an intermittent producer diagnostic on unchanged
ReadByte source (`Console has no member ReadByte`); an immediate retry with the same
input files succeeded. The evidence retains the failed command and records the retry;
backend boundary checks reuse `ReadByte.pe` and its earlier interpreter evidence.
`--reuse-read-byte` reproduces that focused path. This is a pinned Raven reproducibility
limitation, not a compiler issue fixed by the AOT change. `StaticOutcomes.pe` was freshly
compiled from the static factory source and executed natively. All 43 focused load-set
and value-profile tests pass.

## Bounded erased transport (2026-10-07)

The [erased CIL sample](erased-transport.neoil) now executes natively with explicit
`value.pack`, `value.is` and `value.unpack` for Int32, Byte, Boolean and Void. This
hand-authored backend consumer models the byte/EOF/error transport needed by the input
wrapper; it performs no input. [Evidence](erased-validation.json) records interpreter
parity, identical inspection/emission selection, no object imports and standalone ARM64
execution with only libSystem linkage. The actual Raven ReadByte fixture now passes the
Value-storage boundary and stops at selected generic methods. No public Console method
is replaced or recognized specially.

The private native representation is two 32-bit lanes: an exact primitive tag and a
copied payload. Tags are compilation details, not runtime type ordinals, serialized
metadata or a native service ABI. Byte packing uses the runtime's storage truncation;
unpacking a Byte restores the Int32 evaluation-stack category without losing the Byte
tag. Void is a valid erased payload. Boolean, Byte and Int32 remain distinct even when
their numeric payloads coincide.

Erased values can cross direct calls/results, local and argument storage, branch joins,
and ordinary borrowed/output slots. A mismatched unpack returns the existing experimental
RuntimeError status (3), terminates the invocation and leaves the exported result
untouched. No exception crosses the C boundary. The ordinary verifier still checks
initialization, signatures and borrow lifetimes. `initobj Value` is rejected because the
runtime defines no default for Value; a zero tag is not permission to synthesize EOF.
Record fields containing Value, record/reference payloads, nested erasure, generic Value
arguments, reflection, generic methods and native input services remain unsupported.
The exported entry still accepts/returns only the existing Int32 contract.

This implements a bounded subset of the existing [erased-value contract](../../erased-inputs.md).
.NET [boxing/unboxing](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/types/boxing-and-unboxing)
uses an object representation and rejects an incompatible unbox with InvalidCastException
(primary documentation reviewed 2026-10-07). neoCLR's existing explicit erasure similarly
retains the exact payload type, but an invalid unpack is a runtime Fault. This backend
uses an inline primitive representation to avoid needing object lifetime machinery for
this consumer. The cost is a second lane and a strict payload whitelist; it does not
implement general .NET boxing, object identity or a universal Value representation.
General boxes or variable-size tagged payloads would support more types but require
lifetime/layout contracts not exercised by byte input. No performance advantage is claimed.

Four focused erased-value tests cover the complete primitive type-test matrix, copies,
call results, output/address transport, control-flow joins, incorrect unpack propagation
and rejection of defaults, uninitialized slots and unsupported operations even in dead
code. The full isolated AOT suite has 67 tests.

Reproduce the current backend slice without rebuilding unaffected Raven fixtures:

```sh
python3 docs/experiments/aot-input/verify_erased.py \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output target/aot-input-erased-transport
```

The next bounded task is specialization of the ordinary generic primitive test/unpack
helpers with original member identities preserved. Native service binding and explicit
capability/error tests follow; interactive parsing and UTF-8 text ownership remain later.

## Primitive generic method specialization (2026-10-07)

The [Raven generic-method consumer](generic-methods.rvn) now compiles `Identity<T>` and
`Forward<T>` on an ordinary static class for Int32, Byte and Boolean. Both PE and NEOX
inputs execute natively. [Pipeline evidence](method-validation.json) covers freshly
compiled Raven metadata, interpreter parity, matching inspection/emission reports and
standalone ARM64 deployment. `GenericMethods.pe` is the checked producer fixture.

Closed-world preparation binds each original call by name, owner, instance mode, exact
substituted parameters, generic arity and supplied definition identity. It interns
primitive method shapes, copies ordinary bodies, substitutes signatures and type-bearing
instructions, and rewrites calls to explicit private clone identities. This supports
forwarding one method parameter into another generic call and different instantiations
whose nongeneric call signatures are identical. There is no helper-name intrinsic.

The scope is static methods on nongeneric owners (or module functions), up to four
Int32/Byte/Boolean/Void method arguments, 32 clones and 128 selected functions. Generic
constraints, generic instance methods, generic methods on constructed owners, reference
arguments and recursive call graphs remain rejected. Value-type specialization still
permits only one shape per definition. Void method arguments are useful for `value.is`
and `value.unpack`; they do not enable Void local or field storage.

`specialization.methods` records each source definition, source origin, arguments and
private expanded row. Selected function rows retain the original source identity/name,
with `compiledName`, `expandedIndex` and `methodArguments` describing the clone. Excluded
original generic templates are not executable bodies; their selected instances appear
separately. Original metadata is unchanged. Clones receive unused private method tokens
and absent parameter-row markers, retaining assembly/access facts. Original metadata is
verified before this projection, including standalone modules with generic clones, so
new tokens cannot repair an invalid source. These names/tokens are private build details,
not stable native exports or a deployed metadata sidecar.

This extends the existing [.NET Native AOT comparison](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/):
compile closed generic value shapes ahead of execution, accepting code-size growth as a
cost. The bounded compiler uses copied bodies rather than general shared generic code
or runtime dictionaries. This keeps exact primitive type tests straightforward but caps
coverage and duplicates bodies. No performance advantage or general .NET generic support
is claimed. Exact member binding and original-scope verification remain the foundation.

The actual ReadByte call now exposes the next dependency: `RuntimeServices.IsValue<T>`
resolves to a managed IL body in the System seed. That seed remains **validation only**,
so its bodies are not silently imported. The current reproduction scripts assert this
missing-body boundary. Selective managed seed-body compilation needs an explicit opt-in
contract and must still reject internal calls/native services until their bindings exist.

```sh
python3 docs/experiments/aot-input/verify_methods.py \
  --compiler /absolute/path/to/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output target/aot-generic-methods
```

Tests cover multi-shape erased type tests/unpack, forwarded generic arguments, original
library revisions, bad signatures/arity/identities, private access, constraints, recursive
calls, 32-versus-33 clone bounds and malformed original source tokens.

## Explicit managed System bodies (2026-10-07)

Closed-world inspection/emission now accept `--compile-system` together with an explicit
`--system`. This adds the verified seed to the input inventory and selects reachable
managed bodies under the existing admission rules. The default remains validation only.
Reports include `loadSet.runtimeContext.compileSystem`, the seed inventory and original
System member identities, including specialized methods. Duplicate flags and a missing
explicit seed fail before emission. InternalCall/native services remain unsupported.

[The backend consumer](seed-helpers.neoil) calls the pinned seed's actual generic
`RuntimeServices.IsValue<T>` and `UnpackValue<T>` bodies. It checks Byte truncation,
exact Byte/Int32 tests and Void tagging, then exits successfully. Assemble it against the
explicit runtime context first: the AOT tool's standalone text assembler does not resolve
seed types. This is a CIL backend consumer, not an additional Raven producer claim.
[Recorded evidence](seed-validation.json) checks interpreter parity, matching inspection
and emission, preserved System source identities, no unresolved object imports, and an
isolated executable with only the platform libSystem dependency. The 23 focused linking
and inspection tests pass, including nonpublic access and InternalCall rejection.

```sh
python3 docs/experiments/aot-input/verify_seed.py \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output target/aot-explicit-system-code
```

The unchanged Raven `ReadByte.pe` now passes seed helper selection and stops at
`specialization requires closed reference-free local value types: String`: the ordinary
wrapper includes an invalid-status failure path. It still emits no object. Supporting
that path and explicitly binding the native byte-input service are subsequent tasks.
The subsequent [explicit fault slice](../aot-scalar/README.md#explicit-guest-faults-2026-10-07)
adds terminal `UserFault` status propagation in scalar/value code; it does not yet admit
the wrapper's String-based System.Fail call.
Existing reproduction scripts without the opt-in retain the earlier missing-body boundary.

This extends the .NET Native AOT build-time dependency comparison above: managed seed
helpers can be baked into the image, with explicit input selection and bounded native
coverage. This does not make arbitrary runtime services compilable or change the public ABI.

## Immutable literal transport (2026-10-07)

The [Raven literal consumer](literals.rvn) now compiles to native metadata/IL and a
standalone ARM64 executable. It forwards Unicode and empty String literals through an
ordinary managed function and returns 42. [The CIL consumer](literals.neoil) additionally
exercises output slots, indirect loads, copies, branch joins and embedded NUL. This
validates transport and preserved payload bytes, not text operations or console output.

Literal values use one private 64-bit pointer to immutable image data containing a
64-bit UTF-8 byte count followed by the exact bytes. This extends the existing scalar
Hello World representation to value-profile arguments/results, locals and output slots.
All admitted String values originate from literals; the Int32 export cannot import a
String pointer. Original verification still checks initialization and borrowed output
contracts. No allocation, ownership count or general managed String layout is introduced.
Fields, erased String payloads, generic String arguments, defaults, dynamic producers
and String operations remain rejected. The representation is not a public native ABI.

[Recorded evidence](literal-validation.json) includes a fresh producer with the pinned
compiler revision, interpreter/native exit 42, matching inspection/emission, no unresolved
object imports and execution from an otherwise empty directory/environment with only
libSystem. PE and NEOX fixtures pass native tests; object checks preserve UTF-8 lengths
and bytes. The 80-test isolated AOT suite passes, including rejection of unsupported
dead IL (the old String rejection cases now use an unsupported bitwise instruction).

```sh
python3 docs/experiments/aot-input/verify_literals.py \
  --compiler /absolute/path/to/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /absolute/path/to/neoclr-native-poc \
  --output target/aot-literal-transport
```

With `--compile-system`, unchanged ReadByte now passes String signature selection and
fails native admission at `neoCLR.Runtime.ConsoleReadByte: unsupported value member
contract`. Its ordinary System.Fail path is retained and also needs a native failure
service binding. No object is emitted. `verify_seed.py` now asserts this newer boundary;
its earlier recorded evidence remains historical.

This reuses the [.NET String baseline](../../string-storage-design.md#net-baseline-and-layers):
.NET's immutable reference semantics and general managed UTF-16 storage are broader
than this literal-only representation. Static UTF-8 data avoids runtime allocation for
this bounded consumer; the cost is no dynamic text, identity/interning contract or general
String operations. No performance comparison is claimed. Native service bindings are
the next slice; reference counting versus GC for dynamically produced text remains open.

## Failure-service binding and host diagnostics (2026-10-07)

The next [fault diagnostic slice](../aot-fault-details/README.md) supplies an explicit
`--bind-user-fault` binding for the verified runtime failure contracts. It compiles the
ordinary System.Fail wrapper, preserves user messages and captures managed callers in
ABI v3 fault records. The interpreter and native renderer share code/message rules and
a 64-frame truncation limit. The byte-input service remains the next execution boundary.

## Native input (2026-10-08)

The [Console slice](../aot-console/README.md) now supplies the explicit input binding
and runs this ReadByte fixture as a standalone ARM64 executable. The earlier boundary
reports above remain historical evidence and still describe behavior without opt-in.
