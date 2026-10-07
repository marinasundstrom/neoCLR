# Native project root ownership — 2026-10-07

Raven 0f85f53b8 adds RavenNeoClrObjectLibrary to the native project provider. Select
System.Runtime when consuming the separately compiled source runtime. The same exact
registered artifact supplies the semantic Object owner and the --object-root argument
for rvnc neoclr --project ... --run .... Missing or ambiguous names reject before output
replacement; omission retains existing defaults. No metadata format or class-library
API change is involved.

The unchanged HTTP headers consumer now builds and executes from a .rvnproj against
separate Runtime/Data/Networking/Web artifacts, with exact expected output and exit 0.
The previous project command failed emission. Changing the owner to Missing.Owner rejects
and preserves the successful assembly bytes. C# checks cover exact root selection,
artifact watching, missing and ambiguous names alongside existing catalog/project checks.
The editor's metadata provider is covered; this is not an interactive VS Code acceptance.
[Commands and artifact hashes](native-project-root-2026-10-07.json).

Reproduce with matching explicit bootstrap inputs and a fresh output directory:

```sh
python3 scripts/verify-native-library-project.py --compiler /tmp/native-project-compiler1007/rvnc.dll --compiler-revision 0f85f53b8 --core /tmp/failure1006b/Core.dll --runtime-library-directory /tmp/array-runtime1007/runtime-owned --data /tmp/web-build1007final/System.Data.dll --networking /tmp/web-build1007final/System.Networking.dll --web /tmp/void-fixed-web1007-0/System.Web.dll --runtime target/release/neoclr --output /tmp/native-project-new
```

This closes a prerequisite for the real project graph. Native ProjectReference ordering,
transitive artifact catalogs, Platform integration projects and shipping collection remain
next. The ordinary .NET backend is unchanged; native projects continue to use explicit
Core/seed bootstrap dependencies without library-reference projection to CLI.
