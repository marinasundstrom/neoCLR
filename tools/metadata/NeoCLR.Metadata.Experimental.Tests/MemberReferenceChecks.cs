using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class MemberReferenceChecks
{
    internal static void ProducerReferences()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        AssemblyBuilder Library(string name = "Library", bool result = true)
        {
            var assembly = new AssemblyBuilder(new(name, new Version(1, 0, 0, 0)), core);
            var type = assembly.AddType("Example", "Math");
            foreach (int count in new[] { 0, 1 })
            {
                var method = type.AddMethod("Compute", count, result);
                if (result) method.LoadConstant(42);
                method.Return();
            }
            var notify = type.AddMethod("Notify", returnsValue: false); notify.Return();
            return assembly;
        }
        var library = Library();
        var app = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var main = app.AddFunction("Main");
        main.Call(library.Types[0].Methods[2]); main.LoadConstant(20); main.Call(library.Types[0].Methods[1]); main.Return(); app.EntryPoint = main;
        var image = app.Write();
        var consumer = AssemblyDefinition.ReadAssembly(image, false);
        var dependency = AssemblyDefinition.ReadAssembly(library.Write(), false);
        var resolver = new Resolver(dependency);
        var reference = consumer.MainModule.MemberReferences.Single(m => m.Name == "Compute");
        Check(ReferenceEquals(reference.Module, consumer.MainModule) && reference.MetadataToken >> 24 == 0x0a && reference.ParentToken >> 24 == 1, "consuming scope and physical tokens");
        Check(ReferenceEquals(reference, consumer.MainModule.GetMemberReference(reference.MetadataToken)) && consumer.MainModule.GetMemberReference(0x06000001) is null, "reference lookup");
        Check(ReferenceEquals(reference.ResolveMethod(resolver), dependency.MainModule.Types[1].Methods[1]), "overload resolution");
        reference.ResolveMethod(resolver); Check(resolver.Calls == 2, "no hidden resolver cache");
        Check(consumer.MainModule.MemberReferences.Single(m => m.Name == "Notify").ResolveMethod(resolver).TryGetStaticInt32Signature(out _, out var returns) && !returns, "no-result contract");
        var copied = reference.GetSignature(); Array.Clear(copied); Array.Clear(image);
        Check(reference.ResolveMethod(resolver).Name == "Compute", "owned signatures and image");
        Reject(() => reference.ResolveMethod(), "resolver required");
        Reject(() => reference.ResolveMethod(new Resolver(null)), "not found");
        Reject(() => reference.ResolveMethod(new Resolver(AssemblyDefinition.ReadAssembly(Library("Wrong").Write(), false))), "identity mismatch");
        Reject(() => reference.ResolveMethod(new Resolver(AssemblyDefinition.ReadAssembly(Library(result: false).Write(), false))), "missing or ambiguous");
        try { reference.ResolveMethod(new ThrowingResolver()); throw new Exception("resolver failure swallowed"); } catch (IOException error) when (error.Message == "host failure") { }
    }
    internal static void LocalAndUnsupported()
    {
        foreach (string parent in new[] { "type", "reference" })
        {
            var assembly = Read(Image([0, 1, 8, 8], parent: parent));
            Check(ReferenceEquals(assembly.MainModule.MemberReferences[0].ResolveMethod(), assembly.MainModule.Types[1].Methods[0]), "local parent resolution");
        }
        Reject(() => Read(Image([0, 1, 8, 8], duplicate: true)).MainModule.MemberReferences[0].ResolveMethod(), "missing or ambiguous");
        Reject(() => Read(Image([0, 0, 8])).MainModule.MemberReferences[0].ResolveMethod(), "missing or ambiguous");
        Reject(() => Read(Image([0, 1, 8, 8], parent: "module")).MainModule.MemberReferences[0].ResolveMethod(), "unsupported member method parent");
        Reject(() => Read(Image([0x10, 1, 0, 8])).MainModule.MemberReferences[0].ResolveMethod(), "missing or ambiguous");
        foreach (byte[] signature in new byte[][] { [6, 8], [0x20, 1, 8, 8], [0, 0, 0x12, 5], [0], [0, 1, 8, 8, 8] })
        {
            var member = Read(Image(signature)).MainModule.MemberReferences[0];
            Check(member.GetSignature().SequenceEqual(signature), "opaque field/general/malformed signature retained");
            Reject(() => member.ResolveMethod(), "unsupported member method signature");
        }
    }
    internal static void Bounds()
    {
        Check(Read(Image([0, 1, 8, 8], count: 4096)).MainModule.MemberReferences.Count == 4096, "row boundary accepted");
        Reject(() => Read(Image([0, 1, 8, 8], count: 4097)), "too many member references");
        Reject(() => Read(Image([])), "missing or excessive member signature");
        Reject(() => Read(Image(new byte[1024], count: 4096)), "missing or excessive member signature"); // Shared with MethodDef signatures.
        Reject(() => Read(Image([0, 1, 8, 8], parent: "invalid")), "parent outside");
    }
    private static AssemblyDefinition Read(byte[] image) => AssemblyDefinition.ReadAssembly(image, false);
    private static byte[] Image(byte[] signature, int count = 1, bool duplicate = false, string parent = "type")
    {
        var metadata = new MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Fixture.dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Fixture"), new(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
        metadata.AddTypeDefinition(0, default, metadata.GetOrAddString("<Module>"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        var owner = metadata.AddTypeDefinition(TypeAttributes.Public, default, metadata.GetOrAddString("Owner"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        for (int i = 0; i < (duplicate ? 2 : 1); i++)
            metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.Static, 0, metadata.GetOrAddString("Compute"), metadata.GetOrAddBlob(new byte[] { 0, 1, 8, 8 }), -1, MetadataTokens.ParameterHandle(1));
        EntityHandle target = parent switch
        {
            "reference" => metadata.AddTypeReference(MetadataTokens.EntityHandle(1), default, metadata.GetOrAddString("Owner")),
            "module" => metadata.AddModuleReference(metadata.GetOrAddString("Other.netmodule")),
            "invalid" => MetadataTokens.TypeDefinitionHandle(99),
            _ => owner
        };
        for (int i = 0; i < count; i++) metadata.AddMemberReference(target, metadata.GetOrAddString("Compute"), metadata.GetOrAddBlob(signature));
        var pe = new ManagedPEBuilder(new PEHeaderBuilder(), new MetadataRootBuilder(metadata), new BlobBuilder(), strongNameSignatureSize: 0);
        var bytes = new BlobBuilder(); pe.Serialize(bytes); return bytes.ToArray();
    }
    private static void Reject(Action action, string message)
    {
        try { action(); } catch (InvalidDataException error) when (error.Message.Contains(message, StringComparison.Ordinal)) { return; }
        throw new Exception("expected failure: " + message);
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private sealed class Resolver(AssemblyDefinition? assembly) : IAssemblyResolver
    {
        internal int Calls;
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) { Calls++; return assembly; }
    }
    private sealed class ThrowingResolver : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => throw new IOException("host failure");
    }
}
