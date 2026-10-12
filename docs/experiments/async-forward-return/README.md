# Forward async result declaration regression

An extracted preview compiler at Raven `acfe352067b8428f8414d345111527ad6a8b26ef`
intermittently rejected `Task<Result<unit, MappingError>>` when the union was
in another source file. [The retained failing baseline](before.json) reports
RAV2704; other fresh-process builds of the same inputs passed. Early signature
skeleton lookup can see an unresolved generic argument before declarations are
available, and the native target has no non-generic Task fallback.

Raven main `0ccb9a265` defers validation of unresolved skeletons to full binding;
the neoCLR branch deliberately carries it in `6b5e052ce`. Resolved invalid async
returns and unknown names still receive binding diagnostics. No Runtime Contract
configuration, temporary CLI encoding, native metadata format or codegen changes.
This fixes declaration diagnostics, not an application or runtime workaround.

Run against a coherent packaged JIT bundle (the output directory must be new):

```sh
python3 docs/experiments/async-forward-return/verify.py \
  --bundle /path/to/bundle --output target/async-forward-check
```

The default runs three fresh-process compilations for each source order and checks
observable execution. The general .NET check passed 75 async semantic/lowering
cases, including both source orders and unresolved/invalid-return controls.
[Native validation](after.json) passes six fresh-process builds and six default-JIT
runs across both source orders. The rebuilt JSON package consumer also compiles and
runs; complete HTTP/package acceptance is a separate release gate.
