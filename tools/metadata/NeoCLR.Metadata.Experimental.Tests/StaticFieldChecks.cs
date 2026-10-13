using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class StaticFieldChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("StaticFields", new Version(1, 0, 0, 0)), core);
        var catalog = graph.AddType("Example", "Headers");
        var count = catalog.AddField("Count", PrimitiveType.Int32, FieldVisibility.Public, isStatic: true);
        var cache = graph.AddGenericClass("Example", "Cache", ["T"]);
        cache.AddField("InstanceValue", PrimitiveType.Int32);
        var value = cache.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public, isStatic: true);
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.LoadConstant(40); il.StoreField(value.MakeConstructedReference(PrimitiveType.Int32));
        il.LoadConstant(2); il.StoreField(value.MakeConstructedReference(PrimitiveType.String));
        il.LoadField(value.MakeConstructedReference(PrimitiveType.Int32));
        il.LoadField(value.MakeConstructedReference(PrimitiveType.String));
        il.Add(); il.StoreField(count); il.LoadField(count); il.Return();
        var context = new AssemblyLoadContext("static-fields", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(graph.Write()));
            if (!Equals(42, assembly.EntryPoint!.Invoke(null, null))) throw new Exception("CLR static separation failed");
        }
        finally { context.Unload(); }
        var native = RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        var loaded = snapshot.MainModule.Types.Single(t => t.Name == "Cache`1");
        if (loaded.Fields.Count != 2 || loaded.Fields[0].Attributes != 1 || loaded.Fields[1].Attributes != 0x16)
            throw new Exception("Static field flags/layout lost");
        var consumer = new AssemblyBuilder(new("StaticConsumer", new Version(1, 0, 0, 0)), core);
        var imported = consumer.ImportReference(loaded.Fields[1], core);
        if (!imported.IsStatic) throw new Exception("Imported static flag lost");
        var read = consumer.AddFunction("Main"); consumer.EntryPoint = read;
        read.LoadField(imported.MakeConstructedReference(PrimitiveType.Int32)); read.Return();
        _ = consumer.WriteNativeAssembly(); _ = consumer.Write();
        Console.WriteLine("PASS CLR static storage, native declaration roundtrip and constructed field import");
    }
}
