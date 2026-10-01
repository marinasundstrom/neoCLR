using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ShiftChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        foreach (var wide in new[] { false, true })
        foreach (var left in new[] { false, true })
        {
            var op = left ? OpCode.Shl : OpCode.Shr;
            var valueType = wide ? PrimitiveType.Int64 : PrimitiveType.Int32;
            var graph = new AssemblyBuilder(new("Shifts" + wide + left, new Version(1, 0, 0, 0)), core);
            var type = graph.AddType("", "Shifts");
            var method = type.AddMethod("Apply", new(valueType, [valueType, PrimitiveType.Int32]));
            method.LoadArgument(0); method.LoadArgument(1);
            if (wide) method.Emit(op);
            else if (left) method.ShiftLeft(); else method.ShiftRight();
            method.Return();
            var loaded = Assembly.Load(graph.Write()).GetType("Shifts")!.GetMethod("Apply")!;
            foreach (var value in new[] { -1L, 1L, long.MinValue, long.MaxValue, 4294967296L })
            foreach (var count in new[] { 0, 1, wide ? 63 : 31 })
            {
                object operand = wide ? value : (object)unchecked((int)value);
                object expected = wide ? (left ? value << count : value >> count) :
                    (object)(left ? unchecked((int)value) << count : unchecked((int)value) >> count);
                if (!Equals(loaded.Invoke(null, [operand, count]), expected)) throw new Exception("shift result/width");
            }
            _ = graph.WriteNativeAssembly();
            var invalid = type.AddMethod("Invalid", new(valueType, []));
            invalid.LoadConstant(1); invalid.Emit(op); invalid.Return(); Reject();
            invalid.ClearBody(); invalid.LoadConstant(1); invalid.Emit(OpCode.Ldc_I8, 1L); invalid.Emit(op); invalid.Return(); Reject();
            invalid.ClearBody(); invalid.Emit(OpCode.Ldc_Bool, true); invalid.LoadConstant(1); invalid.Emit(op); invalid.Return(); Reject();
            void Reject()
            {
                foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
                {
                    try { write(); throw new Exception("invalid shift accepted"); }
                    catch (InvalidDataException) { }
                }
            }
        }
    }
}
