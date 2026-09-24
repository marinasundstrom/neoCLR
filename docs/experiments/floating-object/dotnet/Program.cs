using System;
using System.Collections.Generic;

foreach (object[] pair in new[] {
    new object[] { 0.0f, -0.0f }, new object[] { 0.0, -0.0 },
    new object[] { float.NaN, BitConverter.Int32BitsToSingle(unchecked((int)0xffc01234)) },
    new object[] { double.NaN, BitConverter.Int64BitsToDouble(unchecked((long)0xfff8000000001234)) }
}) {
    if (!pair[0].Equals(pair[1]) || pair[0].GetHashCode() != pair[1].GetHashCode())
        throw new Exception("Floating Object equivalence mismatch");
    var keys = new Dictionary<object, int> { [pair[0]] = 1 };
    if (keys[pair[1]] != 1 || keys.TryAdd(pair[1], 2)) throw new Exception("Duplicate key mismatch");
}
foreach (var pair in new (object value, int hash)[] {
    (float.NaN, 0x7f800000), (double.NaN, 0x7ff00000),
    (1.5f, 0x3fc00000), (1.5, 0x3ff80000), (float.Epsilon, 1), (double.Epsilon, 1)
}) if (pair.value.GetHashCode() != pair.hash) throw new Exception("Floating hash mismatch");
if (((object)1.0f).Equals(1.0) || ((object)1.0).Equals(1)) throw new Exception("Exact type required");
Console.WriteLine(".NET floating Object baseline: passed");
