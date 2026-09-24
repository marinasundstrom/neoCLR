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
