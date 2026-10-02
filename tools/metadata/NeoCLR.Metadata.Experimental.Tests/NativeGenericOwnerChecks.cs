using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeGenericOwnerChecks
{
    private static (AssemblyBuilder Library, AssemblyBuilder App, byte[] Image) Create(bool binary)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeOwnerLibrary", new Version(1, 0, 0, 0)), core);
        var type = library.AddGenericClass("Example", "Box", ["TItem"]);
        var parameter = SignatureType.TypeParameter(0);
        var field = type.AddField("Value", parameter);
        var ctor = type.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(field); ctor.Return();
        var get = type.AddInstanceMethod("Get", new MethodSignature(parameter, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        var set = type.AddInstanceMethod("Set", new MethodSignature(PrimitiveType.Void, [parameter]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(field); set.Return();
        var array = type.AddMethod("ArrayIdentity", new MethodSignature(SignatureType.ArrayOf(parameter), [SignatureType.ArrayOf(parameter)]));
        array.LoadArgument(0); array.Return();
        SignatureType closed = type.MakeGenericInstance(PrimitiveType.Int32);
        var factoryType = library.AddType("Example", "Factory");
        var factory = factoryType.AddMethod("Create", new MethodSignature(closed, [PrimitiveType.Int32]));
        factory.LoadArgument(0); factory.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32])); factory.Return();
        var echo = factoryType.AddMethod("Echo", new MethodSignature(closed, [closed]));
        echo.LoadArgument(0); echo.Return();
        var openType = type.MakeGenericInstance(SignatureType.MethodParameter(0));
        var openEcho = factoryType.AddMethod("Open", new MethodSignature(openType, [openType], ["T"]));
        openEcho.LoadArgument(0); openEcho.Return();
        var selfType = type.MakeGenericInstance(parameter);
        var same = type.AddInstanceMethod("Same", new MethodSignature(selfType, [selfType]));
        same.LoadArgument(1); same.Return();
        var storage = library.AddClass("Example", "Storage");
        var storedBox = storage.AddField("Box", closed, FieldVisibility.Public);
        var storedBoxes = storage.AddField("Boxes", SignatureType.ArrayOf(closed), FieldVisibility.Public);
        var storageCtor = storage.AddConstructor(new MethodSignature(PrimitiveType.Void, [closed, SignatureType.ArrayOf(closed)]));
        storageCtor.LoadArgument(0); storageCtor.LoadArgument(1); storageCtor.StoreField(storedBox);
        storageCtor.LoadArgument(0); storageCtor.LoadArgument(2); storageCtor.StoreField(storedBoxes); storageCtor.Return();
        var native = library.WriteNativeAssembly();
        var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
        var read = AssemblyDefinition.ReadNativeAssembly(image);
        var definition = read.MainModule.Types.Single(t => t.Name == "Box`1");
        Check(definition.Name == "Box`1" && definition.GenericArity == 1 && definition.GenericParameterNames!.SequenceEqual(new[] { "TItem" }), "owner identity and parameter names");
        Reject<NotSupportedException>(() => ((IList<string>)definition.GenericParameterNames!)[0] = "Changed");
        Check(definition.SpecialConstraints.Count == 0 && definition.GenericConstraints.Count == 0, "unconstrained owner");
        Check(definition.Fields.Single().TryGetSignature(out var fieldSignature) && fieldSignature!.TypeParameterIndex == 0, "type parameter field");
        Check(definition.Methods.Single(m => m.Name == "Get").TryGetSignature(out var signature) && signature!.ReturnType.TypeParameterIndex == 0, "type parameter return");
        Check(definition.Methods.Single(m => m.Name == "ArrayIdentity").TryGetSignature(out var vector) && vector!.ReturnType.ArrayElement!.TypeParameterIndex == 0, "type parameter vector");
        Check(read.Write().SequenceEqual(image), "opaque generic owner roundtrip");
        var readFactory = read.MainModule.Types.Single(t => t.Name == "Factory");
        var createDefinition = readFactory.Methods.Single(m => m.Name == "Create");
        var echoDefinition = readFactory.Methods.Single(m => m.Name == "Echo");
        Check(createDefinition.TryGetSignature(out var createSignature) && echoDefinition.TryGetSignature(out var echoSignature) &&
            createSignature!.ReturnType == echoSignature!.ParameterTypes[0] &&
            ReferenceEquals(createSignature.ReturnType.ReferencedGenericInstance!.Definition.Resolve(), definition) &&
            createSignature.ReturnType.ReferencedGenericInstance.TypeArguments[0].Primitive == PrimitiveType.Int32,
            "structural closed signature identity and canonical definition");
        Reject<NotSupportedException>(() => ((IList<SignatureType>)createSignature!.ReturnType.ReferencedGenericInstance!.TypeArguments)[0] = PrimitiveType.Boolean);
        var app = new AssemblyBuilder(new("NativeOwnerApp", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => app.AddFunction("Unimported", createSignature!));
        Check(!createDefinition.TryGetStaticValueSignature(out _) && !createDefinition.TryGetStaticPrimitiveSignature(out _), "closed construction is not a primitive signature");
        var openDefinition = readFactory.Methods.Single(m => m.Name == "Open");
        Check(openDefinition.TryGetSignature(out var openSignature) && openSignature!.ReturnType.ReferencedGenericInstance!.TypeArguments[0].MethodParameterIndex == 0,
            "method-scoped constructed argument");
        Check(definition.Methods.Single(m => m.Name == "Same").TryGetSignature(out var sameSignature) &&
            sameSignature!.ReturnType.ReferencedGenericInstance!.TypeArguments[0].TypeParameterIndex == 0, "owner-scoped constructed argument");
        var authoredType = app.CreateTypeReference(library.Identity, core,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), definition.Namespace, definition.Name, 1);
        Check(ReferenceEquals(authoredType, app.ImportReference(definition, core)), "authored/read type identity agreement");
        var importedType = authoredType.MakeGenericInstance(PrimitiveType.Int32);
        ImportedConstructedMethodReference Import(string name)
        {
            var signature = name switch
            {
                ".ctor" => new MethodSignature(PrimitiveType.Void, [parameter]),
                "Get" => new MethodSignature(parameter, []),
                "Set" => new MethodSignature(PrimitiveType.Void, [parameter]),
                "Same" => new MethodSignature(authoredType.MakeGenericInstance(parameter), [authoredType.MakeGenericInstance(parameter)]),
                _ => throw new Exception("unexpected authored member " + name)
            };
            return app.CreateMethodReference(authoredType, name, signature).MakeConstructedReference([PrimitiveType.Int32]);
        }
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        IILGenerator main = entry.Definition.GetILGenerator();
        var local = main.DeclareLocal(importedType);
        main.LoadConstant(19); main.Call(app.ImportReference(createDefinition, core)); main.Emit(OpCode.Pop); main.LoadConstant(19); main.NewObject(Import(".ctor")); main.Call(app.ImportReference(echoDefinition, core)); main.Call(app.ImportReference(openDefinition, core).MakeGenericInstance(PrimitiveType.Int32)); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadConstant(42); main.Call(Import("Set"));
        var storageReference = app.CreateTypeReference(library.Identity, core,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "Example", "Storage");
        var boxField = app.CreateFieldReference(storageReference, "Box", importedType, 0);
        var boxesField = app.CreateFieldReference(storageReference, "Boxes", SignatureType.ArrayOf(importedType), 1);
        var storageConstructor = app.CreateMethodReference(storageReference, ".ctor",
            new MethodSignature(PrimitiveType.Void, [importedType, SignatureType.ArrayOf(importedType)]));
        var storageLocal = main.DeclareLocal(storageReference);
        main.LoadLocal(local); main.LoadConstant(1); main.NewArray(importedType);
        main.NewObject(storageConstructor); main.StoreLocal(storageLocal);
        main.LoadLocal(storageLocal); main.LoadField(boxesField); main.LoadConstant(0); main.LoadLocal(local); main.StoreArrayElement(importedType);
        main.LoadLocal(storageLocal); main.LoadLocal(storageLocal); main.LoadField(boxesField); main.LoadConstant(0); main.LoadArrayElement(importedType); main.StoreField(boxField);
        main.LoadLocal(storageLocal); main.LoadField(boxField); main.LoadLocal(local); main.Call(Import("Same")); main.Call(Import("Get")); main.Return();
        _ = app.WriteNativeAssembly();
        return (library, app, image);
    }
    internal static void Run()
    {
        foreach (var binary in new[] { false, true })
        {
            var (library, app, _) = Create(binary);
            var context = new AssemblyLoadContext("native-owner-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check(Equals(42, context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)), "CLR constructed native imports");
            }
            finally { context.Unload(); }
            library.Types.Single(t => t.Name == "Box`1").SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
            Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary)));

        }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        foreach (var binary in new[] { false, true })
        {
            var (_, app, image) = Create(binary);
            var dependency = Path.Combine(directory, "Library-" + binary + ".dll");
            var path = Path.Combine(directory, "App-" + binary + ".dll");
            File.WriteAllBytes(dependency, image);
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (var arg in new[] { command, path, "--module", dependency }) start.ArgumentList.Add(arg);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
                try { await process.WaitForExitAsync(timeout.Token); }
                catch { process.Kill(entireProcessTree: true); throw; }
                var text = await stdout + await stderr;
                Check(process.ExitCode == (command == "verify" ? 0 : 42), command + ": " + text);
            }
        }
        Console.WriteLine("Native generic class import, construction, mutation and execution: 42 (both containers)");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
