static void Check(bool value) { if (!value) throw new Exception("classification mismatch"); }
Check(char.IsDigit('7') && char.IsDigit('٧') && !char.IsAsciiDigit('٧'));
Check(!char.IsDigit('²') && char.IsNumber('²'));
Check(char.IsLetter('é') && char.IsLetter('中') && !char.IsLetter('\u0301'));
Check(char.IsLetter('\u01C5') && !char.IsUpper('\u01C5') && !char.IsLower('\u01C5'));
Check(char.IsWhiteSpace('\u0085') && char.IsWhiteSpace('\u00A0'));
Check(!char.IsWhiteSpace('\u200B') && !char.IsWhiteSpace('\uFEFF'));
Check(char.IsSurrogate('\uD800') && !char.IsLetter('\uD800'));
ulong hash = 14695981039346656037UL;
for (int i = 0; i <= 65535; i++) {
    hash ^= (byte)char.GetUnicodeCategory((char)i);
    hash = unchecked(hash * 1099511628211UL);
}
Console.WriteLine($"BMP category FNV-1a: {hash:X16}");
Console.WriteLine("Character classification checks passed.");
