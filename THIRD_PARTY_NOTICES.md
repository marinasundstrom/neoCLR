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
| cc | 1.4.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/cc-1.4.5/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/cc-1.4.5/LICENSE-MIT) |
| cfg-if | 1.0.4 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/cfg-if-1.0.4/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/cfg-if-1.0.4/LICENSE-MIT) |
| find-msvc-tools | 0.1.12 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/find-msvc-tools-0.1.12/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/find-msvc-tools-0.1.12/LICENSE-MIT) |
| itoa | 1.0.18 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/itoa-1.0.18/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/itoa-1.0.18/LICENSE-MIT) |
| libc | 0.2.189 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/libc-0.2.189/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/libc-0.2.189/LICENSE-MIT) |
| libffi | 5.2.0 | MIT OR Apache-2.0 | [LICENSE-MIT](third-party/licenses/libffi-5.2.0/LICENSE-MIT), [LICENSE-APACHE](third-party/licenses/libffi-5.2.0/LICENSE-APACHE) |
| libffi-sys | 4.2.2 | MIT OR Apache-2.0 | [LICENSE-MIT](third-party/licenses/libffi-sys-4.2.2/LICENSE-MIT), [LICENSE-APACHE](third-party/licenses/libffi-sys-4.2.2/LICENSE-APACHE), [libffi-LICENSE](third-party/licenses/libffi-sys-4.2.2/libffi-LICENSE), [libffi-LICENSE-BUILDTOOLS](third-party/licenses/libffi-sys-4.2.2/libffi-LICENSE-BUILDTOOLS) |
| libloading | 0.8.9 | ISC | [LICENSE](third-party/licenses/libloading-0.8.9/LICENSE) |
| memchr | 2.8.3 | Unlicense OR MIT | [COPYING](third-party/licenses/memchr-2.8.3/COPYING), [LICENSE-MIT](third-party/licenses/memchr-2.8.3/LICENSE-MIT), [UNLICENSE](third-party/licenses/memchr-2.8.3/UNLICENSE) |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/proc-macro2-1.0.107/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/proc-macro2-1.0.107/LICENSE-MIT) |
| quote | 1.0.47 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/quote-1.0.47/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/quote-1.0.47/LICENSE-MIT) |
| serde | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde-1.0.229/LICENSE-MIT) |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_core-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_core-1.0.229/LICENSE-MIT) |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_derive-1.0.229/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_derive-1.0.229/LICENSE-MIT) |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/serde_json-1.0.151/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/serde_json-1.0.151/LICENSE-MIT) |
| shlex | 2.0.1 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/shlex-2.0.1/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/shlex-2.0.1/LICENSE-MIT) |
| syn | 3.0.5 | MIT OR Apache-2.0 | [LICENSE-APACHE](third-party/licenses/syn-3.0.5/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/syn-3.0.5/LICENSE-MIT) |
| unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | [LICENSE-APACHE](third-party/licenses/unicode-ident-1.0.24/LICENSE-APACHE), [LICENSE-MIT](third-party/licenses/unicode-ident-1.0.24/LICENSE-MIT), [LICENSE-UNICODE](third-party/licenses/unicode-ident-1.0.24/LICENSE-UNICODE) |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | [license-apache-2.0](third-party/licenses/windows-link-0.2.1/license-apache-2.0), [license-mit](third-party/licenses/windows-link-0.2.1/license-mit) |
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
