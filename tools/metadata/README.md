# Experimental .NET metadata libraries

`NeoCLR.Metadata.Experimental` is the first .NET-hosted reusable format library for
future Raven symbol-loader and code-generation adapters. `MetadataConformance` is a
separate executable consumer. Both target .NET 10, are non-packable and depend only
on the platform libraries; neither requires Raven or Mono.Cecil.

The current library covers NEOX 0.1 envelope framing, immutable section ownership
and structural signature syntax/context validation. PE transport/recognition, actual
declaration resolution and compiler adapters remain pending. It is not the future neoCLR guest metadata/Introspection/Emit
library. Names and contracts are experimental.

- [Complete API contract](../../api-docs/experimental-metadata.md)
- [Format design](../../docs/design/extended-cli-metadata.md)
- [Wire schemas and probe history](../../docs/experiments/extended-cli-metadata/README.md)

From the repository root:

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet.py
python3 docs/experiments/extended-cli-metadata/verify_dotnet_signatures.py
```

The verifier builds the library/consumer, runs ownership and writer checks, cross-reads
four existing fixtures and independent .NET emission, and compares malformed-input
rejection with Python. Output binaries go under `target/extended-cli-metadata`; a
versioned evidence record goes beside the existing experiment records. No package is
published or installed. Prefer focused conformance over unrelated runtime/website builds.

The signature consumer compares 14 positive payloads and 103 rejection vectors against
Python, including nominal syntax (without resolution), nested Self/generic contexts,
Function modes and no-result, and resource limits. TypeExpression preserves wire syntax;
it is not the eventual Raven symbol model or resolved neoCLR type identity.
