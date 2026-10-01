using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class SpecialConstraintChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = TypeConstraintChecks.Create();
        foreach (var (name, flags) in new[] {
            ("ReferenceOnly", TypeParameterConstraints.ReferenceType),
            ("ValueOnly", TypeParameterConstraints.ValueType | TypeParameterConstraints.DefaultConstructor),
            ("Constructible", TypeParameterConstraints.DefaultConstructor) })
        {
            var owner = graph.AddGenericType("Example", name, ["T"]); owner.SetSpecialConstraints(0, flags);
            var identity = owner.AddMethod("Identity", new MethodSignature(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0)]));
            identity.LoadArgument(0); identity.Return();
        }
        var main = graph.EntryPoint!; main.ClearBody();
        main.LoadConstant(42); main.Call(graph.Types.Single(t => t.Name == "ValueOnly`1").Methods[0].MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create(); var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("value constraint execution");
        var reference = loaded.GetType("Example.ReferenceOnly`1")!;
        var value = loaded.GetType("Example.ValueOnly`1")!;
        var ctor = loaded.GetType("Example.Constructible`1")!;
        reference.MakeGenericType(typeof(string)); value.MakeGenericType(typeof(int)); ctor.MakeGenericType(typeof(int));
        ctor.MakeGenericType(loaded.GetType("Example.Bound")!);
        foreach (var (owner, argument) in new[] { (reference, typeof(int)), (value, typeof(string)), (ctor, typeof(string)) })
            try { owner.MakeGenericType(argument); throw new Exception("invalid CLI special argument"); } catch (ArgumentException) { }
        var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        using var pe = new System.Reflection.PortableExecutable.PEReader(new MemoryStream(projected));
        var reader = System.Reflection.Metadata.PEReaderExtensions.GetMetadataReader(pe);
        foreach (var handle in reader.TypeDefinitions)
        {
            var type = reader.GetTypeDefinition(handle); var name = reader.GetString(type.Name);
            if (name is not ("ReferenceOnly`1" or "ValueOnly`1" or "Constructible`1")) continue;
            var expected = loaded.GetType("Example." + name)!.GetGenericArguments()[0].GenericParameterAttributes;
            if (reader.GetGenericParameter(type.GetGenericParameters().Single()).Attributes != expected) throw new Exception("special flag projection");
        }
        var specialOwner = graph.Types.Single(t => t.Name == "ReferenceOnly`1");
        foreach (var flags in new[] { (TypeParameterConstraints)1, TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType })
        {
            try { specialOwner.SetSpecialConstraints(0, flags); throw new Exception("invalid special flags accepted"); }
            catch (ArgumentException) { }
        }
        foreach (var malformedKind in new[] { "ReferenceType", "ValueType", "Unknown" })
        {
            var malformed = System.Text.Json.Nodes.JsonNode.Parse(graph.WriteNativeAssembly())!;
            var types = malformed["types"]!.AsArray();
            var row = types.Single(t => t!["generic_constraints"] is System.Text.Json.Nodes.JsonArray constraints &&
                constraints.Any(c => c!["kind"]!.ToJsonString() == "\"ReferenceType\""))!;
            row["generic_constraints"]!.AsArray().Add(new System.Text.Json.Nodes.JsonObject { ["parameter"] = 0, ["kind"] = malformedKind });
            try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("malformed special constraint accepted"); }
            catch (InvalidDataException) { }
        }
        var noCtor = graph.AddClass("Example", "NoConstructor");
        var box = graph.AddGenericClass("Example", "NeedsConstructor", ["T"]);
        box.SetSpecialConstraints(0, TypeParameterConstraints.DefaultConstructor);
        graph.EntryPoint!.DeclareLocal(box.MakeGenericInstance(noCtor));
        try { graph.Write(); throw new Exception("missing constructor accepted"); } catch (InvalidDataException) { }
    }
}
