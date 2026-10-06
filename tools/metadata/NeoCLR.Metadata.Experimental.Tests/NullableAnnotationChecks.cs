using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class NullableAnnotationChecks
{
    internal static void Run()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("NullableContracts", new(1, 0, 0, 0)), core);
        var owner = graph.AddClass("Example", "Operations");
        var array = SignatureType.ArrayOf(PrimitiveType.String);
        foreach (bool uniform in new[] { false, true })
        {
            MethodBuilder method;
            if (uniform) method = owner.AddMethod("Uniform", new(array, [array]));
            else
            {
                var definition = new NeoCLR.Metadata.Experimental.Model.MethodDefinition("Nested", 0x16, new(array, [array]));
                owner.Definition.Methods.Add(definition);
                method = MethodBuilder.ForDefinition(definition);
            }
            byte[] flags = uniform ? [2] : [1, 2];
            var annotation = new NullableAnnotation(flags, uniform);
            flags[0] = 0;
            method.SetNullableAnnotation(0, annotation);
            method.Definition.SetNullableAnnotation(-1, annotation);
            var il = method.GetILGenerator(); il.LoadArgument(0); il.Return();
        }
        var image = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, expectedExtended: false);
        var methods = snapshot.MainModule.Types.Single(t => t.Name == "Operations").Methods.Where(m => m.Name is "Nested" or "Uniform").ToArray();
        Check(methods.Length == 2, "methods lost");
        foreach (var method in methods)
        {
            var annotation = method.NullableAnnotations[0];
            Check(annotation.IsUniform == (method.Name == "Uniform"), "constructor form lost");
            Check(annotation.Flags.SequenceEqual(method.Name == "Uniform" ? new byte[] { 2 } : new byte[] { 1, 2 }), "flags changed");
            Check(method.NullableAnnotations[-1].Flags.SequenceEqual(annotation.Flags), "return annotation lost");
        }
        var view = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]).Resolve(snapshot.Identity)
            .GetTypes().Single(t => t.Name == "Operations").GetMethods().Single(m => m.Name == "Nested");
        Check(view.GetParameters()[0].NullableAnnotation!.Flags.SequenceEqual(new byte[] { 1, 2 }), "facade parameter flags lost");
        Check(view.ReturnNullableAnnotation!.Flags.SequenceEqual(new byte[] { 1, 2 }), "facade return flags lost");
        var context = new AssemblyLoadContext("nullable-check", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            var reader = new NullabilityInfoContext();
            foreach (var method in assembly.GetType("Example.Operations")!.GetMethods().Where(m => m.Name is "Nested" or "Uniform"))
            {
                string?[] input = ["retained", null];
                Check(ReferenceEquals(input, method.Invoke(null, [input])), "annotating identity method changed execution");
                var parameter = reader.Create(method.GetParameters()[0]);
                var returns = reader.Create(method.ReturnParameter);
                var root = method.Name == "Uniform" ? NullabilityState.Nullable : NullabilityState.NotNull;
                Check(parameter.ReadState == root && returns.ReadState == root, "CLR root annotation lost");
                Check(parameter.ElementType!.ReadState == NullabilityState.Nullable && returns.ElementType!.ReadState == NullabilityState.Nullable, "CLR array element annotation lost");
            }
        }
        finally { context.Unload(); }
        Reject(() => new NullableAnnotation([]));
        Reject(() => new NullableAnnotation([3]));
        Reject(() => new NullableAnnotation([1, 2], isUniform: true));
        Reject(() => new NullableAnnotation(Enumerable.Repeat((byte)2, 4097)));
        Reject(() => owner.Methods.First().SetNullableAnnotation(-2, new([2])));
        var native = graph.WriteNativeAssembly();
        var binary = NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(native, core);
        var nativeSnapshot = AssemblyDefinition.ReadNativeAssembly(binary);
        var nativeView = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([nativeSnapshot]).Resolve(nativeSnapshot.Identity)
            .GetTypes().Single(t => t.Name == "Operations").GetMethods().Single(m => m.Name == "Nested");
        Check(nativeView.GetParameters()[0].NullableAnnotation!.Flags.SequenceEqual(new byte[] { 1, 2 }), "native facade parameter flags lost");
        Check(nativeView.ReturnNullableAnnotation!.Flags.SequenceEqual(new byte[] { 1, 2 }), "native facade return flags lost");
        var projected = NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.ReadCliProjection(binary);
        Check(projected.MainModule.Types.Single(t => t.Name == "Operations").Methods.Single(m => m.Name == "Uniform").NullableAnnotations[0].IsUniform, "projection lost uniform flag");
        foreach (var invalid in new[] { "position", "flags", "uniform", "duplicate" })
        {
            var root = System.Text.Json.Nodes.JsonNode.Parse(native)!;
            var annotations = root["functions"]![0]!["origin"]!["nullable_annotations"]!.AsArray();
            if (invalid == "position") annotations[0]!["position"] = 1;
            if (invalid == "flags") annotations[0]!["flags"]![0] = 3;
            if (invalid == "uniform") annotations[0]!["uniform"] = true;
            if (invalid == "duplicate") annotations.Add(annotations[0]!.DeepClone());
            var malformed = System.Text.Encoding.UTF8.GetBytes(root.ToJsonString());
            try { AssemblyDefinition.ReadNativeAssembly(NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(malformed, core)); throw new Exception("invalid native annotation accepted"); }
            catch (InvalidDataException) { }
        }
        var output = Environment.GetEnvironmentVariable("NEOCLR_NULLABLE_TEST_ARTIFACT");
        if (!string.IsNullOrEmpty(output)) File.WriteAllBytes(output, binary);
        using var pe = new PEReader(new MemoryStream(image));
        var metadata = pe.GetMetadataReader();
        var scalar = metadata.CustomAttributes.Select(metadata.GetCustomAttribute)
            .Single(a => metadata.GetBlobBytes(a.Value).SequenceEqual(new byte[] { 1, 0, 2, 0, 0 }) &&
                metadata.GetParameter((ParameterHandle)a.Parent).SequenceNumber == 1);
        var offset = pe.PEHeaders.MetadataStartOffset + metadata.GetHeapMetadataOffset(HeapIndex.Blob) + MetadataTokens.GetHeapOffset(scalar.Value);
        // This five-byte payload has a one-byte compressed length prefix.
        foreach (int relativeOffset in new[] { 3, 4 })
        {
            var malformed = (byte[])image.Clone();
            malformed[offset + relativeOffset] = 3;
            try { AssemblyDefinition.ReadAssembly(malformed, expectedExtended: false); throw new Exception("malformed nullable attribute accepted"); }
            catch (InvalidDataException) { }
        }
        var first = owner.Methods.First();
        first.SetNullableAnnotation(0, null);
        Check(!first.Definition.NullableAnnotations.ContainsKey(0) && first.Definition.NullableAnnotations.ContainsKey(-1), "clearing parameter changed return");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or NotSupportedException) { return; }
        throw new Exception("invalid or unsupported nullable metadata accepted");
    }
}
