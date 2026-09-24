using System;
using System.Collections.Generic;

foreach (string text in new[] { "", "A", "hello 👩‍💻", "e\u0301", "a\0b" })
{
    object left = new string(text.ToCharArray());
    object right = new string(text.ToCharArray());
    if (!left.Equals(right) || left.GetHashCode() != right.GetHashCode()
        || left.ToString() != text || left.Equals(null) || left.Equals(42))
        throw new Exception("String Object contract mismatch");
    var map = new Dictionary<object, int> { [left] = 1 };
    if (map[right] != 1) throw new Exception("String key mismatch");
}
if (((object)"é").Equals("e\u0301") || ((object)"A").Equals('A'))
    throw new Exception("Exact content/type required");
Console.WriteLine(".NET String Object content baseline: passed");
