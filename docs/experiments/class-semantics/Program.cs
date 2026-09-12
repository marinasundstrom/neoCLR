using System.Text.Json;
var first = new Counter { Age = 1 };
var alias = first;
Change(alias);
var original = new Point { X = 1 };
var copy = original;
copy.X = 42;
Replace(first);
var afterValueParameter = first.Age;
ReplaceByRef(ref first);
Console.WriteLine(JsonSerializer.Serialize(new {
    SharedMutation = alias.Age,
    IndependentValue = original.X,
    ParameterRebinding = afterValueParameter,
    ByRefRebinding = first.Age
}, new JsonSerializerOptions { WriteIndented = true }));
static void Change(Counter value) => value.Age = 42;
static void Replace(Counter value) => value = new Counter { Age = 9 };
static void ReplaceByRef(ref Counter value) => value = new Counter { Age = 9 };
class Counter { public int Age; }
struct Point { public int X; }
