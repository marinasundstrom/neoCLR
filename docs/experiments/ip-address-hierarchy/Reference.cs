using System.Net;
using System.Text.Json;

using var cases = JsonDocument.Parse(File.ReadAllText(args[0]));
foreach (var item in cases.RootElement.GetProperty("valid").EnumerateObject())
{
    if (!IPAddress.TryParse(item.Name, out var address))
        throw new Exception(".NET rejected shared valid input: " + item.Name);
    if (!address.Equals(IPAddress.Parse(item.Value.GetString()!)))
        throw new Exception("Canonical form changed address bytes: " + item.Name);
    if (!address.Equals(IPAddress.Parse(address.ToString())))
        throw new Exception(".NET round-trip failed: " + item.Name);
}
// Explicitly record intentional lexical-policy differences from .NET 10.
var acceptedByDotNet = new HashSet<string> {
    "127.1", "0xffffffff", "1.1.1.010", "[::1]", "fe80::1%3"
};
foreach (var item in cases.RootElement.GetProperty("invalid").EnumerateArray())
{
    var text = item.GetString()!;
    if (IPAddress.TryParse(text, out _) != acceptedByDotNet.Contains(text))
        throw new Exception("Unexpected .NET acceptance: " + JsonSerializer.Serialize(text));
}
Console.WriteLine(".NET address parsing comparison passed");
