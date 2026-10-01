using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class VectorImportChecks
{
    internal static void Run()
    {
        var coreName = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(coreName.Name!, coreName.Version!, coreName.CultureName ?? "", Convert.ToHexString(coreName.GetPublicKeyToken() ?? []));
        var library = new AssemblyBuilder(new("Vectors", new Version(1, 0, 0, 0)), core);
        var owner = library.AddType("Example", "Vectors");
        foreach (var primitive in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Boolean, PrimitiveType.String })
        {
            var vector = SignatureType.ArrayOf(primitive);
            var method = owner.AddMethod("Identity", new MethodSignature(vector, [vector]));
            method.LoadArgument(0); method.Return();
        }
        foreach (var count in new[] { 0, 127, 128, 256 })
        {
            var method = owner.AddMethod("Count" + count, new MethodSignature(PrimitiveType.Void,
                Enumerable.Repeat(SignatureType.ArrayOf(PrimitiveType.Int32), count)));
            method.Return();
        }
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), false);
        Check(snapshot.MainModule.Types[1].Methods.All(m => m.TryGetStaticValueSignature(out _)), "all vector signatures recognized");
        foreach (var definition in snapshot.MainModule.Types[1].Methods.Where(m => m.Name == "Identity"))
        {
            Check(!definition.TryGetStaticPrimitiveSignature(out _) && !definition.TryGetStaticInt32Signature(out _, out _), "scalar recognizers stay narrow");
            var app = new AssemblyBuilder(new("Consumer" + definition.MetadataToken, new Version(1, 0, 0, 0)), core);
            var imported = app.ImportReference(definition, core);
            Check(ReferenceEquals(imported, app.ImportReference(definition, core)), "vector import interned");
            var entry = app.AddFunction("Main");
            entry.LoadConstant(42); entry.NewArray(imported.Signature.ReturnType.ArrayElement!);
            entry.Call(imported); entry.LoadArrayLength(); entry.Return(); app.EntryPoint = entry;
            var executable = app.Write();
            var consumer = AssemblyDefinition.ReadAssembly(executable, false);
            var context = new AssemblyLoadContext("vector-import-" + definition.MetadataToken, isCollectible: true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                var assembly = context.LoadFromStream(new MemoryStream(executable));
                Check((int)assembly.EntryPoint!.Invoke(null, null)! == 42, "ordinary CLR executes imported primitive vector call");
            }
            finally { context.Unload(); }
            Check(ReferenceEquals(consumer.MainModule.MemberReferences.Single(m => m.Name == "Identity").ResolveMethod(new Resolver(snapshot)), definition), "exact vector overload resolved across assemblies");
            _ = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core);
            var projected = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
            var projectedMethod = projected.MainModule.Methods.Single(m => m.Name == "Identity" && m.GetSignature().SequenceEqual(definition.GetSignature()));
            Check(projectedMethod.TryGetStaticValueSignature(out var projectedSignature) && projectedSignature!.ReturnType == imported.Signature.ReturnType, "native declaration projection preserves vector signature");
        }
    }
    private sealed class Resolver(AssemblyDefinition assembly) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => identity.Equals(assembly.Identity) ? assembly : null;
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
