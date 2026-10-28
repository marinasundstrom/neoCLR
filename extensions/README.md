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

A component can have several source projects and zero or more test projects.
Neo.Localization and Neo.Logging use the same structure. Neo.Configuration is
a planned follow-up, not an implemented API. See the
[service boundaries and migration](../docs/application-services.md) and
[class-library build](../runtime/class-library-projects.md).

The shared test driver selects these projects with `--suite dependency-injection`
and `--suite localization` / `--suite logging`; they share tools/testing/NeoCLR.Testing without
including another suite's test entry point.
