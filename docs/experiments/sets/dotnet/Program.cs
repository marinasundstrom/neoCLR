using System.Runtime.InteropServices;

static void Check(bool condition)
{
    if (!condition) throw new Exception("Set contract failed");
}
var storage = new HashSet<int>(new CollidingIntegers());
ISet<int> set = storage;
IReadOnlySet<int> view = storage;
for (var i = 0; i < 24; i++) Check(set.Add(i) && !set.Add(i));
Check(view.Count == 24 && !view.Contains(25));
Check(set.Remove(12) && !set.Remove(12) && !view.Contains(12));
for (var i = 0; i < 24; i++) Check(i == 12 || view.Contains(i));
Check(set.Add(12) && view.Sum() == 276);
set.Clear();
Check(view.Count == 0 && !set.Remove(0) && set.Add(42));
var words = new HashSet<string>(StringComparer.Ordinal);
Check(words.Add("Café") && !words.Add("Café") && words.Add("café") && words.Add("雪"));
Check(words.Count == 3 && words.Remove("Café") && words.Contains("café"));
words.Clear();
Check(words.Count == 0 && words.Add("雪"));
Console.WriteLine($"PASS: {RuntimeInformation.FrameworkDescription}; collision/growth, duplicates, membership, removal, clear/reuse, ordinal Unicode");

sealed class CollidingIntegers : IEqualityComparer<int>
{
    public bool Equals(int left, int right) => left == right;
    public int GetHashCode(int value) => 1;
}
