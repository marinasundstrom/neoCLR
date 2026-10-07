# Experiments

[Documentation index](../README.md)

Executable probes and local validation support design decisions; their presence
does not imply a shipped runtime capability. Read each experiment's status and
reproduction instructions before using its output.

- [ARM64 AOT Hello World: first standalone executable](aot-hello/README.md)
- [AOT flat value types/members: Raven samples and native validation](aot-values/README.md)
- [ARM64 scalar AOT: native object and C consumer](aot-scalar/README.md)
- [JSON document: a sensor report and acknowledgement](json-document/README.md)
- [JSON message: a bounded string round trip](json-message/README.md)
- [UTF-8 chunks: stateful text over partial byte reads](utf8-chunks/README.md)
- [Byte Copy: managed storage and partial transfers](byte-copy/README.md)
- [External I/O progress: host-side M1/S0 probe](external-io-progress/README.md)
- [Raven target and tooling](raven-target/README.md)
- [CIL decoder](../cil-decoder.md)
- [PE reader](../pe-reader.md)
- [Class semantics](../class-semantics.md)

Generated `bin`, `obj` and local toolchain outputs are not documentation sources.

- [Native Raven union app](aot-union/README.md): Some/None patterns and copies compiled
  to standalone ARM64 under explicit closed-world selection; next-sample reassessment.
