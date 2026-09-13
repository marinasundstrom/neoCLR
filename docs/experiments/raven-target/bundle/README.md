# neoCLR + Raven runtime API proof of concept

This is an experimental macOS arm64 build. It includes neoCLR, its adapted runtime
library, target metadata, a published Raven compiler bridge and a matching language
server. Neither development checkout is required. The matching Raven SDK archive
and VS Code VSIX are distributed separately. This is not a normal Raven release.

Prerequisites: .NET 11 SDK/runtime (the manifest records the tested preview), Python
3.9 or later, and VS Code with the matching experimental Raven extension for editing.

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
2. Run `python3 configure.py` from this folder. If you separately installed the SDK,
   use `python3 configure.py --sdk /absolute/path/to/raven-sdk`.
3. Open the **demo** folder in VS Code. `Main.rvn` demonstrates Result/Option/Void
   propagation. Completion should resolve neoCLR types; for example type
   `System.Date.` or `System.Type.` inside a function.
4. Use **Tasks: Run Task → neoCLR: Run saved project**. Build/run use the published
   bridge and supplied runtime library, not ordinary `dotnet run` on the project.
5. Copy another file from **tools/samples** over **demo/Main.rvn**, save, and rerun.
   Start with `library-files.rvn`, `library-calendar.rvn`, `library-reflection.rvn`,
   `library-value-interfaces.rvn`, or `library-reference-payloads.rvn`. The file sample
   creates `neoclr-file-demo.txt` in the working directory. Native buffers require an
   unsafe context and explicit Free; ordinary objects/arrays use GC.

Terminal equivalent, from this folder:

```sh
python3 tools/run_project.py demo/Demo.rvnproj --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
```

Verify both runtime-level neoIL samples:

```sh
python3 tools/verify_neoil.py --samples samples/neoil --system lib/System.neoil --runtime bin/neoclr
```

Run the repeatable Raven sample suite:

```sh
python3 tools/verify_project.py demo/Demo.rvnproj --collections --bridge tools/bridge/Probe.dll --system lib/System.neoil --runtime bin/neoclr
```

Run `configure.py` again after moving the extracted folder, because VS Code settings
contain its resolved paths. The SDK selection is workspace-local; no global default
is changed. The supplied language server is pinned to the bundle's compiler source.

See **docs/raven-runtime-api-coverage.md** for the existing API surface, deliberate
projection differences and unsupported compiler patterns. Recoverable failures use
Result/Option; terminal faults are not catchable exception objects. The importer is
bounded: arbitrary application types, general closures/virtual dispatch, rectangular
arrays, nullable metadata and fault-unwind cleanup are not claimed. Reflection is
introspection-only. Rough API edges are part of this experiment and are open to feedback.

**manifest.json** records exact source revisions, prerequisites and file hashes.
A separate validation record is added after testing the actual extracted package;
building this folder alone does not establish release readiness.

The bundle carries neoCLR's **LICENSE**, **THIRD_PARTY_NOTICES.md** and preserved
**third-party/** texts, plus Raven's upstream notices under **licenses/Raven/**.
These preserve the repositories' attribution; they do not claim a complete audit of
all dependencies in the separately packaged SDK and VSIX.
