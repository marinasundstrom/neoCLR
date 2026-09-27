let text = "Ae\u{301}👨‍👩‍👧‍👦🇸🇪"
print("graphemes=\(text.count) scalars=\(text.unicodeScalars.count) utf8=\(text.utf8.count) utf16=\(text.utf16.count)")
print("canonical_equal=\("é" == "e\u{301}")")
print("prefix_inside_grapheme=\("e\u{301}".hasPrefix("e"))")
print("joined_graphemes=\(("e" + "\u{301}").count)")
