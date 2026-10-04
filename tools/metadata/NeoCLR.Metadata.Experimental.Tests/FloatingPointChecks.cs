using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class FloatingPointChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("FloatingPoint", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var owner = graph.AddType("Example", "Numbers");
        var single = owner.AddMethod("Single", new(PrimitiveType.Single, [PrimitiveType.Single]));
        var il = single.GetILGenerator();
        il.LoadArgument(0); il.LoadConstant(2.0f); il.Multiply(); il.Return();
        var negativeZero = owner.AddMethod("NegativeZero", new(PrimitiveType.Double, []));
        il = negativeZero.GetILGenerator(); il.Emit(OpCode.Ldc_R8, -0.0); il.Return();
        var nan = owner.AddMethod("NaN", new(PrimitiveType.Double, []));
        il = nan.GetILGenerator(); il.LoadConstant(double.NaN); il.Return();
        var main = owner.AddMethod("Main", new(PrimitiveType.Int32, []));
        il = main.GetILGenerator();
        var failed = il.DefineLabel();
        il.Call(nan); il.Call(nan); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brtrue, failed);
        il.LoadConstant(1.0); il.Call(negativeZero); il.Divide();
        il.LoadConstant(double.NegativeInfinity); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
        il.LoadConstant(20.5f); il.Call(single);
        il.Emit(OpCode.Conv_R8); il.LoadConstant(1.0); il.Add(); il.Emit(OpCode.Conv_I4); il.Return();
        il.MarkLabel(failed); il.LoadConstant(1); il.Return();
        graph.EntryPoint = main;
        try { single.GetILGenerator().Emit(OpCode.Ldc_R8, 1.0f); throw new Exception("wrong float operand admitted"); } catch (ArgumentException) { }
        try { single.GetILGenerator().Emit(OpCode.Ldc_R4, 1.0); throw new Exception("wrong double operand admitted"); } catch (ArgumentException) { }
        var cli = graph.Write();
        var cliSnapshot = AssemblyDefinition.ReadAssembly(cli, expectedExtended: false);
        if (!cliSnapshot.MainModule.Types.Single(t => t.Name == "Numbers").Methods.Single(m => m.Name == "Single").TryGetStaticPrimitiveSignature(out var cliSignature)
            || cliSignature!.ParameterTypes.Single() != PrimitiveType.Single)
            throw new Exception("CLI floating signature round trip");
        var assembly = Assembly.Load(cli);
        var type = assembly.GetType("Example.Numbers")!;
        if ((int)type.GetMethod("Main")!.Invoke(null, null)! != 42) throw new Exception("floating CLI execution");
        if (BitConverter.DoubleToInt64Bits((double)type.GetMethod("NegativeZero")!.Invoke(null, null)!) != long.MinValue) throw new Exception("signed zero lost");
        if (!double.IsNaN((double)type.GetMethod("NaN")!.Invoke(null, null)!)) throw new Exception("NaN lost");
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        if (snapshot.MainModule.Types.Single().Methods.Single(m => m.Name == "Single").TryGetStaticPrimitiveSignature(out var signature) != true || signature!.ReturnType != PrimitiveType.Single)
            throw new Exception("native floating signature round trip");
        var bad = new AssemblyBuilder(new("BadFloat", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var invalid = bad.AddFunction("Invalid", new(PrimitiveType.Double, []));
        il = invalid.GetILGenerator(); il.LoadConstant(1.0); il.LoadConstant(2.0); il.BitwiseAnd(); il.Return();
        try { bad.Write(); throw new Exception("floating bitwise operation admitted"); } catch (InvalidDataException) { }
        if (Environment.GetEnvironmentVariable("NEOCLR_FLOAT_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
    }
}
