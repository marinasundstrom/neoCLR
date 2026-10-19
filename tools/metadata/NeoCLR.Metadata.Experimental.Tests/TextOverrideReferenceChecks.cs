using NeoCLR.Metadata.Experimental.Model;

internal static class TextOverrideReferenceChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyIdentity("Library", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("TextConsumer", new Version(1, 0, 0, 0)), core);
        var hash = new string('a', 64);
        var root = graph.CreateTypeReference(library, core, hash, "System", "Object");
        graph.SetNativeObjectRoot(root);
        var text = graph.CreateTypeReference(library, core, hash, "System", "String");
        var character = graph.CreateValueTypeReference(library, core, hash, "System", "Char");
        graph.SetNativeGrapheme(character);
        foreach (var owner in new[] { text, character })
        {
            PrimitiveType? primitive = owner == text ? PrimitiveType.String : null;
            foreach (var (name, signature) in new[] {
                ("GetHashCode", new MethodSignature(PrimitiveType.Int32, [])),
                ("ToString", new MethodSignature(PrimitiveType.String, [])),
                ("Equals", new MethodSignature(PrimitiveType.Boolean, [root])) })
            {
                var method = graph.CreateMethodReference(owner, name, signature, isOverride: true, nativePrimitive: primitive);
                if (method.RequiresVirtualDispatch != (owner == text))
                    throw new Exception("Text override receiver dispatch changed.");
                if (!ReferenceEquals(method, graph.CreateMethodReference(owner, name, signature, isOverride: true, nativePrimitive: primitive)))
                    throw new Exception("Text override contract was not interned.");
            }
            Reject(() => graph.CreateMethodReference(owner, "Wrong", new(PrimitiveType.Int32, []), isOverride: true, nativePrimitive: primitive));
            Reject(() => graph.CreateMethodReference(owner, "GetHashCode", new(PrimitiveType.String, []), isOverride: true, nativePrimitive: primitive));
            Reject(() => graph.CreateMethodReference(owner, "GetHashCode", new(PrimitiveType.Int32, []), isStatic: true, isOverride: true, nativePrimitive: primitive));
        }
        Reject(() => graph.CreateMethodReference(character, ".ctor", new(PrimitiveType.Void, [])));
        var number = graph.CreateValueTypeReference(library, core, hash, "System", "Int32");
        Reject(() => graph.CreateMethodReference(number, "GetHashCode", new(PrimitiveType.Int32, []), isOverride: true, nativePrimitive: PrimitiveType.Int32));
        Console.WriteLine("PASS exact String/Char Object overrides and rejected invalid contracts");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (ArgumentException) { return; }
        throw new Exception("Invalid text override contract accepted.");
    }
}
