# Install neoCLR

The matched bundle includes the runtime, Raven compiler, VS Code extension and samples.
The current native toolchain download supports **macOS on Apple silicon**.
Windows support for this workflow is being qualified; there is no matching Windows
native toolchain download yet.

## 1. Install prerequisites

Install the [.NET 11 SDK and .NET 10 SDK](https://dotnet.microsoft.com/download),
[Python 3](https://www.python.org/downloads/) and [VS Code](https://code.visualstudio.com/).
.NET is required by the Raven compiler and language server, not by neoCLR itself.

<a id="install"></a>

## 2. Download and extract

[Download neoCLR for macOS arm64](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.12/neoclr-preview12-osx-arm64.tar.gz).
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
<a id="development"></a>

## Help and verification

If a build fails, check the first diagnostic and confirm both .NET SDKs are installed
with `dotnet --list-sdks`. If editor symbols are missing, open the sample folder
containing the `.rvnproj` file and reload VS Code after installing the extension.

To check the complete installation, run this optional command from the extracted
`neoclr-native-poc` folder:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

It compiles and runs the bundled collections, Tasks, JSON and HTTP examples.

[Release notes and downloads](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.12)
include the exact supported toolchain and limitations. Older installation instructions
belong to their [matching release](https://github.com/marinasundstrom/neoCLR/releases).
