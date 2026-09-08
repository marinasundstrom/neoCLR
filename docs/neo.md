# Neo concept language

Neo is a small high-level concept language for exercising NeoCLR end to end. Its
syntax is inspired by Raven: `func`, `record`, `let`/`var`, name-before-type
annotations, braces, and `()` for an empty result. Neo compiles to ordinary neoIL and
format 5 modules, then uses the existing verifier, interpreter and garbage collector.
It is a separate experimental subset, not a Raven compiler or compatibility mode.

## Purpose and maintenance

Keep the Neo compiler updated as neoCLR develops. It is a companion for testing and
explaining the platform: small readable programs should demonstrate runtime behavior
and exercise the source-to-IL-to-runtime path. When a runtime change affects an
exposed feature, update its lowering, examples, regression tests and grammar together.
Keep direct runtime/IL tests as independent coverage, including features Neo does not
yet expose.

In its current form, Neo is deliberately a small concept compiler, not a complex,
full-fledged compiler. Add only the language support needed to demonstrate and test
concrete platform capabilities. Complete language coverage, sophisticated compiler
optimizations and production tooling are not current goals. The
[upcoming slices](neo-roadmap.md) describe the next bounded steps; future
[bootstrapping](neo-bootstrapping.md) does not expand today's scope.

## Runtime contract and source projection

Read the [managed-reference semantics guide](managed-reference-semantics.md) for the
shared runtime/Neo model. T& chooses reference access; the target's storage determines
its lifetime. Frame-backed references cannot outlive their owning frame. Heap-backed
references, including interior field references, keep the owning heap allocation
reachable independently of the allocating method. Copying a reference does not promote
a local or extend its frame's lifetime.

Neo hides managed dereferencing through ordinary reads and assignment. The runtime
still executes explicit checked IL loads/stores. Lexical block visibility is implemented,
but separate runtime block lifetimes and automatic guest destruction are not. Native
pointers retain separate low-level semantics and are not yet exposed by Neo.

## Run the example

From the repository root, with Rust/Cargo installed (Rust 1.85 or newer):

```sh
cargo run -- run examples/source/counter.neo
```

The example prints the untouched value copy, the mutated local, and the heap value:

```text
1
2
42
=> Int32(42)
```

It constructs a local counter, copies it, passes the original by reference, creates
a heap counter in another function, and mutates it through a returned field reference.
Read [counter.neo](../examples/source/counter.neo) for the complete source.

To include GC statistics and recent collection events:

```sh
cargo run -- run examples/source/counter.neo --gc-stats --gc-events
```

GC diagnostics go to stderr; the program's output stays on stdout. This example
allocates one managed heap object and reclaims it after returning the integer result.
The current diagnostics are completion reports, not a live stream.

## Check and compile

```sh
cargo run -- check examples/source/counter.neo
cargo run -- verify examples/source/counter.neo
cargo run -- assemble examples/source/counter.neo counter.neo.json
cargo run -- run counter.neo.json
```

`assemble` creates a normal NeoCLR artifact and refuses to overwrite an existing
file. Choose another output path when compiling again. High-level compilation always
runs typed verification; execution still checks lifetimes and resource limits.
The first front end supports one `.neo` source file with bundled System. Module-set
and custom System flags remain available for neoIL/artifact inputs, but are rejected
for Neo source.

After building, the same commands work with the executable directly:

```sh
cargo build --release
./target/release/neoclr run examples/source/counter.neo --gc-stats
```

## A small source example

```text
record Counter(Age: int)

func MakeCounter() -> Counter& {
    return new Counter(40)
}

func Increment(counter: Counter&) -> () {
    counter.Age = counter.Age + 1
}

func Main() -> int {
    var local = Counter(1)
    let copy = local
    Increment(&local)

    let shared = MakeCounter()
    shared.Age = shared.Age + 2
    Console.WriteLine(shared.Age)
    return shared.Age
}
```

`Counter(1)` constructs an ordinary value. `new Counter(40)` explicitly allocates
managed heap storage and returns `Counter&`; it introduces no boxing step.
`&local` forms a reference to an existing mutable location. Record field access
through a reference automatically addresses its referent. `&shared.Age` returns a
field reference. Managed references are read and written automatically by the compiler;
`age = age + 2` updates the referenced integer. No explicit dereference is required.

`let` prevents rebinding and writable access to a directly held value. `var` permits
assignment and taking its writable address. A `let` binding holding T& still permits
mutation of the referenced object; it only prevents rebinding that reference.
Parameters are immutable bindings, but T& parameters can mutate their referents.

A `record` declaration defines a data shape with ordered fields and positional
construction arguments. It does not introduce a permanent value/reference type flag,
inheritance, synthesized equality methods, or deep-copy behavior.

The [Neo grammar](neo-grammar.md) gives the implemented EBNF and lexical rules.

## Supported subset

- One parameterless `func Main()`, plus typed free functions and forward calls.
- Records, positional construction, nested field access and mutation.
- Source interfaces, record instance methods with managed `this`, explicit `as Contract&` projections and contextual managed reference conversions.
- Initialized `let`/`var` bindings with optional type annotations.
- `int`/`Int32`, `string`/`String`, `bool`/`Boolean`, `()`/`unit`/`Void`, named records and T&.
- Int32 arithmetic with `+`, `-`, `*`, `/`, parentheses and unary minus; runtime arithmetic semantics apply.
- Integer, Boolean and double-quoted string literals; strings use JSON-style escapes.
- Explicit reference formation, automatic managed-reference access, heap construction and typed returns.
- Exhaustive union match expressions/statements, case payload bindings and wildcards.
- Closed generic type annotations and public static/ordinary instance bundled System calls.
- Explicit `int(byteValue)` conversion using checked Int32 conversion.
- `typeof(T)` returning System.Type, and read-only public System property access.
- `Console.WriteLine` for int and string. `import System.Console.*` enables unqualified `WriteLine`.
- `if`/`else`, `while`, integer-range `for`, `loop`, `break` and `continue`.
- Int32 comparisons, Int32/Boolean equality, and short-circuit `&&`/`||` with `!`.
- Newline or semicolon statement separators and `//` comments.

Non-Void functions need an explicit return on every fallthrough path. Both `if` arms
may return; loops conservatively require a following return even when visibly infinite. Void functions may end without one.
Field/parameter lists can span lines. There are no implicit conversions between the
supported numeric types. A T& is automatically read when a T value is needed; this
is managed-reference access, not a numeric conversion. The parser bounds source size
and expression nesting.

## Lifetime checks and current limits

Returning a direct address into the current function is rejected by verification;
runtime provenance checks also prevent indirect escapes. Heap-backed references can
return and keep their entire allocation alive, including through field and interface views.
Reference-valued fields may contain heap-backed references; scoped targets in those
fields fault at runtime. This compiler does not perform complete static lifetime
analysis, so some invalid programs fail only during execution.

Uninitialized bindings, generic declarations, overload declarations,
inheritance, general patterns, native pointers/interop, pinning and full Raven syntax
are not implemented. It does not expose the entire standard library yet. These are
candidate future slices, chosen around end-to-end scenarios rather than added as a
complete language up front.

## Embedding and inspecting generated IL

```rust
let source = include_str!("../examples/source/counter.neo");
let il = neoclr::frontend::lower_to_il(source)?;
let module = neoclr::frontend::compile(source)?;
let result = neoclr::LoadedProgram::new(&module)?.run(neoclr::Limits::default())?;
```

`lower_to_il` exposes the lowering for inspection. `compile` additionally assembles
and verifies it. Front-end diagnostics include source line and column; downstream
metadata/verifier diagnostics currently refer to generated IL, not a source map.

Run the front-end and CLI regressions with:

```sh
cargo test --test neo --test neo_control_flow --test neo_match --test neo_calculator --test neo_typeof --test neo_managed_access --test cli --test cli_modules --test gc_diagnostics
```

See [managed heap references](heap-references.md), [GC](garbage-collection.md) and
[the lifecycle model](lifecycle.md) for the runtime contracts used by Neo.

A future [bootstrapping exercise](neo-bootstrapping.md) may port ordinary library
functions to Neo and later attempt a Neo-written compiler. Neither is part of this
initial implementation.

## Structured control flow

Run `cargo run -- run examples/source/control-flow.neo` (prints and returns 21).
`for i in 0..9` includes 9; `0..<10` excludes 10. Bounds are evaluated once from
left to right. Ranges ascend by one, are empty when their bounds exclude every value,
and stop safely at Int32.MaxValue. The iteration binding is immutable. `continue`
in a for loop advances to the next element; `break` exits the innermost loop.

Block names disappear at block exit and cannot shadow active names. Local storage
currently lasts for the call frame and is reused across loop iterations; lexical
scope does not imply block cleanup. To avoid aliases to reused block-local values,
this subset rejects taking their addresses. Put addressed values in an outer
function local or use managed heap storage. Field mutation within a block and
references to heap objects or existing outer values remain supported.

## Union results and matching

Run `cargo run -- run examples/source/match.neo`. It prints `Invalid number`, then
`42`. `Int32.Parse(text)` returns `Result<int, System.Int32ParseError>`; no exception
handling or boxing is needed to select its result:

```text
let value = Int32.Parse("42") match {
    Ok(let number) => number,
    Error(_) => 0
}
```

Use block arms in a standalone match statement for actions, return or loop control.
Both forms require all cases to be covered; `_` can cover the remainder. Payload
bindings are immutable copies scoped to their arm. Match a nested union with another
match, as demonstrated by `Console.ReadByte()` returning
`Result<Option<byte>, System.IO.ConsoleReadError>`. Without a supplied console,
input reports the recoverable Unavailable case. The CLI supplies process stdin/stdout.

Public System methods bind by exact parameter types after automatic reference reads; `Console` and
`Int32` abbreviate their System names. Current union coverage comes from the trusted
bundled library's marker, constructors and typed case accessors. Runtime faults remain
separate from recoverable results, including malformed union representations. See the
[grammar and matching rules](neo-grammar.md) for limits.

## Bounded console calculator

Run `cargo run -- run examples/source/calculator.neo`, then enter a dividend and a
divisor on separate lines. For example, `84` followed by `2` prints `= 42`. Continue
with another pair or signal EOF (Ctrl-D on Unix with an empty input line).
The [calculator guide](neo-calculator.md) documents redirected input, errors and limits.

This slice adds ordinary System instance calls such as `number.ToString()` and
`text.SliceUtf8(start, length)`, with exact argument types and value receivers. A
T& receiver is read through its reference. Byref receiver/out-parameter methods,
user-defined methods and general overload declarations remain outside this subset.
`int(value)` explicitly converts byte or int to Int32 through `conv.ovf.i4`; no
implicit numeric conversion is introduced. Existing `String.Concat` provides bounded
text assembly in the sample. No new runtime instructions or console ABI are needed.

## Type inspection

Run `cargo run -- run examples/source/typeof.neo` to inspect primitive, record,
managed-reference and closed generic signatures:

```text
let descriptor = typeof(Result<int, System.Int32ParseError>)
Console.WriteLine(descriptor.Name)
Console.WriteLine(descriptor.GenericArgumentCount)
Console.WriteLine(descriptor.GetGenericArgument(0).Name)
```

This prints `System.Result`, `2`, and `System.Int32`. `typeof(int)` and
`typeof(System.Int32)` have the same identity; compare descriptors with `.Equals(...)`.
`typeof(Counter)` and `typeof(Counter&)` describe different signatures. No Counter
instance is created, boxed or inspected. `typeof` accepts a type, not a value expression,
and provides no dynamic object GetType behavior or mandatory Object hierarchy.

Type descriptors are ordinary values with owned read-only metadata handles. Returning
a descriptor does not return a reference into the function's frame. The runtime's
[type-inspection contract](type-inspection.md) defines names, identities and limits.
In particular, Name is a canonical definition name with separate generic arguments,
and invalid GetGenericArgument indices fault. Public non-indexed System properties
are read through their getters; setters and property addresses remain unsupported.

## Managed references are transparent; pointers are explicit

Neo must never require a developer to dereference a managed reference manually.
The compiler emits the needed ldobj/stobj operations for value access and assignment:

```text
let age = Age(shared)    // inferred int&, retains the managed reference
age = age + 2           // reads and updates its target
Console.WriteLine(age)  // reads the integer
let copy: int = age     // explicitly requests a value copy
```

Arithmetic, conditions, range bounds, value parameters and value returns read through
T& automatically. Record values copy normally, including when read through a reference.
Inferred bindings preserve the initializer's type; an explicit T annotation requests
a value, while T& requests a reference. Source function signatures determine whether
a reference is forwarded or read. Ordinary values still require `&` to be passed as
references; there is no automatic address-taking.

Assignment to an existing T& binding writes its target, including a `let` binding or
T& parameter. `left = right` between references copies right's value into left's target.
Retarget a mutable reference binding explicitly with `reference = &other`; a `let`
reference cannot be retargeted. `&reference` forwards its existing target rather than
creating T&&. To retarget to a factory result, use `reference = &MakeCounter()` (or
`reference = &new Counter(0)`). Assignment through reference-valued fields or returned
references follows the same target-write rule. Field-slot rebinding is not exposed.

For overloaded System calls, bare reference arguments are read as values; `&argument`
selects reference access explicitly. Match arms follow an enclosing value/reference
expectation; without one, inferred arm types must agree as usual. Lifetime and GC
validation remain the runtime's responsibility, with existing compiler checks retained.

Raw native pointers are different: they retain explicit low-level access and lifetime
rules. Automatic managed-reference reads do not apply to pointers. Neo does not yet
expose pointer types/dereferencing. Unary `*` is no longer a managed-reference operator;
its future role is pointer dereferencing. Binary `*` remains multiplication.

## Arrays

[Managed arrays](managed-arrays.md) support owned `T[]` values and managed `T[]&`
references. `[1, 2, 3]` and `array(length, initialValue)` create owned values;
`new int[3]` creates a default-initialized managed heap array, and
`new int[3] { 1, 2, 3 }` supplies exact elements. The earlier `new array(3, 0)`
form remains accepted. Local annotations such as
`let items: int[3] = [1, 2, 3]` enforce the extent; signatures still use T[]. Use `items[i]`,
`items[i] = value`, `&items[i]` and `items.Length`. Managed element references
read/write automatically and follow the same lifetime rules as record fields.
Run `cargo run --locked -- run examples/source/arrays.neo --gc-stats`.

## Interfaces

[Declare an interface and project a managed reference](neo-interfaces.md) using
ordinary names such as `Counter`, implemented by `SimpleCounter`. `&local as Counter&`
and `shared as Counter&` preserve the original location and lifetime. Concrete managed
references also project implicitly when an interface reference is expected. Instance methods
use an implicit managed `this` receiver, with automatic reads/writes.
Run `cargo run --locked -- run examples/source/interfaces.neo --gc-stats`.

## Debug a running program

Launch the [interactive terminal debugger](debugger.md):

```sh
cargo run --locked -- debug examples/source/debugger.neo
```

Use `source`, `bt`, `stack`, `heap`, `step`, `next` and `continue`. `watch` enables
live inspection; `pause` freezes execution at an instruction boundary. Neo source
locations and local labels survive compilation into JSON artifacts.

## Library receiver contracts

A managed reference is itself a value: passing `T&` copies the reference, while
access through it automatically reads or writes the referenced storage. Copying the
reference does not copy T or let the callee rebind the caller's reference variable.

Instance calls and property getters honor bundled System receiver metadata. Existing
managed-reference values are forwarded; a byref receiver on a value requires an addressable
mutable binding. Library interface references dispatch virtually. No explicit
dereference is needed. See [API design](api-design.md#neo-projection) for the current
value/reference contract inventory and remaining compiler limitations.

## Closed generic static member calls

`System.Collections.ArrayList<Counter&>.Allocate(0)` selects a static member on a
closed generic type. Type arguments may include managed references and nested closed
types. This adds no generic function declarations or generic method inference. Ordinary
comparisons remain expressions. A reference to a bundled type can implicitly convert
to an interface declared by that closed type, without addressing or boxing a value.
See the [complete collection example](../examples/source/collections.neo).

## Output parameters and uninitialized locals

`func Initialize(out value: Foo&)` declares an unconditional output contract;
`Initialize(out destination)` forms or forwards its destination reference.
`var destination: Foo` declares uninitialized storage. The existing verifier checks
caller initialization; runtime guards enforce callee assignment obligations. Calls
to library conditional outputs are supported on direct success branches. See
[output references](neo-outputs.md) for syntax, examples and limitations.

## Runtime type discovery

`reference.GetType()` returns System.Type for an initialized managed reference's
target. Interface views reveal the concrete implementation type; interior references
reveal the referenced field/element type. Existing declared GetType methods take
precedence. This fallback does not address ordinary values or follow raw pointers.
It lowers to ref.type and GetTypeFromHandle; typeof remains declared-type inspection.

## Reference identity

The preliminary `ReferenceEquals(left, right)` intrinsic compares two explicit managed
references by location, including concrete/interface aliases and interior paths. It
does not dereference arguments for value comparison. See [reference identity](reference-identity.md).

## Library indexers

Use `list[index]` to read and `list[index] = value` to write a bundled library's
declared single-parameter `Item` indexer. Neo resolves its public getter or setter
from property metadata and uses virtual dispatch for interface views. Samples use
brackets rather than calling accessor methods directly.

For ArrayList<Foo&>, reading preserves the stored managed reference and assignment
replaces that reference using the setter. `list[i].Age = 42` instead updates the
referenced object. Value-returning indexers produce copies and are not addressable;
`&list[i]` requires a reference-returning getter. Receiver mutability and argument
types follow the accessor contracts. Declaring indexers in Neo and multi-argument
indexers remain outside the current subset.

The [readonly input-parameter slice](readonly-parameters.md) now enforces restricted
managed access at runtime, with Neo declarations and ParameterInfo.IsReadOnly.
Readonly instance receivers are also implemented, with MethodInfo.IsReadOnly.
[Readonly storage and return signatures](readonly-storage.md) now preserve declared
permissions. The verifier remains conservative about aliases and lifetime provenance.

Record and interface methods may declare `readonly func Read() -> int`. Such methods
can be called through readonly inputs and on immutable owned locals. See the
[receiver contract and runnable example](readonly-parameters.md#readonly-instance-receivers).

See [readonly storage and return signatures](readonly-storage.md) for implemented
readonly T& type positions, checked boundaries and migration. Binding immutability
remains a language feature. [Explicit nullability](nullability.md) is a planned
signature characteristic and special state, not implemented syntax or zeroing.

See [managed constructor chaining](constructor-chaining.md) for explicit init
declarations, base initialization and the local/heap constructor example.

The [reflection hierarchy example](reflection-hierarchy.md) demonstrates inherited
properties, readonly MemberInfo views and heterogeneous managed reference collections.

Explicit interface bodies use `func Interface.Member(...) -> ... { ... }` in a
record, optionally preceded by `readonly`. Calls go through a reference typed as
that interface; the declaration adds no ordinary record method. Run
`cargo run --locked -- run examples/source/explicit-interfaces.neo` for two interfaces
with the same signature and different implementations. See
[explicit implementations](explicit-interfaces.md) for mapping, inheritance and
.NET comparison details. Default interface bodies remain planned.
