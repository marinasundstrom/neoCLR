# Explicit module sets

The bootstrap loader now accepts an application, System, and additional library
modules through `LoadedProgram::with_modules(application, system, dependencies)`.
The caller supplies the complete set explicitly. No filesystem probing, dependency
download, implicit artifact replacement, or assembly resolver callback is involved.

```rust
let modules = neoclr::assembler::assemble_modules(&[app_source, helpers_source])?;
let program = neoclr::LoadedProgram::with_modules(
    &modules[0], neoclr::library::system()?, &modules[1..],
)?;
program.verify()?;
let execution = program.run(neoclr::Limits::default())?;
```

`assemble_modules` parses all sources before resolving field-name aliases and other
references. The first source is the root; the rest are dependencies. Forward references,
generic signatures, cross-module field aliases, and mutually referencing dependencies
are supported. The result retains separate source Module artifacts in input order.

`load_modules` similarly reads a group of JSON artifacts and validates the complete
set with bundled System. It derives omitted legacy definition rows on the resolution
copy. Individual `assemble` and `load` retain their application/System behavior; an
artifact requiring additional modules must be checked with its dependencies present.
The group helpers use bundled System; an explicitly supplied System artifact is
supported by `LoadedProgram::with_modules` when preparing Module values.

## Validation and identities

Every supplied module must use the supported format and have a unique nonempty name.
System remains reserved for the self-contained runtime library. Additional modules
must not declare an entry point; the root can omit one when preparing libraries for
analysis. Unused supplied modules are validated too. A method declaration and its
declaring type must originate in the same module.

Type and function identities are normalized in each source module before combination.
The internal table order is root, System, then dependencies in supplied order. Linked
indices can change with dependency order, while module-local definition rows remain
unchanged. Explicit `@ Models:0` calls keep selecting that definition after reordering.
Generic call binding still happens before specialization, and source artifacts remain
unmodified by preparation.

## Current namespace boundary

All supplied symbols share one namespace. Types must still have globally unique names,
and existing duplicate function-signature rules apply across the set. Duplicate symbols
fail rather than selecting whichever dependency appeared first. Source namespace
prefixes are ordinary parts of names; they do not imply module ownership or imports.

Every supplied module can refer to any other supplied module, apart from System, which
is validated independently. There are no declared import edges, visibility rules,
module-qualified type operands, revision constraints, or side-by-side same-name modules
yet. This slice broadens explicit loading; it does not complete scoped resolution.
Build-local row identities do not establish compatibility across artifact replacements.

The example under `examples/modules/` separates the entry point, an operation, and a
generic Box type into three neoIL sources. `cargo run --example modules` assembles the
set, verifies it, and prints `42`. The existing CLI still operates on one source/artifact
at a time; module-set loading is currently exposed through the Rust API and example.
