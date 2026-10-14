using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class StructuralMethodConstraintChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("StructuralBounds", new Version(1, 0, 0, 0)), core);
        var a = graph.AddInterface("Example", "A");
        var b = graph.AddInterface("Example", "B");
        var finalClass = graph.AddClass("Example", "Final");
        finalClass.SetSealedClass();
        graph.AddClosedClass("Example", "Family");
        var answer = graph.AddFunction("Answer", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        answer.AddInterfaceConstraint(0, a);
        answer.LoadConstant(42);
        answer.Return();
        var main = graph.AddFunction("Main");
        main.LoadConstant(42);
        main.Return();
        graph.EntryPoint = main;
        var original = JsonNode.Parse(graph.WriteNativeAssembly())!;
        var nominal = original["functions"]![0]!["generic_constraints"]![0]!["kind"]!["TypeBound"]!.DeepClone();
        var other = new JsonObject { ["Named"] = original["types"]![1]!["name"]!.DeepClone() };
        JsonNode Compose(string kind, params JsonNode[] operands) => new JsonObject { [kind] = new JsonArray(operands.Select(o => o.DeepClone()).ToArray()) };
        byte[] Image(JsonNode bound, bool binary = false, int ordinal = 0, bool duplicate = false)
        {
            var tree = original.DeepClone();
            var constraints = tree["functions"]![0]!["generic_constraints"]!.AsArray();
            constraints[0]!["kind"]!["TypeBound"] = bound.DeepClone();
            constraints[0]!["parameter"] = ordinal;
            if (duplicate) constraints.Add(constraints[0]!.DeepClone());
            var json = Encoding.UTF8.GetBytes(tree.ToJsonString());
            return binary ? NativeModuleContainer.WriteLibraryBinary(json) : NativeModuleContainer.WriteBinary(json);
        }
        foreach (var kind in new[] { "Union", "Intersection" })
        {
            var bound = Compose(kind, nominal, other);
            foreach (var binary in new[] { false, true })
            {
                var image = Image(bound, binary);
                Reject(() => AssemblyDefinition.ReadNativeAssembly(image));
                var snapshot = AssemblyDefinition.ReadNativeAssembly(image, true);
                Check(snapshot.Write().SequenceEqual(image), "original image preserved");
                var method = snapshot.MainModule.Functions.Single(m => m.Name == "Answer");
                var constraint = method.TypeConstraints.Single();
                Check(constraint.ParameterIndex == 0 && constraint.Bound.Composition!.Kind.ToString() == kind, "complete structural bound");
                Check(constraint.Bound.Composition!.Constituents[0].ReferencedType!.Resolve().Name == "A", "snapshot-owned operand");
                var context = new MetadataLoadContext([snapshot]);
                var parameter = (MethodGenericParameterTypeInfo)context.Resolve(method).GetGenericArguments().Single();
                var view = (ReferenceCompositionTypeInfo)parameter.GetTypeConstraints().Single();
                Check(view.Constituents.Count == 2 && ReferenceEquals(view, context.ResolveSignature(constraint.Bound)), "canonical bound view");
                Reject(() => _ = method.InterfaceConstraints);
                Reject(() => parameter.GetInterfaceConstraints());
                var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
                Reject(() => consumer.ImportReference(method, core));
                Reject(() => consumer.ImportExperimentalCompositionReference(method, core));
                Reject(() => RuntimeAssemblyContainer.Write(NativeModuleContainer.Read(image), core));
            }
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(bound, ordinal: -1), true));
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(bound, ordinal: 1), true));
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(bound, duplicate: true), true));
            foreach (var invalid in new JsonNode[] { JsonValue.Create("Int32")!, new JsonObject { ["MethodParameter"] = 0 }, new JsonObject { ["Named"] = "Missing" } })
                Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(Compose(kind, nominal, invalid)), true));
            var mixed = Compose(kind, nominal, Compose(kind == "Union" ? "Intersection" : "Union", nominal, other));
            var mixedRead = AssemblyDefinition.ReadNativeAssembly(Image(mixed), true);
            Check(mixedRead.MainModule.Functions[0].TypeConstraints[0].Bound.Composition!.Constituents[1].Composition is not null, "mixed grouping retained");
            var collapsed = AssemblyDefinition.ReadNativeAssembly(Image(Compose(kind, nominal, nominal)), true);
            Check(collapsed.MainModule.Functions[0].TypeConstraints[0].Bound.ReferencedType!.Name == "A", "duplicate normalization");
            Reject(() => _ = collapsed.MainModule.Functions[0].InterfaceConstraints);
        }
        JsonNode Named(int index) => new JsonObject { ["Named"] = original["types"]![index]!["name"]!.DeepClone() };
        Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(Compose("Intersection", nominal, Named(2))), true));
        _ = AssemblyDefinition.ReadNativeAssembly(Image(Compose("Union", nominal, Named(2))), true);
        _ = AssemblyDefinition.ReadNativeAssembly(Image(Compose("Intersection", nominal, Named(3))), true);
        var nominalRead = AssemblyDefinition.ReadNativeAssembly(Image(nominal));
        var nominalMethod = nominalRead.MainModule.Functions[0];
        Check(nominalMethod.InterfaceConstraints.Count == 1 && nominalMethod.TypeConstraints.Count == 1, "nominal API compatibility");
        Console.WriteLine("PASS structural method constraint metadata and introspection");
    }

    private static void Check(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is InvalidDataException or ArgumentException or NotSupportedException) { return; }
        throw new Exception("expected explicit rejection");
    }
}
