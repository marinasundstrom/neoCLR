using System.Text;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class EmitChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("OpcodeConsumer", new Version(1, 0, 0, 0)), core);
        var helper = graph.AddFunction("Compute", 2);
        var main = graph.AddType("Example", "Program").AddMethod("Main"); graph.EntryPoint = main;
        helper.LoadArgument(0); helper.LoadArgument(1); helper.Add(); helper.LoadConstant(3); helper.Multiply(); helper.LoadConstant(0); helper.Subtract(); helper.Return();
        main.LoadConstant(10); main.LoadConstant(4); main.Call(helper); main.Return();
        var expectedPe = graph.Write(); var expectedNative = graph.WriteNativeAssembly();
        helper.ClearBody(); main.ClearBody();
        helper.Emit(OpCode.Ldarg, 0); helper.Emit(OpCode.Ldarg, 1); helper.Emit(OpCode.Add);
        helper.Emit(OpCode.Ldc_I4, 3); helper.Emit(OpCode.Mul); helper.Emit(OpCode.Ldc_I4, 0); helper.Emit(OpCode.Sub); helper.Emit(OpCode.Ret);
        main.Emit(OpCode.Ldc_I4, 10); main.Emit(OpCode.Ldc_I4, 4); main.Emit(OpCode.Call, helper); main.Emit(OpCode.Ret);
        Check(graph.Write().SequenceEqual(expectedPe) && graph.WriteNativeAssembly().SequenceEqual(expectedNative), "helper and raw emission differ");
        Check((int)System.Reflection.Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null)! == 42, "raw CLI execution");
        foreach (var action in new Action[] {
            () => main.Emit(OpCode.Call), () => main.Emit(OpCode.Ldarg), () => main.Emit(OpCode.Add, 1),
            () => main.Emit(OpCode.Call, 42), () => main.Emit(OpCode.Ret, helper),
            () => main.Emit((OpCode)999), () => main.Emit((OpCode)999, 1), () => main.Emit((OpCode)999, helper),
            () => main.Emit(OpCode.Call, (MethodBuilder)null!)
        })
        {
            Reject<ArgumentException>(action);
            Check(graph.WriteNativeAssembly().SequenceEqual(expectedNative), "rejected instruction mutated body");
        }
        var dependency = new AssemblyBuilder(new("OpcodeDependency", new Version(1, 0, 0, 0)), core);
        var target = dependency.AddType("Example", "Library").AddMethod("Value"); target.LoadConstant(42); target.Return();
        var snapshot = AssemblyDefinition.ReadAssembly(dependency.Write(), false);
        var definition = snapshot.MainModule.Types.Single(t => t.Name == "Library").Methods.Single();
        var imported = graph.ImportReference(definition, core);
        main.ClearBody(); main.Call(imported); main.Return();
        var importedPe = graph.Write(); var importedNative = graph.WriteNativeAssembly();
        main.ClearBody(); main.Emit(OpCode.Call, imported); main.Emit(OpCode.Ret);
        Check(graph.Write().SequenceEqual(importedPe) && graph.WriteNativeAssembly().SequenceEqual(importedNative), "imported operand encoding");
        Reject<ArgumentException>(() => main.Emit(OpCode.Ldc_I4, imported));
        Reject<ArgumentException>(() => target.Emit(OpCode.Call, imported));
        Reject<ArgumentNullException>(() => main.Emit(OpCode.Call, (ImportedMethodReference)null!));
        const string systemJson = """
            {"format":5,"name":"System","entry":"","types":[{"name":"System.Math","fields":[]}],"functions":[
            {"name":"System.Math.Min","owner":{"Named":"System.Math"},"parameters":["Int32","Int32"],"returns":"Int32","body":[]}]}
            """;
        var system = NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(systemJson)));
        var native = system.Functions.Single();
        main.ClearBody(); main.LoadConstant(42); main.LoadConstant(99); main.Call(native); main.Return();
        var nativeBytes = graph.WriteNativeAssembly();
        main.ClearBody(); main.Emit(OpCode.Ldc_I4, 42); main.Emit(OpCode.Ldc_I4, 99); main.Emit(OpCode.Call, native); main.Emit(OpCode.Ret);
        Check(graph.WriteNativeAssembly().SequenceEqual(nativeBytes), "native operand encoding");
        Reject<ArgumentException>(() => main.Emit(OpCode.Add, native));
        Reject<ArgumentNullException>(() => main.Emit(OpCode.Call, (NativeFunctionDefinition)null!));
        Reject<InvalidDataException>(() => graph.Write());
        main.ClearBody(); main.Emit(OpCode.Ldarg, -1); main.Emit(OpCode.Ret);
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        main.ClearBody(); main.Emit(OpCode.Add); main.Emit(OpCode.Ret);
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("invalid raw emission was accepted");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
