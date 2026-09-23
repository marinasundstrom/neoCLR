using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json;

// Compare decoded text, not .NET UTF-16 char counts with neoCLR grapheme counts.
using var corpus = JsonDocument.Parse(File.ReadAllText(args[0]));
var outcomes = new List<object>();
foreach (var item in corpus.RootElement.EnumerateArray())
{
    var data = Convert.FromHexString(item.GetProperty("hex").GetString()!);
    var decoder = new UTF8Encoding(false, true).GetDecoder();
    var output = new StringBuilder();
    var accepted = true;
    var start = 0;
    try
    {
        foreach (var stop in item.GetProperty("stops").EnumerateArray())
        {
            var end = stop.GetInt32();
            var chars = new char[data.Length + 4];
            var count = decoder.GetChars(data, start, end - start, chars, 0, false);
            output.Append(chars, 0, count);
            start = end;
        }
        var finalChars = new char[data.Length + 4];
        var finalCount = decoder.GetChars(data, start, data.Length - start, finalChars, 0, true);
        output.Append(finalChars, 0, finalCount);
    }
    catch (DecoderFallbackException)
    {
        accepted = false;
    }
    outcomes.Add(new { accepted, text = accepted ? output.ToString() : "" });
}
Console.WriteLine(JsonSerializer.Serialize(outcomes));
