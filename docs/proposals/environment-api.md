# NeoCLR Proposal: Shell Environment and Contextual Capability Projection

## Summary

NeoCLR should introduce `ShellEnvironment` as the standard contextual entry point for capabilities and state supplied by an application's surrounding shell.

A shell may be a **terminal shell, desktop shell, or another host shell**. Applications should not need to belong to fundamentally different runtime models merely because they interact through a terminal or graphical desktop.

Rather than defining every possible shell capability directly on `ShellEnvironment`, NeoCLR libraries may use **extension members** to project capabilities onto it as if they were intrinsic members of the type.

For example:

```raven
ShellEnvironment.Storage
ShellEnvironment.CurrentDirectory
ShellEnvironment.Terminal
ShellEnvironment.Desktop
```

Likewise, application-scoped capabilities may be projected onto an `Application` context:

```raven
Application.Storage
```

This provides a discoverable platform API without introducing global `Provider.Instance` patterns or requiring the core shell abstraction to depend on every optional subsystem.

---

## 1. Motivation

Traditional platform APIs often assume a singular ambient “environment.”

For example, APIs may expose concepts such as:

```text
Environment.CurrentDirectory
Environment.GetEnvironmentVariable(...)
Environment.SpecialFolder
```

This conflates several different contexts:

- runtime state;
- process state;
- operating-system information;
- shell state;
- application state;
- platform capabilities.

NeoCLR should model these concepts according to their actual ownership.

In particular, many things conventionally described as belonging to “the environment” are more precisely properties or capabilities of the **shell environment surrounding the application**.

NeoCLR therefore introduces:

```raven
ShellEnvironment
```

rather than treating a generic `Environment` as a universal platform context.

---

## 2. What is a shell?

NeoCLR uses *shell* in a broader sense than command-line shell.

A shell is the user-facing host environment through which an application participates in the surrounding system.

Two important forms are:

```text
Shell
├── Terminal shell
└── Desktop shell
```

A terminal application interacts primarily with terminal facilities.

A desktop application interacts with graphical shell facilities such as windows, displays, clipboard services, notifications, file associations, and application launching.

These are not necessarily mutually exclusive environments.

A desktop application may still possess:

- environment variables;
- a current working directory;
- command-line arguments;
- storage access;
- process state;
- terminal facilities when attached to a terminal.

A desktop application is therefore not fundamentally a different kind of executable. It is an application using additional shell services to present itself graphically.

---

# 3. `ShellEnvironment`

`ShellEnvironment` represents the shell context surrounding the currently executing application.

Conceptually:

```text
Application
     │
     ▼
ShellEnvironment
     │
     ├── Storage
     ├── CurrentDirectory
     ├── Terminal
     ├── Desktop
     └── other shell capabilities
     │
     ▼
Host / operating system
```

`ShellEnvironment` does **not** need to define all of these capabilities itself.

It serves as a stable **extension anchor**.

---

## 4. Contextual capability projection

NeoCLR's extension-member mechanism allows independently packaged APIs to attach properties and other members to existing types.

This should be used deliberately as a platform-composition mechanism.

For example, the Storage package could declare:

```raven
extension ShellEnvironment
{
    static Storage: StorageProvider { get; }

    static CurrentDirectory: Path
    {
        get;
        set;
    }
}
```

Consumer code simply sees:

```raven
ShellEnvironment.Storage
ShellEnvironment.CurrentDirectory
```

The API therefore behaves as though these members were originally defined by `ShellEnvironment`.

However, the package containing `ShellEnvironment` has **no dependency on Storage**.

This gives NeoCLR both modularity and a cohesive discoverable API.

---

# 5. Storage example

The Storage API defines the abstraction:

```raven
interface StorageProvider
{
    // Storage operations
}
```

It does not expose a singleton:

```raven
// Avoid
StorageProvider.Instance
```

Instead, a shell capable of general storage access exposes its provider through:

```raven
ShellEnvironment.Storage : StorageProvider
```

Usage becomes:

```raven
let file = await ShellEnvironment.Storage.GetFile(path)?;
```

This has stronger semantics than:

```raven
StorageProvider.Instance
```

The latter means:

> There happens to be a global instance of this provider.

The former means:

> This is the storage provider exposed by the current shell environment.

---

# 6. Current working directory

The current working directory demonstrates why `ShellEnvironment` is more than a provider registry.

```raven
ShellEnvironment.CurrentDirectory
```

is actual contextual shell state.

The Storage package may contribute:

```raven
extension ShellEnvironment
{
    static CurrentDirectory: Path
    {
        get;
        set;
    }
}
```

For example:

```raven
ShellEnvironment.CurrentDirectory = projectDirectory;
```

The current directory should not belong to `StorageProvider` itself.

A provider represents storage operations. The current directory represents the shell context from which relative paths may be interpreted.

This establishes an important distinction:

> **Providers own domain operations. Context objects own contextual state and expose the providers available within that context.**

---

# 7. Terminal capabilities

Terminal support can similarly extend the shell environment:

```raven
extension ShellEnvironment
{
    static Terminal: Terminal { get; }
}
```

This could expose terminal-specific facilities such as input, output and terminal capabilities.

For example:

```raven
ShellEnvironment.Terminal.Output.WriteLine("Hello");
```

The precise Terminal API should be designed separately.

The important architectural point is that terminal functionality does not define `ShellEnvironment`; it is one capability that may be projected onto it.

---

# 8. Desktop capabilities

A desktop capability pack could similarly expose:

```raven
extension ShellEnvironment
{
    static Desktop: DesktopShell { get; }
}
```

`DesktopShell` could eventually provide access to concepts such as:

```text
DesktopShell
├── Displays
├── Clipboard
├── Notifications
├── Windows
├── Associations
└── Launching
```

The exact division should be determined by later desktop API proposals.

The significant point is that desktop functionality builds **on the same shell model** as terminal applications.

NeoCLR therefore does not need a fundamental runtime hierarchy such as:

```text
Application
├── ConsoleApplication
└── DesktopApplication
```

Instead, these are compositions of available capabilities.

---

# 9. Application context

Not every capability belongs to the shell.

NeoCLR application models may additionally provide an `Application` context representing resources belonging specifically to the application.

For example:

```raven
Application.Storage
```

could also have the type:

```raven
StorageProvider
```

giving us:

```raven
ShellEnvironment.Storage : StorageProvider
Application.Storage      : StorageProvider
```

but with different semantics.

### `ShellEnvironment.Storage`

Storage exposed through the surrounding shell environment.

It may represent broad access to the host's storage namespace according to the application's permissions.

### `Application.Storage`

Storage owned or allocated specifically to the application.

It might represent:

- application data;
- sandboxed persistent storage;
- application-local files;
- host-managed application storage.

An implementation of `Application.Storage` may ultimately use `ShellEnvironment.Storage`, but consumers do not need to know this.

---

# 10. Not all hosts require `ShellEnvironment`

`ShellEnvironment` should **not be a fundamental CLR assumption**.

Some NeoCLR execution environments may not have a meaningful shell.

For example, future hosts could include:

- embedded environments;
- specialized runtimes;
- sandboxed execution;
- hosted components;
- application models that expose only explicitly granted capabilities.

Such environments should not be forced to emulate a shell merely to satisfy the base runtime.

Therefore:

> `ShellEnvironment` belongs to shell-capable application/platform layers, not necessarily the fundamental NeoCLR runtime.

Similarly, `Application` belongs to application models that define such a context.

---

# 11. Dependency injection

`ShellEnvironment` is **not intended to be an injectable abstraction**.

There should be no requirement for something like:

```raven
interface ShellEnvironment
```

merely to make the platform context mockable.

Instead, the individual capabilities exposed through it may themselves be abstractions:

```raven
ShellEnvironment.Storage : StorageProvider
```

Code requiring substitutable storage should depend directly on that capability:

```raven
class ConfigurationLoader(StorageProvider storage)
{
    ...
}
```

rather than:

```raven
class ConfigurationLoader(ShellEnvironment environment)
{
    ...
}
```

This preserves explicit dependency injection where it matters without forcing the entire platform context behind an abstraction.

Applications that need their own injectable concept called `Environment` or `ShellEnvironment` remain free to define one.

---

# 12. Extension members as platform architecture

This proposal establishes a broader NeoCLR principle.

Extension members are not merely syntactic convenience for attaching helper methods to foreign types.

They can serve as a **platform composition mechanism**.

A package may expose its functionality through the context where that functionality naturally belongs:

```raven
// System.Storage

extension ShellEnvironment
{
    static Storage: StorageProvider { get; }
    static CurrentDirectory: Path { get; set; }
}
```

```raven
// System.Terminal

extension ShellEnvironment
{
    static Terminal: Terminal { get; }
}
```

```raven
// System.Desktop

extension ShellEnvironment
{
    static Desktop: DesktopShell { get; }
}
```

```raven
// Application storage capability

extension Application
{
    static Storage: StorageProvider { get; }
}
```

To application developers, this appears as one coherent API:

```raven
ShellEnvironment.Storage
ShellEnvironment.CurrentDirectory
ShellEnvironment.Terminal
ShellEnvironment.Desktop

Application.Storage
```

Yet these members may originate from completely separate packages.

---

# 13. Package dependency model

This avoids forcing the base shell package to reference every subsystem:

```text
System.Shell
    └── ShellEnvironment

System.Storage
    ├── StorageProvider
    ├── Path
    └── extensions for ShellEnvironment

System.Terminal
    ├── Terminal
    └── extensions for ShellEnvironment

System.Desktop
    ├── DesktopShell
    └── extensions for ShellEnvironment
```

The dependency direction is therefore:

```text
System.Storage ────► System.Shell
System.Terminal ───► System.Shell
System.Desktop ────► System.Shell
```

rather than:

```text
                 ┌── Storage
                 ├── Terminal
System.Shell ────┼── Desktop
                 ├── Clipboard
                 ├── ...
                 └── everything else forever
```

This is particularly important for NeoCLR because it allows the platform API to grow without recreating the historical problem of a monolithic base framework.

---

# 14. Design principles

I think the proposal can boil down to a small set of rules:

1. **Use precise contexts.** `ShellEnvironment` represents the surrounding shell rather than an ambiguous universal “environment.”
2. **Treat shell models as capability compositions.** Terminal and desktop shells are not fundamental CLR application categories.
3. **Use extension members for contextual projection.** Optional packages can add capabilities to `ShellEnvironment`, `Application`, and future context objects without modifying their defining assemblies.
4. **Expose providers through their natural context.** Prefer `ShellEnvironment.Storage` over `StorageProvider.Instance`.
5. **Keep abstractions at the capability level.** Inject `StorageProvider` when storage needs substitution; don't make `ShellEnvironment` injectable merely for testing.
6. **Separate shell and application scope.** `ShellEnvironment.Storage` and `Application.Storage` can expose the same abstraction with different ownership and semantics.
7. **Do not require a shell universally.** `ShellEnvironment` is part of appropriate application/platform models, not an assumption built into NeoCLR itself.

The larger architectural idea is especially strong: **extension members allow NeoCLR's platform to be modular in implementation while still feeling integrated in use.** That's probably worth elevating beyond this proposal into the eventual NeoCLR architecture document as a general platform-design mechanism.