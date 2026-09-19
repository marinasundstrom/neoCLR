# neoCLR Preview 8 — runtime library, Introspection and text

Version: **0.1.0-preview.8** · Tag: **v0.1.0-preview.8**.
Release scope selected **2026-09-19**. The attached manifest records the actual
publication evidence for this version.

This preview ports System.Runtime to Raven and starts a unified Introspection API.
It also makes ordinary character handling use grapheme clusters, with explicit
Unicode scalar and UTF-8 operations. The goal is a small, working API that shows
where neoCLR is heading and invites feedback. These contracts can still change.

## Install and try

The matching asset set is:

- `neoclr-0.1.0-preview.8-osx-arm64.tar.gz`
- `raven-sdk-0.1.12-neoclr.15-osx-arm64.tar.gz`
- `raven-vscode-0.1.12-neoclr.15.vsix`
- `raven-toolchain-notices.tar.gz`
- source archives, validation evidence, `release-manifest.json` and `SHA256SUMS`.

Prebuilt tools support **macOS arm64**. The Raven compiler, MSBuild, importer and
Raven Language Server require .NET SDK **11.0.100-rc.1.26425.128**. The VS Code
extension connects to that language server. Python 3.9+ is used for configuration
and validation. **neoCLR and programs running on neoCLR do not depend on .NET.**
Source CI covers Linux, macOS and Windows; binary packages do not claim those other
hosts. Use the matching packages together rather than mixing previous previews.

Extract the runtime and SDK separately, install the VSIX, then run from the
runtime bundle:

```sh
python3 configure.py --sdk /absolute/path/to/extracted/raven-sdk
code msbuild-demo
```

Use **Tasks: Run Build Task** to compile, or **neoCLR: Run (MSBuild)** to build and
run. Save Main.rvn first. The ordinary Raven toolbar targets .NET and is not the
neoCLR run/debug path. The project uses standalone `.rvnproj` build assets.

Copy `tools/samples/library-introspection-tour.rvn` or
`tools/samples/library-grapheme-strings.rvn` over `msbuild-demo/Main.rvn` to try the
new APIs. Their expected output files accompany them. The text sample counts
4 graphemes, 12 scalars and 37 UTF-8 bytes in the same string; it also exercises
character literals, matching, arrays, fields and copying.

The application/library project-reference demo and direct neoIL samples remain
available. See the bundled README for commands and [MSBuild limits](raven-msbuild.md).

## What changed

- System.Runtime has 77 reproducible Raven implementation slices. Native services
  remain explicit runtime boundaries; the port does not make every operation
  managed code or change the archived Neo frontend into the current language.
- `typeof(T)` and `Object.GetType()` return `System.Introspection.TypeInfo` directly.
  `RuntimeContext.Current.ExecutingAssembly` exposes assembly/module/type discovery.
  AssemblyInfo reports ReferencedAssemblies, including System.Runtime.
- The Info model uses sealed interfaces. TypeInfo extends MemberInfo alongside
  FieldInfo, MethodInfo and PropertyInfo. MetadataToken is exposed on the Info
  interfaces and scoped by module for definitions. Collection contracts return
  Sequence<T>. Discovery describes retained loaded metadata; it does not load code.
- String is immutable Unicode text. Char is one extended grapheme cluster,
  independent of encoding. UTF-8 remains canonical storage. String.Length counts
  graphemes and iteration yields Char; GetScalars exposes Sequence<uint> and
  UnicodeScalar classification handles explicit numeric scalar values.
- Utf8.Encode/Decode expose Sequence<byte> and strict typed decoding errors.
  Conversion preserves BOM, NUL and normalization forms; malformed bytes are not
  silently replaced. Ordinal comparison follows UTF-8/scalar order.
- Website feature pages explain current behavior using tested samples, with a
  separate proposal overview. Feature boxes link to those pages, and the Raven
  and try-it guides explain the project workflow without requiring repository visits.

## Breaking changes and migration

Rebuild applications and references with the matching compiler, importer and runtime.
System.Type and TypeOf<T>.Of are retired from the Raven API. Remove the `.Info` hop;
use TypeInfo directly. Add TypeInfo to exhaustive MemberInfo matches. DeclaringType
is Option<TypeInfo>; extract its payload with patterns. Use Sequence Count rather
than array Length in Introspection code. BindingFlags is an ordinary enum.

Char is no longer an integer or a fixed-width native value. Numeric casts and
arithmetic are rejected; use explicit scalar access for Unicode algorithms.
String.IsEmpty is a property. GetUtf8ByteCount remains explicit byte measurement.
Data sorted using the previous UTF-16 ordinal rule may need re-sorting: U+10000
now sorts after U+E000. Raven's unit keyword and () still map to System.Void.

## Preview limits and direction

Grapheme segmentation is pinned to Unicode 16. A grapheme approximates a perceived
character; it is not a guarantee of one rendered glyph. Length scans and current
iteration creates snapshots. Integer string indexing and cursors are deferred.
Equality remains ordinal without automatic normalization. Literal diagnostics use
host Unicode rules while runtime construction enforces pinned target rules; CLI
constant fields are outside the tested Char surface. Dedicated scalar values,
normalization, collation and efficient traversal remain open design work.

Possible Utf8String and AsciiString types may later offer encoding-specific
functionality alongside neutral String/Char. They are not included in this preview.
Introspection does not yet provide dynamic loading, reflection invocation or emit.
Runtime async, broad globalization, fault-unwind cleanup, a JIT and full Raven
source debugging in VS Code remain outside this release.

Compiler target policies stay on Raven's neoclr branch. General compiler fixes
are reviewed and integrated independently into Raven main. This experimental SDK
is not a normal Raven release or evidence of NanoFramework hardware testing.

## Validation

Publication requires the exact candidate's six stable/minimum-Rust source CI jobs,
extracted runtime/SDK/VSIX checks and notice/hash audits. The attached manifest and
validation archive record the actual revisions and results; this document
alone is not evidence of a passed release gate.
