# Source runtime friend attribute

Provider uses the real `System.Runtime.CompilerServices.InternalsVisibleToAttribute`
from a rebuilt System.Runtime library. Two assembly grants verify AllowMultiple;
assembly annotations precede the module declaration. Consumer constructs the attribute,
checks its read-only AssemblyName, then calls an internal provider method and returns 42.
The runtime attribute derives from qualified `System.Attribute`, as UnionAttribute does,
to select the source-owned base during the bootstrap build.

Run against matching, freshly built libraries and compiler:

```sh
python3 scripts/validate-runtime-friends.py \
  --compiler /path/to/rvnc.dll --libraries /path/to/native-libraries \
  --runtime /path/to/neoclr --aot /path/to/neoclr-aot-poc \
  --output /path/to/fresh-validation
```

The macOS ARM64 harness checks interpreted and compiled execution, unrelated-consumer
rejection (RAV0500, no output), and invalid class application (RAV0502, no output).
It writes command results and artifact hashes to validation.json. It uses the ordinary
project loader, explicit library context and AOT reference-arena profile. No reflection
root or execution of a metadata attribute constructor is required for friend access;
the consumer separately constructs an instance to test the public property contract.

The native identity profile remains unsigned/unqualified ASCII names. Windows execution,
full-key/quoted/Unicode identities and Utf8 internalization remain separate work. No
production runtime bundle pin changes. This is an integration fixture, not a new testing
framework convention. See validation.json for the local completed check.
