static void Check(bool value) { if (!value) throw new Exception("ordinal mismatch"); }
foreach (var (left, right, sign) in new[] {
    ("", "", 0), ("", "x", -1), ("x", "", 1), ("A", "a", -1),
    ("é", "e\u0301", 1), ("\U00010000", "\uE000", -1),
    ("🌍", "🌎", -1), ("a\0", "a", 1), ("abc", "abcd", -1)
}) Check(Math.Sign(string.CompareOrdinal(left, right)) == sign);
foreach (var (text, pattern, contains, starts, ends) in new[] {
    ("", "", true, true, true), ("abc", "", true, true, true),
    ("", "x", false, false, false), ("café🌍", "é", true, false, false),
    ("café🌍", "café", true, true, false), ("café🌍", "🌍", true, false, true),
    ("a\0b", "\0", true, false, false), ("é", "e\u0301", false, false, false),
    ("e\u0301", "\u0301", true, false, true), ("Neo", "neo", false, false, false),
    ("x", "xx", false, false, false)
}) {
    Check(text.Contains(pattern, StringComparison.Ordinal) == contains);
    Check(text.StartsWith(pattern, StringComparison.Ordinal) == starts);
    Check(text.EndsWith(pattern, StringComparison.Ordinal) == ends);
}
Console.WriteLine("Ordinal ordering and matching checks passed.");
