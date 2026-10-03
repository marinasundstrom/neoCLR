using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class ParameterModeChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new("ParameterModes", new Version(1, 0, 0, 0)), core);
        var contract = builder.AddGenericInterface("Example", "Output", ["T"]);
        var signature = new MethodSignature(PrimitiveType.Boolean,
            [SignatureType.TypeParameter(0), SignatureType.ByReference(SignatureType.TypeParameter(0)),
             SignatureType.ByReference(SignatureType.ArrayOf(SignatureType.TypeParameter(0)))], outParameters: [2]);
        contract.AddInterfaceMethod("Try", signature);
        contract.Definition.Methods.Add(new MethodDefinition("Manual", 0x546, signature));
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteBinary(builder), RuntimeAssemblyContainer.Write(builder.WriteNativeAssembly(), core) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var method = snapshot.MainModule.Types.Single(t => t.Name == "Output`1").Methods.First();
            Check(method.TryGetSignature(out var decoded) && decoded!.OutParameters.SequenceEqual([2]) && decoded.ParameterTypes[1].ByReferenceElement?.TypeParameterIndex == 0, "native signature retains ref/out");
            var context = new MetadataLoadContext([snapshot]);
            var owner = context.Resolve(snapshot.Identity).GetTypes().Single();
            foreach (var view in owner.GetMethods())
            {
                var parameters = view.GetParameters();
                Check(parameters.Select(p => p.PassingMode).SequenceEqual([ParameterPassingMode.Value, ParameterPassingMode.Ref, ParameterPassingMode.Out]), "all passing modes");
                Check(ReferenceEquals(parameters[0].ParameterType, parameters[1].ParameterType), "ref exposes canonical element type");
                Check(ReferenceEquals(((ArrayTypeInfo)parameters[2].ParameterType).ElementType, parameters[0].ParameterType), "out vector retains scope");
            }
            var closed = owner.MakeGenericType(context.ResolveSignature(PrimitiveType.Int32));
            Check(closed.GetMethods().First().GetParameters()[1].ParameterType is PrimitiveTypeInfo { Kind: PrimitiveType.Int32 }, "constructed byref substitution");
            var second = AssemblyDefinition.ReadNativeAssembly(snapshot.Write());
            Check(second.MainModule.Types.Single(t => t.Name == "Output`1").Methods.First().TryGetSignature(out var roundTrip) && roundTrip!.OutParameters.SequenceEqual([2]), "snapshot round trip");
        }
        foreach (var mode in new[] { System.Reflection.ParameterAttributes.None, System.Reflection.ParameterAttributes.Out, System.Reflection.ParameterAttributes.In })
        {
            var snapshot = AssemblyDefinition.ReadAssembly(CliParameter(mode), expectedExtended: false);
            var method = snapshot.MainModule.Functions.Single();
            if (mode == System.Reflection.ParameterAttributes.In)
                Check(!method.TryGetSignature(out _), "readonly/in mode must not become writable ref");
            else
            {
                var context = new MetadataLoadContext([snapshot]);
                var parameter = context.Resolve(snapshot.Identity).GetModules().Single().GetFunctions().Single().GetParameters().Single();
                Check(parameter.PassingMode == (mode == System.Reflection.ParameterAttributes.Out ? ParameterPassingMode.Out : ParameterPassingMode.Ref), "CLI parameter flag parity");
            }
        }
        var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var external = consumer.CreateInterfaceReference(builder.Identity, core, new string('A', 64), "Example", "Output`1", 1);
        var imported = consumer.CreateMethodReference(external, "Try", signature);
        Check(imported.Signature.OutParameters.SequenceEqual([2]), "symbol-authored reference retains modes");
        Reject<InvalidDataException>(() => consumer.CreateMethodReference(external, "Try", new MethodSignature(PrimitiveType.Boolean, signature.ParameterTypes)));
        consumer.CompleteInterfaceReference(external);
        Reject<ArgumentException>(() => new MethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32], outParameters: [0]));
    }
    private static byte[] CliParameter(System.Reflection.ParameterAttributes mode)
    {
        var metadata = new System.Reflection.Metadata.Ecma335.MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Modes.dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Modes"), new Version(1, 0, 0, 0), default, default, 0, System.Reflection.AssemblyHashAlgorithm.None);
        metadata.AddTypeDefinition(0, default, metadata.GetOrAddString("<Module>"), default,
            System.Reflection.Metadata.Ecma335.MetadataTokens.FieldDefinitionHandle(1), System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1));
        metadata.AddParameter(mode, metadata.GetOrAddString("value"), 1);
        metadata.AddMethodDefinition(System.Reflection.MethodAttributes.Public | System.Reflection.MethodAttributes.Static,
            System.Reflection.MethodImplAttributes.Runtime, metadata.GetOrAddString("Write"), metadata.GetOrAddBlob(new byte[] { 0, 1, 1, 0x10, 8 }), -1,
            System.Reflection.Metadata.Ecma335.MetadataTokens.ParameterHandle(1));
        var image = new System.Reflection.Metadata.BlobBuilder();
        new System.Reflection.PortableExecutable.ManagedPEBuilder(new System.Reflection.PortableExecutable.PEHeaderBuilder(),
            new System.Reflection.Metadata.Ecma335.MetadataRootBuilder(metadata), new System.Reflection.Metadata.BlobBuilder(), strongNameSignatureSize: 0).Serialize(image);
        return image.ToArray();
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
