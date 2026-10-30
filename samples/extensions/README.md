# Configuration, DI, logging and localization together

`ExtensionsDemo` composes the default DI container, configuration, console logging and JSON
localization. It resolves a scoped GreetingService with Configuration, a typed logger and typed
localizer, reads Greeting:Name, formats the greeting, and logs it. Disposing the root provider disposes
its default logger factory and console provider. appsettings.json supplies Greeting:Name and Localization:Directory through
AddJsonFile. Run from this directory so the configuration and resource paths resolve.
AddConfiguration borrows the root, which is disposed after the DI container.
The supplied localizer factory
is caller-owned. Resources are read from `resources` relative to the working directory.

`ExtensionsConsumer` compiles the same application service using only System.Runtime
and the four Neo `.Abstractions` assemblies. It does not reference containers,
console providers or JSON providers. The executable compiles this source directly
alongside its composition entry point; the compile-only project demonstrates the
contract dependency boundary without introducing another deployment dependency.

From the repository root, with a matching development bundle:

```sh
python3 scripts/build-native-project.py \
  --project samples/extensions/src/ExtensionsDemo/ExtensionsDemo.rvnproj \
  --bundle target/neo-configuration-20261029/json-bundle/bundle \
  --aot target/debug/neoclr-codegen \
  --profile http \
  --output target/configuration-demo
```

The current development host profile `http` supplies the resource-file services;
this sample does not start a web server. The checked-in retention file keeps the
constructor metadata needed for DI and the typed logging/localization adapters.
Run from the sample directory:

```sh
cd samples/extensions
../../target/configuration-demo/app
```

Expected output:

```text
info: Examples.Extensions.GreetingService[0] Hej Ada! Logging and localization are composed through DI.
```

Compile the independent service contract consumer from the repository root:

```sh
dotnet /path/to/rvnc.dll neoclr \
  --project samples/extensions/src/ExtensionsConsumer/ExtensionsConsumer.rvnproj \
  --no-build-references
```

Build the runtime and Neo abstraction projects first using the matching compiler.
No application behavior depends on a service locator inside GreetingService;
all its dependencies are constructor parameters.

The JSON configuration provider uses the System.Data parser with comments and
trailing commas enabled, as demonstrated by appsettings.json. It accepts an initial
UTF-8 BOM and up to 1 MiB of input with 64 container levels. Value and container
entry counts are no longer capped at prototype sizes.

## Introductory examples

The four introductory projects are also shown on their feature and API pages:

| Project | Introduces | Output |
| --- | --- | --- |
| DependencyInjectionIntro | Register and resolve a caller-owned instance | Hello from DI |
| LoggingIntro | Factory, console provider and category logger | info: Introduction[0] Hello from logging |
| LocalizationIntro | JSON catalogue and formatted lookup | Hej Ada! |
| ConfigurationIntro | appsettings.json and colon-key lookup | Ada |

Build one from the repository root, substituting the project name as needed:

```sh
python3 scripts/build-native-project.py \
  --project samples/extensions/src/ConfigurationIntro/ConfigurationIntro.rvnproj \
  --bundle target/neo-configuration-20261029/json-bundle/bundle \
  --aot target/debug/neoclr-codegen --profile http \
  --output target/configuration-intro
cd samples/extensions
../../target/configuration-intro/app
```

ConfigurationIntro uses the shared appsettings.json. The other introductory
programs have no file dependencies. ExtensionsDemo shows all four services together.

### Observe configuration changes

Run ExtensionsDemo with `--watch` from this directory. It prints a greeting every
two seconds for 30 seconds. Edit `Greeting:Name` in appsettings.json while it runs;
the same DI-resolved service reads the reloaded value. Without `--watch`, it prints
once and does not start a watcher. Localization:Directory is a startup setting.
The sample disposes its service scope/container before the borrowed configuration
root; root disposal stops automatic reload. Background failures preserve valid data
and can be inspected through JsonConfigurationProvider.LastReloadError.
