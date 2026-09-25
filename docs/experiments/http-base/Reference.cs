using System;
using System.IO;
using System.Text.Json;

using var cases = JsonDocument.Parse(File.ReadAllText(args[0]));
foreach (var row in cases.RootElement.GetProperty("resolved").EnumerateArray())
{
    var baseUri = new Uri(row[0].GetString()!);
    var reference = row[1].GetString()!;
    var resolved = new Uri(baseUri, reference);
    // neoCLR preserves encoded dots under its lexical URI contract. .NET decodes
    // these unreserved characters and removes the resulting parent path segment.
    var expected = reference == "%2e%2e/item" ? "/item" : row[2].GetString();
    if (resolved.PathAndQuery != expected)
        throw new Exception($"Unexpected .NET URI resolution: {resolved}");
}
var configured = new Uri("http://example.test/api/");
if (new Uri(configured, "http://other.test/item").Host != "other.test"
    || new Uri(configured, "//other.test/item").Host != "other.test")
    throw new Exception("Expected .NET absolute/authority replacement behavior");
Console.WriteLine(".NET URI baseline passed; authority overrides and encoded-dot normalization differ explicitly");
