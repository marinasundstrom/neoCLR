using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ReferenceOperationChecks
{
    private static AssemblyBuilder Create(AssemblyIdentity core, Action<AssemblyBuilder>? configure = null)
    {
        var graph = new AssemblyBuilder(new("ReferenceOperations", new Version(1, 0, 0, 0)), core);
        configure?.Invoke(graph);
        var check = graph.AddFunction("IsAbsent", new(PrimitiveType.Boolean, [graph.CoreObjectType], ["T"]));
        var il = check.GetILGenerator(); il.LoadArgument(0); il.IsInstance(SignatureType.MethodParameter(0)); il.IsNull(); il.Return();
        var cast = graph.AddFunction("CastString", new(PrimitiveType.String, [graph.CoreObjectType]));
        il = cast.GetILGenerator(); il.LoadArgument(0); il.CastReference(PrimitiveType.String); il.Return();
        var owner = graph.AddClass("Example", "Reference");
        var constructor = owner.AddConstructor(new MethodSignature(PrimitiveType.Void, []));
        constructor.GetILGenerator().Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main; il = main.GetILGenerator();
        var fail = il.DefineLabel();
        il.NewObject(constructor); il.IsInstance(owner); il.IsNull(); il.Emit(OpCode.Brtrue, fail);
        il.LoadConstant(42); il.Box(PrimitiveType.Int32); il.IsInstance(owner); il.IsNull(); il.Emit(OpCode.Brfalse, fail);
        il.LoadDefault(graph.CoreObjectType); il.IsNull(); il.Emit(OpCode.Brfalse, fail);
        il.LoadDefault(PrimitiveType.String); il.Emit(OpCode.ReferenceIsNull); il.Emit(OpCode.Brfalse, fail);
        il.Emit(OpCode.Ldstr, "text"); il.Box(PrimitiveType.String); il.Emit(OpCode.Isinst, (SignatureType)PrimitiveType.String); il.IsNull(); il.Emit(OpCode.Brtrue, fail);
        il.LoadConstant(42); il.Box(PrimitiveType.Int32); il.IsInstance(PrimitiveType.String); il.IsNull(); il.Emit(OpCode.Brfalse, fail);
        il.LoadConstant(42); il.Box(PrimitiveType.Int32); il.IsInstance(PrimitiveType.Int32); il.IsNull(); il.Emit(OpCode.Brtrue, fail);
        il.Emit(OpCode.Ldstr, "text"); il.Box(PrimitiveType.String); il.Call(cast); il.IsNull(); il.Emit(OpCode.Brtrue, fail);
        il.LoadConstant(42); il.Return(); il.MarkLabel(fail); il.LoadConstant(1); il.Return();
        return graph;
    }
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = Create(core);
        var context = new AssemblyLoadContext("reference-operations", true);
        try
        {
            var bytes = graph.Write(); _ = AssemblyDefinition.ReadAssembly(bytes, false);
            var loaded = context.LoadFromStream(new MemoryStream(bytes));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("null/type-test result mismatch");
            var check = loaded.ManifestModule.GetMethods().Single(m => m.Name == "IsAbsent");
            foreach (var type in new[] { typeof(int), typeof(string), typeof(object) })
            {
                if (!Equals(true, check.MakeGenericMethod(type).Invoke(null, [null]))) throw new Exception("generic test matched null");
                object value = type == typeof(int) ? 42 : type == typeof(string) ? "text" : new object();
                if (!Equals(false, check.MakeGenericMethod(type).Invoke(null, [value]))) throw new Exception("generic test missed exact value");
            }
            var text = new string('x', 3);
            if (!ReferenceEquals(text, loaded.ManifestModule.GetMethods().Single(m => m.Name == "CastString").Invoke(null, [text]))) throw new Exception("cast changed string identity");
        }
        finally { context.Unload(); }
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        var il = graph.AddFunction("Rejected", new(PrimitiveType.Void, [])).GetILGenerator();
        foreach (var bad in new[] { (SignatureType)PrimitiveType.Void, SignatureType.ByReference(PrimitiveType.Int32), SignatureType.MethodParameter(0) })
            Reject<ArgumentException>(() => il.IsInstance(bad));
        il.Return(); _ = graph.Write();
        var invalid = new AssemblyBuilder(new("InvalidReferenceTest", new Version(1, 0, 0, 0)), core);
        il = invalid.AddFunction("Bad", new(PrimitiveType.Boolean, [])).GetILGenerator();
        il.LoadConstant(42); il.IsNull(); il.Return();
        Reject<InvalidDataException>(() => invalid.Write());
    }
    internal static async Task RunRuntime(string runtime, string referencePath, string seedPath, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var reference = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(referencePath), false);
        var graph = Create(reference.Identity, output => output.BindNativeLibrary(reference,
            NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath)), reference.Identity));
        var path = Path.Combine(directory, "ReferenceOperations.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        _ = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(path));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path, "--system", seedPath }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42) || command == "run" && text.Length != 0) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Null checks, primitive/reference type tests and String casts execute on CLR/NeoCLR: 42");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
