# Structural value union integration

The runtime and companion Raven compiler keep structural unions as values, whether
an alternative contains a value or an object reference.

- `project/`: boxing/extraction and ref/out writeback, exit 42.
- `task-project/`: awaited `Task<Result<Response, HttpError | JsonError>>` with
  completed/queued success and both imported error alternatives, exit 42.
  `task-results.json` records the command and artifact hashes.
- `callback-project/`: an exact callback returning a structural union, exit 42.
- `fresh-bundle-qualification.json`: the HTTP-header consumer compiled and executed
  against the fresh v3 Runtime/Data/Networking/Web bundle, exit 0, plus invalid
  configuration rejection without replacing the successful output. Includes commands
  and artifact hashes. Runtime evidence covers 53 focused structural tests.

The fresh local bundle is `target/structural-integrated-libraries-v3`; its props
file is `NeoCLR.ClassLibrary.props`. Bootstrap and source-build provenance are in
that directory's `build-evidence.json` and manifest. The qualified project is
`target/structural-integrated-libraries-v3-qualified/Headers.rvnproj`.

Reproduce qualification with `scripts/verify-native-library-project.py`, using the
paths and compiler revision recorded in the evidence and a new output directory.
These are development feature-branch artifacts. This result does not qualify every
HTTP endpoint, all open structural operands, or JIT/AOT execution.

Run the task sample against the same v3 bundle with:

```sh
dotnet /path/to/rvnc.dll neoclr --project docs/experiments/value-unions/task-project/App.rvnproj --run /path/to/neoclr
```

An exit status of 42 means all six checks passed. The sample uses native union
extraction from a nominal Result carrier. Explicit whole-union boxing and final
boxed identity remain unqualified; see the integration document for the temporary
interpreter representation.

## Direct unions and generic alternatives

- `direct-task-project/`: `Task<Response | HttpError | JsonError>`, exit 42.
- `closed-generic-task-project/`: `Task<Response<int> | HttpError | JsonError>`, exit 42.
- `open-generic-project/`: a generic method with `Response<T> | HttpError | JsonError`
  now executes with `T = int`, exit 42. `open-generic-results.json` records the
  successful follow-up; bare `T` alternatives remain unsupported.

Run each with the command above, substituting the project directory. Both task
consumers check all three alternatives through completed, queued and async-forwarded
tasks, plus preserved reference identity (ten checks each). They use native union
signatures with imported value errors and a reference response. The callback
identity check captures the response reference; directly capturing a union local
remains unsupported. `direct-task-results.json` records the successful executions
and the historical open-generic rejection with source/tool/bundle hashes. Focused validation passes
21 compiler tests and 55 runtime tests.
