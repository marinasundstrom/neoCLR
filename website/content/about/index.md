---
toc: false
---

# About neoCLR

neoCLR is an experimental application platform. It brings together application APIs, managed execution, language integration and development tools. Working programs guide which .NET and CLR contracts to retain and where to investigate alternatives.

**Early development.** Preview 9 is the published baseline, including Tasks, async execution and isolated workers. Streams, Storage and Encoding are the next development focus. The project is open source under the MIT license.

## Background

The project starts from .NET’s managed-platform model: value and reference types, generics, interfaces, metadata and garbage collection. The question is which parts to preserve and which alternatives are worth testing when building an independent application platform.

An earlier small language experiment, Neo, exercised the runtime through readable programs compiled to neoIL. It remains in the repository as historical work. The current frontend is [Raven](../raven/), a separate language that normally targets .NET. Its compiler emits CLI metadata and IL; the neoCLR integration imports the supported subset.

Using Raven lets application samples, library code and compiler integration develop together. Runtime guarantees also need direct IL tests, so their correctness does not depend solely on checks in one source language.

<a id="goals"></a>

## What the project aims to establish

The goal is to support the path from writing an application to building, running and inspecting it. The runtime, APIs, compiler integration, project tooling and editor support are parts of that platform. Each should be evaluated through complete programs, with foundational primitives and small library operations tested before larger applications.

Each proposed difference needs a concrete benefit, an account of its costs and executable evidence. Keeping the existing .NET behavior is a valid outcome. Starting independently does not by itself make an implementation faster, safer or simpler.

Proposals are inputs to this process. They can conflict, and their API shapes are not commitments. Samples should expose problems early enough to revise the contracts.

<a id="dotnet"></a>

## Relationship to .NET

neoCLR retains the distinction between copied values and shared class or array references, managed garbage collection, generics and ordinary method dispatch. CLI metadata provides a compiler boundary. The runtime and System library are independent implementations; arbitrary unchanged .NET assemblies are not supported.

| Area | Current neoCLR difference | Consequence |
| --- | --- | --- |
| [Errors and absence](../features/outcomes/) | Option and Result represent optional values and expected failures. | Callers match or propagate outcomes. APIs using exceptions or null in .NET need adaptation. |
| Generic Void | Void can be a type argument, including in Func and Result. | One generic family covers results with or without a payload; the compiler must support this target contract. |
| [Arrays](../features/arrays/) | Mutable arrays are invariant and expose a generic API shape. | Reference-array covariance is unavailable; code relying on those .NET conversions must change. |
| [Text](../features/strings/) | String uses UTF-8 storage; Char represents a grapheme, with explicit scalar and byte operations. | Indexing and conversion differ from .NET’s UTF-16 code-unit contracts. |
| [Tasks](../features/tasks/) | Development Task outcomes are completed or cancelled. Expected errors are Result values; terminal Fault is separate. | There is no .NET-style faulted Task state. Error handling and producer completion use different contracts. |

These are selected differences, not a claim of full feature coverage or superiority. Feature pages describe availability, examples and limitations.

<a id="implementation"></a>

## Current implementation

The runtime is a Rust interpreter with a nonmoving tracing collector and an initial terminal debugger. The library includes arrays, collections, typed outcomes, text conversion, date/time values, bounded file helpers and runtime metadata discovery. Preview 9 includes Task/Promise, async state machines and isolated workers. Development extends storage, streams and introspection.

neoCLR and its guest programs run without .NET. Raven compilation, the import bridge, MSBuild and the Raven Language Server use .NET. The [build and run guide](../try/) describes the matching toolchain and distinguishes published examples from development APIs.

Major limits include the bounded importer, no general socket or HTTP API, incomplete cleanup during terminal faults, and no JIT backend. The [guides](../guides/) record the smaller supported subsets.

<a id="direction"></a>

## Roadmap

Preview 9 delivered the [async and Tasks checkpoint](../features/tasks/#release-checkpoint). Current work develops Streams, Storage and Encoding through runnable samples before networking. No date is promised for the next release.

The first major application milestone is a Raven HTTP client/server pair exchanging UTF-8 text and JSON. The sequence starts with byte copy, text and JSON transformation, and controlled delayed operations that test GC lifetimes. A bounded file transformer and TCP echo precede HTTP.

Byte-copy, UTF-8, JSON and reduced VM completion checkpoints now run. Completion progresses between returning default-queue callbacks, including self-reposting work. Operation cancellation, queue affinity and buffer ownership still need work before general I/O. The [on-site roadmap overview](../proposals/#http-poc) describes the current evidence and remaining scope.

Later candidates include a file catalog, download queue, time-aware report, assembly explorer and portable sample pack. Their order can change. Memory views, nullability, runtime suspension and alternative execution backends remain research topics rather than prerequisites for every sample.

<a id="participate"></a>

## Participation

neoCLR is an early project initiated by Marina Sundström. It is open to people interested in managed runtimes, compilers, API design, tooling and documentation. Questions and critical comparisons are useful contributions alongside code.

A starting point is to run a sample and report what worked or failed, improve an explanation, add a focused test, or describe an application the current API cannot express. For substantial changes, discuss the scope first so the work fits the active milestone or answers an explicit research question.

[Contribution guidance](../#feedback) covers reports and proposals. [GitHub Issues](https://github.com/marinasundstrom/neoCLR/issues) is the current place for questions and design discussion; code and documentation changes can be submitted as pull requests.

### Development records

The repository retains the [development conversation record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/development-timeline.md), including author directions, AI-assisted work and subsequent corrections. The [working roadmap](https://github.com/marinasundstrom/neoCLR/blob/main/docs/platform-roadmap.md) links implementation evidence, and the [research process](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design-research.md) describes how design alternatives are compared.
