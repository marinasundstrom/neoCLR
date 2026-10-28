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
Neo.Localization uses the same structure. Neo.Configuration and Neo.Logging are
planned follow-ups, not implemented APIs. See the
[service boundaries and migration](../docs/application-services.md) and
[class-library build](../runtime/class-library-projects.md).

The shared test driver selects these projects with `--suite dependency-injection`
and `--suite localization`; they share tools/testing/NeoCLR.Testing without
including another suite's test entry point.
