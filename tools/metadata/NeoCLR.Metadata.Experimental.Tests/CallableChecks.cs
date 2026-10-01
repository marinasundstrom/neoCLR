using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class CallableChecks
{
    internal static void RoundTrip()
    {
        var builder = new AssemblyBuilder(new("Callables", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var function = builder.AddFunction("Main"); function.LoadConstant(42); function.Return(); builder.EntryPoint = function;
        var noResult = builder.AddFunction("Notify", returnsValue: false); noResult.Return();
        var type = builder.AddType("Example", "Math");
        foreach (int count in new[] { 0, 1, 127, 128, 256 })
        {
            var method = type.AddMethod("Compute", count); method.LoadConstant(count); method.Return();
        }
        var image = builder.Write();
        var read = AssemblyDefinition.ReadAssembly(image, false);
        var module = read.MainModule;
        Check(module.Methods.Count == 7 && module.Functions.Count == 2, "callable collections");
        Check(ReferenceEquals(read.EntryPoint, module.Functions[0]), "entry definition identity");
        Check(module.Functions.All(m => m.DeclaringType is null && ReferenceEquals(m.Module, module)), "global ownership");
        Check(module.Types[0].Methods.Count == 0, "pseudo-type does not own model functions");
        Check(module.Types[1].Methods.Count == 5, "type methods");
        foreach (var method in module.Types[1].Methods)
        {
            Check(ReferenceEquals(method.DeclaringType, module.Types[1]), "owned type identity");
            Check(ReferenceEquals(method, module.GetMethodDefinition(method.MetadataToken)), "token lookup identity");
            Check(method.TryGetStaticInt32Signature(out var count, out var result) && result && count == new[] { 0, 1, 127, 128, 256 }[module.Types[1].Methods.ToList().IndexOf(method)], "typed signatures at count boundaries");
            Check(method.IsStatic && method.GenericArity == 0 && method.ImplementationAttributes == 0 && method.Attributes == 0x96, "physical attributes");
        }
        Check(module.Functions[1].TryGetStaticInt32Signature(out var parameters, out var returns) && parameters == 0 && !returns, "CLI void remains no-result");
        Check(module.GetMethodDefinition(0x02000001) is null && module.GetMethodDefinition(0x060000ff) is null, "missing token");
        var signature = read.EntryPoint!.GetSignature(); Array.Clear(signature); Array.Clear(image);
        Check(read.EntryPoint.TryGetStaticInt32Signature(out _, out var retained) && retained, "owned input and returned signature copies");
        Check(AssemblyDefinition.ReadAssembly(read.Write(), false).MainModule.Methods.Count == 7, "snapshot preservation");
    }
    internal static void SignatureRecognition()
    {
        foreach (byte[] signature in new byte[][] {
            [0, 0, 0x1c], // Object result, outside the supported built-in subset.
            [0x20, 0, 8], // Instance calling convention.
            [0x10, 1, 0, 8], // Generic calling convention.
            [5, 0, 8], // Vararg.
            [0, 0x80, 0, 8], // Noncanonical compressed count.
            [0, 0xc0, 0, 0, 0, 8], // Four-byte count outside bounded canonical subset.
            [0, 1, 8], // Missing parameter.
            [0, 0, 8, 8], // Trailing data.
            [0], // Truncation.
            [0, 0, 0x10, 8], // Managed-reference result.
            [0, 0, 0x1d], // Truncated vector.
            [0, 0, 0x1d, 0x1d, 8], // Nested vector.
            [0, 0, 0x1d, 1], // Void vector result.
            [0, 1, 1, 0x1d, 1], // Void vector parameter.
            [0, 1, 1, 0x1d, 0x1c], // Unsupported element.
            [0, 0, 0x1d, 8, 8], // Trailing vector signature data.
            [0, 1, 0x1d, 8] // Missing vector parameter.
        })
        {
            var method = AssemblyDefinition.ReadAssembly(Image(1, signature), false).MainModule.Functions.Single();
            Check(!method.TryGetStaticInt32Signature(out var count, out var result) && count == 0 && !result, "unsupported signature cannot masquerade as Int32");
            Check(!method.TryGetStaticValueSignature(out var value) && value is null, "unsupported value signature rejected");
            var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
            try { consumer.ImportReference(method, consumer.CoreLibrary); throw new Exception("unsupported signature imported"); }
            catch (InvalidDataException) { }
            Check(method.GetSignature().SequenceEqual(signature), "opaque signature preserved exactly");
        }
        var instance = AssemblyDefinition.ReadAssembly(Image(1, [0, 0, 8], global: false, isStatic: false), false).MainModule.Methods.Single();
        Check(!instance.TryGetStaticInt32Signature(out _, out _), "attributes checked as well as signature");
        var output = new AssemblyBuilder(new("Output", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        foreach (var unsupported in new[] { instance,
            AssemblyDefinition.ReadAssembly(Image(1, [0, 0, 8], global: false, nested: true), false).MainModule.Methods.Single(),
            AssemblyDefinition.ReadAssembly(Image(1, [0, 0, 8], global: false, genericOwner: true), false).MainModule.Methods.Single() })
        {
            try { output.ImportReference(unsupported, output.CoreLibrary); throw new Exception("unsupported owner/instance imported"); }
            catch (InvalidDataException) { }
        }
    }
    internal static void GenericSignatureRecognition()
    {
        foreach (var arity in new[] { 1, 32 })
        {
            byte[] signature = [0x10, (byte)arity, 1, 0x1e, (byte)(arity - 1), 0x1d, 0x1e, 0];
            var method = AssemblyDefinition.ReadAssembly(Image(1, signature, genericArity: arity), false).MainModule.Functions.Single();
            Check(method.TryGetStaticGenericValueSignature(out var value) && value!.GenericParameterNames.Count == arity, "bounded generic arity recognized");
        }
        foreach (byte[] signature in new byte[][] {
            [0x10, 1, 0, 0x1e], // Missing index.
            [0x10, 1, 0, 0x1e, 1], // Index outside scope.
            [0x10, 1, 0, 0x1e, 0x80, 0], // Noncanonical index.
            [0x10, 1, 0, 0x13, 0], // Declaring-type parameter.
            [0x10, 1, 0, 0x1d, 0x1d, 0x1e, 0], // Nested vector.
            [0x10, 2, 0, 8], // Signature/table arity mismatch.
            [0x10, 0, 0, 8], // Generic bit with zero arity.
            [0x10, 0x80, 1, 0, 8], // Noncanonical arity.
            [0x10, 1, 1, 1, 1], // Void parameter.
            [0x10, 1, 0, 0x1e, 0, 8] // Trailing data.
        })
            RejectGeneric(Image(1, signature, genericArity: 1));
        RejectGeneric(Image(1, [0x10, 33, 0, 8], genericArity: 33));
        foreach (var flags in new[] { GenericParameterAttributes.ReferenceTypeConstraint, GenericParameterAttributes.NotNullableValueTypeConstraint,
            GenericParameterAttributes.DefaultConstructorConstraint, GenericParameterAttributes.Covariant })
            RejectGeneric(Image(1, [0x10, 1, 0, 8], genericArity: 1, genericFlags: flags));
        RejectGeneric(Image(1, [0x10, 1, 0, 8], genericArity: 1, typeConstraint: true));
        RejectGeneric(Image(1, [0x10, 1, 0, 8], genericArity: 1, invalidIndex: true));
        static void RejectGeneric(byte[] image)
        {
            var method = AssemblyDefinition.ReadAssembly(image, false).MainModule.Functions.Single();
            Check(!method.TryGetStaticGenericValueSignature(out var value) && value is null, "unsupported generic declaration rejected");
            var output = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
            try { output.ImportReference(method, output.CoreLibrary); throw new Exception("unsupported generic import admitted"); }
            catch (InvalidDataException) { }
        }
    }
    internal static void Limits()
    {
        Check(AssemblyDefinition.ReadAssembly(Image(4096, [0, 0, 8]), false).MainModule.Functions.Count == 4096, "4096 accepted");
        Reject(Image(4097, [0, 0, 8]), "too many method definitions");
        Reject(Image(1, []), "missing or excessive method signature");
        Reject(Image(4096, new byte[1025]), "missing or excessive method signature");
        Reject(Image(1, [0, 0, 8], isStatic: false), "global function must be static");
    }
    internal static byte[] Image(int count, byte[] signature, bool global = true, bool isStatic = true, bool nested = false, bool genericOwner = false, int genericArity = 0, GenericParameterAttributes genericFlags = 0, bool typeConstraint = false, bool invalidIndex = false)
    {
        var metadata = new MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Fixture.dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Fixture"), new(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
        metadata.AddTypeDefinition(0, default, metadata.GetOrAddString("<Module>"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        if (!global)
        {
            var owner = metadata.AddTypeDefinition(TypeAttributes.Public, default, metadata.GetOrAddString("Owner"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
            if (genericOwner) metadata.AddGenericParameter(owner, 0, metadata.GetOrAddString("T"), 0);
            if (nested)
            {
                var child = metadata.AddTypeDefinition(TypeAttributes.NestedPublic, default, metadata.GetOrAddString("Child"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
                metadata.AddNestedType(child, owner);
            }
        }
        for (int i = 0; i < count; i++)
            metadata.AddMethodDefinition(MethodAttributes.Public | (isStatic ? MethodAttributes.Static : 0), 0,
                metadata.GetOrAddString("Method"), metadata.GetOrAddBlob(signature), -1, MetadataTokens.ParameterHandle(1));
        for (int i = 0; i < genericArity; i++)
        {
            var parameter = metadata.AddGenericParameter(MetadataTokens.MethodDefinitionHandle(1), genericFlags,
                metadata.GetOrAddString("T" + i), invalidIndex ? i + 1 : i);
            if (typeConstraint) metadata.AddGenericParameterConstraint(parameter, MetadataTokens.TypeDefinitionHandle(1));
        }
        var pe = new ManagedPEBuilder(new PEHeaderBuilder(), new MetadataRootBuilder(metadata), new BlobBuilder(), strongNameSignatureSize: 0);
        var image = new BlobBuilder(); pe.Serialize(image); return image.ToArray();
    }
    private static void Reject(byte[] image, string expected)
    {
        try { AssemblyDefinition.ReadAssembly(image, false); }
        catch (InvalidDataException error) when (error.Message.Contains(expected, StringComparison.Ordinal)) { return; }
        throw new Exception("expected reader rejection: " + expected);
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
