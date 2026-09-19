# NeoCLR Strings & Encoding Proposal

**Implementation scope clarified 2026-09-19:** after the basic Introspection API,
review string handling and implement a minimal UTF-8-oriented surface. The encoding
API remains undecided. Specialized string classes, including Utf8String, are out
of scope for that step. The broader proposal below is preserved as design history;
its encoding-specific types are not a current implementation commitment. See
[the work sequence](../roadmap.md#immediate-sequence-clarified-2026-09-19).

The model has now converged on a fairly simple principle:

> **Unicode defines NeoCLR's text semantics. UTF-8 is NeoCLR's native and canonical text representation. `String` and `Char` remain the familiar programming abstractions, while encoding-specific types and `Encoding` expose representation when it matters.**

This keeps the everyday experience close to .NET while removing the historical coupling between `Char` and UTF-16.

## 1. `String` is the canonical text type

`String` is the normal immutable text container used throughout NeoCLR and Raven.

```raven
let text: String = "Hello 🌍"

text.Length
text[0]
text.Contains("Hello")
text.Substring(2)

for char in text {
    ...
}
```

Semantically:

```text
String = sequence of Char
```

Its contract is Unicode text, while its native storage representation is well-formed UTF-8:

```text
String
├── semantics: Unicode
├── elements: Char
└── native representation: UTF-8
```

A `String` does **not** carry an encoding:

```raven
text.Encoding       // no
String<Utf8>        // no
String<Utf16>       // no
```

There is one canonical `String` type.

---

## 2. `Char` is a Unicode scalar value

NeoCLR deliberately breaks the .NET relationship between `Char` and UTF-16.

```text
.NET                         NeoCLR

Char                         Char
 │                            │
UTF-16 code unit             Unicode scalar value
```

Consequently:

```raven
let c: Char = '😀'

let text = "A😀B"

text.Length // 3
text[1]     // '😀'
```

Surrogate values cannot exist as valid `Char` values.

How a `Char` is encoded is a separate concern:

```text
                 '😀'

                  Char
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
      UTF-8      UTF-16     UTF-32
     4 bytes     2 units     1 unit
```

So:

> **Unicode determines what a `Char` is. Encoding determines how that value is represented.**

---

## 3. `Char` has Unicode semantics

Classification belongs naturally on `Char`:

```raven
char.IsLetter
char.IsDigit
char.IsNumber
char.IsLetterOrDigit

char.IsWhitespace
char.IsUpper
char.IsLower

char.IsControl
char.IsPunctuation
char.IsSymbol
char.IsSeparator
char.IsMark

char.Category
```

Properties make simple classification ergonomic:

```raven
if char.IsLetter {
    ...
}
```

Full text transformations belong to `String`:

```raven
text.ToUpper()
text.ToLower()
text.Normalize(...)
```

because casing, normalization, grapheme handling and similar operations can involve multiple Unicode scalar values.

`Char` is therefore a **Unicode scalar**, not necessarily a complete user-perceived grapheme.

---

## 4. UTF-8 is a platform convention

UTF-8 isn't merely the default argument chosen independently by various APIs.

It is the **native NeoCLR text representation**.

Therefore NeoCLR-owned APIs use UTF-8 whenever they require an encoded representation and no other encoding has been specified.

For example:

```raven
File.WriteText(path, text)
```

means UTF-8 by default.

Likewise:

```raven
new StreamWriter(stream)
```

uses UTF-8.

Serialization APIs that produce textual bytes should likewise naturally prefer UTF-8 where the format permits it.

This establishes a platform-wide rule:

> **When NeoCLR defines the encoding convention, UTF-8 is the default. When an external protocol, file format or ABI defines the encoding, its contract takes precedence.**

So UTF-8 isn't forced onto external systems.

---

## 5. Representation-specific string types

Ordinary code should not need to reason about UTF-8 code units.

When it does, the programmer explicitly changes abstraction:

```raven
let utf8 = text.AsUtf8()
```

NeoCLR can provide representation-specific types such as:

```raven
Utf8String
Utf16String
AsciiString
```

Their meaningful elements correspond to their representation:

```text
String
└── Char

Utf8String
└── Byte / UTF-8 code unit

Utf16String
└── UInt16 / UTF-16 code unit

AsciiString
└── AsciiChar
```

Therefore:

```raven
let text = "A😀B"

text[1]          // Char('😀')
text.Length      // 3

let utf8 = text.AsUtf8()

utf8[1]          // UTF-8 code unit
utf8.Length      // 6
```

This gives `Length` a simple rule:

> **`Length` counts the elements exposed by that particular abstraction.**

---

## 6. `Utf8String` exposes native representation

Because `String` already has a canonical UTF-8 representation:

```raven
text.AsUtf8()
```

can normally produce a lightweight zero-copy view.

It can expose representation-oriented facilities:

```raven
utf8.Length
utf8[index]

utf8.Bytes
utf8.Slice(...)
utf8.IsAscii
```

This is preferable to putting:

```raven
text.Bytes
text.ByteLength
```

directly on ordinary `String`.

The abstraction boundary remains explicit:

```text
String
   │
   │ AsUtf8()
   ▼
Utf8String
   │
   ▼
UTF-8 code units / bytes
```

---

## 7. `AsciiString` represents a stronger invariant

ASCII is a Unicode subset and also maps directly into UTF-8.

`AsciiString` can therefore represent text known to satisfy the ASCII invariant:

```text
AsciiString
   │
   └── AsciiChar
```

`AsciiChar` can expose cheap, precise operations:

```raven
char.IsLetter
char.IsDigit
char.IsLetterOrDigit
char.IsHexDigit
char.IsUpper
char.IsLower
char.IsWhitespace
```

For `AsciiChar`, `IsDigit` means exactly `0`–`9`.

For ordinary Unicode `Char`, `IsDigit` follows Unicode semantics.

This makes `AsciiString` particularly useful for protocols, headers, tokens, identifiers and parsers.

---

## 8. `Encoding` converts text to and from bytes

`Encoding` is separate from representation-specific string types.

The distinction is:

```text
Utf8String
    "I am working with a valid UTF-8 representation."

Encoding.Utf8
    "I want to convert Unicode text to/from UTF-8."
```

NeoCLR can provide:

```raven
Encoding.Utf8
Encoding.Utf16
Encoding.Utf16BE
Encoding.Utf32

Encoding.Ascii
Encoding.Latin1
```

The basic model is:

```text
String ─── Encode ───► Bytes

String ◄── Decode ───── Bytes
```

For example:

```raven
let bytes = text.Encode(Encoding.Utf16)

let text = Encoding.Utf16.Decode(bytes)?
```

Whether `Encode` lives primarily on `String` or `Encoding` can be settled during detailed API design. The important part is that direct conversion remains ergonomic.

---

## 9. Views and encoding are deliberately different

These operations answer different questions.

```raven
text.AsUtf8()
```

means:

> Work with this String's UTF-8 representation.

```raven
text.Encode(Encoding.Utf8)
```

means:

> Produce binary data encoded as UTF-8.

```raven
Encoding.Utf8.Decode(bytes)
```

means:

> Interpret these bytes as UTF-8 and produce a `String`.

So conceptually:

```text
AsUtf8()       String → representation view

Encode()       String → Bytes

Decode()       Bytes → String
```

Because UTF-8 is native, `AsUtf8()` and UTF-8 encoding can have optimized paths without changing their semantic contracts.

---

## 10. `Option`, `Result`, and Fault follow NeoCLR conventions

String APIs don't retain .NET sentinel values simply for familiarity.

For example:

```raven
text.IndexOf("foo")
    -> Option<Int>
```

instead of `-1`.

Parsing becomes:

```raven
Int.Parse(text)
    -> Result<Int, ParseError>
```

rather than separate `Parse` and `TryParse` families.

Invalid encoded data is an expected failure:

```raven
Encoding.Utf8.Decode(bytes)
    -> Result<String, DecodeError>
```

Likewise, attempting to represent Unicode text in a restricted encoding can fail:

```raven
text.Encode(Encoding.Ascii)
    -> Result<Bytes, EncodeError>
```

Lossy conversion must be requested explicitly.

Unrecoverable runtime failures remain **Faults**.

---

## 11. Encoding is also an interop boundary property

A particularly important consequence is P/Invoke/native interoperability.

A native declaration can specify the representation expected at the boundary:

```raven
extern func NativeFunction(
    [NativeString(Utf16)] String name
)
```

The application still passes an ordinary `String`:

```raven
NativeFunction("Hello 🌍")
```

NeoCLR handles:

```text
String
native UTF-8 representation
        │
        │ boundary requires UTF-16
        ▼
    transcoding
        │
        ▼
native UTF-16 string
```

For an API accepting UTF-8:

```raven
[NativeString(Utf8)] String name
```

the runtime can potentially use the native representation directly, subject to ABI requirements such as ownership, lifetime and null termination.

The programmer can still take explicit control by working with `Utf8String`, `Utf16String`, etc. when required.

Thus:

> **Encoding at an interop boundary is a property of the boundary contract, not of the semantic `String` value.**

---

## 12. Higher Unicode concepts sit above the core model

NeoCLR doesn't need to pretend that one Unicode scalar always equals one visible character.

The layers are:

```text
encoded code units
       │
       ▼
Unicode scalar
       │
       ▼
grapheme cluster
```

NeoCLR defines:

```text
Char = Unicode scalar
```

Higher-level facilities can provide concepts such as:

```text
TextElement
Normalization
Case folding
Unicode categories
Grapheme segmentation
Collation
Culture-sensitive comparison
```

without complicating the fundamental `String` representation.

---

# Overall model

```text
                       TEXT

                      String
               immutable Unicode text
                         │
                  elements = Char
                         │
                         ▼
                       Char
                Unicode scalar value


                NATIVE REPRESENTATION

                       UTF-8
                         │
             NeoCLR platform convention
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
     Utf8String      Utf16String    AsciiString
          │              │              │
         Byte          UInt16        AsciiChar


                     ENCODING

                      String
                         │
                    Encode(...)
                         ▼
                       Bytes

                       Bytes
                         │
                    Decode(...)
                         ▼
                      String


                     INTEROP

                      String
                         │
                  boundary metadata
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        UTF-8          UTF-16         other
```

The proposal can ultimately be expressed in five rules:

1. **`String` is NeoCLR's normal Unicode text type.**
2. **`Char` is a Unicode scalar value, not an encoding code unit.**
3. **UTF-8 is NeoCLR's native/canonical encoded representation and the default for NeoCLR-owned text boundaries.**
4. **`Utf8String`, `Utf16String`, `AsciiString`, etc. expose representation-specific semantics when needed.**
5. **`Encoding` and interop projections determine the actual encoded representation when text crosses a boundary.**

That gives NeoCLR the familiar `String`/`Char` ergonomics of .NET while making UTF-8 a coherent **platform convention** rather than something individual libraries repeatedly have to choose as their own special-case default.
