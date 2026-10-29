# Configuration JSON reader qualification — 2026-10-29

This author-directed slice removes prototype DOM reader limits and adds
System.Text.Json-style document options. Configuration enables skipped comments
and trailing commas, accepts an initial UTF-8 BOM and uses a 1 MiB input limit.

The focused validation uses Raven `codex/neoclr` at `34f14c8ce`, rebuilt native
libraries and the matching compiler bundle. Results are recorded in [validation.json](validation.json). No compiler or runtime ABI change is required.

Coverage includes strict/default syntax; independent comment/comma options;
malformed comments and separators; depth 64/65 and custom depths; invalid options;
large objects and Unicode strings beyond 64 KiB; configuration file loading,
flattening, reload and failure preservation; stream ownership and decoding; and
the integrated DI/logging/localization/configuration sample.

JSON DOM/configuration native tests use a 32 MiB host heap for the large input and
1 MiB rejection boundaries, with the default 64 KiB byte-array limit. The parser
chunks token decoding rather than requiring a document-sized byte array. The
integrated sample uses the normal HTTP-profile host budget. Runtime resource
budgets remain distinct from parser syntax/size policy.


## Results

- Native: all 24 DOM tests and all 10 configuration tests passed. These include
  128-field documents, 64/65 depth boundaries, strings above 64 KiB, comments and
  trailing commas, BOM handling, malformed input, and the 1 MiB file rejection path.
- Streams: 20 contracts passed unchanged. The remaining test contained an obsolete
  assertion that numbers above 1024 bytes must fail; the corrected exact test
  function passed in a single-test native registry. It now checks preserved number
  text and the separate writer limit. A broad rerun was unnecessary.
- Interpreter: the focused options/depth filter passed both tests. The full debug
  interpreter DOM attempt exceeded its 180-second wall limit and is **not claimed
  as qualified**; large-input evidence comes from native execution.
- The integrated native sample read the commented appsettings.json and printed the
  expected localized DI/logging greeting. The sample uses the normal host budget.
- Native API rendering, four snapshot unit tests and the isolated pre-rename
  aggregate reference check passed. The website built and checked 6573 pages.

The author concurrently renamed Fail to Panic. All library/test evidence here
uses the pinned pre-rename bundle. JSON changes and their aggregate snapshot were
isolated against the base commit before committing; the author's working-tree
migration and its newer reference snapshot were preserved. These results do not
qualify the rename. The generated large-file fixtures avoid selecting the separate
native guest-file-write gap encountered during initial fixture preparation.

## Reproduction

Use the matching compiler/native bundle and the JSON slice before the concurrent
Panic migration, or rebuild a coherent bundle after integrating that migration.
From the repository root:

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/json-reader-20261029/test-bundle/bundle \
  --runtime target/debug/neoclr --aot target/debug/neoclr-codegen \
  --output target/json-reader-verification \
  --suite json-dom --suite json-streams --suite configuration --mode native
python3 -m unittest discover -s scripts -p 'test_native_api_snapshot.py'
python3 scripts/build-api-docs.py --check
python3 scripts/build-website.py
```

Full-library build, test-build commands, source fingerprints and compiler hashes
are retained in the local target/json-reader-20261029 build directories. Committed
validation.json preserves the relevant outcomes, input hashes and the targeted
interpreter command. The web preview is local, not a publication.
