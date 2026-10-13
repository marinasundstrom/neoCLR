# Install neoCLR

The matched bundle includes the runtime, Raven compiler, VS Code extension and samples.
Preview 14 provides matching packages for **Windows x64** and **macOS on Apple silicon**.
JIT is the default on both; interpreter execution remains available explicitly.
AOT is a separate experimental publishing workflow for supported applications.

## 1. Install prerequisites

Install the [.NET 11 SDK and .NET 10 SDK](https://dotnet.microsoft.com/download),
[Python 3](https://www.python.org/downloads/) and [VS Code](https://code.visualstudio.com/).
The bundles were tested with .NET 11 SDK **11.0.100-rc.1.26425.128** (RC1)
and a .NET 10 SDK. .NET is required by the Raven compiler and language server,
not by neoCLR itself.

<a id="install"></a>

## 2. Download and extract

- [Download for macOS ARM64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/neoclr-preview14-osx-arm64.tar.gz)
- [Download for Windows x64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/neoclr-preview14-win-x64.tar.gz)

Extract the archive and keep its folders together.

In VS Code, open **Extensions → ⋯ → Install from VSIX…** and select
`editor/raven-vscode.vsix` from the extracted folder.

<a id="run"></a>

## 3. Run a sample

In VS Code, choose **File → Open Folder** and open `samples/collections` inside
the extracted folder. Open `application-order-collections.rvn`, then choose
**Terminal → Run Task → neoCLR: Run**. Save edits before running again.

The task compiles your Raven code and runs it on neoCLR. Try the `tasks` and `json`
sample folders next.

[Learn Raven →](../raven/) · [Explore the APIs →](../docs/)

<a id="project"></a>
<a id="explore"></a>

## Your project

Start with a copy of a bundled sample folder, kept under `samples/`. Its `.rvnproj`
file selects the native libraries and its VS Code settings select the bundled compiler.
Keep compiler, runtime, library and extension versions together when updating.

Hover over a type or member to see its signature and available API documentation.
Libraries can supply XML or Markdown documentation beside their assemblies.
Source debugging on neoCLR is not available; use the run task and printed output.

<a id="limits"></a>

## Help and verification

If a build fails, check the first diagnostic and confirm both .NET SDKs are installed
with `dotnet --list-sdks`. If editor symbols are missing, open the sample folder
containing the `.rvnproj` file and reload VS Code after installing the extension.

To check the complete installation, run this optional command from the extracted
`neoclr-native-poc` folder:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

On Windows, use `python` in place of `python3`. It compiles and runs the bundled
collections, Tasks, JSON and HTTP examples.

[Release notes and downloads](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.14)
include the exact supported toolchain and limitations. Older installation instructions
belong to their [matching release](https://github.com/marinasundstrom/neoCLR/releases).

<a id="development"></a>

## Preview 14 capabilities

HTTP client/server, typed JSON and HTTPS checks pass in interpreter, JIT and AOT
modes on macOS ARM64 and Windows x64. Windows JIT uses compiler-generated async
state machines; runtime-owned suspension has separate target-specific coverage.

The optional PostgreSQL provider is included in the macOS bundle. The Windows
bundle contains the four base libraries without that provider. Start with the
[PostgreSQL sample](/cases/postgresql/) for local database queries.

For native publishing, follow the
[project build instructions](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.14/docs/native-poc-bundle.md)
with the matching compiler, runtime, libraries and host tools. AOT remains bounded;
JIT package acceptance does not imply every application can be published natively.

[Preview 14 release notes](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.14) ·
[Qualification results](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.14/docs/experiments/release-20261013/README.md)
