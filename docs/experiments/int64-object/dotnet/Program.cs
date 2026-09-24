using System;

foreach (long value in new[] { long.MinValue, -4294967296L, -1L, 0L, 1L, 4294967296L, 4294967297L, long.MaxValue })
{
    object first = value;
    object second = value;
    object collision = value ^ 0x0000000100000001L;
    int expected = unchecked((int)value) ^ (int)(value >> 32);
    if (!first.Equals(second) || ReferenceEquals(first, second)
        || first.Equals(collision) || first.GetHashCode() != collision.GetHashCode()
        || first.GetHashCode() != expected || first.Equals(null))
        throw new Exception("Int64 Object contract mismatch");
}
object one = 1L;
if (one.Equals(1) || one.Equals(true)) throw new Exception("Int64 equality must require Int64");
Console.WriteLine(".NET Int64 equality and hash baseline: passed");

foreach (object value in new object[] { int.MinValue, int.MaxValue, 0, long.MinValue, long.MaxValue, 4294967297L, true, false })
{
    string expected = value switch
    {
        int number => number.ToString(System.Globalization.CultureInfo.InvariantCulture),
        long number => number.ToString(System.Globalization.CultureInfo.InvariantCulture),
        bool flag => flag ? "True" : "False",
        _ => throw new Exception("Unexpected type")
    };
    if (value.ToString() != expected) throw new Exception("Object display mismatch");
}
Console.WriteLine(".NET primitive Object display baseline: passed");
