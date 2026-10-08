using System.Reflection.Emit;
static Func<object?, object?> Test(Type target) {
    var method = new DynamicMethod("Test", typeof(object), new[] { typeof(object) });
    var il = method.GetILGenerator();
    il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Isinst, target); il.Emit(OpCodes.Ret);
    return method.CreateDelegate<Func<object?, object?>>();
}
var values = new A[0];
var exact = Test(typeof(A[]));
var wrong = Test(typeof(B[]));
if (!ReferenceEquals(exact(values), values) || wrong(values) is not null || wrong(null) is not null
    || Test(typeof(bool[]))(new int[0]) is not null) return 1;
Console.WriteLine("exact identity, unrelated arrays, primitive arrays and null: passed");
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
return 0;
class A { }
class B { }
