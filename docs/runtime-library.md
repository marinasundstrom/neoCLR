# Runtime library boundary

The basic runtime library is written **for neoCLR**, assembled into its metadata
and instruction representation, and executed by neoCLR. Host Rust implements the
interpreter and unavoidable bootstrap services; it should not become the BCL's
implementation language by accident.

[System.neoil](../runtime/System.neoil) currently provides six platform-written methods and three native declarations:

- `System.Console.WriteLine(string)` calls the host output primitive.
- `System.Console.WriteLine(int32)` calls the Int32 receiver's `ToString()` and then
  the string overload.
- `System.Int32.ToString()` is an instance method that calls the formatting helper.
- `System.Int32.Parse(string)` wraps a temporary host parsing primitive.
- `System.Int32.Divide(int32, int32)` checks zero and overflow, then executes `div`
  or returns an Error. Its control flow and Result construction are platform IL.
- `System.Math.Abs(int32)` computes absolute value and represents overflow as an
  Error. Its logic is entirely platform IL.

The remaining host calls are `neoCLR.Runtime.WriteLine(string) -> Void`,
`neoCLR.Runtime.Int32ToString(int32) -> String`, and
`neoCLR.Runtime.ParseInt32(string) -> Result<Int32,Error>`. Parsing and formatting
are temporary host implementations until character/string operations can support
their platform versions. Console output is buffered until successful execution.

The public `System.*` methods are not hard-coded interpreter dispatch cases.
Their declaring types are explicit, including runtime-known `System.Int32`.
They consume guest frames and instruction budget, and their metadata/IL bodies are
serialized just like application functions. Tests replace the compiled Divide body
and confirm execution follows the replacement, rather than a hidden intrinsic.

## Build and use

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
executes as IL; names do not implicitly activate native code. This is intentionally one-library
bootstrap linking: general assembly identities, version binding, import tables,
visibility, and isolated dependency graphs are not implemented. Application
assembly/loading currently validates against the bundled System API, so an alternate
compiled System artifact must retain the application's required bundled signatures;
adding new external APIs needs the forthcoming reference-resolution model.

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
familiar attribute form to the same metadata. General custom attributes are not
implemented yet.

The runtime owns a typed registry in `src/native.rs`. `call` first resolves the
metadata declaration; an InternalCall declaration binds to the corresponding Rust
implementation. Parameter and return contracts are checked. Dynamic library imports
use a separate [P/Invoke path](native-interop.md). `InternalCall` is a CLR implementation
mechanism; it is distinct from the `Runtime` code-type flag and does not claim that
all existing CLI tools support it unchanged.

Runtime helper names are implementation details, not the intended consumer API.
Visibility/access enforcement is still pending, so the prototype does not yet
prevent applications from naming those declared helpers directly.
