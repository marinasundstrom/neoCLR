---
uid: T:System.ConsoleKeyInfo
---
A key value containing logical key identity, reported modifiers and optional Unicode
scalar text. TextScalar is not a grapheme or an allocated String.

Development equality compares all fields, including the active TextScalar case and
payload. Equal keys have equal hashes. None, Some(0) and uninitialized Optional storage
remain distinct. Explicit boxing still allocates a value box.

[Complete equality and hash member reference](/docs/console.html#key-value-equality)
includes both operator signatures. The pinned renderer currently gives `==` and `!=`
the same generated URL, so that manual entry is authoritative for their signatures.

The compiler supplies `EquatableTo<ConsoleKeyInfo>` automatically. Typed interface
calls use the generated equality method; the record uses ordinary value-type layout
and interface dispatch.
