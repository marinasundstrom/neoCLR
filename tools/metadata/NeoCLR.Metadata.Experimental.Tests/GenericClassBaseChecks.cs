using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericClassBaseChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("GenericClassBase", new(1, 0, 0, 0)), core);
        var parent = graph.AddClass("Example", "Base");
        var number = parent.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
        var parentCtor = parent.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32]));
        parentCtor.LoadArgument(0); parentCtor.LoadArgument(1); parentCtor.StoreField(number); parentCtor.Return();
        var box = graph.AddGenericClass("Example", "Box", ["T"], parent);
        var field = box.AddField("Value", SignatureType.TypeParameter(0), FieldVisibility.Public);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32, SignatureType.TypeParameter(0)]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.LoadConstant(1); ctor.Add(); ctor.Call(parentCtor);
        ctor.LoadArgument(0); ctor.LoadArgument(2); ctor.StoreField(field); ctor.Return();
        var constantCtor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [SignatureType.TypeParameter(0)]));
        constantCtor.LoadArgument(0); constantCtor.LoadConstant(2); constantCtor.Call(parentCtor);
        constantCtor.LoadArgument(0); constantCtor.LoadArgument(1); constantCtor.StoreField(field); constantCtor.Return();
        var upcast = graph.AddFunction("Upcast", new(parent, [box.MakeGenericInstance(PrimitiveType.Int32)]));
        upcast.LoadArgument(0); upcast.Return();
        var producer = graph.AddGenericInterface("Example", "Producer", ["T"]);
        producer.SetVariance(0, GenericVariance.Covariant);
        producer.AddInterfaceMethod("Get", new(SignatureType.TypeParameter(0), []));
        var widen = graph.AddFunction("Widen", new(producer.MakeGenericInstance(parent), [producer.MakeGenericInstance(box.MakeGenericInstance(PrimitiveType.Int32))]));
        widen.LoadArgument(0); widen.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var local = main.DeclareLocal(box.MakeGenericInstance(PrimitiveType.Int32));
        main.LoadConstant(1); main.LoadConstant(20); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32])); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadField(field.MakeConstructedReference(PrimitiveType.Int32));
        main.LoadLocal(local); main.Call(upcast); main.LoadField(number); main.Add();
        main.LoadConstant(18); main.NewObject(constantCtor.MakeConstructedReference([PrimitiveType.Int32])); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadField(field.MakeConstructedReference(PrimitiveType.Int32)); main.Add();
        main.LoadLocal(local); main.Call(upcast); main.LoadField(number); main.Add(); main.Return();
        var image = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
        if (snapshot.MainModule.Types.Single(t => t.Name == "Box`1").BaseType!.Name != "Base")
            throw new Exception("generic class base metadata lost");
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        var actual = loaded.EntryPoint!.Invoke(null, null);
        if (!Equals(actual, 42)) throw new Exception("generic inherited storage failed: " + actual);

        var constructed = loaded.GetType("Example.Box`1")!.MakeGenericType(typeof(int));
        var value = Activator.CreateInstance(constructed, 1, 40);
        if (!ReferenceEquals(loaded.ManifestModule.GetMethods().Single(m => m.Name == "Upcast").Invoke(null, [value]), value))
            throw new Exception("generic base upcast changed identity");
        var strings = loaded.GetType("Example.Box`1")!.MakeGenericType(typeof(string));
        var text = Activator.CreateInstance(strings, 1, "kept");
        if (!Equals(strings.GetField("Value")!.GetValue(text), "kept") || !Equals(strings.GetField("Number")!.GetValue(text), 2))
            throw new Exception("generic reference and inherited fields overlap");
        if (Environment.GetEnvironmentVariable("NEOCLR_GENERIC_BASE_ARTIFACT") is { } path) File.WriteAllBytes(path, image);
        ctor.ClearBody(); ctor.Return();
        try { graph.WriteNativeAssembly(); throw new Exception("missing base initialization accepted"); }
        catch (InvalidDataException) { }
        Console.WriteLine("PASS generic derived class authoring, storage, constructor chaining and CLR identity");
    }
}
