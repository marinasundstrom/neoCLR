# Runtime attribute data

Development after Preview 10. Read application attributes from types, fields,
properties, constructors, methods and parameters through MemberInfo and ParameterInfo.

```sh
python3 docs/experiments/attribute-introspection/verify.py \
  --toolchain-root /path/to/development-bundle --runner /path/to/measure_async
```

Main.rvn checks String/Int32/Boolean argument types and values, unannotated types,
repeated reads, defensive copies, union cases and separate/grouped/repeated attribute
declarations (including AllowMultiple). Routes.rvn inspects catalog route templates
at startup without source generation. Every NoteAttribute constructor faults: success
proves that inspection never instantiates the attributes. The verifier also checks
that unsupported Int64 constants and named arguments fail import instead of disappearing.

See [the contract](../../attribute-introspection.md) for retention limits, the
nullable-string source-emission gap, and the future cached route mapper. This is
metadata reading; it does not add general value-case construction or route dispatch.
