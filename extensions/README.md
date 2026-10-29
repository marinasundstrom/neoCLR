# Application extensions

Optional application services reference the runtime libraries without becoming
part of the runtime layer. Each component groups related source and test projects:

```text
Neo.DependencyInjection/
  src/Neo.DependencyInjection/
    Neo.DependencyInjection.rvnproj
    Neo/DependencyInjection/...
  test/Neo.DependencyInjection.Tests/
    Neo.DependencyInjection.Tests.rvnproj
```

Each component now has a second source project, `<Component>.Abstractions`,
containing provider-neutral contracts and helpers. Implementation projects reference
their abstractions; logging/localization/configuration DI integration references DI abstractions.
A component can have several source projects and zero or more test projects.
Neo.Localization, Neo.Logging and Neo.Configuration use the same structure. See the
[service boundaries and migration](../docs/application-services.md) and
[class-library build](../runtime/class-library-projects.md).

The shared test driver selects these projects with `--suite dependency-injection`
and `--suite localization` / `--suite logging` / `--suite configuration`; they share tools/testing/NeoCLR.Testing without
including another suite's test entry point.
