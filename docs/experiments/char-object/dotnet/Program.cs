using System;
using System.Globalization;

object first = 'A';
object repeated = 'A';
if (!first.Equals(repeated) || ReferenceEquals(first, repeated) || first.Equals(65)
    || first.ToString() != "A" || first.GetHashCode() != repeated.GetHashCode())
    throw new Exception(".NET Char Object contract mismatch");
foreach (string text in new[] { "A", "é", "e\u0301", "👩‍💻" })
    if (StringInfo.ParseCombiningCharacters(text).Length != 1)
        throw new Exception("Expected one text element");
if ("👩‍💻".Length <= 1 || "e\u0301".Length <= 1 || string.Equals("é", "e\u0301", StringComparison.Ordinal))
    throw new Exception("UTF-16 and ordinal comparison baseline mismatch");
Console.WriteLine(".NET Char and text-element comparison: passed");
