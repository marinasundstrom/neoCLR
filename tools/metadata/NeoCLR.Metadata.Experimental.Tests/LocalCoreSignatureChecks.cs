using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class LocalCoreSignatureChecks
{
    internal static void Run()
    {
        foreach (var cycle in new[] { false, true })
        {
            var metadata = new MetadataBuilder();
            metadata.AddModule(0, metadata.GetOrAddString("LocalCore.dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
            metadata.AddAssembly(metadata.GetOrAddString("LocalCore"), new Version(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
            var fields = MetadataTokens.FieldDefinitionHandle(1);
            var methods = MetadataTokens.MethodDefinitionHandle(1);
            metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default, fields, methods);
            var valueBase = metadata.AddTypeDefinition(TypeAttributes.Public, metadata.GetOrAddString("System"), metadata.GetOrAddString("ValueType"), default, fields, methods);
            metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Sealed, metadata.GetOrAddString("System"), metadata.GetOrAddString("Void"), valueBase, fields, methods);
            metadata.AddTypeDefinition(TypeAttributes.Public, metadata.GetOrAddString("System"), metadata.GetOrAddString("Action"), default, fields, methods);
            metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed, default, metadata.GetOrAddString("Tools"), default, fields, methods);
            metadata.AddTypeReference(cycle ? MetadataTokens.TypeReferenceHandle(1) : MetadataTokens.EntityHandle(1), metadata.GetOrAddString("System"), metadata.GetOrAddString("Void"));
            metadata.AddTypeReference(MetadataTokens.EntityHandle(1), metadata.GetOrAddString("System"), metadata.GetOrAddString("Action"));
            // bool Try(out valuetype local-System.Void, class local-System.Action)
            var signature = metadata.GetOrAddBlob(new byte[] { 0, 2, 2, 0x10, 0x11, 5, 0x12, 9 });
            metadata.AddParameter(ParameterAttributes.Out, metadata.GetOrAddString("value"), 1);
            metadata.AddParameter(0, metadata.GetOrAddString("callback"), 2);
            metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.Static, MethodImplAttributes.Runtime,
                metadata.GetOrAddString("Try"), signature, -1, MetadataTokens.ParameterHandle(1));
            var blob = new BlobBuilder();
            new ManagedPEBuilder(new PEHeaderBuilder(), new MetadataRootBuilder(metadata), new BlobBuilder(), strongNameSignatureSize: 0).Serialize(blob);
            try
            {
                var snapshot = AssemblyDefinition.ReadAssembly(blob.ToArray(), expectedExtended: false);
                var app = new AssemblyBuilder(new("LocalCoreConsumer", new Version(1, 0, 0, 0)), snapshot.Identity);
                var imported = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Tools").Methods.Single(), snapshot.Identity);
                if (cycle) throw new Exception("cyclic local TypeRef admitted");
                if (imported.Signature.ParameterTypes[0].ByReferenceElement?.ImportedType is not { IsValueType: true, Name: "Void" } ||
                    imported.Signature.ParameterTypes[1].FunctionSignature is not { NoResult: true }) throw new Exception("local core signature lost");
            }
            catch (InvalidDataException) when (cycle) { }
        }
    }
}
