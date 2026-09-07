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
field reference; use `*reference` to read or write a scalar referent explicitly.

`let` prevents rebinding and writable access to a directly held value. `var` permits
assignment and taking its writable address. A `let` binding holding T& still permits
mutation of the referenced object; it only prevents rebinding that reference.
Parameters are immutable bindings, but T& parameters can mutate their referents.

A `record` declaration defines a data shape with ordered fields and positional
construction arguments. It does not introduce a permanent value/reference type flag,
inheritance, synthesized equality methods, or deep-copy behavior.

The [Neo grammar](neo-grammar.md) gives the implemented EBNF and lexical rules.

## Supported first slice

- One parameterless `func Main()`, plus typed free functions and forward calls.
- Records, positional construction, nested field access and mutation.
- Initialized `let`/`var` bindings with optional type annotations.
- `int`/`Int32`, `string`/`String`, `bool`/`Boolean`, `()`/`unit`/`Void`, named records and T&.
- Int32 arithmetic with `+`, `-`, `*`, `/`, parentheses and unary minus; runtime arithmetic semantics apply.
- Integer, Boolean and double-quoted string literals; strings use JSON-style escapes.
- Explicit references, dereferencing, heap construction and typed returns.
- `Console.WriteLine` for int and string. `import System.Console.*` enables unqualified `WriteLine`.
- Newline or semicolon statement separators and `//` comments.

Non-Void functions need an explicit return. Void functions may end without one.
Field/parameter lists can span lines. There are no implicit conversions between the
supported source types. The parser bounds source size and expression nesting.

## Lifetime checks and current limits

Returning a direct address into the current function is rejected by verification;
runtime provenance checks also prevent indirect escapes. Heap-backed references can
return and keep their entire allocation alive, including through field views.
Reference-valued fields may contain heap-backed references; scoped targets in those
fields fault at runtime. This compiler does not perform complete static lifetime
analysis, so some invalid programs fail only during execution.

The current subset has function scopes only. Nested block scopes, uninitialized
bindings, loops/branches, generics, overload declarations, instance methods,
inheritance, pattern matching, native pointers/interop, pinning and full Raven syntax
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
cargo test --test neo --test cli --test cli_modules --test gc_diagnostics
```

See [managed heap references](heap-references.md), [GC](garbage-collection.md) and
[the lifecycle model](lifecycle.md) for the runtime contracts used by Neo.

A future [bootstrapping exercise](neo-bootstrapping.md) may port ordinary library
functions to Neo and later attempt a Neo-written compiler. Neither is part of this
initial implementation.
