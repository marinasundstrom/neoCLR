using System;
using System.Reflection;

[Flags] enum Access { None = 0, Read = 1, Write = 2, Both = 3, All = -1 }
class Program {
    static void Main() {
        var value = Access.Read | Access.Write;
        Console.WriteLine($"bits={(int)value}, zero={(int)default(Access)}, unknown={(int)(Access)42}");
        Console.WriteLine($"contains-zero={value.HasFlag(Access.None)}, read={value.HasFlag(Access.Read)}");
        Console.WriteLine(string.Join(",", typeof(Access).GetEnumNames()));
        Console.WriteLine($"underlying={typeof(Access).GetEnumUnderlyingType().Name}");
        Console.WriteLine($"binding={(int)(BindingFlags.Public | BindingFlags.Instance)}");
        Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
    }
}
