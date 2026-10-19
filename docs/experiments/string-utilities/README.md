# String predicates and comparison utilities — 2026-10-19

Raven String adds IsNullOrEmpty(string?), IsNullOrWhiteSpace(string?),
Equals(string, StringComparison) and CompareTo(string), implementing
ComparableTo<string>. These use existing byte views and comparison services; no
runtime/compiler representation or intrinsic changes are introduced.

The nullable predicates accept null intentionally. Whitespace uses Unicode 17's
25 White_Space scalars, including empty text. Combining marks and format characters
such as U+200B/U+FEFF/U+180E are not whitespace. The implementation scans borrowed
UTF-8 instead of materializing Trim(). Protocol-specific whitespace remains separate.
Equals with a policy delegates to the same comparison engine as String.Compare;
unknown values fault even when the operands are identical. CompareTo requires
non-null text and uses canonical NFD ordering, not locale collation. Use the
corresponding StringComparer for matching equality and hash policies in collections.

Main.rvn covers all whitespace scalars, null/empty/NUL, mixed text and emoji,
canonical/ordinal/simple-fold distinctions, ordering signs, interface dispatch and
preserved original bytes. Invalid.rvn checks the invalid-mode fault. Allocation.rvn
compares 128 predicate iterations with a baseline, using the existing native GC
counter host; it does not measure CPU or HTTP throughput.

```sh
python3 docs/experiments/string-utilities/check.py \
  target/string-utilities-bundle-v1/bundle target/string-utilities-public-v1 \
  --invalid target/string-utilities-invalid-v2 \
  --output target/string-utilities-validation-final.json
python3 docs/experiments/string-utilities/measure.py \
  target/string-utilities-allocation-v1 --output target/string-utilities-allocation.json
```

Build fresh libraries with build-native-class-library.py, prepare a matched bundle
with prepare-jit-bundle.py and build each rvnproj with build-native-project.py's
console profile. Artifacts use Raven compiler code 2d22055dc, metadata adapter
e4709c451, neoCLR backend 6e7bf1d9a and libraries rebuilt from the utility working
tree atop 900e75455. Pinned Xcode/SDK, macOS ARM64, native eager-boundary GC.
These are development checks, not extracted release-package qualification.

The string-as-view/value representation decision is explicitly deferred. Current
String storage, copying and null semantics remain unchanged by these utilities.


All six public/fault executions pass across interpreter/JIT/AOT; see
[validation.json](validation.json). The native allocation check reports two
allocations in both baseline and predicate runs, all reclaimed. Eager boundary
collections are 517 versus 6,533: no throughput improvement is claimed. Both API
snapshots and four native snapshot tests pass.

The local website build checks 4,556 pages; publication remains separate.
