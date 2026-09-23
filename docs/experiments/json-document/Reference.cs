using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json;

// Fixture policy is explicit: .NET itself accepts duplicate names by default.
static bool Validate(JsonElement value, int depth, ref int nodes)
{
    if (++nodes > 32) return false;
    if (value.ValueKind == JsonValueKind.String) _ = value.GetString();
    if (value.ValueKind == JsonValueKind.Object)
    {
        if (depth >= 4) return false;
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (var field in value.EnumerateObject())
            if (!names.Add(field.Name) || !Validate(field.Value, depth + 1, ref nodes)) return false;
    }
    if (value.ValueKind == JsonValueKind.Array)
    {
        if (depth >= 4) return false;
        foreach (var item in value.EnumerateArray())
            if (!Validate(item, depth + 1, ref nodes)) return false;
    }
    return true;
}

using var corpus = JsonDocument.Parse(File.ReadAllText(args[0]));
var outcomes = new List<object>();
foreach (var item in corpus.RootElement.EnumerateArray())
{
    var input = item.GetString()!;
    var accepted = false;
    string? integer = null;
    try
    {
        using var document = JsonDocument.Parse(input);
        var nodes = 0;
        accepted = Encoding.UTF8.GetByteCount(input) <= 128 && Validate(document.RootElement, 0, ref nodes);
        if (accepted && document.RootElement.ValueKind == JsonValueKind.Number && document.RootElement.TryGetInt32(out var number))
            integer = number.ToString(System.Globalization.CultureInfo.InvariantCulture);
    }
    catch (JsonException) { }
    catch (InvalidOperationException) { }
    outcomes.Add(new { accepted, integer });
}
// Verify the baseline difference directly, not only the fixture's duplicate filter.
using var duplicates = JsonDocument.Parse("{\"a\":1,\"\\u0061\":2}");
if (duplicates.RootElement.GetProperty("a").GetInt32() != 2) throw new Exception("Duplicate baseline changed");
Console.WriteLine(JsonSerializer.Serialize(outcomes));
