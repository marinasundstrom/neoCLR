using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericObjectRootChecks
{
    internal static AssemblyBuilder Create(bool manual = false)
    {
        var graph = new AssemblyBuilder(new("GenericObjectRoot", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var root = graph.AddNativeObjectRoot();
        var rootCtor = root.AddConstructor([], MethodVisibility.Protected);
        rootCtor.GetILGenerator().Return();
        var display = root.AddNativeObjectSlot("ToString", new(PrimitiveType.String, []));
        display.GetILGenerator().Emit(OpCode.Ldstr, "object"); display.GetILGenerator().Return();
        var hash = root.AddNativeObjectSlot("GetHashCode", new(PrimitiveType.Int32, []));
        hash.GetILGenerator().LoadConstant(2); hash.GetILGenerator().Return();
        var equals = root.AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [root]));
        equals.GetILGenerator().Emit(OpCode.Ldc_Bool, false); equals.GetILGenerator().Return();
        TypeBuilder box;
        if (manual)
        {
            var definition = new TypeDefinition("Example", "Box", 1, root.Definition.ToReference(), ["T"]);
            graph.Definition.MainModule.Types.Add(definition);
            box = graph.Types.Last();
        }
        else box = graph.AddGenericClass("Example", "Box", ["T"], root);
        var parameter = SignatureType.TypeParameter(0);
        var field = box.AddField("Value", parameter, FieldVisibility.Public);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        var il = ctor.GetILGenerator();
        il.LoadArgument(0); il.Call(rootCtor);
        il.LoadArgument(0); il.LoadArgument(1); il.StoreField(field); il.Return();
        var get = box.AddInstanceMethod("Get", new(parameter, []));
        il = get.GetILGenerator(); il.LoadArgument(0); il.LoadField(field); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator(); il.LoadConstant(40); il.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32]));
        var saved = il.DeclareLocal(PrimitiveType.Int32);
        il.Duplicate(); il.Call(get.MakeConstructedReference([PrimitiveType.Int32])); il.StoreLocal(saved);
        il.CallVirtual(hash); il.LoadLocal(saved); il.Add(); il.Return();
        return graph;
    }

    internal static void WriteRuntime(string path)
    {
        var library = Create();
        library.EntryPoint = null;
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(library);
        File.WriteAllBytes(path, image);
        var app = new AssemblyBuilder(new("GenericObjectConsumer", new(1, 0, 0, 0)), library.CoreLibrary);
        var call = app.CreateFunctionReference(library.Identity, library.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "", "Main", new(PrimitiveType.Int32, []));
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.GetILGenerator().Call(call); main.GetILGenerator().Return();
        File.WriteAllBytes(path + ".app", RuntimeAssemblyContainer.WriteBinary(app));
    }

    internal static void WriteConsumer(string libraryPath, string corePath, string output)
    {
        var image = File.ReadAllBytes(libraryPath);
        var library = AssemblyDefinition.ReadNativeAssembly(image);
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var app = new AssemblyBuilder(new("SourceGenericObjectConsumer", new(1, 0, 0, 0)), core.Identity);
        var call = app.CreateFunctionReference(library.Identity, core.Identity,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "", "RunGeneric", new(PrimitiveType.Int32, []));
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.GetILGenerator().Call(call); main.GetILGenerator().Return();
        File.WriteAllBytes(output, RuntimeAssemblyContainer.WriteBinary(app));
    }

    internal static void Run()
    {
        var restricted = new AssemblyBuilder(new("UnsupportedGenericBases", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var ordinary = restricted.AddClass("Example", "Base");
        try { restricted.AddGenericClass("Example", "Bad", ["T"], ordinary); throw new Exception("ordinary generic base admitted"); }
        catch (ArgumentException) { }
        var missing = Create();
        var constructor = missing.Types.Last().Methods.Single(m => m.IsConstructor);
        constructor.Definition.Body.ClearInstructions();
        constructor.GetILGenerator().Return();
        try { RuntimeAssemblyContainer.WriteBinary(missing); throw new Exception("missing base initialization accepted"); }
        catch (InvalidDataException) { }

        foreach (bool manual in new[] { false, true })
        {
            var graph = Create(manual);
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            var loaded = AssemblyDefinition.ReadNativeAssembly(image);
            var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([loaded]);
            var views = context.Resolve(loaded.Identity).GetTypes();
            if (!ReferenceEquals(views.Single(t => t.Name == "Box`1").BaseType, views.Single(t => t.Name == "Object")))
                throw new Exception("source root facade identity lost");
            var box = loaded.MainModule.Types.Single(t => t.Name == "Box`1");
            if (box.BaseType?.Resolve().Name != "Object" || box.GenericArity != 1)
                throw new Exception("generic source-root relationship lost");
        }
    }
}
