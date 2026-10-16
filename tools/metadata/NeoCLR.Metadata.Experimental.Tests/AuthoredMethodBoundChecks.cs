using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredMethodBoundChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var dependency = new AssemblyIdentity("NumericLibrary", new Version(1, 0, 0, 0));
        var app = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var hash = new string('a', 64);
        var owner = app.CreateTypeReference(dependency, core, hash, "Example", "Arithmetic");
        var bound = app.CreateInterfaceReference(dependency, core, hash, "Example", "Addition");
        app.CompleteInterfaceReference(bound);
        var value = app.CreateTypeReference(dependency, core, hash, "Example", "Value");
        app.AddInterfaceConversion(value, bound);
        var signature = new MethodSignature(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]);
        var bounds = new[] { new GenericMethodTypeConstraint(0, bound) };
        var reference = app.CreateMethodReference(owner, "Identity", signature, isStatic: true, interfaceBounds: bounds);
        if (!ReferenceEquals(reference, app.CreateMethodReference(owner, "Identity", signature, isStatic: true, interfaceBounds: bounds)))
            throw new Exception("constrained reference interning");
        bounds[0] = new(0, value); // Input storage must not mutate the reference.
        _ = reference.MakeGenericInstance(value);
        Reject<ArgumentException>(() => reference.MakeGenericInstance(owner));
        Reject<InvalidDataException>(() => app.CreateMethodReference(owner, "Identity", signature, isStatic: true));
        Reject<ArgumentException>(() => app.CreateMethodReference(owner, "Invalid", signature, isStatic: true, interfaceBounds: bounds));
        Reject<ArgumentException>(() => app.CreateMethodReference(owner, "Invalid", signature, isStatic: true, interfaceBounds: [new(1, bound)]));
        Reject<ArgumentException>(() => app.CreateMethodReference(owner, "Invalid", signature, isStatic: true, interfaceBounds: [new(0, bound), new(0, bound)]));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        var foreignBound = foreign.CreateInterfaceReference(dependency, core, hash, "Example", "Addition");
        Reject<ArgumentException>(() => app.CreateMethodReference(owner, "Invalid", signature, isStatic: true, interfaceBounds: [new(0, foreignBound)]));
        Console.WriteLine("PASS authored nominal method bounds: interning, copied input, conformance and malformed contracts");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
