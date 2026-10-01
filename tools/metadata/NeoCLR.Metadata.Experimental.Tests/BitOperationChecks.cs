using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class BitOperationChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        foreach (var op in new[] { OpCode.And, OpCode.Or, OpCode.Xor })
        {
            var graph = new AssemblyBuilder(new("Bits" + op, new Version(1, 0, 0, 0)), core);
            var type = graph.AddType("", "Bits");
            foreach (var wide in new[] { false, true })
            {
                var valueType = wide ? PrimitiveType.Int64 : PrimitiveType.Int32;
                var method = type.AddMethod(wide ? "Wide" : "Narrow", new(valueType, [valueType, valueType]));
                method.LoadArgument(0); method.LoadArgument(1);
                if (wide) method.Emit(op);
                else if (op == OpCode.And) method.BitwiseAnd();
                else if (op == OpCode.Or) method.BitwiseOr();
                else method.BitwiseXor();
                method.Return();
            }
            var loaded = Assembly.Load(graph.Write()).GetType("Bits")!;
            foreach (var (left, right) in new[] { (0L, 0L), (-1L, 42L), (long.MinValue, long.MaxValue), (4294967296L, 42L) })
            {
                var expected = op switch { OpCode.And => left & right, OpCode.Or => left | right, _ => left ^ right };
                if (!Equals(loaded.GetMethod("Wide")!.Invoke(null, [left, right]), expected) ||
                    !Equals(loaded.GetMethod("Narrow")!.Invoke(null, [unchecked((int)left), unchecked((int)right)]), unchecked((int)expected))) throw new Exception("bitwise result");
            }
            _ = graph.WriteNativeAssembly();
            var bad = type.AddMethod("Invalid");
            bad.LoadConstant(1); bad.Emit(op); bad.Return(); Reject();
            bad.ClearBody(); bad.LoadConstant(1); bad.Emit(OpCode.Ldc_I8, 2L); bad.Emit(op); bad.Return(); Reject();
            bad.ClearBody(); bad.Emit(OpCode.Ldc_Bool, true); bad.Emit(OpCode.Ldc_Bool, false); bad.Emit(op); bad.Return(); Reject();
            void Reject()
            {
                foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
                {
                    try { write(); throw new Exception("invalid bitwise operands accepted"); }
                    catch (InvalidDataException) { }
                }
            }
        }
    }
}
