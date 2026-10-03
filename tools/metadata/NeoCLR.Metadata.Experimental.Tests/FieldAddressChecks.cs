using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class FieldAddressChecks
{
    private static (AssemblyBuilder Graph, FieldBuilder Field, MethodBuilder Constructor) Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("FieldAddress", new Version(1, 0, 0, 0)), core);
        var point = graph.AddValueType("Example", "Point");
        var x = point.AddField("X", PrimitiveType.Int32, FieldVisibility.Public);
        var pointCtor = point.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32]));
        var il = pointCtor.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(x); il.Return();
        var box = graph.AddGenericClass("Example", "Box", ["T"]);
        var t = SignatureType.TypeParameter(0);
        var payload = box.AddField("Value", t, FieldVisibility.Public);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        il = ctor.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(payload); il.Return();
        var setter = box.AddInstanceMethod("Set", new(PrimitiveType.Void, [t]));
        il = setter.GetILGenerator(); il.LoadArgument(0); il.LoadFieldAddress(payload); il.LoadArgument(1); il.StoreObject(t); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator();
        var local = il.DeclareLocal(box.MakeGenericInstance(point));
        var alias = il.DeclareLocal(box.MakeGenericInstance(point));
        il.LoadConstant(1); il.NewObject(pointCtor); il.NewObject(ctor.MakeConstructedReference([point])); il.StoreLocal(local);
        il.LoadLocal(local); il.StoreLocal(alias);
        il.LoadLocal(local); il.LoadConstant(2); il.NewObject(pointCtor); il.Call(setter.MakeConstructedReference([point]));
        il.LoadLocal(local); il.LoadFieldAddress(payload.MakeConstructedReference(point)); il.Emit(OpCode.Ldflda, x);
        il.LoadConstant(42); il.StoreObject(PrimitiveType.Int32);
        il.LoadLocal(alias); il.LoadField(payload.MakeConstructedReference(point)); il.LoadField(x); il.Return();
        return (graph, x, pointCtor);
    }
    internal static void Run()
    {
        var (graph, _, _) = Create();
        var context = new AssemblyLoadContext("field-address", true);
        try
        {
            var bytes = graph.Write();
            _ = AssemblyDefinition.ReadAssembly(bytes, false);
            var loaded = context.LoadFromStream(new MemoryStream(bytes));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("nested field address did not mutate aliased storage");
        }
        finally { context.Unload(); }
        _ = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var uninitialized in new[] { false, true })
        {
            var (bad, field, constructor) = Create();
            var main = bad.EntryPoint!; main.ClearBody(); var il = main.GetILGenerator();
            if (uninitialized) il.LoadLocalAddress(il.DeclareLocal(field.DeclaringType));
            else { il.LoadConstant(1); il.NewObject(constructor); }
            il.LoadFieldAddress(field); il.LoadObject(PrimitiveType.Int32); il.Return();
            Reject<InvalidDataException>(() => bad.Write());
            Reject<InvalidDataException>(() => bad.WriteNativeAssembly());
        }
        var owner = graph.AddClass("Example", "Readonly");
        var readOnly = owner.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public, isReadOnly: true);
        var body = graph.AddFunction("Invalid", new(PrimitiveType.Void, [])).GetILGenerator();
        Reject<ArgumentException>(() => body.LoadFieldAddress(readOnly));
        Reject<ArgumentException>(() => body.LoadFieldAddress(Create().Field));
        body.Return(); _ = graph.Write();
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var (graph, _, _) = Create();
        var path = Path.Combine(directory, "FieldAddress.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42) || command == "run" && text.Length != 0) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Nested field addresses preserve generic storage and object aliases on CLR/NeoCLR: 42");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
