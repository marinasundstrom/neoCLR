# neoCLR + Raven runtime API proof of concept

This is an experimental macOS arm64 build. It includes neoCLR, its adapted runtime
library, target metadata, a published Raven compiler bridge and a matching language
server. Neither development checkout is required. The matching Raven SDK archive
and VS Code VSIX are distributed separately. This is not a normal Raven release.

Prerequisites: .NET 11 SDK/runtime (the manifest records the tested preview), Python
3.9 or later, and VS Code with the matching experimental Raven extension for editing.

neoCLR and its guest programs do not depend on .NET. The compiler, importer,
MSBuild and Raven Language Server use .NET; the VS Code extension connects to
that language server.

You can first try the runtime directly, without the Raven tools:

```sh
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

Expect `42`, `7`, `9` from the first, and `Completed`, `Not saved` from the second.
The [release walkthrough](docs/runtime-raven-preview.md) explains both runtime and
Raven entry points. Continue with Raven and VS Code:

1. Extract this folder. Install the matching `raven-vscode.vsix` using VS Code's
   **Extensions: Install from VSIX** command, or `code --install-extension PATH`.
2. Extract the matching Raven SDK and run
   `python3 configure.py --sdk /absolute/path/to/raven-sdk` from this folder.
3. Open the **msbuild-demo** folder in VS Code. `Main.rvn` demonstrates Result/Option/Void
   propagation. Completion should resolve neoCLR types; for example type
   `System.Date.` or `System.Runtime.RuntimeContext.` inside a function.
4. Use **Tasks: Run Task → neoCLR: Run (MSBuild)**. It first runs the default
   MSBuild build task, then executes the verified program. **Tasks: Run Build Task**
   compiles without running. No Microsoft.NET.Sdk import or guest .NET target framework
   is used. The compiler and importer remain separate stages.
5. Copy another file from **tools/samples** over **msbuild-demo/Main.rvn**, save, and rerun.
   Start with `library-grapheme-strings.rvn`, `library-introspection-tour.rvn`,
   `library-files.rvn`, `library-calendar.rvn`, `library-reflection.rvn`,
   `library-value-interfaces.rvn`, or `library-reference-payloads.rvn`. The file sample
   creates `neoclr-file-demo.txt` in the working directory. Native buffers require an
   unsafe context and explicit Free; ordinary objects/arrays use GC.
6. Try `application-types.rvn`, `application-interfaces.rvn`,
   `application-inheritance.rvn`, and `application-delegates.rvn` for application
   classes, value copies, dispatch and shared or escaping lambda captures.
   `application-orders.rvn` combines these library contracts in a small order workflow;
   it creates or replaces `neoclr-orders.txt` in the working directory.

Terminal equivalent, from this folder:

```sh
dotnet msbuild msbuild-demo/Demo.rvnproj -p:RavenSdkRoot=/absolute/path/to/raven-sdk
./bin/neoclr run msbuild-demo/bin/neoclr/Debug/App.neoil --system msbuild-demo/bin/neoclr/Debug/System.neoil
```

For an application referencing a library, open **project-reference-demo/App** and
use the same tasks. Its `ProjectReference` builds `Library/Library.rvnproj` before
compiling the application. The program prints `42` and `Library call`. The language
server resolves the library's public members from the project graph before building.
The current build supports one directly referenced library, not a general project graph.

The older **demo** folder and `tools/run_project.py` remain an advanced compiler/import
runner and a fixture for the compatibility suites below. They are not the primary
build workflow. See [MSBuild scope](docs/raven-msbuild.md).

Verify both runtime-level neoIL samples:

```sh
python3 tools/verify_neoil.py --samples samples/neoil --system lib/System.neoil --runtime bin/neoclr
```

## Async Workbench

The development async/Tasks surface includes Task/Promise, named async functions,
await, Map/Then/MapResult, Result propagation and producer cancellation. Workers are
isolated and exchange text; ordinary queued joins can block the default dispatcher.
The separate nonblocking notification adapter and guest cancellation tokens are not
part of this sample set.

Run eight saved-source examples through the packaged MSBuild workflow:

```sh
python3 tools/verify_async_workbench.py --bundle . --sdk /absolute/path/to/raven-sdk --report async-workbench.json
```

The check uses fresh temporary projects and asserts exact output for default-queue
workers, dedicated/pool workers, Promise, composition, Result mapping/propagation,
cancelled await and pending await. Copy any matching `tools/samples/library-task-*.rvn`
or `library-async-*.rvn` over `msbuild-demo/Main.rvn` to explore it in the editor.

The deeper probes are included under `tools/task-contract`. For example:

```sh
python3 tools/task-contract/verify_workers.py demo/Demo.rvnproj --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
python3 tools/task-contract/verify_async.py demo/Demo.rvnproj --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
python3 tools/task-contract/verify_editor.py msbuild-demo
```

These are focused checks, not a complete release or platform certification.

Run the repeatable Raven sample suite:

```sh
python3 tools/verify_project.py demo/Demo.rvnproj --collections --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
```

Check application semantics and the order workflow in temporary directories:

```sh
python3 tools/verify_application.py demo/Demo.rvnproj --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
python3 tools/verify_orders.py demo/Demo.rvnproj --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
```

These checks include value copies versus class aliases, interface/virtual dispatch,
callbacks surviving GC, saved-source rebuilding, file round trips and rejected writes.

Run `configure.py` again after moving the extracted folder, because VS Code settings
contain its resolved paths. The SDK selection is workspace-local; no global default
is changed. The supplied language server is pinned to the bundle's compiler source.

See **docs/raven-runtime-api-coverage.md** for the existing API surface, deliberate
projection differences and unsupported compiler patterns. Recoverable failures use
Result/Option; terminal faults are not catchable exception objects. The importer is
bounded: generic application definitions, custom delegate declarations, value-receiver
method groups, explicit/default interfaces, rectangular arrays, nullable metadata and
fault-unwind cleanup are not claimed. See [application types](docs/raven-application-types.md)
and [delegate boundaries](docs/raven-delegates-lambdas.md). Reflection is
introspection-only. Rough API edges are part of this experiment and are open to feedback.

**manifest.json** records exact source revisions, prerequisites and file hashes.
A separate validation record is added after testing the actual extracted package;
building this folder alone does not establish release readiness.

The bundle carries neoCLR's **LICENSE**, **THIRD_PARTY_NOTICES.md** and preserved
**third-party/** texts, plus Raven's upstream notices under **licenses/Raven/**.
The version-specific tool dependency inventory is in **third-party/raven-tools/**.
It supplements the upstream notices and accompanies the separate SDK/VSIX assets.


Validate the primary build workflow and project-reference support:

```sh
python3 tools/verify_msbuild.py --bundle "$PWD" --sdk /absolute/path/to/raven-sdk
python3 tools/verify_editor.py project-reference-demo/App --files --project-references
```
