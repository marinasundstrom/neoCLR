---
toc: false
---

# About neoCLR

neoCLR is an open-source project developing a managed runtime, a class library and
tools for Raven applications. It was initiated by Marina Sundström and is available
under the MIT license.

<a id="goals"></a>

<a id="background"></a>

## Project scope

The platform brings compilation, execution, application APIs and editor support together.
Raven programs use modules to organize code and access libraries for text, collections,
files, Tasks, networking and HTTP. The runtime manages memory and supplies the services
those libraries need.

The project is experimental. The [installation guide](../try/) describes the available
toolchain; [feature guides](../guides/) explain supported behavior and practical limits.

<a id="dotnet"></a>

## Relationship to .NET

neoCLR uses familiar managed-platform concepts: values and objects, generics,
interfaces, metadata and garbage collection. Its runtime and class library are
independent implementations. UTF-8 text and explicit Option/Result outcomes are
important parts of its programming model.

Raven can target both .NET and neoCLR, each with its own library contracts.
neoCLR does not run arbitrary .NET applications. See the
[platform comparison](../comparison/) for the differences that affect programs.

<a id="implementation"></a>

## How it works

The runtime is implemented in Rust. Raven supplies the source language, compiler and
editor integration. The interpreter executes neoCLR assemblies; a developing ARM64
AOT backend compiles supported applications into standalone executables.

[Architecture](../architecture/) introduces these layers, and [metadata](../metadata/)
explains how compiled libraries connect tools and execution.

<a id="direction"></a>

## Project direction

The project develops through working applications and library use cases.
[Project direction](../proposals/) summarizes open areas such as broader native
execution, HTTP capabilities and module features. These are areas of work and
exploration, not a release schedule.

<a id="participate"></a>

## Contributing

Run a sample, report a problem, improve a guide or propose a change with a concrete
use case. For substantial contributions, open an issue to discuss scope.

[Source and pull requests](https://github.com/marinasundstrom/neoCLR) ·
[Issues and questions](https://github.com/marinasundstrom/neoCLR/issues)

The website explains the platform and how to use it. The repository's
[documentation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/README.md)
contains implementation contracts, design research, validation procedures and
[development history](https://github.com/marinasundstrom/neoCLR/blob/main/docs/development-timeline.md)
for contributors.
