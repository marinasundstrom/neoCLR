# Who Raven and neoCLR are for

Raven and neoCLR are for people who want a modern programming experience based
on .NET: familiar C# ergonomics, expressive types and APIs, automatic memory
management, and runtime services, together with unions, pattern matching and
explicit optional values and errors.

neoCLR is a new, independent platform. **It is not compatible with existing .NET
libraries.** Familiar concepts and API organization do not mean that a .NET package
can be used unchanged; libraries must target neoCLR and its supported contracts.

Raven is the language and also targets .NET, where it can use .NET libraries.
neoCLR supplies its own runtime, class library and toolchain for the neoCLR target.
Explore the language on .NET or try the combined platform through the
[neoCLR samples](../try/).

## Raven: modern and familiar language concepts

Raven draws influences from C#, Swift, Rust and Kotlin. It combines familiar
classes, value types, interfaces and generics with functions, unions, type inference
and expressive pattern-matching syntax.

Patterns are part of everyday control flow: match alternatives, destructure their
payloads, or bind a value only when a pattern succeeds. If you like Rust's enums
for optional values and errors, Raven's unions may feel familiar. These concepts
work alongside object-oriented programming, so you can choose the representation
that suits your domain.

Type inference keeps local code concise while public signatures describe the
contracts between parts of an application. Explore the
[Raven language introduction](../raven/) for syntax and examples.

## neoCLR: managed runtime services and modern APIs

neoCLR carries forward the managed programming experience: automatic memory
management and garbage collection, asynchronous work, and runtime services such as
[introspection](../features/introspection/) and [reflection](../features/reflection/).
Libraries can inspect types and support tasks such as object mapping, including
on supported native execution paths.

Its APIs are designed afresh, drawing on .NET without carrying its accumulated
compatibility requirements. That freedom lets the platform make deliberate choices
for modern application code:

- **UTF-8 strings** use the encoding common to network and file formats, with
  Unicode-aware text operations and explicit byte access.
- **Optional for absence** expresses an optional value through Some and None instead
  of relying on nullable values as the API convention.
- **Result for recoverable errors** exposes success and failure in the return type,
  with patterns for handling and `?` for propagation.
- **Slice and Memory for efficient buffer access** provide views over existing
  storage. Slices borrow storage for immediate work; Memory retains an owner and
  a window that can be kept across asynchronous operations, avoiding unnecessary
  copies where APIs accept these views.

These choices shape the platform's API design; adoption is incremental as existing
APIs evolve. They do not imply .NET library compatibility. See
[strings](../features/strings/), [Optional and Result](../features/outcomes/),
[error handling](../features/error-handling/), [slices and memory](../features/memory/)
and the [platform comparison](../comparison/) for current behavior.

## Cross-platform applications with AOT-first deployment

neoCLR's direction is cross-platform application development with ahead-of-time
compilation as the intended deployment path. A supported application compiles to
a native executable with the required runtime services, including managed memory.
It does not need a separately installed neoCLR or .NET runtime.

JIT execution and the interpreter also support development and testing. Current
toolchains target macOS ARM64 and Windows x64; broader platform coverage remains
work ahead. Native compilation supports a narrower set of programs than the
interpreter, and reflection can require explicit metadata retention. See
[native compilation](../features/native-compilation/) for supported targets,
build requirements and deployment details.

## Room to explore and contribute

The project may also interest developers building web and backend applications,
library authors exploring explicit API contracts, and people curious about how
languages, metadata and managed runtimes fit together. Working examples cover
[HTTP applications](../features/web/), [SQL](../features/sql/) and
[asynchronous work](../features/tasks/).

neoCLR is experimental. It is a place to try ideas and help shape the platform;
applications that need .NET's ecosystem and production maturity should account
for that difference when evaluating it. Start with a small sample, or contribute
a use case, a bug report, documentation or code through the
[project's contribution paths](../about/#participate).
