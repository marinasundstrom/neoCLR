using System.Diagnostics;
using System.Text.Json;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using TypeDefinition = NeoCLR.Metadata.Experimental.Model.TypeDefinition;
using FieldDefinition = NeoCLR.Metadata.Experimental.Model.FieldDefinition;

internal static class AuthoredDefinitionChecks
{
    internal static AssemblyDefinition Create()
    {
        var coreName = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(coreName.Name!, coreName.Version!, "", Convert.ToHexString(coreName.GetPublicKeyToken()!));
        var assembly = AssemblyDefinition.CreateAssembly(new("ManualStruct", new Version(1, 0, 0, 0)), core);
        var module = assembly.MainModule;
        var valueBase = module.ImportReference(core, "System", "ValueType");
        var type = new TypeDefinition("Example", "MyStruct", (uint)(TypeAttributes.Public | TypeAttributes.SequentialLayout | TypeAttributes.Sealed), valueBase);
        var field = new FieldDefinition("Original", (ushort)FieldAttributes.Public, PrimitiveType.Int32);
        type.Fields.Add(field);
        module.Types.Add(type);
        field.Name = "MyField";
        return assembly;
    }
    internal static void Run()
    {
        var executable = Executable();
        var executionContext = new AssemblyLoadContext("manual-execution", isCollectible: true);
        try
        {
            if ((int)executionContext.LoadFromStream(new MemoryStream(executable.Write())).EntryPoint!.Invoke(null, null)! != 42)
                throw new Exception("manual definition CLR execution");
        }
        finally { executionContext.Unload(); }
        var assembly = Create();
        var type = assembly.MainModule.Types.Single(); var field = type.Fields.Single();
        var builder = AssemblyBuilder.ForDefinition(assembly);
        if (!ReferenceEquals(assembly.MainModule.Fields.Single(), field)) throw new Exception("module field view");
        var entry = executable.EntryPoint!;
        if (!ReferenceEquals(entry, executable.MainModule.Functions.Single(m => m.Name == "Main")) ||
            !ReferenceEquals(entry, executable.MainModule.Methods.Single(m => m.Name == "Main")) || entry.DeclaringType is not null ||
            !ReferenceEquals(entry.AuthoredSignature, AssemblyBuilder.ForDefinition(executable).EntryPoint!.Signature))
            throw new Exception("authored function definition identity");
        Reject<InvalidOperationException>(() => entry.GetSignature());
        Reject<ArgumentException>(() => assembly.EntryPoint = entry);
        Reject<NotSupportedException>(() => executable.MainModule.Functions.Clear());
        var writtenEntry = AssemblyDefinition.ReadAssembly(executable.Write(), false).EntryPoint!;
        if (entry.Attributes != writtenEntry.Attributes || entry.Name != writtenEntry.Name) throw new Exception("method declaration round trip");
        if (!ReferenceEquals(builder.Definition, assembly) || !ReferenceEquals(builder.Types[0].Definition, type) ||
            !ReferenceEquals(builder.Types[0].Fields[0].Definition, field) || builder.Types[0].Fields[0].Name != "MyField" ||
            !ReferenceEquals(type.ToReference().Resolve(), type) || !ReferenceEquals(field.DeclaringType, type))
            throw new Exception("definition/facade identity");
        var second = builder.Types[0].AddField("Second", PrimitiveType.Int64, FieldVisibility.Public);
        if (!ReferenceEquals(type.Fields[1], second.Definition)) throw new Exception("builder adds canonical definition");
        second.Definition.Name = "Renamed";
        Reject<ArgumentException>(() => second.Definition.Name = "MyField");
        Reject<ArgumentException>(() => assembly.MainModule.Types.Add(type));
        Reject<ArgumentException>(() => type.Fields.Add(field));
        Reject<NotSupportedException>(() => type.Fields.Remove(field));
        var foreign = AssemblyDefinition.CreateAssembly(new("Other", new Version(1, 0, 0, 0)), builder.CoreLibrary);
        Reject<ArgumentException>(() => foreign.MainModule.Types.Add(type));
        var scoped = new MethodDefinition("Scoped", new MethodSignature(builder.Types[0], Array.Empty<SignatureType>()));
        Reject<ArgumentException>(() => foreign.MainModule.Functions.Add(scoped));
        Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(scoped));
        assembly.MainModule.Functions.Add(scoped);
        var scopedBody = MethodBuilder.ForDefinition(scoped);
        scopedBody.LoadDefault(builder.Types[0]); scopedBody.Return();
        Reject<ArgumentException>(() => assembly.MainModule.Functions.Add(new MethodDefinition("Scoped", scoped.AuthoredSignature!)));
        var method = builder.Types[0].AddMethod("get_Answer");
        method.LoadConstant(42); method.Return();
        builder.Types[0].AddProperty("Answer", PrimitiveType.Int32, getter: method);
        var contract = builder.AddInterface("Example", "IAnswer");
        var required = contract.AddInterfaceMethod("get_Answer", PrimitiveMethodSignature.Int32(0, true));
        contract.AddProperty("Answer", PrimitiveType.Int32, getter: required);
        if (!ReferenceEquals(type.Methods.Single(), method.Definition) ||
            !ReferenceEquals(method.Definition.DeclaringType, type) ||
            !ReferenceEquals(method.Definition.Module, assembly.MainModule)) throw new Exception("type method definition identity");
        var image = assembly.Write();
        if (!image.SequenceEqual(builder.Write())) throw new Exception("facade writer differs");
        var context = new AssemblyLoadContext("manual-definitions", isCollectible: true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(image)).GetType("Example.MyStruct")!;
            if (!loaded.IsValueType || loaded.GetField("MyField")!.FieldType != typeof(int) || loaded.GetField("Renamed")!.FieldType != typeof(long))
                throw new Exception("manual CLI shape");
        }
        finally { context.Unload(); }
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        foreach (var declared in assembly.MainModule.Methods)
        {
            var physical = snapshot.MainModule.Methods.Single(m => m.Name == declared.Name && m.DeclaringType?.Name == declared.DeclaringType?.Name);
            if (declared.Attributes != physical.Attributes || declared.GenericArity != physical.GenericArity || declared.IsStatic != physical.IsStatic)
                throw new Exception("authored method flags differ from encoding");
        }
        Reject<InvalidOperationException>(() => snapshot.EntryPoint = null);
        Reject<InvalidOperationException>(() => snapshot.MainModule.Functions.Add(new MethodDefinition("New", PrimitiveMethodSignature.Int32(0, true))));
        Reject<InvalidOperationException>(() => snapshot.MainModule.Types.Add(type));
        Reject<InvalidOperationException>(() => snapshot.MainModule.Types.Single(t => t.Name == "MyStruct").Fields[0].Name = "Changed");
        Reject<InvalidOperationException>(() => AssemblyBuilder.ForDefinition(snapshot));
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), builder.CoreLibrary));
        if (!projection.MainModule.Types.Single(t => t.Name == "MyStruct").IsValueType) throw new Exception("native category");
    }
    private static AssemblyDefinition Executable()
    {
        var assembly = Create(); var builder = AssemblyBuilder.ForDefinition(assembly);
        var type = builder.Types.Single(); var field = type.Fields.Single();
        var helperDefinition = new MethodDefinition("Answer", PrimitiveMethodSignature.Int32(0, true), @namespace: "Example");
        Reject<InvalidOperationException>(() => MethodBuilder.ForDefinition(helperDefinition));
        assembly.MainModule.Functions.Add(helperDefinition);
        Reject<ArgumentException>(() => assembly.MainModule.Functions.Add(helperDefinition));
        var helper = MethodBuilder.ForDefinition(helperDefinition);
        helper.LoadConstant(42); helper.Return();
        var entryDefinition = new MethodDefinition("Main", PrimitiveMethodSignature.Int32(0, true));
        assembly.MainModule.Functions.Add(entryDefinition);
        assembly.EntryPoint = entryDefinition;
        var entry = MethodBuilder.ForDefinition(entryDefinition);
        var local = entry.DeclareLocal(type);
        entry.LoadDefault(type); entry.StoreLocal(local);
        entry.LoadLocalAddress(local); entry.Call(helper); entry.StoreField(field);
        entry.LoadLocal(local); entry.LoadField(field); entry.Return();
        return assembly;
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var assembly = Executable(); var builder = AssemblyBuilder.ForDefinition(assembly);
        var path = Path.Combine(output, "ManualStruct.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), builder.CoreLibrary));
        foreach (var (command, expected) in new[] { ("verify", 0), ("run", 42) })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout; var error = await stderr;
            if (process.ExitCode != expected) throw new Exception(command + ": " + process.ExitCode + " " + text + error);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new { verified = true, result = 42,
            scope = "manual assembly/type/field/function construction and entry-point assignment; helper call through same definitions in body builders; native write/load/execute; type-method construction, body definition migration and loaded editing remain pending" }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
