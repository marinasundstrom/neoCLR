using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json;

using var corpus = JsonDocument.Parse(File.ReadAllText(args[0]));
var outcomes = new List<object>();
foreach (var item in corpus.RootElement.EnumerateArray())
{
    var payload = item.GetString()!;
    var accepted = false;
    var text = "";
    try
    {
        using var parsed = JsonDocument.Parse(payload);
        if (parsed.RootElement.ValueKind == JsonValueKind.String)
        {
            text = parsed.RootElement.GetString()!;
            // The experiment's size limit is separate from .NET's JSON grammar.
            accepted = Encoding.UTF8.GetByteCount(payload) <= 128;
            var written = JsonSerializer.Serialize(text);
            if (JsonSerializer.Deserialize<string>(written) != text)
                throw new Exception(".NET round trip failed");
        }
    }
    catch (JsonException) { }
    catch (InvalidOperationException) { } // Invalid escaped surrogate at GetString.
    outcomes.Add(new { accepted, text = accepted ? text : "" });
}
Console.WriteLine(JsonSerializer.Serialize(outcomes));
