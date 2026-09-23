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
var identityHash = first!.GetHashCode();
first.Number++;
GC.Collect(GC.MaxGeneration, GCCollectionMode.Forced, blocking: true, compacting: true);
Check(ReferenceEquals(first, alias) && alias.GetHashCode() == identityHash,
    "Default class identity and hash survive mutation and a requested compacting collection");
var array = new[] { 1, 2 };
Check(ReferenceEquals(array, (object)array) && !array.Equals(new[] { 1, 2 }),
    "Array Object views preserve identity; equal elements do not imply Object equality");
var text = new string(new[] { 's', 'a', 'm', 'e' });
object textView = text;
Check(ReferenceEquals(textView, (object)text) && ReferenceEquals(textView, (object)(string)textView),
    "String Object conversions preserve identity");
var sameText = new string(new[] { 's', 'a', 'm', 'e' });
Check(!ReferenceEquals(text, sameText) && text.Equals(sameText) && text.GetHashCode() == sameText.GetHashCode(),
    "Separate strings can be value-equal with equal hashes and distinct identity");
var key = new Key(42);
var equalKey = new Key(42);
Check(key.Equals(equalKey) && !ReferenceEquals(key, equalKey) && key.GetHashCode() == equalKey.GetHashCode(),
    "Custom equality and hash agree without changing reference identity");
Check(object.Equals(key, equalKey) && object.Equals(null, null) && !object.Equals(key, null),
    "Static Object equality handles null and dispatches the instance override");
var record = new KeyRecord(42);
var sameRecord = new KeyRecord(42);
Check(ReferenceEquals(record, record) && !ReferenceEquals(record, sameRecord)
    && record.Equals(sameRecord) && ((object)record).Equals(sameRecord)
    && record == sameRecord && record != new KeyRecord(7)
    && record.GetHashCode() == sameRecord.GetHashCode(),
    "Record syntax generates value equality and matching hashes while preserving class identity");
object integer = 42;
object equalInteger = 42;
Check(integer.Equals(equalInteger) && !ReferenceEquals(integer, equalInteger),
    "Boxed Int32 uses value equality without sharing identity");
Check(!integer.Equals(7) && !integer.Equals(42L) && !integer.Equals(null)
    && !integer.Equals("42"), "Boxed Int32 equality requires the same concrete type and value");
foreach (var value in new[] { int.MinValue, -1, 0, 1, int.MaxValue })
{
    object boxedInteger = value;
    Check(boxedInteger.GetHashCode() == value, $"Boxed Int32 hash matches its value: {value}");
}
Console.WriteLine($"Runtime: {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}");

sealed class Cell { public int Number; }
struct Pair { public int Number; public Cell Reference; }

sealed class Key(int number)
{
    public int Number { get; } = number;
    public override bool Equals(object? other) => other is Key key && key.Number == Number;
    public override int GetHashCode() => Number;
}

sealed record KeyRecord(int Number);
