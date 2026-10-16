using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredMethodBoundChecks
{
    internal static void Run()
    {
        Check(false);
        Check(true);
        Console.WriteLine("PASS authored nominal method/function bounds: interning, copied input, conformance and malformed contracts");
    }

    private static void Check(bool function)
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
        ImportedMethodReference Create(string name, IReadOnlyList<GenericMethodTypeConstraint>? constraints = null) => function
            ? app.CreateFunctionReference(dependency, core, hash, "Example", name, signature, interfaceBounds: constraints)
            : app.CreateMethodReference(owner, name, signature, isStatic: true, interfaceBounds: constraints);
        var reference = Create("Identity", bounds);
        if (!ReferenceEquals(reference, Create("Identity", bounds)))
            throw new Exception("constrained reference interning");
        bounds[0] = new(0, value); // Input storage must not mutate the reference.
        _ = reference.MakeGenericInstance(value);
        Reject<ArgumentException>(() => reference.MakeGenericInstance(PrimitiveType.Int32));
        var scalar = app.CreateValueTypeReference(dependency, core, hash, "System", "Int32");
        app.SetNativePrimitive(scalar, PrimitiveType.Int32);
        Reject<ArgumentException>(() => reference.MakeGenericInstance(PrimitiveType.Int32));
        app.AddInterfaceConversion(scalar, bound);
        _ = reference.MakeGenericInstance(PrimitiveType.Int32);
        Reject<ArgumentException>(() => reference.MakeGenericInstance(PrimitiveType.Double));
        Reject<ArgumentException>(() => reference.MakeGenericInstance(owner));
        Reject<InvalidDataException>(() => Create("Identity"));
        Reject<ArgumentException>(() => Create("Invalid", bounds));
        Reject<ArgumentException>(() => Create("Invalid", [new(1, bound)]));
        Reject<ArgumentException>(() => Create("Invalid", [new(0, bound), new(0, bound)]));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        var foreignBound = foreign.CreateInterfaceReference(dependency, core, hash, "Example", "Addition");
        Reject<ArgumentException>(() => Create("Invalid", [new(0, foreignBound)]));
        Reject<ArgumentException>(() => Create("Invalid", [null!]));
        Reject<ArgumentException>(() => Create("Invalid", [new(-1, bound)]));
        Reject<ArgumentException>(() => Create("Invalid", Enumerable.Range(0, 129).Select(_ => new GenericMethodTypeConstraint(0, bound)).ToArray()));
        _ = Create("Invalid", [new(0, bound)]); // Rejection must not reserve the callable name.
        _ = Create("Unconstrained");
        Reject<InvalidDataException>(() => Create("Unconstrained", [new(0, bound)]));
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
