static void Check(bool value) { if (!value) throw new Exception("Math mismatch"); }
Check(Math.Sign(int.MinValue) == -1 && Math.Sign(0) == 0);
Check(Math.Min(int.MinValue, int.MaxValue) == int.MinValue);
Check(Math.Clamp(100, 0, 42) == 42);
try { Math.Clamp(0, 2, 1); throw new Exception("expected invalid bounds"); }
catch (ArgumentException) { }
Check(Math.Round(2.5) == 2 && Math.Round(3.5) == 4 && Math.Round(-2.5) == -2);
Check(BitConverter.DoubleToInt64Bits(Math.Round(-0.5)) == long.MinValue);
Check(BitConverter.DoubleToInt64Bits(Math.Abs(-0.0)) == 0);
foreach (var (x, y) in new[] { (0.0, -0.0), (-0.0, 0.0) }) {
    Check(BitConverter.DoubleToInt64Bits(Math.Min(x, y)) == long.MinValue);
    Check(BitConverter.DoubleToInt64Bits(Math.Max(x, y)) == 0);
}
Check(double.IsNaN(Math.Min(double.NaN, 1)) && double.IsNaN(Math.Max(1, double.NaN)));
Check(double.IsNaN(Math.Sqrt(-1)) && double.IsNaN(Math.Pow(-2, 0.5)));
Check(Math.Pow(double.NaN, 0) == 1 && Math.Pow(1, double.NaN) == 1);
Check(Math.Log(0) == double.NegativeInfinity && double.IsNaN(Math.Log(-1)));
Check(Math.Exp(1000) == double.PositiveInfinity);
Check(Math.Sqrt(25) == 5 && Math.Pow(2, 3) == 8);
Check(Math.Floor(-1.2) == -2 && Math.Ceiling(-1.2) == -1 && Math.Truncate(-1.2) == -1);
Check(Math.Sin(0) == 0 && Math.Cos(0) == 1 && Math.Tan(0) == 0 && Math.Log10(100) == 2);
Console.WriteLine("Integer, rounding, signed-zero, NaN and floating Math checks passed.");
