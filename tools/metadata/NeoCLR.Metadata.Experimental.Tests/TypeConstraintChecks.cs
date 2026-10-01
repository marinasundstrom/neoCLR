using System.Diagnostics;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class TypeConstraintChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = GenericClassChecks.Create();
        var bound = graph.AddClass("Example", "Bound");
        var ctor = bound.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var owner = graph.AddGenericClass("Example", "Restricted", ["Element"]);
        owner.AddBaseTypeConstraint(0, bound);
        var t = SignatureType.TypeParameter(0);
        var field = owner.AddField("Value", t, FieldVisibility.Public);
        var create = owner.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        create.LoadArgument(0); create.LoadArgument(1); create.StoreField(field); create.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        main.NewObject(ctor); main.NewObject(create.MakeConstructedReference([bound]));
        main.LoadField(field.MakeConstructedReference(bound)); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create(); var image = graph.Write(); var loaded = Assembly.Load(image);
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("bounded class execution");
        var type = loaded.GetType("Example.Restricted`1")!;
        if (type.GetGenericArguments()[0].GetGenericParameterConstraints().Single().Name != "Bound") throw new Exception("CLI bound");
        try { type.MakeGenericType(typeof(int)); throw new Exception("CLI accepted invalid bound"); } catch (ArgumentException) { }
        void Reject(Action action) { try { action(); } catch (ArgumentException) { return; } throw new Exception("invalid type bound accepted"); }
        var owner = graph.Types.Last(); var bound = graph.Types[^2];
        Reject(() => owner.MakeGenericInstance(PrimitiveType.Int32));
        Reject(() => owner.AddBaseTypeConstraint(0, bound));
        Reject(() => owner.AddBaseTypeConstraint(1, bound));
        var late = Create(); var lateOwner = late.AddGenericClass("Example", "Late", ["T"]);
        late.EntryPoint!.DeclareLocal(lateOwner.MakeGenericInstance(PrimitiveType.Int32));
        lateOwner.AddBaseTypeConstraint(0, late.Types.Single(t => t.Name == "Bound"));
        try { late.Write(); throw new Exception("late constraint failed to invalidate existing use"); } catch (InvalidDataException) { }
        var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        using var pe = new PEReader(new MemoryStream(projected)); var reader = pe.GetMetadataReader();
        var definition = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(t => reader.GetString(t.Name) == "Restricted`1");
        var parameter = reader.GetGenericParameter(definition.GetGenericParameters().Single());
        var constraint = reader.GetGenericParameterConstraint(parameter.GetConstraints().Single());
        if (reader.GetString(reader.GetTypeDefinition((TypeDefinitionHandle)constraint.Type).Name) != "Bound") throw new Exception("projected type bound");
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        malformed["types"]!.AsArray().Last()!["generic_constraints"]![0]!["parameter"] = 1;
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("invalid bound ordinal"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); if (Directory.Exists(output)) throw new IOException("output must be fresh"); Directory.CreateDirectory(output);
        var graph = Create(); var path = Path.Combine(output, "TypeConstraints.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        // Corrupt the concrete constructor argument in the native input. Body references
        // are opaque to the declaration reader; the runtime must enforce the contract.
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        var main = malformed["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Main")!;
        var call = main["body"]!.AsArray().Single(i => i!["op"]!.GetValue<string>() == "newobj.ctor" && i["arg"]!["owner"]?["Constructed"] is not null)!;
        call["arg"]!["owner"]!["Constructed"]!["arguments"]![0] = "Int32";
        call["arg"]!["parameters"]![0] = "Int32";
        var invalidPath = Path.Combine(output, "InvalidTypeConstraint.dll");
        File.WriteAllBytes(invalidPath, RuntimeAssemblyContainer.WriteBinary(Encoding.UTF8.GetBytes(malformed.ToJsonString()), graph.CoreLibrary));
        var invalid = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
        invalid.ArgumentList.Add("verify"); invalid.ArgumentList.Add(invalidPath);
        using var verification = Process.Start(invalid)!;
        var invalidOut = verification.StandardOutput.ReadToEndAsync(); var invalidError = verification.StandardError.ReadToEndAsync();
        await verification.WaitForExitAsync(); var failure = await invalidOut + await invalidError;
        if (verification.ExitCode == 0 || !failure.Contains("constraint", StringComparison.OrdinalIgnoreCase))
            throw new Exception("native constraint rejection missing: " + failure);
        Console.WriteLine("PASS nominal type constraints: CLI/native binary verify/run 42 and invalid concrete bound rejected");
    }
}
