using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExternalClassBaseAuthoringChecks
{
    internal static void Run(string? output = null)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ExternalBaseLibrary", new Version(1, 0, 0, 0)), core);
        var parent = library.AddClass("Example", "Base");
        parent.SetAbstractClass();
        parent.AddConstructor([], MethodVisibility.Protected).Return();
        var libraryImage = RuntimeAssemblyContainer.WriteLibraryBinary(library);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(libraryImage);
        var app = new AssemblyBuilder(new("ExternalBaseApp", new Version(1, 0, 0, 0)), core);
        var baseType = app.ImportReference(snapshot.MainModule.Types.Single(), core);
        Reject<ArgumentException>(() => app.AddClass("Example", "Undeclared", baseType));
        app.DeclareFieldlessClassBase(baseType);
        var derived = app.AddClass("Example", "Derived", baseType);
        var field = derived.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
        var baseConstructor = app.ImportReference(snapshot.MainModule.Methods.Single(), core);
        var constructor = derived.AddConstructor([]);
        constructor.LoadArgument(0); constructor.GetILGenerator().Emit(OpCode.Call, baseConstructor);
        constructor.LoadArgument(0); constructor.LoadConstant(42); constructor.StoreField(field); constructor.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.NewObject(constructor); main.LoadField(field); main.Return();
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(app);
        var read = AssemblyDefinition.ReadNativeAssembly(image);
        Check(read.MainModule.Types.Single().BaseType!.Resolve(new Resolver(snapshot)).Name == "Base", "external base lost");
        var context = new AssemblyLoadContext("external-base", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var executable = context.LoadFromStream(new MemoryStream(app.Write()));
            Check((int)executable.EntryPoint!.Invoke(null, null)! == 42, "CLR external constructor chaining");
            Check(executable.GetType("Example.Derived")!.BaseType!.Assembly.GetName().Name == "ExternalBaseLibrary", "CLR base scope");
        }
        finally { context.Unload(); }
        if (output is not null)
        {
            Directory.CreateDirectory(output);
            File.WriteAllBytes(Path.Combine(output, "Library.dll"), libraryImage);
            File.WriteAllBytes(Path.Combine(output, "App.dll"), image);
        }
        constructor.ClearBody(); constructor.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        constructor.ClearBody();
        constructor.LoadArgument(0); constructor.GetILGenerator().Emit(OpCode.Call, baseConstructor);
        constructor.LoadArgument(0); constructor.GetILGenerator().Emit(OpCode.Call, baseConstructor); constructor.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        constructor.ClearBody(); constructor.LoadArgument(0); constructor.GetILGenerator().Emit(OpCode.Call, baseConstructor); constructor.Return();
        main.ClearBody(); main.GetILGenerator().Emit(OpCode.Newobj, baseConstructor); main.Emit(OpCode.Pop); main.LoadConstant(0); main.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        CheckVirtualBase(core, output);
        Console.WriteLine("PASS external base storage, virtual dispatch and constructor chaining");
    }
    private static void CheckVirtualBase(AssemblyIdentity core, string? output)
    {
        var library = new AssemblyBuilder(new("VirtualBaseLibrary", new Version(1, 0, 0, 0)), core);
        var parent = library.AddClass("Example", "Base");
        parent.SetAbstractClass();
        var storage = parent.AddField("Number", PrimitiveType.Int32, FieldVisibility.Private);
        var constructor = parent.AddConstructor([], MethodVisibility.Protected);
        constructor.LoadArgument(0); constructor.LoadConstant(40); constructor.StoreField(storage); constructor.Return();
        var signature = new MethodSignature(PrimitiveType.Int32, []);
        var value = parent.AddVirtualMethod("Value", signature);
        value.LoadArgument(0); value.LoadField(storage); value.Return();
        parent.AddAbstractMethod("Extra", signature);
        var libraryImage = RuntimeAssemblyContainer.WriteLibraryBinary(library);
        var app = new AssemblyBuilder(new("VirtualBaseApp", new Version(1, 0, 0, 0)), core);
        var hash = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(libraryImage));
        var imported = app.CreateTypeReference(library.Identity, core, hash, "Example", "Base");
        var baseValue = app.CreateMethodReference(imported, "Value", signature, isVirtual: true);
        var baseExtra = app.CreateMethodReference(imported, "Extra", signature, isVirtual: true, isAbstract: true);
        Reject<InvalidDataException>(() => app.CreateMethodReference(imported, "Value", signature));
        Reject<ArgumentException>(() => app.CreateMethodReference(imported, "Bad", signature, isAbstract: true));
        Reject<ArgumentException>(() => app.CompleteClassBaseReference(imported, [baseValue, baseValue]));
        app.CompleteClassBaseReference(imported, [baseValue, baseExtra]);
        Reject<ArgumentException>(() => app.CompleteClassBaseReference(imported, [baseValue]));
        var derived = app.AddClass("Example", "Derived", imported);
        var derivedConstructor = derived.AddConstructor([]);
        var baseConstructor = app.CreateMethodReference(imported, ".ctor", new MethodSignature(PrimitiveType.Void, []), visibility: MethodVisibility.Protected);
        derivedConstructor.LoadArgument(0); derivedConstructor.GetILGenerator().Emit(OpCode.Call, baseConstructor); derivedConstructor.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.NewObject(derivedConstructor); main.GetILGenerator().Emit(OpCode.Callvirt, baseValue); main.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        var extra = derived.AddOverride("Extra", signature); extra.LoadConstant(2); extra.Return();
        var overridden = derived.AddOverride("Value", signature);
        overridden.LoadArgument(0); overridden.GetILGenerator().Emit(OpCode.Call, baseValue);
        overridden.LoadArgument(0); overridden.Call(extra); overridden.Emit(OpCode.Add); overridden.Return();
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(app);
        var context = new AssemblyLoadContext("virtual-external-base", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var executable = context.LoadFromStream(new MemoryStream(app.Write()));
            Check((int)executable.EntryPoint!.Invoke(null, null)! == 42, "CLR external virtual dispatch and inherited private storage");
        }
        finally { context.Unload(); }
        if (output is not null)
        {
            File.WriteAllBytes(Path.Combine(output, "VirtualLibrary.dll"), libraryImage);
            File.WriteAllBytes(Path.Combine(output, "VirtualApp.dll"), image);
        }
    }

    private sealed class Resolver(AssemblyDefinition candidate) : IAssemblyResolver
    { public AssemblyDefinition? Resolve(AssemblyIdentity identity) => candidate; }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
