using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class TypeHandleChecks
{
    private static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("TypeHandles", new(1, 0, 0, 0)), core);
        var local = graph.AddClass("Example", "Item");
        var token = graph.AddFunction("Token", new(PrimitiveType.RuntimeTypeHandle, []));
        token.GetILGenerator().LoadTypeToken(local);
        token.Return();
        var generic = graph.AddFunction("GenericToken", new(PrimitiveType.RuntimeTypeHandle, [], ["T"]));
        generic.GetILGenerator().Emit(OpCode.Ldtoken, SignatureType.MethodParameter(0));
        generic.Return();
        var box = graph.AddGenericClass("Example", "Box", ["T"]);
        var ownerToken = box.AddMethod("OwnerToken", new(PrimitiveType.RuntimeTypeHandle, []));
        ownerToken.GetILGenerator().LoadTypeToken(SignatureType.TypeParameter(0));
        ownerToken.Return();
        var constructed = graph.AddFunction("ConstructedToken", new(PrimitiveType.RuntimeTypeHandle, []));
        constructed.GetILGenerator().LoadTypeToken(box.MakeGenericInstance(PrimitiveType.Int32));
        constructed.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.Call(token); il.Emit(OpCode.Pop);
        il.Call(generic.MakeGenericInstance(PrimitiveType.String)); il.Emit(OpCode.Pop);
        il.Call(constructed); il.Emit(OpCode.Pop);
        il.LoadTypeToken(SignatureType.ArrayOf(PrimitiveType.Int32)); il.Emit(OpCode.Pop);
        il.LoadConstant(42); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var cli = graph.Write();
        var context = new AssemblyLoadContext("type-handles", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(cli));
            var token = (RuntimeTypeHandle)assembly.ManifestModule.GetMethods().Single(m => m.Name == "Token").Invoke(null, null)!;
            if (Type.GetTypeFromHandle(token) != assembly.GetType("Example.Item")) throw new Exception("local type identity lost");
            var generic = assembly.ManifestModule.GetMethods().Single(m => m.Name == "GenericToken").MakeGenericMethod(typeof(string));
            if (Type.GetTypeFromHandle((RuntimeTypeHandle)generic.Invoke(null, null)!) != typeof(string)) throw new Exception("generic type identity lost");
            var box = assembly.GetType("Example.Box`1")!.MakeGenericType(typeof(int));
            var constructed = assembly.ManifestModule.GetMethods().Single(m => m.Name == "ConstructedToken");
            if (Type.GetTypeFromHandle((RuntimeTypeHandle)constructed.Invoke(null, null)!) != box) throw new Exception("constructed identity lost");
            if (Type.GetTypeFromHandle((RuntimeTypeHandle)box.GetMethod("OwnerToken")!.Invoke(null, null)!) != typeof(int)) throw new Exception("owner parameter identity lost");
        }
        finally { context.Unload(); }
        var snapshot = AssemblyDefinition.ReadAssembly(cli, false);
        var output = new AssemblyBuilder(new("Consumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var imported = output.ImportReference(snapshot.MainModule.Methods.Single(m => m.Name == "Token"), graph.CoreLibrary);
        if (imported.Signature.ReturnType.Primitive != PrimitiveType.RuntimeTypeHandle) throw new Exception("CLI handle import failed");
        var native = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        if (!native.MainModule.Methods.Single(m => m.Name == "Token").TryGetStaticPrimitiveSignature(out var signature) || signature!.ReturnType != PrimitiveType.RuntimeTypeHandle)
            throw new Exception("native handle round trip failed");
        var il = output.AddFunction("Invalid", new(PrimitiveType.Void, [])).GetILGenerator();
        foreach (var bad in new[] { (SignatureType)PrimitiveType.Void, SignatureType.Self, SignatureType.ByReference(PrimitiveType.Int32), SignatureType.MethodParameter(0), (SignatureType)graph.AddClass("Example", "Foreign") })
        {
            try { il.LoadTypeToken(bad); throw new Exception("invalid token operand accepted"); }
            catch (ArgumentException) { }
        }
    }

    private static byte[] Image() { var graph = Create(); return RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary); }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var path = Path.Combine(directory, "TypeHandles.dll");
        File.WriteAllBytes(path, Image());
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
    }
}
