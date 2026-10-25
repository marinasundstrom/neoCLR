# Install neoCLR

The matched bundle includes the runtime, Raven compiler, VS Code extension and samples.
Preview 17 provides matching packages for **Windows x64** and **macOS on Apple silicon**.
JIT is the default on both; interpreter execution remains available explicitly.
AOT is a separate experimental publishing workflow for supported applications.

**In preparation:** Preview 18 is being qualified for both platforms. It includes
API migrations (`Optional<T>`, `Result.Failure`), HTTP/value improvements and runtime
fixes. Synchronous GC remains the default; concurrent GC is still experimental.
[Candidate scope and migration notes](https://github.com/marinasundstrom/neoCLR/blob/main/docs/next-preview-release-notes.md).
The downloads below remain the published Preview 17 packages.

## 1. Install prerequisites

Install the [.NET 11 SDK and .NET 10 SDK](https://dotnet.microsoft.com/download),
[Python 3](https://www.python.org/downloads/) and [VS Code](https://code.visualstudio.com/).
The bundles were tested with .NET 11 SDK **11.0.100-rc.1.26425.128** (RC1)
and a .NET 10 SDK. .NET is required by the Raven compiler and language server,
not by neoCLR itself.

<a id="install"></a>

## 2. Download and extract

- [Download for macOS ARM64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.17/neoclr-preview17-osx-arm64.tar.gz)
- [Download for Windows x64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.17/neoclr-preview17-win-x64.tar.gz)

Extract the archive and keep its folders together.

In VS Code, open **Extensions → ⋯ → Install from VSIX…** and select
`editor/raven-vscode.vsix` from the extracted folder.

<a id="run"></a>

## 3. Run a sample

In VS Code, choose **File → Open Folder** and open `samples/collections` inside
the extracted folder. Open `application-order-collections.rvn`, then choose
**Terminal → Run Task → neoCLR: Build and Run (JIT)**. Save edits before running again.

The task compiles your Raven code and runs it on neoCLR. Try the `tasks` and `json`
sample folders next.

**neoCLR: Build and Run (JIT)** is also the default build task. The separate
**neoCLR: Compile to Native** action requires the matching
[macOS ARM64 build kit](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.17/neoclr-native-build-kit-osx-arm64.tar.gz),
Apple build tools and a fresh output directory. It builds without launching the result.

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
bundle root folder:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

On Windows, use `python` in place of `python3`. It compiles and runs the bundled
collections, Tasks, JSON and HTTP examples.

[Release notes and downloads](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.17)
include the exact supported toolchain and limitations.

<a id="development"></a>

<a id="preview-15-capabilities"></a>
<a id="preview-16-capabilities"></a>

## Published bundle capabilities

HTTP client/server and typed JSON checks pass in interpreter, JIT and AOT
modes on macOS ARM64 and Windows x64. Security-provider checks cover all three modes;
the source-built HTTPS server has separate matching-package qualification. Runtime-owned suspension and resumption
are the default for Raven async code on both platforms.

The optional PostgreSQL provider is included in the macOS bundle. The Windows
bundle contains the four base libraries without that provider. Start with the
[PostgreSQL sample](/cases/postgresql/) for local database queries.

For native publishing, follow the
[project build instructions](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.17/docs/native-poc-bundle.md)
with the matching compiler, runtime, libraries and host tools. AOT remains bounded;
JIT package acceptance does not imply every application can be published natively.

[Preview 17 release notes](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.17) ·
[Qualification results](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.17/docs/validation/release-20261023/README.md)
