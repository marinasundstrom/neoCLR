using System.Diagnostics;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ValueConstructorChecks
{
    static (AssemblyBuilder Library, AssemblyBuilder App, MethodBuilder Constructor, FieldBuilder Field) Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var library = new AssemblyBuilder(new("ValueConstructorLibrary", new Version(1, 0, 0, 0)), core);
        var number = library.AddValueType("Example", "Number");
        var value = number.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var constructor = number.AddConstructor([PrimitiveType.Int32]);
        constructor.LoadArgument(0); constructor.LoadArgument(1); constructor.StoreField(value); constructor.Return();
        var read = number.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        read.LoadArgument(0); read.LoadField(value); read.Return();
        var box = library.AddGenericValueType("Example", "Box", ["T"]);
        var t = SignatureType.TypeParameter(0); var field = box.AddField("Value", t, FieldVisibility.Public);
        var init = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        init.LoadArgument(0); init.LoadArgument(1); init.StoreField(field); init.Return();
        var get = box.AddInstanceMethod("Get", new MethodSignature(t, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        var referenceType = library.AddClass("Example", "ReferenceBox");
        var referenceField = referenceType.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var referenceConstructor = referenceType.AddConstructor([PrimitiveType.Int32]);
        referenceConstructor.LoadArgument(0); referenceConstructor.LoadArgument(1); referenceConstructor.StoreField(referenceField); referenceConstructor.Return();
        var referenceRead = referenceType.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        referenceRead.LoadArgument(0); referenceRead.LoadField(referenceField); referenceRead.Return();
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        var app = new AssemblyBuilder(new("ValueConstructorApp", new Version(1, 0, 0, 0)), core);
        var numberType = projection.MainModule.Types.Single(t => t.Name == "Number");
        var numberCtor = app.ImportReference(numberType.Methods.Single(m => m.Name == ".ctor"), core);
        var numberRead = app.ImportReference(numberType.Methods.Single(m => m.Name == "Read"), core);
        var boxType = projection.MainModule.Types.Single(t => t.Name == "Box`1");
        var boxCtor = app.ImportReference(boxType.Methods.Single(m => m.Name == ".ctor"), core).MakeConstructedReference([PrimitiveType.Int32]);
        var boxRead = app.ImportReference(boxType.Methods.Single(m => m.Name == "Get"), core).MakeConstructedReference([PrimitiveType.Int32]);
        if (!numberCtor.IsConstructor || !boxCtor.Definition.IsConstructor) throw new Exception("constructor identity lost");
        var referenceDefinition = projection.MainModule.Types.Single(t => t.Name == "ReferenceBox");
        var importedReferenceConstructor = app.ImportReference(referenceDefinition.Methods.Single(m => m.Name == ".ctor"), core);
        var importedReferenceRead = app.ImportReference(referenceDefinition.Methods.Single(m => m.Name == "Read"), core);
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(app.ImportReference(numberType, core)); var boxed = main.DeclareLocal(boxCtor.DeclaringType);
        main.LoadConstant(42); main.NewObject(numberCtor); main.StoreLocal(local);
        main.LoadLocalAddress(local); main.Call(numberRead); main.NewObject(boxCtor); main.StoreLocal(boxed);
        main.LoadLocalAddress(boxed); main.Call(boxRead); main.Emit(OpCode.Newobj, importedReferenceConstructor); main.Call(importedReferenceRead); main.Return();
        try { main.Call(numberCtor); throw new Exception("ordinary constructor call admitted"); } catch (ArgumentException) { }
        return (library, app, constructor, value);
    }
    internal static void Run()
    {
        var (library, app, constructor, field) = Create();
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
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory); var (library, app, _, _) = Create();
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
