# Logical module consumer

Development sample, 2026-10-09. `Numbers.rvn` declares a function and an empty module;
`Main.rvn` imports the function with the existing wildcard import syntax. Compile
both into an assembly named `DifferentPackage`, independently of their module paths.

Use a fresh project importing the current bundle's `NeoCLR.ClassLibrary.props`, with
`TargetFramework` net10.0, `OutputType` Exe and those two source files. Run:

```sh
dotnet /path/to/rvnc.dll neoclr --project /path/to/Demo.rvnproj --no-build-references
```

The compiler and metadata library must contain the matching module changes. For the
separate-assembly check, compile Numbers.rvn as a Library named NumbersPackage and
Main.rvn as ConsumerPackage with a Reference/HintPath to the resulting library.

Run with the matching bundle's System.runtime.neox, all four library modules and
System.Runtime.dll as the explicit object root. `neoclr run ... --show-result` prints
`Int32(42)` and returns the guest exit code 42. Use the AOT tool's `--closed-world`,
`@entry` and the same dependency context to emit an object. The checked `host.c` uses
the existing v2 ABI (including its unused argument slot for parameterless entries).
The native executable prints 42 and exits zero when the result matches.

No new module-private access or loading policy is exercised. The existing module
command-line options refer to physical execution images, not declaration modules.
The module table is read-only at runtime and does not change code generation.
See [validation](validation.json) and the [format contract](../../declaration-modules.md).
