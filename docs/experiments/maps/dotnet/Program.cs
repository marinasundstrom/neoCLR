// Observable .NET comparison, not an opcode or performance test.
var comparer = EqualityComparer<int>.Create((a, b) => a == b, _ => int.MinValue);
var map = new Dictionary<int, string>(comparer);
Console.WriteLine(map.TryAdd(42, "Pending"));
Console.WriteLine(map.TryAdd(42, "Duplicate"));
Console.WriteLine(map[42]);
Console.WriteLine(map.TryGetValue(99, out _));
var liveKeys = map.Keys;
map[42] = "Shipped";
map[99] = "Received";
Console.WriteLine(map[42]);
Console.WriteLine(liveKeys.Count);
for (var i = 0; i <= 20; i++) map[i] = "Stored";
Console.WriteLine(map.Count);
Console.WriteLine(map[0]);
Console.WriteLine(map[20]);
try { _ = new Dictionary<string, int>().TryAdd(null!, 1); }
catch (ArgumentNullException) { Console.WriteLine("Null key rejected"); }
