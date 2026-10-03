using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class BoxingChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("BoxingChecks", new Version(1, 0, 0, 0)), core);
        Check(ReferenceEquals(graph.CoreObjectType, graph.CoreObjectType), "canonical core object reference");
        var owner = graph.AddType("Example", "Boxing");
        var t = SignatureType.MethodParameter(0);
        var generic = owner.AddMethod("Generic", new(graph.CoreObjectType, [t], ["T"]));
        var il = generic.GetILGenerator(); il.LoadArgument(0); il.Box(t); il.Return();
        var primitive = owner.AddMethod("Integer", new(graph.CoreObjectType, [PrimitiveType.Int32]));
        il = primitive.GetILGenerator(); il.LoadArgument(0); il.Emit(OpCode.Box, (SignatureType)PrimitiveType.Int32); il.Return();
        var genericOwner = graph.AddGenericType("Example", "Owner", ["T"]);
        var ownerParameter = SignatureType.TypeParameter(0);
        il = genericOwner.AddMethod("Box", new(graph.CoreObjectType, [ownerParameter])).GetILGenerator();
        il.LoadArgument(0); il.Box(ownerParameter); il.Return();
        var bytes = graph.Write();
        _ = AssemblyDefinition.ReadAssembly(bytes, expectedExtended: false);
        var context = new AssemblyLoadContext("boxing-checks", isCollectible: true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(bytes));
            var type = loaded.GetType("Example.Boxing")!;
            Check(Equals(loaded.GetType("Example.Owner`1")!.MakeGenericType(typeof(int)).GetMethod("Box")!.Invoke(null, [42]), 42), "owner generic scope executes");
            Check(Equals(type.GetMethod("Integer")!.Invoke(null, [42]), 42), "primitive box executes");
            var method = type.GetMethod("Generic")!;
            Check(Equals(method.MakeGenericMethod(typeof(int)).Invoke(null, [42]), 42), "generic value box executes");
            var instance = new object();
            Check(ReferenceEquals(method.MakeGenericMethod(typeof(object)).Invoke(null, [instance]), instance), "generic reference preserves identity");
            var text = new string('x', 3);
            Check(ReferenceEquals(method.MakeGenericMethod(typeof(string)).Invoke(null, [text]), text), "generic string preserves identity");
            Check(method.MakeGenericMethod(typeof(object)).Invoke(null, [null]) is null, "generic null preserved");
        }
        finally { context.Unload(); }
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        var rejected = owner.AddMethod("Rejected", new(PrimitiveType.Void, []));
        foreach (var invalid in new[] { (SignatureType)PrimitiveType.Void, SignatureType.ByReference(PrimitiveType.Int32), SignatureType.MethodParameter(0) })
            Reject<ArgumentException>(() => rejected.GetILGenerator().Box(invalid));
        rejected.GetILGenerator().Return();
        _ = graph.Write(); // rejected instructions did not mutate the body
        var bad = new AssemblyBuilder(new("BadBox", new Version(1, 0, 0, 0)), core);
        il = bad.AddFunction("Mismatch", new(bad.CoreObjectType, [])).GetILGenerator();
        il.LoadConstant(42); il.Box(PrimitiveType.Int64); il.Return();
        Reject<InvalidDataException>(() => bad.Write());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
