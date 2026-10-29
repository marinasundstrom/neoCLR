# Configuration: providers and hierarchical settings

Development (2026-10-29): Neo.Configuration composes ordered configuration sources
behind provider-independent contracts. Later providers take precedence; colon paths
provide sections and immediate child enumeration. Sections remain live after writes
and explicit reloads. In-memory and optional JSON file providers supply the initial implementation.

Consumer contracts live in `Neo.Configuration.Abstractions`. The implementation
assembly supplies the default builder/root/sections, memory source/provider and
AddConfiguration integration with DI abstractions. The container borrows the
configuration root; the application controls its lifetime.

The [integrated sample](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md)
reads appsettings.json to choose a greeting name and JSON localization directory,
then injects Configuration alongside typed logging and localization into its service.

See the [configuration API reference](/docs/configuration.html) for members,
ownership and null semantics. The JSON file provider accepts comments, trailing
commas and a UTF-8 BOM, like Microsoft.Extensions.Configuration. It reads up to
1 MiB with a maximum nesting depth of 64. Change notifications,
binding and options remain future work; access is synchronous and serialized.

## Try it

Build configuration from appsettings.json, then read a colon-delimited key. The file is read during Build; `use` disposes the root.

Run from `samples/extensions`, containing this appsettings.json:

```json
{
  // Configuration accepts comments and trailing commas, like Microsoft's provider.
  "Greeting": { "Name": "Ada", },
  "Localization": { "Directory": "resources", },
}
```

```raven
module Examples.Extensions.Introductions.Configuration

import System.*
import Neo.Configuration.*

func Main() -> int {
    let builder: ConfigurationBuilder = DefaultConfigurationBuilder()
    _ = builder.AddJsonFile("appsettings.json")
    use configuration = builder.Build()
    Console.WriteLine(configuration["Greeting:Name"] ?? "world")
    return 0
}
```

[Complete sample project](https://github.com/marinasundstrom/neoCLR/tree/main/samples/extensions/src/ConfigurationIntro) · [Build and run instructions](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md#introductory-examples)
