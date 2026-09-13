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
