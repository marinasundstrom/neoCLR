using System.Reflection;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericMethodConstraintChecks
{
    internal static void Run()
    {
        foreach (var detached in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("MethodBounds" + detached, new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var bound = graph.AddInterface("Example", "Number");
            var implementation = graph.AddValueType("Example", "Count");
            implementation.AddInterfaceImplementation(bound);
            var signature = new MethodSignature(PrimitiveType.Int32, [], ["T"]);
            MethodBuilder method;
            if (detached)
            {
                var definition = new MethodDefinition("Answer", signature);
                definition.AddInterfaceConstraint(0, bound.Definition);
                graph.Definition.MainModule.Functions.Add(definition);
                method = MethodBuilder.ForDefinition(definition);
            }
            else
            {
                method = graph.AddFunction("Answer", signature);
                method.AddInterfaceConstraint(0, bound);
            }
            method.GetILGenerator().LoadConstant(42); method.GetILGenerator().Return();
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(0, bound));
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(1, bound));
            Reject<ArgumentException>(() => method.AddInterfaceConstraint(0, implementation));
            Reject<ArgumentException>(() => method.MakeGenericInstance(PrimitiveType.Int32));
            Reject<ArgumentException>(() => method.MakeGenericInstance(SignatureType.MethodParameter(0)));
            var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
            main.GetILGenerator().Call(method.MakeGenericInstance(implementation)); main.GetILGenerator().Return();
            graph.EntryPoint = main;
            var cli = graph.Write();
            var loaded = Assembly.Load(cli);
            if ((int)loaded.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("CLI bounded generic execution");
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
            foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, expectedExtended: false), AssemblyDefinition.ReadNativeAssembly(native), AssemblyDefinition.ReadAssembly(projected, expectedExtended: false) })
            {
                var definition = snapshot.MainModule.Functions.Single(m => m.Name == "Answer");
                if (definition.InterfaceConstraints.Single().InterfaceType.Name != "Number") throw new Exception("method constraint lost");
                var context = new MetadataLoadContext([snapshot]);
                var view = context.Resolve(definition);
                var parameter = (MethodGenericParameterTypeInfo)view.GetGenericArguments().Single();
                if (!ReferenceEquals(parameter.GetInterfaceConstraints().Single(), context.GetType(definition.InterfaceConstraints.Single().InterfaceType))) throw new Exception("method bound facade");
                Reject<InvalidOperationException>(() => definition.AddInterfaceConstraint(0, definition.InterfaceConstraints[0].InterfaceType));
                var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
                Reject<NotSupportedException>(() => consumer.ImportReference(definition, graph.CoreLibrary));
            }
            var json = graph.WriteNativeAssembly();
            foreach (var bad in new[] { -1, 1 })
            {
                var tree = JsonNode.Parse(json)!;
                var function = tree["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Answer")!;
                function["generic_constraints"]![0]!["parameter"] = bad;
                Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(tree.ToJsonString())));
            }
            var duplicate = JsonNode.Parse(json)!;
            var rows = duplicate["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Answer")!["generic_constraints"]!.AsArray();
            rows.Add(rows[0]!.DeepClone());
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(duplicate.ToJsonString())));
            if (Environment.GetEnvironmentVariable("NEOCLR_METHOD_BOUND_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
            var extraBound = graph.AddInterface("Example", "Other");
            method.AddInterfaceConstraint(0, extraBound);
            Reject<InvalidDataException>(() => graph.Write());
        }
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
