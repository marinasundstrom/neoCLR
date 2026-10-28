# DI, logging and localization together

`ExtensionsDemo` composes the default DI container, console logging and JSON
localization. It resolves a scoped GreetingService with a typed logger and typed
localizer, formats the greeting, and logs it. Disposing the root provider disposes
its default logger factory and console provider. The supplied localizer factory
is caller-owned. Resources are read from `resources` relative to the working directory.

`ExtensionsConsumer` compiles the same application service using only System.Runtime
and the three Neo `.Abstractions` assemblies. It does not reference containers,
console providers or JSON providers. The executable compiles this source directly
alongside its composition entry point; the compile-only project demonstrates the
contract dependency boundary without introducing another deployment dependency.

From the repository root, with a matching development bundle:

```sh
python3 scripts/build-native-project.py \
  --project samples/extensions/src/ExtensionsDemo/ExtensionsDemo.rvnproj \
  --bundle target/neo-abstractions-20261028/test-bundle/bundle \
  --aot target/debug/neoclr-codegen \
  --profile http \
  --output target/extensions-demo
```

The current development host profile `http` supplies the resource-file services;
this sample does not start a web server. The checked-in retention file keeps the
constructor metadata needed for DI and the typed logging/localization adapters.
Run from the sample directory:

```sh
cd samples/extensions
../../target/extensions-demo/app
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
