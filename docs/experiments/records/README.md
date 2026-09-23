# Records and hash components

Development sample for a matching neoCLR bundle and experimental Raven compiler:

```sh
python3 verify.py --toolchain-root /path/to/bundle
```

Key and Pair use record syntax. Assignment shares class identity; separate records
can compare equal by their integer components. The sample checks typed, Object and Equatable
Equals, generated operators and hash agreement, display, deconstruction and mutable
HashCode value copies. Numeric hash values are not printed or treated as unique IDs.

The opt-in RuntimeRecordContract currently supports non-generic record classes
with Int32 components and Object as their direct base. Unsupported component types,
record structs and generic/inherited records report RAVT004. This is an initial
record implementation, not full Raven/.NET record parity. HashCode also has a
non-null string overload, independently tested; string record components are not
part of this compiler contract yet. No deep equality or implicit boxing is added.

The importer recognizes init-only property metadata, checks readonly-field writes,
and supports the integer output parameters used by generated Deconstruct. Property
initialization restrictions are checked by Raven; runtime reflection over application
properties and a persistent runtime init-only field flag are not provided here.
