using System.Diagnostics;
using System.Runtime.Loader;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ImportedTypeChecks
{
    private static (AssemblyBuilder Library, AssemblyBuilder App) Build()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ExternalTypes", new Version(1, 0, 0, 0)), core);
        library.AddGenericClass("Collections", "Box", ["T"]);
        library.AddInterface("Collections", "View");
        library.AddType("Collections", "Static");
        library.AddClass("Collections", "Hidden", TypeVisibility.Internal);
        library.AddGenericClass("Collections", "Constrained", ["T"]).SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), false);
        var app = new AssemblyBuilder(new("TypeConsumer", new Version(1, 0, 0, 0)), core);
        var boxDefinition = snapshot.MainModule.Types.Single(t => t.Name == "Box`1");
        var box = app.ImportReference(boxDefinition, core);
        Check(ReferenceEquals(box, app.ImportReference(boxDefinition, core)), "definition reference interned");
        var order = app.AddClass("Example", "Order");
        SignatureType[] arguments = [order];
        var constructed = box.MakeGenericInstance(arguments); arguments[0] = PrimitiveType.Int32;
        Check(constructed.Equals(box.MakeGenericInstance(order)) && constructed.GetHashCode() == box.MakeGenericInstance(order).GetHashCode(), "structural construction identity and argument copy");
        Check(!constructed.Equals(box.MakeGenericInstance(PrimitiveType.Int32)), "distinct arguments");
        var view = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "View"), core);
        order.AddField("View", view);
        order.AddField("Box", constructed);
        var accept = app.AddType("Example", "Consumer").AddMethod("Accept", new MethodSignature(PrimitiveType.Int32, [(SignatureType)constructed]));
        accept.LoadConstant(42); accept.Return();
        var forwardType = box.MakeGenericInstance(SignatureType.MethodParameter(0));
        var forward = app.AddFunction("Forward", new MethodSignature(forwardType, [(SignatureType)forwardType], ["T"]));
        forward.LoadArgument(0); forward.Return();
        var holder = app.AddGenericClass("Example", "Holder", ["T"]);
        var ownerType = box.MakeGenericInstance(SignatureType.TypeParameter(0));
        var field = holder.AddField("Value", ownerType);
        Check(field.MakeConstructedReference(order).FieldType.ImportedType!.Equals(constructed), "field owner substitution");
        var echo = holder.AddMethod("Echo", new MethodSignature(ownerType, [(SignatureType)ownerType]));
        echo.LoadArgument(0); echo.Return();
        var boundEcho = echo.MakeConstructedReference([(SignatureType)order]);
        Check(boundEcho.Signature.ReturnType.ImportedType!.Equals(constructed), "method owner substitution");
        var entry = app.AddFunction("Main");
        entry.LoadDefault(view); entry.Emit(OpCode.Pop);
        entry.LoadDefault(constructed); entry.Emit(OpCode.Pop);
        entry.LoadConstant(1); entry.NewArray(constructed); entry.LoadConstant(0); entry.LoadArrayElement(constructed);
        entry.Call(forward.MakeGenericInstance(order)); entry.Call(boundEcho); entry.Call(accept); entry.Return(); app.EntryPoint = entry;
        foreach (var unsupported in new[] { "Static", "Hidden", "Constrained`1" })
            Reject<InvalidDataException>(() => app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == unsupported), core));
        Reject<ArgumentNullException>(() => app.ImportReference((TypeDefinition)null!, core));
        Reject<ArgumentException>(() => { SignatureType open = box; });
        Reject<ArgumentException>(() => box.MakeGenericInstance());
        Reject<ArgumentException>(() => box.MakeGenericInstance(PrimitiveType.Void));
        Reject<ArgumentException>(() => constructed.MakeGenericInstance(order));
        Reject<ArgumentNullException>(() => box.MakeGenericInstance(null!));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => foreign.AddFunction("Use", new MethodSignature(PrimitiveType.Void, [(SignatureType)constructed])));
        Reject<InvalidDataException>(() => app.ImportReference(boxDefinition, new("WrongCore", new Version(1, 0, 0, 0))));
        var replacement = new AssemblyBuilder(library.Identity, core); replacement.AddGenericClass("Collections", "Box", ["T"]);
        Reject<InvalidDataException>(() => app.ImportReference(AssemblyDefinition.ReadAssembly(replacement.Write(), false).MainModule.Types.Single(t => t.Name == "Box`1"), core));
        return (library, app);
    }
    internal static void Run()
    {
        var (library, app) = Build();
        var context = new AssemblyLoadContext("imported-types", isCollectible: true);
        try
        {
            var dependency = context.LoadFromStream(new MemoryStream(library.Write()));
            var consumer = context.LoadFromStream(new MemoryStream(app.Write()));
            Check((int)consumer.EntryPoint!.Invoke(null, null)! == 42, "CLR executes external generic signature");
            var parameter = consumer.GetType("Example.Consumer")!.GetMethod("Accept")!.GetParameters()[0].ParameterType;
            Check(parameter.GetGenericTypeDefinition().Assembly == dependency && parameter.GetGenericArguments()[0].Assembly == consumer, "TypeRef/GENERICINST scopes preserved");
        }
        finally { context.Unload(); }
        var typeFixture = AssemblyDefinition.ReadAssembly(ValueTypeFixture(), false);
        foreach (var valueType in typeFixture.MainModule.Types.Where(t => t.Name is "Number" or "Kind"))
            Reject<InvalidDataException>(() => app.ImportReference(valueType, app.CoreLibrary));
        var binary = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary);
        var projected = RuntimeAssemblyContainer.ReadCliProjection(binary);
        Check(projected.MainModule.AssemblyReferences.Any(r => r.Identity.Equals(library.Identity)), "native projection retains external signature dependency");
        Check(projected.MainModule.TypeReferences.Any(t => t.Name == "Box`1") && projected.MainModule.TypeReferences.Any(t => t.Name == "View"), "native projection retains external type references");
        var malformed = System.Text.Json.Nodes.JsonNode.Parse(app.WriteNativeAssembly())!;
        malformed["types"]![0]!["fields"]![1]!["ty"]!["Constructed"]!["arguments"] = new System.Text.Json.Nodes.JsonArray();
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        malformed = System.Text.Json.Nodes.JsonNode.Parse(app.WriteNativeAssembly())!;
        malformed["references"] = new System.Text.Json.Nodes.JsonArray();
        malformed["assemblies"]![0]!["references"] = new System.Text.Json.Nodes.JsonArray();
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var (library, app) = Build();
        var libraryPath = Path.Combine(output, "ExternalTypes.dll"); var appPath = Path.Combine(output, "TypeConsumer.dll");
        File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var (command, expected) in new[] { ("verify", 0), ("run", 42) })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, appPath, "--module", libraryPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout; var error = await stderr;
            if (process.ExitCode != expected) throw new Exception(command + ": " + process.ExitCode + " " + text + error);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new { verified = true, result = 42,
            scope = "API-produced external reference/interface and Box<owned Order> signatures, fields, default locals and CLI projection; no imported member dispatch or translated-System mapping" }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    }
    private static byte[] ValueTypeFixture()
    {
        var metadata = new System.Reflection.Metadata.Ecma335.MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Fixture.dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Fixture"), new Version(1, 0, 0, 0), default, default, 0, System.Reflection.AssemblyHashAlgorithm.None);
        var firstField = System.Reflection.Metadata.Ecma335.MetadataTokens.FieldDefinitionHandle(1);
        var firstMethod = System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1);
        metadata.AddTypeDefinition(0, default, metadata.GetOrAddString("<Module>"), default, firstField, firstMethod);
        var valueBase = metadata.AddTypeDefinition(System.Reflection.TypeAttributes.Public, metadata.GetOrAddString("System"), metadata.GetOrAddString("ValueType"), default, firstField, firstMethod);
        metadata.AddTypeDefinition(System.Reflection.TypeAttributes.Public, metadata.GetOrAddString("Example"), metadata.GetOrAddString("Number"), valueBase, firstField, firstMethod);
        var core = metadata.AddAssemblyReference(metadata.GetOrAddString("Core"), new Version(1, 0, 0, 0), default, default, 0, default);
        var enumBase = metadata.AddTypeReference(core, metadata.GetOrAddString("System"), metadata.GetOrAddString("Enum"));
        metadata.AddTypeDefinition(System.Reflection.TypeAttributes.Public, metadata.GetOrAddString("Example"), metadata.GetOrAddString("Kind"), enumBase, firstField, firstMethod);
        var pe = new System.Reflection.PortableExecutable.ManagedPEBuilder(new System.Reflection.PortableExecutable.PEHeaderBuilder(),
            new System.Reflection.Metadata.Ecma335.MetadataRootBuilder(metadata), new System.Reflection.Metadata.BlobBuilder(), strongNameSignatureSize: 0);
        var image = new System.Reflection.Metadata.BlobBuilder(); pe.Serialize(image); return image.ToArray();
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
