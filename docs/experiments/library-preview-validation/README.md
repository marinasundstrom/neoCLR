# Library-preview validation evidence

Recorded 2026-09-08 on macOS 26.6.2 ARM64. This is development evidence, not a
published release or a claim of Linux/Windows validation.

The two archive reports validate commit
`eb4a7dd74977da4f0195fde66b38af42af4cbe0c` with stable Rust 1.95.0 and minimum Rust
1.85.0. Both independently built the extracted source, audited 47 dependencies and
94 notice files, and passed 26 Neo source/artifact scenarios plus native interop.
Their archive SHA-256 is identical:
`89055544180d6de79a243c30d4c2f72cf23495f729595da78c43b5866644d006`.

- [Stable archive report](macos-stable-archive.json)
- [Minimum-Rust archive report](macos-minimum-archive.json)

Both reports deliberately retain `full_tests: false`: the archives were run with
`--smoke-only`. They must not be presented as full six-job release certification.
The runtime code in this commit is unchanged from `e5220e0`; the new commit repairs
notice packaging and broadens the archive validator.

Reproduce the archive checks with:

```sh
python3 scripts/validate-release.py --revision eb4a7dd --toolchain stable --smoke-only
python3 scripts/validate-release.py --revision eb4a7dd --toolchain 1.85.0 --smoke-only
```

The full release gate remains the [platform/toolchain matrix](../../next-preview-validation.md)
for the exact selected release commit. Adding this evidence creates another commit;
it does not retroactively certify that later tree.

## All-targets test run and correction

The [test summary](macos-tests.json) records the complete local all-targets run and
its targeted correction. The first run completed 147 targets: 917 passed tests,
one failed assertion and 0 ignored tests. Its sole failure counted all Math.Abs
overloads when checking the original Int32 Result API. The test now selects the
parameter signature, consistent with the [Math contract](../../math.md). No runtime
behavior changed. All 11 console tests passed on rerun; all other targets passed in
the initial full run. The full suite was not repeated after this test-only change.

Rust 1.85 all-target compilation, strict stable Clippy and formatting checks also
passed. Linux/Windows execution and six-job full-archive certification remain
pending for an explicitly selected release commit.
