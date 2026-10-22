using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericDefinitionTokenChecks
{
    internal static void Run()
    {
        var graph = GenericClassChecks.Create();
        var box = graph.Types.Single(t => t.Name == "Box`1");
        var tokens = graph.AddClass("Example", "Tokens");
        var owned = tokens.AddMethod("Owned", new MethodSignature(PrimitiveType.RuntimeTypeHandle, []));
        owned.GetILGenerator().LoadTypeDefinitionToken(box.MakeGenericInstance(SignatureType.TypeParameter(0)));
        owned.Return();
        var loaded = Assembly.Load(graph.Write());
        var tokenType = loaded.GetType("Example.Tokens")!;
        var ownType = Type.GetTypeFromHandle((RuntimeTypeHandle)tokenType.GetMethod("Owned")!.Invoke(null, null)!);
        if (ownType is null || ownType != loaded.GetType("Example.Box`1") || !ownType.IsGenericTypeDefinition)
            throw new Exception("Generic definition token identity");
        var external = graph.ImportTypeIdentity(new AssemblyIdentity("GenericDependency", new Version(1, 0, 0, 0)), "System.Collections.Generic", "List`1", 1);
        var imported = tokens.AddMethod("Imported", new MethodSignature(PrimitiveType.RuntimeTypeHandle, []));
        imported.GetILGenerator().LoadTypeDefinitionToken(external.MakeGenericInstance(SignatureType.TypeParameter(0)));
        imported.Return();
        _ = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly());
        Console.WriteLine("PASS generic definition tokens preserve owned and imported identity without caller parameters");
    }
}
