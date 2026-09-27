using System.Globalization;
using System.Text;
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
var text = "Ae\u0301👨‍👩‍👧‍👦🇸🇪";
Console.WriteLine($"graphemes={new StringInfo(text).LengthInTextElements} scalars={text.EnumerateRunes().Count()} utf8={Encoding.UTF8.GetByteCount(text)} utf16={text.Length}");
Console.WriteLine($"ordinal_canonical_equal={String.Equals("é", "e\u0301", StringComparison.Ordinal)}");
Console.WriteLine($"prefix_inside_grapheme={"e\u0301".StartsWith("e", StringComparison.Ordinal)}");
Console.WriteLine($"ordinal_ignore_case_kelvin={String.Equals("K", "k", StringComparison.OrdinalIgnoreCase)}");
