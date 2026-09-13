// .NET behavior comparison; not a timing benchmark.
int[] empty = [], one = [0], many = [0, 7, 42];
Console.WriteLine(empty.FirstOrDefault());
Console.WriteLine(one.FirstOrDefault());
Console.WriteLine(many.First());
Console.WriteLine(many.Last());
Console.WriteLine(one.Single());
foreach (var values in new[] { empty, many })
{
    try { Console.WriteLine(values.Single()); }
    catch (InvalidOperationException) { Console.WriteLine("Cardinality exception"); }
}
foreach (var operation in new Func<int>[] { () => empty.First(), () => empty.Last() })
{
    try { Console.WriteLine(operation()); }
    catch (InvalidOperationException) { Console.WriteLine("Empty exception"); }
}

Console.WriteLine(many.First(value => value == 0));
Console.WriteLine(many.Last(value => value == 0));
Console.WriteLine(many.Single(value => value == 42));
foreach (var operation in new Func<int>[] {
    () => many.First(value => value > 100),
    () => many.Last(value => value > 100),
    () => many.Single(value => value > 100),
    () => many.Single(value => value >= 0)
})
{
    try { Console.WriteLine(operation()); }
    catch (InvalidOperationException) { Console.WriteLine("Matching cardinality exception"); }
}
var calls = 0;
Console.WriteLine(many.ToList().Last(value => { calls++; return value > 0; }));
Console.WriteLine($"List Last predicate calls: {calls}");
IEnumerable<int> Forward() { foreach (var value in many) yield return value; }
calls = 0;
Console.WriteLine(Forward().Last(value => { calls++; return value > 0; }));
Console.WriteLine($"Iterable Last predicate calls: {calls}");
