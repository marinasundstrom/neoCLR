using System.Text.Json;

using var document = JsonDocument.Parse(File.ReadAllText(args[0]));
var data = document.RootElement;
var basis = new Uri(data.GetProperty("base").GetString()!);
Console.WriteLine($".NET {Environment.Version} URI comparison");
var differences = 0;
foreach (var item in data.GetProperty("resolution").EnumerateObject())
{
    try
    {
        var actual = new Uri(basis, item.Name).AbsoluteUri;
        if (actual != item.Value.GetString())
        {
            Console.WriteLine($"Resolution difference {JsonSerializer.Serialize(item.Name)}: {actual}");
            differences++;
        }
    }
    catch (UriFormatException)
    {
        Console.WriteLine($"Resolution rejected: {JsonSerializer.Serialize(item.Name)}");
        differences++;
    }
}
foreach (var relative in new[] { "g", "../g", "?y", "#s", "g/../h" })
    if (new Uri(basis, relative).AbsoluteUri != data.GetProperty("resolution").GetProperty(relative).GetString())
        throw new Exception("Common .NET resolution contract changed");
foreach (var item in data.GetProperty("invalid").EnumerateArray())
    if (Uri.TryCreate(item.GetString(), UriKind.RelativeOrAbsolute, out _))
        Console.WriteLine($".NET accepts text rejected by strict neoCLR grammar: {JsonSerializer.Serialize(item.GetString())}");
Console.WriteLine($"Recorded {differences} resolution differences; common contract checks passed");
