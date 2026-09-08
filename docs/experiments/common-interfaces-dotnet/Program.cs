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
