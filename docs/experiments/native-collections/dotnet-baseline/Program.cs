var pairs = new[] { new KeyValuePair<string, int>("key", 1), new KeyValuePair<string, int>("KEY", 2) };
foreach (var create in new Func<Dictionary<string, int>>[] {
    () => new(pairs, StringComparer.OrdinalIgnoreCase),
    () => pairs.ToDictionary(StringComparer.OrdinalIgnoreCase),
    () => pairs.ToDictionary(p => p.Key, p => p.Value, StringComparer.OrdinalIgnoreCase) }) {
    try { create(); throw new Exception("duplicate admitted"); }
    catch (ArgumentException) { Console.WriteLine("duplicate rejected"); }
}
var original = new Dictionary<string, List<int>> { ["key"] = new() { 1 } };
var copy = new Dictionary<string, List<int>>(original, StringComparer.Ordinal);
original["key"].Add(2);
original["later"] = new();
if (copy.Count != 1 || copy["key"].Count != 2) throw new Exception("shallow copy mismatch");
Console.WriteLine("shallow independent storage passed");
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);

var numbers = new[] { 1, 2, 3, 4 };
if (!new List<int>(numbers).SequenceEqual(numbers)) throw new Exception("list order");
if (!new Queue<int>(numbers).SequenceEqual(numbers)) throw new Exception("queue order");
var stack = new Stack<int>(numbers);
if (!stack.SequenceEqual(numbers.Reverse())) throw new Exception("stack order");
if (!new Stack<int>(stack).SequenceEqual(numbers)) throw new Exception("stack copy reversal");
if (new HashSet<string>(new[] { "key", "KEY" }, StringComparer.OrdinalIgnoreCase).Count != 1) throw new Exception("set duplicates");
Console.WriteLine("collection initialization order and set deduplication passed");
