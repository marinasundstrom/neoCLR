using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

// Semantic comparison only; this is not a cross-runtime performance benchmark.
var encoding = new UTF8Encoding(false, true);
var encoder = encoding.GetEncoder();
var chars = "é😀e\u0301".ToCharArray();
var output = new byte[4];
var bytes = new List<byte>();
var used = 0;
var completed = false;
while (!completed)
{
    encoder.Convert(chars, used, chars.Length - used, output, 0, output.Length,
        true, out var charsUsed, out var bytesUsed, out completed);
    if (charsUsed == 0 && bytesUsed == 0 && !completed)
        throw new Exception("No progress");
    used += charsUsed;
    bytes.AddRange(output.Take(bytesUsed));
}
if (used != 5 || bytes.Count != 9 || !bytes.SequenceEqual(encoding.GetBytes("é😀e\u0301")))
    throw new Exception("Unexpected conversion result");
var tooSmall = false;
try
{
    encoding.GetEncoder().Convert(new[] { 'é' }, 0, 1, new byte[1], 0, 1, true,
        out _, out _, out _);
}
catch (ArgumentException)
{
    tooSmall = true;
}
if (!tooSmall)
    throw new Exception("Expected scalar output capacity failure");
Console.WriteLine($".NET {Environment.Version}: Convert consumed 5 UTF-16 units and produced 9 UTF-8 bytes; one-byte é destination rejected");
