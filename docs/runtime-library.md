# Runtime library boundary

Try the latest local collection APIs with the [Raven .12 installation](raven-collections-local-build.md).

The [API design policy and contract inventory](api-design.md) defines when System
APIs use values, managed references, reference receivers, outputs, and native pointers.

The basic runtime library is written **for neoCLR**, assembled into its metadata
and instruction representation, and executed by neoCLR. Host Rust implements the
interpreter and unavoidable bootstrap services; it should not become the BCL's
implementation language by accident.

The Raven-target experiment should expose an adapted version of this same library,
previously demonstrated through Neo. Its compiler reference artifact is a declaration
view, not a separate executable BCL. The desired demo is Raven source using useful
existing library contracts and executing their platform implementations. See the
[shared-library demo direction](raven-target-experiment.md#desired-demo-the-existing-runtime-library-through-raven-2026-09-12).

For new features, first try composing existing IL operations. If that fails, identify
whether the gap is a platform fundamental or an unavoidable host operation. An IL
wrapper alone does not make host-implemented policy a platform-written implementation.
Keep temporary bootstrap helpers explicit and describe what is needed to replace them.
In particular, the current whole-file text helper is a bounded demonstration, not the
foundation of a permanent Stream API. General I/O abstractions should follow concrete
needs and the primitives required to implement them in platform code.

[System.neoil](../runtime/System.neoil) provides platform-written methods and explicit native declarations:

- [Eager ArrayList filtering and searches](arraylist-filtering.md) use Func<T,Boolean>
  and ordinary iterator/Option IL. Find returns Option<T> without invalid defaults.
- [Comparable, Iterable and Iterator](common-interfaces.md) provide scalar ordering
  and managed ArrayList traversal, implemented entirely in platform IL. List<T>
  inherits Iterable<T>; Iterator<T> inherits Disposable.
- Ordinary `System.Option<T>` and `System.Result<T,E>` provide variant constructors, predicates and checked accessors. Their Option.None/Some and Result.Ok/Error cases are ordinary nested types. The carrier methods are platform IL; see the [member convention](union-convention.md).
- `System.IO.File` provides bounded ReadAllText; see [file input](file-input.md).
- `System.IO.File.ReadAllText` adapts bounded file failures to the ordinary
  `System.Result<String,System.IO.FileReadError>` carrier; the canonical method now uses nested cases.
- `System.Error` provides FromMessage, get_Message, and ToString, with explicit Message property metadata; see [Error values](errors.md).
- `System.Array<T>` provides six IL methods for explicit allocation, length, checked
  access, element addresses, and free, with explicit Length property metadata; see [arrays and pointers](arrays-and-pointers.md).
- `System.String` provides Concat, Equals, IsEmpty, GetUtf8ByteCount, and SliceUtf8;
  see [the text contract](text-model.md).
- The UnionAttribute marker has an ordinary IL constructor.
- `System.Console.ReadByte()` adapts the host console boundary to the ordinary nested
  `System.Result<System.Option<Byte>,System.IO.ConsoleReadError>` carrier. EOF is `Option.None`,
  a byte is `Option.Some<Byte>`, and host read failures are `Result.Error<System.IO.ConsoleReadError>`.
- `System.Console.WriteLine(string)` calls the host output primitive.
- `System.Console.WriteLine(int32)` calls the Int32 receiver's `ToString()` and then
  the string overload.
- `System.Int32.ToString()` is an instance method that calls the formatting helper.
- `System.Int32ParseError` models InvalidFormat and Overflow with ordinary nested cases.
- `System.Int32.Parse(string)` constructs an ordinary System.Result<Int32,Int32ParseError> from an erased
  host parsing payload; see [the migrated parsing boundary](int32-parse.md).
- `System.Int32.Divide(int32, int32)` checks zero and overflow, then executes `div`
  or returns IntegerDivisionError with DivisionByZero/Overflow cases. Its control flow
  and ordinary Result construction are platform IL.
- `System.Math.Abs(int32)` computes absolute value and represents overflow as an
  ordinary OverflowError. Its logic is entirely platform IL.

The remaining host calls are `neoCLR.Runtime.WriteLine(string) -> Void`,
`neoCLR.Runtime.Int32ToString(int32) -> String`, and
`neoCLR.Runtime.ParseInt32(string) -> System.Value`, plus StringConcat,
StringByteCount, StringSliceUtf8, ErrorFromMessage, ErrorMessage, ReadAllText, and ConsoleReadByte. Parsing and formatting
are temporary host implementations until character/string operations can support
their platform versions. Console output is captured by default in Rust embedding, or delivered immediately to
an explicitly supplied host console. The CLI uses live output.

The public `System.*` methods are not hard-coded interpreter dispatch cases.
Their declaring types are explicit, including runtime-known `System.Int32`.
They consume guest frames and instruction budget, and their metadata/IL bodies are
serialized just like application functions. Tests replace the compiled Divide body
and confirm execution follows the replacement, rather than a hidden intrinsic.

The [Result API review](runtime-error-contracts.md) inventories current failure outcomes
and proposed operation-specific error types. All six reviewed Result APIs now use specific error types. See [arithmetic contracts](arithmetic-errors.md).

## Build and use

`runtime/System.neoil` is an ordered `.include` manifest. Class and feature sources
live under namespace folders: for example, `System/Console.neoil`,
`System/IO/File.neoil`, and `System/Runtime/CompilerServices/UnionAttribute.neoil`.
Generic carriers and their companions share `System/Option.neoil` and
`System/Result.neoil`. InternalCall declarations live under `neoCLR/Runtime/`.

These are source fragments, not separate modules. Manifest order determines the
combined declaration order and metadata rows. The initial split preserves the previous
metadata and IL exactly. Add new files explicitly to the manifest; directory enumeration
does not determine build order. Cargo's build script expands the manifest and embeds
the result, rebuilding when runtime sources change. It requires no installed neoCLR
executable and reads no runtime files when an application executes.

The CLI expands `.include "path"` when reading `.neoil` files, including `--system`
inputs. Paths resolve relative to the including file. Nested includes are supported;
cycles and nesting beyond 64 files are rejected. Includes accept a trailing semicolon
comment, but no wildcard expansion or string escape processing. Include loader errors
identify the source path; subsequent assembler diagnostics use expanded line numbers.

Embedding code can call `neoclr::source::read_source(path)` before `assemble`.
The string-only assembler remains independent of filesystem access. The fully expanded
bundled source is available through `neoclr::library::system_source()`.

```sh
cargo run -- assemble runtime/System.neoil System.neo.json
cargo run -- check System.neo.json
cargo run -- run examples/hello.neoil System.neo.json
```

The library has `.module System` and no entry point. Application entry points remain
parameterless functions. The CLI refuses to execute a library directly. By default,
the runtime embeds the System source, assembles it once into a cached module, and
links its types/functions into the application's execution image. Supplying a
compiled artifact uses that library instead. The embedding API exposes
`run_with_library` for the same purpose.

The System module name is reserved for this library role. The linker verifies its
metadata and bodies, rejects signature collisions, and resolves exact overloads.
Native declarations carry `.methodimpl InternalCall`, mapping to CLR-compatible
implementation flags. The runtime validates name, ordered parameter types, and return
type against its binding registry. Unknown bindings, unexpected flags, and native
declarations with IL bodies/locals are rejected. A matching name without the flag
executes as IL; names do not implicitly activate native code. Prepared programs now
support explicit module sets, module/revision identities,
and direct reference lists; see [module sets](module-sets.md). The CLI and embedding APIs
can select a supplied System artifact during initial assembly and loading. Native import
metadata and typed binding checks remain separate from that module selection.

The `.neo.json` artifact contains prototype metadata and IL instructions, not final
CLI binary tables or byte streams. Platform-written library execution is implemented;
CLI-format binary emission remains a separate goal.

## MethodImpl metadata

```text
.function neoCLR.Runtime.WriteLine(string) -> Void
    .methodimpl InternalCall
.end
```

This corresponds to the familiar .NET
[`MethodImpl(MethodImplOptions.InternalCall)`](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.methodimploptions?view=net-10.0)
concept. The assembly directive emits an implementation flag on the function, not
an ordinary custom-attribute blob. The prototype `impl_flags` field uses the CLR
[`MethodImplAttributes.InternalCall`](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.methodimplattributes?view=net-10.0)
value `0x1000`; ordinary IL uses zero. A future higher-level compiler can lower the
familiar attribute form to the same metadata. Marker custom attributes are implemented separately; see [custom attributes](custom-attributes.md).

The runtime owns a typed registry in `src/native.rs`. `call` first resolves the
metadata declaration; an InternalCall declaration binds to the corresponding Rust
implementation. Parameter and return contracts are checked. Dynamic library imports
use a separate [P/Invoke path](native-interop.md). `InternalCall` is a CLR implementation
mechanism; it is distinct from the `Runtime` code-type flag and does not claim that
all existing CLI tools support it unchanged.

Runtime helper names are implementation details, not the intended consumer API.
Visibility/access enforcement is still pending, so the prototype does not yet
prevent applications from naming those declared helpers directly.

## Reflection introspection

System.Type exposes GetFields, GetMethods and GetProperties, returning ordinary typed
value arrays of independent System.Reflection descriptors. Parameter and accessor
metadata preserve managed-reference signatures and receiver/output contracts. See the
[reflection API guide](reflection.md) for filtering, type-shape queries, storage and
identity contracts, limitations, and runnable Neo/IL examples.

[Ordinal text operations](ordinal-text.md) add String.CompareOrdinal and readonly
ContainsOrdinal, StartsWithOrdinal and EndsWithOrdinal. See the text contract for
UTF-16 ordering over UTF-8 storage, empty patterns and current copying costs.

[Character classification](character-classification.md) adds familiar System.Char predicates and
Neo character literals, including Unicode IsDigit and explicit IsAsciiDigit.

[Fundamental Math operations](math.md) add integer helpers, typed Clamp and core
Double functions. Neo supports Double literals and same-type arithmetic/comparison;
`examples/source/math.neo` demonstrates the APIs.

[Date and Time core values](date-time.md) provide validated factories, readonly
components and value comparison. Run `examples/source/date-time.neo` for leap-date,
time-of-day and typed-error handling; parsing and formatting remain planned.

The [local system clock](local-clock.md) provides Date, Time and UTC offset in one reading.
Run its Neo sample with `cargo run --locked -- run examples/source/local-clock.neo`.

The [Environment API](environment.md) exposes guest arguments, current directory and
optional process variables; see `examples/source/environment.neo`.

[Lexical paths](path.md) provide System.IO.Path.Combine and GetFileName.

The [file report example](file-output.md) combines guest arguments, paths and bounded
UTF-8 input/output with typed Results.

## Raven as the library source language (2026-09-13)

The author now directs ordinary runtime class-library development toward Raven,
while retaining neoIL as an important platform facility. Begin the migration while
the library is still relatively small, rather than increasing the amount of later
translation. Take a source milestone checkpoint before starting the transition.
This is a change of implementation language, not a decision to alter public API
contracts, value/reference categories, the instruction set or the runtime's role.

Raven should author ordinary algorithms, collections, query operators and error-flow
helpers. Keep neoIL where low-level control, bootstrap definitions or direct runtime
conformance tests justify it. Host code still supplies unavoidable platform services.
Generated neoIL can remain an executable/debuggable intermediate; the source language
and executable representation are separate layers.

This follows the managed-library pattern visible in .NET's
[C# LINQ implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Linq/src/System/Linq/Last.cs),
using a high-level language for ordinary algorithms without moving their policy into
the runtime engine. Raven adds a useful integration test of neoCLR's compiler-facing
surface. Benefits are readability, maintainability and earlier discovery of target
gaps; costs include compiler/bootstrap dependencies and the risk of introducing
translation regressions. Retaining hand-written neoIL avoids that bootstrap cost
but becomes more expensive to maintain as algorithms and APIs grow.

### First migration groundwork

The current bridge is a bounded application importer, not a general library compiler.
[ApplicationTypes.cs](experiments/raven-target/ApplicationTypes.cs) rejects generic
application type/method bodies and assigns application-specific type identities;
[QueryBindings.cs](experiments/raven-target/QueryBindings.cs) admits calls to known
generic library methods, whose executable bodies still come from neoIL. Therefore
successful Raven calls to a generic API are not evidence that its implementation
can already be authored and imported from Raven.

Start by establishing the repeatable build path for a small scalar library helper,
then a generic collection or query method. Preserve stable System type/member
identities and test imported bodies with existing direct IL and Raven scenarios.
The necessary sequence is: bootstrap reference metadata, compile Raven library
sources against that surface, import the implementation, then compile and run a
consumer against the matching library contract. Determine how to derive or verify
the reference surface from the authoritative sources so declarations do not drift
from implementations. Do not create a circular dependency on a library that cannot
yet be built. Keep Raven changes on its experimental feature branch.

For each port, compare results, fault/cleanup boundaries, callback order, allocation
behavior and retained references under GC before replacing its hand-written body.
Do not change the API merely to make the port easier. Keep the remaining neoIL
bodies executable while migrating incrementally; remove duplicate authoritative
implementations once a port is verified. Finish with a clean-checkout bootstrap and
an extracted-bundle consumer test. Broad generic importing and the reference-assembly
build strategy are groundwork to implement, not completed capabilities. No library
method has been ported to Raven in the predicate-overload checkpoint itself.

### Migration paused for target evaluation (2026-09-14)

The author subsequently directed a pause before authoring System classes. Evaluate
Raven compiler fixes and coherent .NET/neoCLR targeting first. The scalar pilot remains
locally preserved, not adopted; see the [assessment and resumption criteria](raven-target-evaluation.md).
