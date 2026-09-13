// API behavior comparison. Not an allocation or elapsed-time benchmark.
List<int> values = [0, 7, 42, 7];
Console.WriteLine(values.Find(v => v == 7));
Console.WriteLine(values.FindLast(v => v > 0));
Console.WriteLine(values.FindIndex(v => v == 7));
Console.WriteLine(values.FindLastIndex(v => v == 7));
Console.WriteLine(values.FindIndex(v => v < 0));
Console.WriteLine(values.Exists(v => v == 7));
Console.WriteLine(values.TrueForAll(v => v > 0));
var selected = values.FindAll(v => v > 0);
Console.WriteLine(string.Join(",", selected));
selected[0] = 99;
Console.WriteLine(values[1]);
Console.WriteLine(new List<int>().TrueForAll(_ => false));
Console.WriteLine(new List<int>().FindAll(_ => true).Count);
