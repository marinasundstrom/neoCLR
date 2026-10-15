using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ReferenceCompositionChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new("CompositionRead", new Version(1, 0, 0, 0)), core);
        var a = builder.AddInterface("Example", "A");
        var b = builder.AddInterface("Example", "B");
        var echo = builder.AddFunction("Echo", new MethodSignature(a, new SignatureType[] { a }));
        echo.LoadArgument(0); echo.Return();
        var main = builder.AddFunction("Main"); main.LoadConstant(42); main.Return(); builder.EntryPoint = main;
        var source = JsonNode.Parse(builder.WriteNativeAssembly())!.AsObject();
        var originalType = source["functions"]![0]!["parameters"]![0]!.DeepClone();
        var otherType = new JsonObject { ["Named"] = source["types"]![1]!["name"]!.DeepClone() };
        JsonNode Compose(string kind, params JsonNode[] types) => new JsonObject { [kind] = new JsonArray(types.Select(t => t.DeepClone()).ToArray()) };
        JsonNode SetSignature(JsonNode signature, JsonNode? returns = null)
        {
            var root = source.DeepClone();
            root["functions"]![0]!["parameters"]![0] = signature.DeepClone();
            root["functions"]![0]!["returns"] = (returns ?? signature).DeepClone();
            return root;
        }
        byte[] Image(JsonNode root, bool binary = false)
        {
            var json = Encoding.UTF8.GetBytes(root.ToJsonString());
            return binary ? NativeModuleContainer.WriteLibraryBinary(json) : NativeModuleContainer.WriteBinary(json);
        }
        foreach (var kind in new[] { "Union", "Intersection" })
        {
            var bound = Compose(kind, originalType, otherType);
            var reversed = Compose(kind, otherType, originalType);
            foreach (var binary in new[] { false, true })
            {
                var image = Image(SetSignature(bound, reversed), binary);
                var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
                Check(snapshot.Write().SequenceEqual(image), "original image preservation");
                var method = snapshot.MainModule.Functions.Single(m => m.Name == "Echo");
                Check(method.TryGetSignature(out var signature), "materialized signature");
                var input = signature!.ParameterTypes.Single();
                Check(input == signature.ReturnType && input.GetHashCode() == signature.ReturnType.GetHashCode(), "order-independent identity");
                Check(input.Composition!.Kind.ToString() == kind && input.Composition.Constituents.Count == 2, "kind and constituents");
                Check(input.Composition.Constituents[0].ReferencedType!.Resolve().Name == "A", "snapshot-owned bound");
                Check(input.ToString().Contains(kind), "diagnostic display");
                var context = new MetadataLoadContext([snapshot]);
                var view = (ReferenceCompositionTypeInfo)context.ResolveSignature(input);
                Check(ReferenceEquals(view, context.ResolveSignature(signature.ReturnType)), "canonical reordered view");
                Check(view.Kind == input.Composition.Kind && !view.IsNominalType, "non-nominal composition view");
                Check(ReferenceEquals(view.Constituents[0], context.Resolve(input.Composition.Constituents[0].ReferencedType!)), "canonical nominal view");
                Check(view.DisplayName.Contains(kind), "composition view display");
                var functionView = context.GetModules().SelectMany(m => m.GetFunctions()).Single(m => m.Name == "Echo");
                Check(ReferenceEquals(view, functionView.ReturnType), "function discovery projects canonical result");

                try { ((IList<TypeInfo>)view.Constituents).Clear(); throw new Exception("mutable view bounds"); }
                catch (NotSupportedException) { }
                Check(!ReferenceEquals(view, new MetadataLoadContext([snapshot]).ResolveSignature(input)), "context-scoped view identity");
                Reject(() => new MetadataLoadContext([]).ResolveSignature(input));

                try { ((IList<SignatureType>)input.Composition.Constituents).Clear(); throw new Exception("mutable constituents"); }
                catch (NotSupportedException) { }
                Reject(() => RuntimeAssemblyContainer.Write(NativeModuleContainer.Read(image), core));
                var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
                Reject(() => consumer.ImportReference(method, core));
                Reject(() => consumer.AddFunction("Bad", new MethodSignature(input, Array.Empty<SignatureType>())));
                var another = AssemblyDefinition.ReadNativeAssembly(image);
                Check(another.MainModule.Functions[0].TryGetSignature(out var other), "separate snapshot");
                Check(input != other!.ParameterTypes[0], "snapshot identity is not display-name identity");
            }
            var distinctKinds = AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(bound,
                Compose(kind == "Union" ? "Intersection" : "Union", originalType, otherType))));
            Check(distinctKinds.MainModule.Functions[0].TryGetSignature(out var distinctSignature), "distinct kind signature");
            var distinctContext = new MetadataLoadContext([distinctKinds]);
            Check(!ReferenceEquals(distinctContext.ResolveSignature(distinctSignature!.ReturnType),
                distinctContext.ResolveSignature(distinctSignature.ParameterTypes[0])), "kind participates in canonical identity");
            var mixed = Compose(kind, originalType, Compose(kind == "Union" ? "Intersection" : "Union", originalType, otherType));
            var mixedRead = AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(mixed)));
            Check(mixedRead.MainModule.Functions[0].TryGetSignature(out var mixedSignature) && mixedSignature!.ReturnType.Composition!.Constituents[1].Composition is not null, "preserve mixed composition");
            var mixedContext = new MetadataLoadContext([mixedRead]);
            var mixedView = (ReferenceCompositionTypeInfo)mixedContext.ResolveSignature(mixedSignature!.ReturnType);
            Check(mixedView.Constituents[1] is ReferenceCompositionTypeInfo inner && inner.Kind != mixedView.Kind, "mixed view retains distinct kinds");
            var nested = Compose(kind, originalType, Compose(kind, otherType, originalType));
            var read = AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(nested, bound)));
            Check(read.MainModule.Functions[0].TryGetSignature(out var flattened) && flattened!.ReturnType == flattened.ParameterTypes[0], "flatten and deduplicate");
            var singleton = AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(Compose(kind, originalType, originalType))));
            Check(singleton.MainModule.Functions[0].TryGetSignature(out var single) && single!.ReturnType.Composition is null, "singleton collapse");
            foreach (var bad in new[] { Compose(kind, originalType), Compose(kind, originalType, JsonValue.Create(kind == "Union" ? "Void" : "Int32")!), Compose(kind, originalType, new JsonObject { ["Named"] = "Missing" }) })
                Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(bad))));
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(new JsonObject { ["ArrayRef"] = bound.DeepClone() }))));
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(new JsonObject { ["ByRef"] = bound.DeepClone() }))));
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(Compose(kind, Enumerable.Repeat(originalType, 257).ToArray())))));
            var wide = Compose(kind, Enumerable.Repeat(Compose(kind, Enumerable.Repeat(originalType, 256).ToArray()), 17).ToArray());
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(wide))));
            var generic = SetSignature(bound);
            generic["functions"]![0]!["generic_parameters"] = new JsonArray("T");
            var genericRead = AssemblyDefinition.ReadNativeAssembly(Image(generic));
            Check(genericRead.MainModule.Functions[0].TryGetSignature(out var genericSignature) &&
                genericSignature!.GenericParameterNames.SequenceEqual(new[] { "T" }) &&
                genericSignature.ReturnType.Composition is not null, "generic method retains structural signature");
            var local = source.DeepClone();
            local["functions"]![0]!["locals"] = new JsonArray(bound.DeepClone());
            _ = AssemblyDefinition.ReadNativeAssembly(Image(local));
            var malformed = bound.DeepClone().AsObject(); malformed["Unexpected"] = true;
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(malformed))));
            var deep = bound;
            for (var i = 0; i < 34; i++) deep = Compose(i % 2 == 0 ? "Union" : "Intersection", originalType, deep);
            Reject(() => AssemblyDefinition.ReadNativeAssembly(Image(SetSignature(deep))));
        }
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is InvalidDataException or ArgumentException) { return; }
        throw new Exception("expected explicit rejection");
    }
}
