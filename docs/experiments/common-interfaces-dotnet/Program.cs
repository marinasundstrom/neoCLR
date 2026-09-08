using System.Collections;

static void Check(bool ok) { if (!ok) throw new Exception("comparison failed"); }
Check(int.MinValue.CompareTo(int.MaxValue) < 0);
Check(uint.MaxValue.CompareTo(0u) > 0);
Check(double.NaN.CompareTo(0d) < 0);
Check(double.NaN.CompareTo(double.NaN) == 0);
Check((-0d).CompareTo(0d) == 0);
IEnumerable<int> values = new List<int> { 20, 22 };
using (var first = values.GetEnumerator())
using (var second = values.GetEnumerator()) {
    Check(first.MoveNext() && first.Current == 20);
    Check(second.MoveNext() && second.Current == 20);
    Check(first.MoveNext() && first.Current == 22);
    Check(!first.MoveNext() && !first.MoveNext());
}
var mutable = new List<int> { 1 };
using var iterator = mutable.GetEnumerator();
mutable[0] = 42;
try { iterator.MoveNext(); throw new Exception("expected version check"); }
catch (InvalidOperationException) { }
Console.WriteLine("Ordering, independent iteration, exhaustion and List mutation checks passed.");

var searched = new List<int> { 0, 42, 7 };
int calls = 0;
Check(searched.Find(value => { calls++; return value == 0; }) == 0 && calls == 1);
Check(searched.Find(value => value < 0) == 0); // Same result as a successful zero match.
Check(searched.FindIndex(value => value == 42) == 1);
Check(searched.FindIndex(value => value < 0) == -1);
Check(searched.Exists(value => value == 42));
IEquatable<Point> point = new Point(42);
Check(point.Equals(new Point(42)) && !point.Equals(new Point(7)));
Console.WriteLine("Typed equality and eager predicate search checks passed.");
readonly struct Point(int value) : IEquatable<Point> {
    private readonly int value = value;
    public bool Equals(Point other) => value == other.value;
}
