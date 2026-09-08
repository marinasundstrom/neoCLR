using System;
using System.Reflection.Emit;
struct Counter { public int Value; public void Increment() => Value++; }
class Program {
    static int Observe(in Counter value) { value.Increment(); return value.Value; }
    static void Main() {
        Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
        var counter = new Counter();
        Console.WriteLine($"C# in: observed={Observe(in counter)}, original={counter.Value}");
        var method = new DynamicMethod("Mutate", typeof(void), new[] {typeof(Counter[])});
        var il = method.GetILGenerator();
        il.Emit(OpCodes.Ldarg_0); il.Emit(OpCodes.Ldc_I4_0);
        il.Emit(OpCodes.Readonly); il.Emit(OpCodes.Ldelema, typeof(Counter));
        il.Emit(OpCodes.Call, typeof(Counter).GetMethod(nameof(Counter.Increment)));
        il.Emit(OpCodes.Ret);
        var array = new Counter[1];
        ((Action<Counter[]>)method.CreateDelegate(typeof(Action<Counter[]>)))(array);
        Console.WriteLine($"CLR readonly. + mutator: original={array[0].Value}");
    }
}
