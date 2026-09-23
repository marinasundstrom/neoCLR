static void Check(bool condition, string scenario)
{
    if (!condition) throw new Exception(scenario);
    Console.WriteLine(scenario);
}

var first = new Cell { Number = 1 };
var alias = first;
alias.Number = 2;
Check(first.Number == 2 && ReferenceEquals(first, alias), "Class assignment shares identity");
Check(!first.Equals(new Cell { Number = 2 }), "Object equality defaults to identity");
var copy = new Pair { Number = 1, Reference = first };
var other = copy;
other.Number = 9;
other.Reference.Number = 3;
Check(copy.Number == 1 && copy.Reference.Number == 3, "Value copy is shallow, not a deep clone");
object boxed = copy;
copy.Number = 7;
Check(((Pair)boxed).Number == 1 && boxed.GetType() == typeof(Pair), "Boxing copies a value and preserves its type");
object secondBox = (Pair)boxed;
Check(!ReferenceEquals(boxed, secondBox) && boxed.Equals(secondBox), "Separate boxes can be value-equal without sharing identity");
Check(boxed.GetHashCode() == secondBox.GetHashCode(), "Equal values have equal hashes; unequal hashes are not required");
Check(ReferenceEquals(null, null) && !ReferenceEquals(first, null), "Reference equality handles null");
Check(new Cell().ToString() == typeof(Cell).FullName, "Object formatting defaults to the runtime type name");
Console.WriteLine($"Runtime: {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}");

sealed class Cell { public int Number; }
struct Pair { public int Number; public Cell Reference; }
