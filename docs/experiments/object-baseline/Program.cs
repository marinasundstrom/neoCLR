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
KeyRecord? absentRecord = null;
Check(!record.Equals(absentRecord) && !record.Equals(null)
    && !((IEquatable<KeyRecord>)record).Equals(absentRecord),
    "Typed record-class equality accepts nullable comparisons");
var nullability = new System.Reflection.NullabilityInfoContext();
Check(nullability.Create(typeof(KeyRecord).GetMethod("Equals", [typeof(KeyRecord)])!.GetParameters()[0]).ReadState
    == System.Reflection.NullabilityState.Nullable
    && nullability.Create(typeof(Coordinate).GetMethod("Equals", [typeof(Coordinate)])!.GetParameters()[0]).ReadState
    == System.Reflection.NullabilityState.NotNull,
    "Typed equality annotates record-class arguments but keeps record-struct values non-nullable");

KeyRecord? nullableRecord = sameRecord;
KeyRecord? differentRecord = new KeyRecord(7);
Check(nullableRecord == record && nullableRecord != differentRecord
    && nullableRecord != absentRecord && absentRecord != nullableRecord
    && absentRecord == null && null == absentRecord
    && nullableRecord != null && null != nullableRecord,
    "Nullable record operators preserve component equality and null symmetry");
Check(new[] { "op_Equality", "op_Inequality" }.All(name =>
    typeof(KeyRecord).GetMethod(name)!.GetParameters().All(parameter =>
        nullability.Create(parameter).ReadState == System.Reflection.NullabilityState.Nullable)
    && typeof(Coordinate).GetMethod(name)!.GetParameters().All(parameter =>
        nullability.Create(parameter).ReadState == System.Reflection.NullabilityState.NotNull)),
    "Record-class operators annotate both references while struct operators take values");

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
var point = new Coordinate(42, 7);
var copiedPoint = point;
object boxedPoint = point;
point.X = 99;
Check(copiedPoint.X == 42 && ((Coordinate)boxedPoint).X == 42, "Record struct assignment and boxing copy fields");
Check(boxedPoint.Equals(copiedPoint) && !boxedPoint.Equals(new Coordinate(7, 42))
    && !boxedPoint.Equals(null) && !boxedPoint.Equals(new KeyRecord(42)), "Record struct Object equality requires exact type and equal fields");
Check(boxedPoint.GetHashCode() == copiedPoint.GetHashCode()
    && ((IEquatable<Coordinate>)boxedPoint).Equals(copiedPoint), "Boxed record struct hash and interface equality agree");
Check(default(Coordinate) == new Coordinate(0, 0), "Record struct default initializes value fields");
var corner = new Coordinate(1, 2);
var bounds = new Rectangle(corner, new Coordinate(3, 4));
var equalBounds = new Rectangle(new Coordinate(1, 2), new Coordinate(3, 4));
corner.X = 99;
var (start, end) = bounds;
start.X = 77;
Check(bounds == equalBounds && bounds.Start.X == 1 && end.Y == 4,
    "Nested record structs copy constructor and deconstruction values");
Check(((object)bounds).Equals(equalBounds) && ((IEquatable<Rectangle>)bounds).Equals(equalBounds)
    && bounds.GetHashCode() == equalBounds.GetHashCode()
    && bounds.ToString() == "Rectangle { Start = Coordinate { X = 1, Y = 2 }, End = Coordinate { X = 3, Y = 4 } }",
    "Nested record struct equality, hash and display use component semantics");
var defaultText = default(TextRecord);
var otherDefaultText = default(TextRecord);
Check(defaultText.Text is null && defaultText == otherDefaultText && defaultText != new TextRecord(""),
    "Non-nullable string annotations do not change struct zero initialization");
Check(defaultText.GetHashCode() == otherDefaultText.GetHashCode()
    && defaultText.ToString() == "TextRecord { Text =  }",
    "Default string components hash and display without dereferencing null");
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

record struct Coordinate(int X, int Y);

record struct Rectangle(Coordinate Start, Coordinate End);

record struct TextRecord(string Text);
