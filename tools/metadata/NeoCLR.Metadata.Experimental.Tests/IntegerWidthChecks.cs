using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class IntegerWidthChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("IntegerWidths", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var owner = graph.AddType("Example", "Numbers");
        var storage = graph.AddValueType("Example", "Storage");
        var cases = new[] {
            (PrimitiveType.SByte, OpCode.Conv_I1, -128L), (PrimitiveType.Byte, OpCode.Conv_U1, 255L),
            (PrimitiveType.Int16, OpCode.Conv_I2, -32768L), (PrimitiveType.UInt16, OpCode.Conv_U2, 65535L),
            (PrimitiveType.UInt32, OpCode.Conv_U4, -1L), (PrimitiveType.UInt64, OpCode.Conv_U8, -1L)
        };
        var main = owner.AddMethod("Main", new(PrimitiveType.Int32, []));
        var il = main.GetILGenerator();
        var failed = il.DefineLabel();
        foreach (var (kind, conversion, bits) in cases)
        {
            storage.AddField(kind.ToString(), kind);
            var method = owner.AddMethod(kind.ToString(), new(kind, [kind]));
            var body = method.GetILGenerator();
            var local = body.DeclareLocal(kind);
            body.LoadArgument(0); body.StoreLocal(local); body.LoadLocal(local); body.Return();
            var getter = owner.AddMethod("get_" + kind, new(kind, []));
            body = getter.GetILGenerator();
            if (kind == PrimitiveType.UInt64) body.Emit(OpCode.Ldc_I8, bits); else body.LoadConstant((int)bits);
            body.Emit(conversion); body.Return();
            owner.AddProperty(kind.ToString(), kind, getter, null);
            il.Call(getter); il.Call(method);
            if (kind == PrimitiveType.UInt64) il.Emit(OpCode.Ldc_I8, bits); else il.LoadConstant((int)bits);
            il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
        }
        foreach (var wide in new[] { false, true })
        {
            void Constant(long value) { if (wide) il.Emit(OpCode.Ldc_I8, value); else il.LoadConstant((int)value); }
            void Equal(long value) { Constant(value); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed); }
            Constant(-1); Constant(2); il.Emit(OpCode.Div_Un); Equal(wide ? long.MaxValue : int.MaxValue);
            Constant(-1); Constant(2); il.Emit(OpCode.Rem_Un); Equal(1);
            Constant(-1); il.LoadConstant(1); il.Emit(OpCode.Shr_Un); Equal(wide ? long.MaxValue : int.MaxValue);
            Constant(-1); Constant(0); il.Emit(OpCode.Cgt_Un); il.Emit(OpCode.Brfalse, failed);
            Constant(0); Constant(-1); il.Emit(OpCode.Clt_Un); il.Emit(OpCode.Brfalse, failed);
            Constant(-1); il.Emit(OpCode.Conv_R_Un); il.LoadConstant(wide ? (double)ulong.MaxValue : uint.MaxValue);
            il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
        }
        il.LoadConstant(-1); il.Emit(OpCode.Conv_U8); il.Emit(OpCode.Ldc_I8, (long)uint.MaxValue);
        il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
        il.LoadConstant(42); il.Return();
        il.MarkLabel(failed); il.LoadConstant(1); il.Return();
        graph.EntryPoint = main;
        var cli = graph.Write();
        if ((int)Assembly.Load(cli).GetType("Example.Numbers")!.GetMethod("Main")!.Invoke(null, null)! != 42)
            throw new Exception("CLI integer width execution");
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, false), AssemblyDefinition.ReadNativeAssembly(native) })
        {
            var numbers = snapshot.MainModule.Types.Single(t => t.Name == "Numbers");
            var fields = snapshot.MainModule.Types.Single(t => t.Name == "Storage");
            foreach (var (kind, _, _) in cases)
            {
                if (!numbers.Methods.Single(m => m.Name == kind.ToString()).TryGetStaticPrimitiveSignature(out var signature)
                    || signature!.ReturnType != kind || signature.ParameterTypes.Single() != kind
                    || !fields.Fields.Single(f => f.Name == kind.ToString()).TryGetPrimitiveType(out var field) || field != kind
                    || !numbers.Properties.Single(p => p.Name == kind.ToString()).TryGetPrimitiveSignature(out var property, out var isStatic)
                    || property != kind || !isStatic)
                    throw new Exception("integer signature round trip: " + kind);
            }
        }
        foreach (var opcode in new[] { OpCode.Div_Un, OpCode.Rem_Un, OpCode.Shr_Un, OpCode.Conv_R_Un })
        {
            var bad = new AssemblyBuilder(new("Bad" + opcode, new Version(1, 0, 0, 0)), graph.CoreLibrary);
            var method = bad.AddFunction("Bad", new(PrimitiveType.Double, []));
            var body = method.GetILGenerator(); body.LoadConstant(1.0);
            if (opcode == OpCode.Shr_Un) body.LoadConstant(1);
            else if (opcode != OpCode.Conv_R_Un) body.LoadConstant(2.0);
            body.Emit(opcode); body.Return();
            try { bad.Write(); throw new Exception("floating unsigned operation admitted"); } catch (InvalidDataException) { }
        }
        // The primitive overload must accept all supported scalar locals, including floats.
        var floats = owner.AddMethod("Locals", new(PrimitiveType.Void, []));
        floats.GetILGenerator().DeclareLocal(PrimitiveType.Single);
        floats.GetILGenerator().DeclareLocal(PrimitiveType.Double);
        floats.GetILGenerator().Return();
        _ = graph.Write();
        if (Environment.GetEnvironmentVariable("NEOCLR_INTEGER_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
    }
}
