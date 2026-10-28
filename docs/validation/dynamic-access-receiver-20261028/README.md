# Descriptor receiver requirements — 2026-10-28

The [native consumer](../../experiments/dynamic-access/receiver/Main.rvn) calls
GetConstructors/GetProperties directly on `typeof(Message)`. The runtime API receiver
annotations infer flags 515 for source row 6. No user annotation or retention file
is supplied. Execution exits 0 and prints `1` then `1`. Only the existing DAM2004
dynamic-implementation warning remains; no unknown receiver or contract mismatch warns.

33 focused analyzer tests pass, including known/unknown receiver values, propagation
through self, invalid static placement, interface mismatch and inherited descriptor
override mismatch. Five runtime libraries rebuild; native rendering, API fingerprint
checks and four snapshot tests pass. The old storage-only backend rejects the new
method placement with DAM1001. [Results and hashes](results.json) retain commands.

Tested bundle: `target/dynamic-access-20261028/bundle-receiver`; backend:
`receiver-codegen`; executable: `receiver-native/app` under that target directory.
The compiler is the previously validated return/static-field metadata build; no new
compiler emission rule is involved. Existing Runtime Contract settings remain.
BindingFlags overloads, generic annotations and broader member categories are still
outside this slice. No performance or deployment claim is made.

```sh
cargo test -q --lib dynamic_access::tests
python3 scripts/build-api-docs.py --check
python3 scripts/test_native_api_snapshot.py
```
