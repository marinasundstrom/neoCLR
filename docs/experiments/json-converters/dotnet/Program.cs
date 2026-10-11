using System.Text.Json;

Reading number = 42;
Reading text = "hello";
foreach (var value in new[] { number, text, default(Reading) })
{
    var json = JsonSerializer.Serialize(value);
    var copy = JsonSerializer.Deserialize<Reading>(json);
    if (JsonSerializer.Serialize(copy) != json) throw new Exception("Round trip mismatch");
    Console.WriteLine(json);
}
try
{
    JsonSerializer.Deserialize<Ambiguous>("1");
    throw new Exception("Ambiguous numeric cases were accepted");
}
catch (JsonException) { Console.WriteLine("ambiguous"); }
try
{
    JsonSerializer.Deserialize<Reading>("true");
    throw new Exception("Unknown shape was accepted");
}
catch (JsonException) { Console.WriteLine("unknown-shape"); }

public union Reading(int, string);
public union Ambiguous(int, long);
