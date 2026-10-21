# Scoped greeting sample

Development API. `Greeting.rvn` is also compiled directly into the built-in DI
suite, so the website example uses the tested implementation.

Build with a matching development bundle and backend:

```sh
python3 scripts/build-native-project.py \
  --project docs/experiments/dependency-injection/sample/App.rvnproj \
  --bundle /path/to/bundle \
  --aot /path/to/neoclr-codegen \
  --reflection-constructor-type Examples.DependencyInjection.Greeter \
  --output target/di-greeting-sample
./target/di-greeting-sample/app
```

Expected output: `Hello, Ada!`

The settings instance belongs to the caller. The provider constructs `Greeter`
inside a scope. `use` disposes the scope before the root provider, including on
propagated errors. This example requires only Greeter constructor retention; it
neither enumerates methods nor needs methodTypes retention.

Run the built-in coverage with `scripts/test-runtime-library.py --bundle ...
--aot ... --suite dependency-injection --mode interpreter --mode native --mode jit
--output target/di-tests`.
