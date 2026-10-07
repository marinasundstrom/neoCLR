# Third-party notices

neoCLR's original source is provided under the [MIT license](LICENSE). Its Rust
build uses the following dependencies pinned by Cargo.lock, including transitive,
build-time and target-specific packages. Their licenses are listed as declared by
the packages; this inventory does not relicense dependency code.

The source preview does not vendor these packages. Cargo downloads them separately.
License texts are retained here for attribution and review. The inventory also covers
windows-link, even though it was not compiled in the local macOS validation.

| Package | Locked version | Declared license expression | Preserved texts |
| --- | --- | --- | --- |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/block-buffer-0.10.4/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/block-buffer-0.10.4/LICENSE-MIT) |
| ciborium | 0.2.2 | Apache-2.0 | [LICENSE](third-party/licenses/ciborium-0.2.2/LICENSE) |
| ciborium-io | 0.2.2 | Apache-2.0 | [LICENSE](third-party/licenses/ciborium-io-0.2.2/LICENSE) |
| ciborium-ll | 0.2.2 | Apache-2.0 | [LICENSE](third-party/licenses/ciborium-ll-0.2.2/LICENSE) |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/cpufeatures-0.2.17/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/cpufeatures-0.2.17/LICENSE-MIT) |
| crunchy | 0.2.4 | MIT | [LICENSE](third-party/licenses/crunchy-0.2.4/LICENSE) |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/crypto-common-0.1.7/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/crypto-common-0.1.7/LICENSE-MIT) |
| digest | 0.10.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/digest-0.10.7/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/digest-0.10.7/LICENSE-MIT) |
| generic-array | 0.14.7 | MIT | [LICENSE](third-party/licenses/generic-array-0.14.7/LICENSE) |
| half | 2.7.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/half-2.7.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/half-2.7.1/LICENSE-MIT) |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/sha2-0.10.9/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/sha2-0.10.9/LICENSE-MIT) |
| typenum | 1.20.1 | MIT OR Apache-2.0 | [LICENSE](third-party/licenses/typenum-1.20.1/LICENSE), [LICENSE-APACHE](third-party/licenses/typenum-1.20.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/typenum-1.20.1/LICENSE-MIT) |
| version_check | 0.9.5 | MIT/Apache-2.0 | [LICENSE-APACHE](third-party/licenses/version_check-0.9.5/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/version_check-0.9.5/LICENSE-MIT) |
| zerocopy | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT | [LICENSE-APACHE](third-party/licenses/zerocopy-0.8.59/LICENSE-APACHE), [LICENSE-BSD](third-party/licenses/zerocopy-0.8.59/LICENSE-BSD), [LICENSE-MIT](third-party/licenses/zerocopy-0.8.59/LICENSE-MIT) |
| zerocopy-derive | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT | [LICENSE-APACHE](third-party/licenses/zerocopy-derive-0.8.59/LICENSE-APACHE), [LICENSE-BSD](third-party/licenses/zerocopy-derive-0.8.59/LICENSE-BSD), [LICENSE-MIT](third-party/licenses/zerocopy-derive-0.8.59/LICENSE-MIT) |
| chrono-tz | 0.10.4 | MIT OR Apache-2.0 | [LICENSE](third-party/licenses/chrono-tz-0.10.4/LICENSE), [LICENSE](third-party/licenses/chrono-tz-0.10.4/tz/LICENSE) |
| phf | 0.12.1 | MIT | [LICENSE](third-party/licenses/phf-0.12.1/LICENSE) |
| phf_shared | 0.12.1 | MIT | [LICENSE](third-party/licenses/phf_shared-0.12.1/LICENSE) |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | [COPYING](third-party/licenses/siphasher-1.0.4/COPYING), [LICENSE-APACHE](third-party/licenses/siphasher-1.0.4/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/siphasher-1.0.4/LICENSE-MIT) |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/sys-locale-0.3.2/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/sys-locale-0.3.2/LICENSE-MIT) |
| socket2 | 0.6.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/socket2-0.6.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/socket2-0.6.1/LICENSE-MIT) |
| windows-sys | 0.60.2 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-sys-0.60.2/license-apache-2.0), [license-mit](third-party/licenses/windows-sys-0.60.2/license-mit) |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-targets-0.53.5/license-apache-2.0), [license-mit](third-party/licenses/windows-targets-0.53.5/license-mit) |
| windows_aarch64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_aarch64_gnullvm-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_aarch64_gnullvm-0.53.1/license-mit) |
| windows_aarch64_msvc | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_aarch64_msvc-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_aarch64_msvc-0.53.1/license-mit) |
| windows_i686_gnu | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_i686_gnu-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_i686_gnu-0.53.1/license-mit) |
| windows_i686_gnullvm | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_i686_gnullvm-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_i686_gnullvm-0.53.1/license-mit) |
| windows_i686_msvc | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_i686_msvc-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_i686_msvc-0.53.1/license-mit) |
| windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_x86_64_gnu-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_x86_64_gnu-0.53.1/license-mit) |
| windows_x86_64_gnullvm | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_x86_64_gnullvm-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_x86_64_gnullvm-0.53.1/license-mit) |
| windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows_x86_64_msvc-0.53.1/license-apache-2.0), [license-mit](third-party/licenses/windows_x86_64_msvc-0.53.1/license-mit) |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/android_system_properties-0.1.6/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/android_system_properties-0.1.6/LICENSE-MIT) |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | [LICENSE-APACHE](third-party/licenses/autocfg-1.5.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/autocfg-1.5.1/LICENSE-MIT) |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/bumpalo-3.20.3/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/bumpalo-3.20.3/LICENSE-MIT) |
| cc | 1.4.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/cc-1.4.5/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/cc-1.4.5/LICENSE-MIT) |
| cfg-if | 1.0.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/cfg-if-1.0.4/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/cfg-if-1.0.4/LICENSE-MIT) |
| chrono | 0.4.45 | MIT OR Apache-2.0 | [LICENSE.txt](third-party/licenses/chrono-0.4.45/LICENSE.txt) |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/core-foundation-sys-0.8.7/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/core-foundation-sys-0.8.7/LICENSE-MIT) |
| find-msvc-tools | 0.1.12 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/find-msvc-tools-0.1.12/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/find-msvc-tools-0.1.12/LICENSE-MIT) |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/futures-core-0.3.34/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/futures-core-0.3.34/LICENSE-MIT) |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/futures-task-0.3.34/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/futures-task-0.3.34/LICENSE-MIT) |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/futures-util-0.3.34/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/futures-util-0.3.34/LICENSE-MIT) |
| iana-time-zone | 0.1.65 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/iana-time-zone-0.1.65/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/iana-time-zone-0.1.65/LICENSE-MIT) |
| iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/iana-time-zone-haiku-0.1.2/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/iana-time-zone-haiku-0.1.2/LICENSE-MIT) |
| itoa | 1.0.18 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/itoa-1.0.18/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/itoa-1.0.18/LICENSE-MIT) |
| js-sys | 0.3.105 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/js-sys-0.3.105/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/js-sys-0.3.105/LICENSE-MIT) |
| libc | 0.2.189 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/libc-0.2.189/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/libc-0.2.189/LICENSE-MIT) |
| libffi | 5.2.0 | MIT OR Apache-2.0 | [LICENSE-MIT](third-party/licenses/libffi-5.2.0/LICENSE-MIT), [LICENSE-APACHE](third-party/licenses/libffi-5.2.0/LICENSE-APACHE) |
| libffi-sys | 4.2.2 | MIT OR Apache-2.0 | [LICENSE-MIT](third-party/licenses/libffi-sys-4.2.2/LICENSE-MIT), [LICENSE-APACHE](third-party/licenses/libffi-sys-4.2.2/LICENSE-APACHE), [libffi-LICENSE](third-party/licenses/libffi-sys-4.2.2/libffi-LICENSE), [libffi-LICENSE-BUILDTOOLS](third-party/licenses/libffi-sys-4.2.2/libffi-LICENSE-BUILDTOOLS) |
| libloading | 0.8.9 | ISC | [LICENSE](third-party/licenses/libloading-0.8.9/LICENSE) |
| log | 0.4.34 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/log-0.4.34/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/log-0.4.34/LICENSE-MIT) |
| memchr | 2.8.3 | Unlicense OR MIT | [COPYING](third-party/licenses/memchr-2.8.3/COPYING), [LICENSE-MIT](third-party/licenses/memchr-2.8.3/LICENSE-MIT), [UNLICENSE](third-party/licenses/memchr-2.8.3/UNLICENSE) |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/num-traits-0.2.19/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/num-traits-0.2.19/LICENSE-MIT) |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/once_cell-1.21.4/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/once_cell-1.21.4/LICENSE-MIT) |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | [LICENSE-APACHE](third-party/licenses/pin-project-lite-0.2.17/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/pin-project-lite-0.2.17/LICENSE-MIT) |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/proc-macro2-1.0.107/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/proc-macro2-1.0.107/LICENSE-MIT) |
| quote | 1.0.47 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/quote-1.0.47/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/quote-1.0.47/LICENSE-MIT) |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/rustversion-1.0.23/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/rustversion-1.0.23/LICENSE-MIT) |
| serde | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde-1.0.229/LICENSE-MIT) |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_core-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_core-1.0.229/LICENSE-MIT) |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_derive-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_derive-1.0.229/LICENSE-MIT) |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_json-1.0.151/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_json-1.0.151/LICENSE-MIT) |
| shlex | 2.0.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/shlex-2.0.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/shlex-2.0.1/LICENSE-MIT) |
| slab | 0.4.12 | MIT | [LICENSE](third-party/licenses/slab-0.4.12/LICENSE) |
| syn | 2.0.119 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/syn-2.0.119/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/syn-2.0.119/LICENSE-MIT) |
| syn | 3.0.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/syn-3.0.5/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/syn-3.0.5/LICENSE-MIT) |
| unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | [LICENSE-APACHE](third-party/licenses/unicode-ident-1.0.24/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/unicode-ident-1.0.24/LICENSE-MIT), [LICENSE-UNICODE](third-party/licenses/unicode-ident-1.0.24/LICENSE-UNICODE) |
| unicode-segmentation | 1.12.0 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/unicode-segmentation-1.12.0/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/unicode-segmentation-1.12.0/LICENSE-MIT), [COPYRIGHT](third-party/licenses/unicode-segmentation-1.12.0/COPYRIGHT) |
| wasm-bindgen | 0.2.128 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/wasm-bindgen-0.2.128/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/wasm-bindgen-0.2.128/LICENSE-MIT) |
| wasm-bindgen-macro | 0.2.128 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/wasm-bindgen-macro-0.2.128/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/wasm-bindgen-macro-0.2.128/LICENSE-MIT) |
| wasm-bindgen-macro-support | 0.2.128 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/wasm-bindgen-macro-support-0.2.128/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/wasm-bindgen-macro-support-0.2.128/LICENSE-MIT) |
| wasm-bindgen-shared | 0.2.128 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/wasm-bindgen-shared-0.2.128/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/wasm-bindgen-shared-0.2.128/LICENSE-MIT) |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-core-0.62.2/license-apache-2.0), [license-mit](third-party/licenses/windows-core-0.62.2/license-mit) |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-implement-0.60.2/license-apache-2.0), [license-mit](third-party/licenses/windows-implement-0.60.2/license-mit) |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-interface-0.59.3/license-apache-2.0), [license-mit](third-party/licenses/windows-interface-0.59.3/license-mit) |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-link-0.2.1/license-apache-2.0), [license-mit](third-party/licenses/windows-link-0.2.1/license-mit) |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-result-0.4.1/license-apache-2.0), [license-mit](third-party/licenses/windows-result-0.4.1/license-mit) |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-strings-0.5.1/license-apache-2.0), [license-mit](third-party/licenses/windows-strings-0.5.1/license-mit) |
| zmij | 1.0.23 | MIT | [LICENSE-MIT](third-party/licenses/zmij-1.0.23/LICENSE-MIT) |

## Native libffi and build tooling

libffi-sys bundles native libffi. Its native LICENSE is preserved separately from
the Rust wrapper licenses, together with LICENSE-BUILDTOOLS. The latter identifies
separately licensed build/test utilities and includes GPLv2 terms. The native LICENSE
identifies libffi's own copyright and permission notice. Keep this distinction when
reviewing or redistributing dependency source; the Cargo crate license expression
alone does not describe every bundled file.

The source-preview archive contains these notice texts, not the bundled libffi source
or build utilities. Prebuilt binaries, dependency-vendored distributions and modified
dependency source need a distribution-specific notice/source review before release.
This inventory does not claim to audit every embedded third-party file individually.

## Provenance and maintenance

[The manifest](third-party/manifest.json) records each locked package, registry
checksum, notice origin and SHA-256 of the retained notice bytes. Most notices come
from the downloaded registry package. libffi and libffi-sys omit their Rust workspace
license files from their crate roots, so those two sets were retrieved from the
upstream commits recorded in each package's .cargo_vcs_info.json. URLs in the manifest
pin those exact commits; the native libffi notices come from the libffi-sys crate.

When Cargo.lock changes, review the full resolved package set with
`cargo metadata --locked --format-version 1`, update this inventory and manifest,
and preserve new or changed notices. Include target-specific and build dependencies.
The recorded registry checksums identify package artifacts; notice hashes check the
copied texts, not the complete provenance of neoCLR's source.

## Derived Unicode data

The generated scalar-category and simple-case-fold tables use Unicode data under
[Unicode License V3](third-party/unicode/LICENSE.txt). Case folding uses Unicode
17.0.0 CaseFolding.txt; its pinned source URL and SHA-256 are recorded in
`scripts/generate-string-case-folding.py` and `src/string_case_folding.rs`.

## Isolated AOT experiment

The optional source-only [scalar AOT tool](tools/aot-poc/Cargo.toml) has a separate
lockfile and [dependency inventory](tools/aot-poc/THIRD_PARTY_NOTICES.md), including
Cranelift 0.121.2 under Apache-2.0 WITH LLVM-exception. These compiler dependencies
are not added to the runtime distribution. The experiment's emitted scalar object
has no Cranelift or neoCLR runtime imports.
