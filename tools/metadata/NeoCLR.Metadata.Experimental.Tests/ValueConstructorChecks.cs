using System.Diagnostics;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ValueConstructorChecks
{
    static (AssemblyBuilder Library, AssemblyBuilder App, MethodBuilder Constructor, FieldBuilder Field) Create(bool nested = false, bool native = false, bool authored = false)
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var library = new AssemblyBuilder(new("ValueConstructorLibrary", new Version(1, 0, 0, 0)), core);
        var container = nested ? library.AddType("Example", "Container") : null;
        var number = nested ? container!.AddNestedValueType("Number") : library.AddValueType("Example", "Number");
        var value = number.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var constructor = number.AddConstructor([PrimitiveType.Int32]);
        constructor.LoadArgument(0); constructor.LoadArgument(1); constructor.StoreField(value); constructor.Return();
        var read = number.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        read.LoadArgument(0); read.LoadField(value); read.Return();
        var box = nested ? container!.AddNestedGenericValueType("Box", ["T"]) : library.AddGenericValueType("Example", "Box", ["T"]);
        var t = SignatureType.TypeParameter(0); var field = box.AddField("Value", t, FieldVisibility.Public);
        var init = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        init.LoadArgument(0); init.LoadArgument(1); init.StoreField(field); init.Return();
        var get = box.AddInstanceMethod("Get", new MethodSignature(t, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        var referenceType = nested ? container!.AddNestedClass("ReferenceBox") : library.AddClass("Example", "ReferenceBox");
        var referenceField = referenceType.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var referenceConstructor = referenceType.AddConstructor([PrimitiveType.Int32]);
        referenceConstructor.LoadArgument(0); referenceConstructor.LoadArgument(1); referenceConstructor.StoreField(referenceField); referenceConstructor.Return();
        var referenceRead = referenceType.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        referenceRead.LoadArgument(0); referenceRead.LoadField(referenceField); referenceRead.Return();
        var image = RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core);
        var projection = native ? AssemblyDefinition.ReadNativeAssembly(image) : RuntimeAssemblyContainer.ReadCliProjection(image);
        var app = new AssemblyBuilder(new("ValueConstructorApp", new Version(1, 0, 0, 0)), core);
        var numberType = projection.MainModule.Types.Single(t => t.Name == "Number");
        var digest = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image));
        ImportedTypeReference Describe(TypeDefinition type)
        {
            if (type.DeclaringType is { } parent)
                return app.CreateNestedTypeReference(Describe(parent), type.Name, type.GenericArity, type.IsValueType);
            return type.IsValueType
                ? app.CreateValueTypeReference(library.Identity, core, digest, type.Namespace, type.Name, type.GenericArity)
                : app.CreateTypeReference(library.Identity, core, digest, type.Namespace, type.Name, type.GenericArity);
        }
        ImportedMethodReference Reference(TypeDefinition type, string name, MethodSignature signature) => authored
            ? app.CreateMethodReference(Describe(type), name, signature)
            : app.ImportReference(type.Methods.Single(m => m.Name == name), core);
        if (authored)
        {
            var root = app.CreateTypeReference(library.Identity, core, digest, "Example", "Container");
            var foreign = new AssemblyBuilder(new("OtherOutput", new Version(1, 0, 0, 0)), core);
            try { foreign.CreateNestedTypeReference(root, "Child"); throw new Exception("foreign parent admitted"); } catch (ArgumentException) { }
            try { app.CreateValueTypeReference(library.Identity, core, new string('0', 64), "Example", "Other"); throw new Exception("conflicting snapshot admitted"); } catch (InvalidDataException) { }
        }
        var numberCtor = Reference(numberType, ".ctor", new(PrimitiveType.Void, [PrimitiveType.Int32]));
        var numberRead = Reference(numberType, "Read", new(PrimitiveType.Int32, []));
        var boxType = projection.MainModule.Types.Single(t => t.Name == "Box`1");
        var boxCtor = Reference(boxType, ".ctor", new(PrimitiveType.Void, [t])).MakeConstructedReference([PrimitiveType.Int32]);
        var boxRead = Reference(boxType, "Get", new(t, [])).MakeConstructedReference([PrimitiveType.Int32]);
        if (!numberCtor.IsConstructor || !boxCtor.Definition.IsConstructor) throw new Exception("constructor identity lost");
        var referenceDefinition = projection.MainModule.Types.Single(t => t.Name == "ReferenceBox");
        var importedReferenceConstructor = Reference(referenceDefinition, ".ctor", new(PrimitiveType.Void, [PrimitiveType.Int32]));
        var importedReferenceRead = Reference(referenceDefinition, "Read", new(PrimitiveType.Int32, []));
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(authored ? Describe(numberType) : app.ImportReference(numberType, core)); var boxed = main.DeclareLocal(boxCtor.DeclaringType);
        main.LoadConstant(42); main.NewObject(numberCtor); main.StoreLocal(local);
        main.LoadLocalAddress(local); main.Call(numberRead); main.NewObject(boxCtor); main.StoreLocal(boxed);
        main.LoadLocalAddress(boxed); main.Call(boxRead); main.Emit(OpCode.Newobj, importedReferenceConstructor); main.Call(importedReferenceRead); main.Return();
        try { main.Call(numberCtor); throw new Exception("ordinary constructor call admitted"); } catch (ArgumentException) { }
        return (library, app, constructor, value);
    }
    internal static void Run(bool nested = false, bool native = false, bool authored = false)
    {
        var (library, app, constructor, field) = Create(nested, native, authored);
        var context = new AssemblyLoadContext("value-constructors", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("CLI constructor roundtrip");
        }
        finally { context.Unload(); }
        constructor.ClearBody(); constructor.Return(); Reject(library);
        constructor.ClearBody(); constructor.LoadArgument(0); constructor.LoadField(field); constructor.Emit(OpCode.Pop); constructor.Return(); Reject(library);
        constructor.ClearBody(); constructor.LoadArgument(0); constructor.LoadObject(field.DeclaringType); constructor.Emit(OpCode.Pop); constructor.Return(); Reject(library);
        constructor.ClearBody(); var skip = constructor.DefineLabel(); constructor.Emit(OpCode.Ldc_Bool, true); constructor.Emit(OpCode.Brtrue, skip);
        constructor.LoadArgument(0); constructor.LoadArgument(1); constructor.StoreField(field); constructor.MarkLabel(skip); constructor.Return(); Reject(library);
        constructor.ClearBody(); constructor.Fail("construction aborted"); library.Write(); library.WriteNativeAssembly();
    }
    static void Reject(AssemblyBuilder assembly)
    {
        try { assembly.Write(); throw new Exception("invalid constructor accepted"); } catch (InvalidDataException) { }
        try { assembly.WriteNativeAssembly(); throw new Exception("invalid native constructor accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory, bool nested = false, bool native = false)
    {
        Run(nested, native); Directory.CreateDirectory(directory); var (library, app, _, _) = Create(nested, native);
        var libraryPath = Path.Combine(directory, "Library.dll"); var appPath = Path.Combine(directory, "App.dll");
        File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, appPath, "--module", libraryPath }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Imported value/generic constructors execute on CLR and neoCLR: 42");
    }
}
