using System.Text.Json;

if (args.Contains("--casing-integer")) {
    var text = new[] { "Straße ﬃ", "İıI", "ΟΣ", "ΟΣΑ", "AΣ́", "𐐀𐐨👩‍💻", "é é" };
    var integers = new[] { "9223372036854775807", "-9223372036854775808", "+42", "-0", " 1", "1 ", "１２", "9223372036854775808", "99999999999999999999999999999x" };
    Console.WriteLine(JsonSerializer.Serialize(new {
        runtime = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription,
        platform = System.Runtime.InteropServices.RuntimeInformation.OSDescription,
        casing = text.Select(value => new { input = value, lower = value.ToLowerInvariant(), upper = value.ToUpperInvariant() }),
        integers = integers.Select(value => {
            try { var parsed = long.Parse(value, System.Globalization.CultureInfo.InvariantCulture);
                return new { input = value, output = parsed.ToString(System.Globalization.CultureInfo.InvariantCulture), error = "" }; }
            catch (Exception error) { return new { input = value, output = "", error = error.GetType().Name }; }
        })
    }, new JsonSerializerOptions { WriteIndented = true }));
    return;
}

if (args.Contains("--ignore-case")) {
    var pairs = new (string, string)[] {
        ("A", "a"), ("Å", "å"), ("Σ", "ς"), ("ß", "ss"), ("ẞ", "ß"),
        ("K", "k"), ("ı", "I"), ("İ", "i"), ("ſ", "S"), ("𐐀", "𐐨"),
        ("é", "e\u0301"), ("\U00010000", "\uE000"), ("[", "A")
    };
    Console.WriteLine(JsonSerializer.Serialize(new {
        runtime = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription,
        platform = System.Runtime.InteropServices.RuntimeInformation.OSDescription,
        results = pairs.Select(pair => new {
            left = pair.Item1,
            right = pair.Item2,
            compare = Math.Sign(string.Compare(pair.Item1, pair.Item2, StringComparison.OrdinalIgnoreCase)),
            sameHash = StringComparer.OrdinalIgnoreCase.GetHashCode(pair.Item1)
                == StringComparer.OrdinalIgnoreCase.GetHashCode(pair.Item2)
        })
    }, new JsonSerializerOptions { WriteIndented = true }));
    return;
}

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
