using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class IntegerArithmeticChecks
{
    internal static void Division() => Run(false);
    internal static void Remainder() => Run(true);

    private static void Run(bool remainder)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new(remainder ? "Remainder" : "Division", new Version(1, 0, 0, 0)), core);
        var type = graph.AddType("", "Arithmetic");
        var narrow = type.AddMethod("Narrow", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Int32]));
        narrow.LoadArgument(0);
        narrow.LoadArgument(1);
        if (remainder) narrow.Remainder(); else narrow.Divide();
        narrow.Return();
        var wide = type.AddMethod("Wide", new(PrimitiveType.Int64, [PrimitiveType.Int64, PrimitiveType.Int64]));
        wide.LoadArgument(0); wide.LoadArgument(1); wide.Emit(remainder ? OpCode.Rem : OpCode.Div); wide.Return();
        var loaded = Assembly.Load(graph.Write()).GetType("Arithmetic")!;
        foreach (var (left, right, quotient) in new[] { (43, 2, 21), (-43, 2, -21), (43, -2, -21), (-43, -2, 21), (1, 2, 0) })
        {
            var expected = remainder ? left % right : quotient;
            if (!Equals(loaded.GetMethod("Narrow")!.Invoke(null, [left, right]), expected) ||
                !Equals(loaded.GetMethod("Wide")!.Invoke(null, [(long)left, (long)right]), (long)expected)) throw new Exception("signed arithmetic result");
        }
        foreach (var name in new[] { "Narrow", "Wide" })
        {
            object one = name == "Narrow" ? (object)1 : 1L;
            object zero = name == "Narrow" ? (object)0 : 0L;
            object minimum = name == "Narrow" ? (object)int.MinValue : long.MinValue;
            object negativeOne = name == "Narrow" ? (object)(-1) : -1L;
            Fault<DivideByZeroException>([one, zero]); Fault<ArithmeticException>([minimum, negativeOne]);
            void Fault<T>(object[] values) where T : Exception
            {
                try { loaded.GetMethod(name)!.Invoke(null, values); throw new Exception("arithmetic did not fault"); }
                catch (TargetInvocationException error) when (error.InnerException is T) { }
            }
        }
        _ = graph.WriteNativeAssembly();
        var bad = type.AddMethod("Invalid");
        bad.LoadConstant(1); bad.Emit(remainder ? OpCode.Rem : OpCode.Div); bad.Return(); Reject();
        bad.ClearBody(); bad.LoadConstant(1); bad.Emit(OpCode.Ldc_I8, 2L); bad.Emit(remainder ? OpCode.Rem : OpCode.Div); bad.Return(); Reject();
        bad.ClearBody(); bad.Emit(OpCode.Ldc_Bool, true); bad.Emit(OpCode.Ldc_Bool, false); bad.Emit(remainder ? OpCode.Rem : OpCode.Div); bad.Return(); Reject();
        void Reject()
        {
            foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
            {
                try { write(); throw new Exception("invalid arithmetic accepted"); }
                catch (InvalidDataException) { }
            }
        }
    }
}
