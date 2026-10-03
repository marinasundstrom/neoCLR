using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NominalMethodImportChecks
{
    internal static void Run()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NominalLibrary", new Version(1, 0, 0, 0)), core);
        var item = library.AddClass("Example", "Item");
        var value = item.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var constructor = item.AddConstructor([]); constructor.Return();
        var operations = library.AddType("Example", "Operations");
        var create = operations.AddMethod("Create", new MethodSignature(item, []));
        create.NewObject(constructor); create.Duplicate(); create.LoadConstant(42); create.StoreField(value); create.Return();
        var read = operations.AddMethod("Read", new MethodSignature(PrimitiveType.Int32, [item]));
        read.LoadArgument(0); read.LoadField(value); read.Return();
        var box = library.AddGenericClass("Example", "Box", ["T"]);
        var t = SignatureType.MethodParameter(0);
        var boxOfT = box.MakeGenericInstance(t);
        var forward = operations.AddMethod("Forward", new MethodSignature(boxOfT, [boxOfT], ["T"]));
        forward.LoadArgument(0); forward.Return();
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), false);
        var app = new AssemblyBuilder(new("NominalConsumer", new Version(1, 0, 0, 0)), core);
        ImportedMethodReference Import(string method) => app.ImportReference(snapshot.MainModule.Methods.Single(m => m.Name == method), core);
        var factory = Import("Create");
        var reader = Import("Read");
        if (factory.Signature.ReturnType != reader.Signature.ParameterTypes[0]) throw new Exception("nominal signature identity");
        var forwarded = Import("Forward").MakeGenericInstance(PrimitiveType.Int32);
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        entry.LoadDefault(forwarded.Signature.ParameterTypes[0]); entry.Call(forwarded); entry.Emit(OpCode.Pop);
        entry.Call(factory); entry.Call(reader); entry.Return();
        var bypass = new AssemblyBuilder(new("Bypass", new Version(1, 0, 0, 0)), core);
        var bypassEntry = bypass.AddFunction("Main");
        bypassEntry.Call(create); bypassEntry.Call(read); bypassEntry.Return();
        try { bypass.Write(); throw new Exception("mutable foreign nominal call bypassed import"); }
        catch (InvalidDataException) { }
        var image = app.Write();
        var context = new AssemblyLoadContext("nominal-import", isCollectible: true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(image));
            if ((int)loaded.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("CLR nominal cross-assembly execution");
        }
        finally { context.Unload(); }
        _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
        var projected = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        foreach (byte[] malformed in new byte[][] {
            [0, 0, 0x12],                 // Truncated nominal token.
            [0, 0, 0x12, 0x80, 4],       // Noncanonical token.
            [0, 0, 0x12, 0x7c],          // Missing TypeDef.
            [0, 0, 0x12, 5],             // TypeRef requires a resolver contract.
            [0, 0, 0x11, 4],             // Value type is not a reference.
            [0, 0, 0x15, 0x11, 4, 1, 8], // Value-type construction.
            [0, 0, 0x1e, 0],             // Unscoped method parameter.
            [0, 0, 0x1d, 0x1d, 0x10, 8],       // By-reference vector element.
            [0, 0, 8, 0]                 // Trailing bytes.
        })
        {
            var fixture = AssemblyDefinition.ReadAssembly(CallableChecks.Image(1, malformed), false);
            var isolated = new AssemblyBuilder(new("Reject", new Version(1, 0, 0, 0)), core);
            try { isolated.ImportReference(fixture.MainModule.Methods.Single(), core); }
            catch (InvalidDataException) { continue; }
            throw new Exception("malformed/unsupported nominal signature admitted");
        }
        var projectedApp = new AssemblyBuilder(new("ProjectedConsumer", new Version(1, 0, 0, 0)), core);
        _ = projectedApp.ImportReference(projected.MainModule.Methods.Single(m => m.Name == "Forward"), core);
    }
}
