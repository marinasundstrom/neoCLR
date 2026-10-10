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


## Host introspection consolidation (2026-10-10)

The matching host ModuleInfo now represents the logical namespace of declarations.
Raven 1a0c4e627 traverses all modules when importing free functions and the aggregate
assembly type view when importing types. The expanded `--check-module-consumer`
metadata check validates the compiled sample through those host views, including
empty modules and canonical member ownership. No parent modules are synthesized.

[Host validation](host-validation.json) records the 170 metadata checks, focused
ownership/packaging checks, real compiled consumer, five discovery rejection cases
and all three Raven suites passing in interpreter/macOS ARM64 AOT. Generated test IDs
now use logical module names; invocation still uses typed adapters. The report records
the matching compiler hashes and reused development-library provenance. This does not
qualify guest ModuleInfo migration or guest AOT module scanning. The new Windows
compiler pin is coordinated with this host API migration; its result is pending.
