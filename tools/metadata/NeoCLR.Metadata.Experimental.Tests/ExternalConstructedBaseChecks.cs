using System.Runtime.Loader;
using System.Security.Cryptography;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExternalConstructedBaseChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ExternalGenericLibrary", new(1, 0, 0, 0)), core);
        var t = SignatureType.TypeParameter(0);
        var view = library.AddGenericInterface("Example", "View", ["T"]);
        view.AddInterfaceMethod("Get", new(t, []));
        var parent = library.AddGenericClass("Example", "Base", ["T"]);
        parent.AddInterfaceImplementation(view.MakeGenericInstance(t));
        var value = parent.AddField("value", t, FieldVisibility.Private);
        var constructor = parent.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]), MethodVisibility.Protected);
        constructor.LoadArgument(0); constructor.LoadArgument(1); constructor.StoreField(value); constructor.Return();
        var get = parent.AddVirtualMethod("Get", new(t, []));
        get.LoadArgument(0); get.LoadField(value); get.Return();
        var echo = parent.AddVirtualMethod("Echo", new(t, [t]));
        echo.LoadArgument(1); echo.Return();
        var libraryImage = RuntimeAssemblyContainer.WriteLibraryBinary(library);
        var hash = Convert.ToHexString(SHA256.HashData(libraryImage));
        var app = new AssemblyBuilder(new("ExternalGenericApp", new(1, 0, 0, 0)), core);
        var imported = app.CreateTypeReference(library.Identity, core, hash, "Example", "Base`1", 1);
        var importedView = app.CreateInterfaceReference(library.Identity, core, hash, "Example", "View`1", 1);
        var viewGet = app.CreateMethodReference(importedView, "Get", new(t, []));
        app.CompleteInterfaceReference(importedView);
        app.AddInterfaceConversion(imported, importedView.MakeGenericInstance(t));
        var importedGet = app.CreateMethodReference(imported, "Get", new(t, []), isVirtual: true);
        var importedEcho = app.CreateMethodReference(imported, "Echo", new(t, [t]), isVirtual: true);
        var importedCtor = app.CreateMethodReference(imported, ".ctor", new(PrimitiveType.Void, [t]), visibility: MethodVisibility.Protected);
        app.CompleteClassBaseReference(imported, [importedGet, importedEcho], 1);
        Reject<ArgumentException>(() => app.CompleteClassBaseReference(imported.MakeGenericInstance(PrimitiveType.Int32), [], 1));
        var derived = app.AddGenericClass("Example", "Derived", ["Unused", "T"]);
        var second = SignatureType.TypeParameter(1);
        derived.SetBaseType(imported.MakeGenericInstance(second));
        var extra = derived.AddField("Extra", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = derived.AddConstructor(new MethodSignature(PrimitiveType.Void, [second, PrimitiveType.Int32]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.Call(importedCtor.MakeConstructedReference([second]));
        ctor.LoadArgument(0); ctor.LoadArgument(2); ctor.StoreField(extra); ctor.Return();
        var derivedGet = derived.AddOverride("Get", new(second, []));
        derivedGet.LoadArgument(0); derivedGet.Call(importedGet.MakeConstructedReference([second])); derivedGet.Return();
        var derivedEcho = derived.AddOverride("Echo", new(second, [second]));
        derivedEcho.LoadArgument(0); derivedEcho.Call(importedGet.MakeConstructedReference([second])); derivedEcho.Return();
        var leaf = app.AddClass("Example", "Leaf", derived.MakeGenericInstance(PrimitiveType.Boolean, PrimitiveType.Int32));
        var leafCtor = leaf.AddConstructor([]);
        leafCtor.LoadArgument(0); leafCtor.LoadConstant(40); leafCtor.LoadConstant(2);
        leafCtor.Call(ctor.MakeConstructedReference([PrimitiveType.Boolean, PrimitiveType.Int32])); leafCtor.Return();
        var upcast = app.AddFunction("Upcast", new(imported.MakeGenericInstance(PrimitiveType.Int32), [leaf]));
        upcast.LoadArgument(0); upcast.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(leaf);
        main.NewObject(leafCtor); main.StoreLocal(local);
        var reference = main.DeclareLocal(derived.MakeGenericInstance(PrimitiveType.Boolean, leaf));
        main.LoadLocal(local); main.LoadConstant(2); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Boolean, leaf])); main.StoreLocal(reference);
        main.LoadLocal(reference); main.CallVirtual(importedGet.MakeConstructedReference([leaf]));
        main.CallVirtual(viewGet.MakeConstructedReference([PrimitiveType.Int32]));
        main.LoadLocal(local); main.LoadField(extra.MakeConstructedReference(PrimitiveType.Boolean, PrimitiveType.Int32)); main.Add();
        main.LoadLocal(local); main.LoadConstant(99); main.CallVirtual(importedEcho.MakeConstructedReference([PrimitiveType.Int32])); main.Add();
        main.LoadLocal(local); main.LoadConstant(40); main.Call(importedEcho.MakeConstructedReference([PrimitiveType.Int32])); main.Subtract();
        main.Return();
        Reject<ArgumentException>(() => ctor.Call(importedCtor.MakeConstructedReference([PrimitiveType.Int32])));
        Reject<ArgumentException>(() => main.Call(importedCtor.MakeConstructedReference([PrimitiveType.Int32])));
        var pending = app.AddClass("Example", "Pending");
        Reject<ArgumentException>(() => pending.SetBaseType(imported));
        Reject<ArgumentException>(() => pending.SetBaseType(imported.MakeGenericInstance(t)));
        pending.SetBaseType(imported.MakeGenericInstance(PrimitiveType.Int32));
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(app);
        var context = new AssemblyLoadContext("external-constructed", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var executable = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(executable.EntryPoint!.Invoke(null, null), 42)) throw new Exception("external constructed CLR storage/dispatch");
        }
        finally { context.Unload(); }
        if (Environment.GetEnvironmentVariable("NEOCLR_EXTERNAL_CONSTRUCTED_OUTPUT") is { } output)
        {
            Directory.CreateDirectory(output);
            File.WriteAllBytes(Path.Combine(output, "Library.dll"), libraryImage);
            File.WriteAllBytes(Path.Combine(output, "App.dll"), image);
        }
        var invalid = app.AddFunction("Invalid", new(imported.MakeGenericInstance(PrimitiveType.Boolean), [leaf]));
        invalid.LoadArgument(0); invalid.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        invalid.ClearBody(); invalid.GetILGenerator().LoadDefault(imported.MakeGenericInstance(PrimitiveType.Boolean)); invalid.Return();
        ctor.ClearBody(); ctor.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
        Console.WriteLine("PASS external constructed bases, protected initialization, inherited storage/interface and invariant rejection");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
