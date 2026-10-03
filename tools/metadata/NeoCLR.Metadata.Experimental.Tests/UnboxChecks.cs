using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;

internal static class UnboxChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("UnboxChecks", new Version(1, 0, 0, 0)), core);
        var owner = graph.AddType("Example", "Casts");
        var parameter = SignatureType.MethodParameter(0);
        var method = owner.AddMethod("Extract", new(parameter, [graph.CoreObjectType], ["T"]));
        var il = method.GetILGenerator();
        il.LoadArgument(0); il.UnboxAny(parameter); il.Return();
        var raw = owner.AddMethod("Integer", new(PrimitiveType.Int32, [graph.CoreObjectType]));
        raw.GetILGenerator().LoadArgument(0);
        raw.GetILGenerator().Emit(OpCode.UnboxAny, (SignatureType)PrimitiveType.Int32);
        raw.GetILGenerator().Return();
        var genericOwner = graph.AddGenericType("Example", "Owner", ["T"]);
        var ownerMethod = genericOwner.AddMethod("Extract", new(SignatureType.TypeParameter(0), [graph.CoreObjectType]));
        ownerMethod.GetILGenerator().LoadArgument(0);
        ownerMethod.GetILGenerator().UnboxAny(SignatureType.TypeParameter(0));
        ownerMethod.GetILGenerator().Return();
        var image = graph.Write();
        var ownerType = Assembly.Load(image).GetType("Example.Owner`1")!.MakeGenericType(typeof(int));
        if (!Equals(ownerType.GetMethod("Extract")!.Invoke(null, [42]), 42)) throw new Exception("owner scope");
        var loaded = Assembly.Load(graph.Write()).GetType("Example.Casts")!;
        var generic = loaded.GetMethod("Extract")!;
        if (!Equals(generic.MakeGenericMethod(typeof(int)).Invoke(null, [42]), 42) ||
            !Equals(loaded.GetMethod("Integer")!.Invoke(null, [42]), 42)) throw new Exception("value unboxing");
        var instance = new object();
        if (!ReferenceEquals(generic.MakeGenericMethod(typeof(object)).Invoke(null, [instance]), instance)) throw new Exception("reference identity");
        if (generic.MakeGenericMethod(typeof(string)).Invoke(null, [null]) is not null) throw new Exception("reference null");
        foreach (var input in new object?[] { null, 42L })
        {
            try { generic.MakeGenericMethod(typeof(int)).Invoke(null, [input]); throw new Exception("invalid box admitted"); }
            catch (TargetInvocationException error) when (error.InnerException is InvalidCastException or NullReferenceException) { }
        }
        foreach (var invalid in new[] { (SignatureType)PrimitiveType.Void, SignatureType.ByReference(PrimitiveType.Int32), SignatureType.MethodParameter(1) })
        {
            try { il.UnboxAny(invalid); throw new Exception("invalid target admitted"); } catch (ArgumentException) { }
        }
        var bad = new AssemblyBuilder(new("BadUnbox", new Version(1, 0, 0, 0)), core);
        var body = bad.AddFunction("Bad", new(PrimitiveType.Int32, [])).GetILGenerator();
        body.LoadConstant(42); body.UnboxAny(PrimitiveType.Int32); body.Return();
        try { bad.Write(); throw new Exception("unboxed input admitted"); } catch (InvalidDataException) { }
    }
}
