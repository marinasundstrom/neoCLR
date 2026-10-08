using System.Reflection.Emit;

var count = 0;
foreach (var (op, ordered, unordered) in new[] {
    (OpCodes.Ceq, 0, 0), (OpCodes.Cgt, 1, 0), (OpCodes.Clt, 0, 0),
    (OpCodes.Cgt_Un, 1, 1), (OpCodes.Clt_Un, 0, 1) })
{
    var method = new DynamicMethod("Compare", typeof(int), [typeof(double), typeof(double)]);
    var il = method.GetILGenerator();
    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldarg_1); il.Emit(op); il.Emit(OpCodes.Ret);
    var compare = method.CreateDelegate<Func<double, double, int>>();
    Check(compare(2, 1) == ordered);
    Check(compare(double.NaN, 1) == unordered);
    Check(compare(1, double.NaN) == unordered);
    Check(compare(double.NaN, double.NaN) == unordered);
}
foreach (var (op, expected) in new[] {
    (OpCodes.Beq, 0), (OpCodes.Bne_Un, 1), (OpCodes.Bgt, 0), (OpCodes.Blt, 0),
    (OpCodes.Bge, 0), (OpCodes.Ble, 0), (OpCodes.Bgt_Un, 1), (OpCodes.Blt_Un, 1),
    (OpCodes.Bge_Un, 1), (OpCodes.Ble_Un, 1) })
{
    var method = new DynamicMethod("Branch", typeof(int), [typeof(double), typeof(double)]);
    var il = method.GetILGenerator();
    var yes = il.DefineLabel();
    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldarg_1); il.Emit(op, yes);
    il.Emit(OpCodes.Ldc_I4_0); il.Emit(OpCodes.Ret);
    il.MarkLabel(yes); il.Emit(OpCodes.Ldc_I4_1); il.Emit(OpCodes.Ret);
    Check(method.CreateDelegate<Func<double, double, int>>()(double.NaN, 1) == expected);
}
foreach (var (op, left, right, expected) in new[] {
    (OpCodes.Add, 7.5, 2.0, 9.5), (OpCodes.Sub, 7.5, 2.0, 5.5),
    (OpCodes.Mul, Math.PI, 2.0, Math.Tau), (OpCodes.Div, 7.5, 2.0, 3.75),
    (OpCodes.Div, 1.0, -0.0, double.NegativeInfinity),
    (OpCodes.Mul, 1e308, 2.0, double.PositiveInfinity),
    (OpCodes.Div, 0.0, 0.0, double.NaN) })
{
    var method = new DynamicMethod("Arithmetic", typeof(double), [typeof(double), typeof(double)]);
    var il = method.GetILGenerator();
    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldarg_1); il.Emit(op); il.Emit(OpCodes.Ret);
    var result = method.CreateDelegate<Func<double, double, double>>()(left, right);
    Check(double.IsNaN(expected) ? double.IsNaN(result) : result == expected);
}
Check(0.0 == -0.0);
Console.WriteLine($"PASS {count} CLR floating instruction controls; {System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription}; {System.Runtime.InteropServices.RuntimeInformation.ProcessArchitecture}");
void Check(bool ok) { if (!ok) throw new Exception($"Control {count} failed"); count++; }
