using Mono.Cecil;
using System.Text.Json;

static class SignatureProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var corePath = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(corePath, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(corePath);
        var module = core.MainModule;
        var result = module.GetType("System.Result`2");
        var owner = new GenericInstanceType(result);
        owner.GenericArguments.Add(module.TypeSystem.String);
        owner.GenericArguments.Add(module.GetType(FileBindings.ReadError));
        var checks = new List<string>();
        void Check(string name, bool success)
        {
            if (!success) throw new Exception("Signature check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action)
        {
            try { action(); }
            catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Malformed signature accepted: " + name);
        }
        var ok = module.GetType("System.Result").NestedTypes.Single(t => t.Name == "Ok`1");
        var nested = new GenericInstanceType(ok);
        nested.GenericArguments.Add(new ArrayType(result.GenericParameters[0]));
        var shape = new ByReferenceType(nested);
        var before = shape.FullName;
        Check("Recursive type argument under vector and byref", RuntimeSignatures.Close(shape, owner).FullName
            == "System.Result/Ok`1<System.String[]>&");
        Check("Input signature remains unchanged", shape.FullName == before);
        var extraction = result.Methods.Single(m => m.Name == "TryGetResidual");
        var reference = Reference(extraction, owner);
        var matched = RuntimeSignatures.Match(reference, extraction, FileBindings.Type);
        Check("Closed residual parameter", matched.Args.SequenceEqual(new[] { FileBindings.ReadError + "&" }) && matched.Result == "Boolean");
        reference.ReturnType = module.TypeSystem.Int32;
        Reject("Return mismatch", () => RuntimeSignatures.Match(reference, extraction, FileBindings.Type));
        reference = Reference(extraction, owner);
        reference.Parameters[0].ParameterType = new ByReferenceType(module.TypeSystem.String);
        Reject("Parameter mismatch", () => RuntimeSignatures.Match(reference, extraction, FileBindings.Type));
        reference = Reference(extraction, owner);
        reference.HasThis = false;
        Reject("Receiver mismatch", () => RuntimeSignatures.Match(reference, extraction, FileBindings.Type));
        var shortOwner = new GenericInstanceType(result);
        shortOwner.GenericArguments.Add(module.TypeSystem.String);
        Reject("Owner arity mismatch", () => RuntimeSignatures.Match(Reference(extraction, shortOwner), extraction, FileBindings.Type));
        Reject("Out of range generic index", () => RuntimeSignatures.Close(result.GenericParameters[1], shortOwner));
        var openMethod = new MethodReference("Open", module.TypeSystem.Void, result);
        var methodParameter = new GenericParameter("M", openMethod);
        openMethod.GenericParameters.Add(methodParameter);
        Reject("Open method parameter", () => RuntimeSignatures.Close(methodParameter, owner));
        Reject("Pointer requires separate admission", () => RuntimeSignatures.Close(new PointerType(module.TypeSystem.Int32), owner));
        Reject("Multidimensional array requires separate admission", () => RuntimeSignatures.Close(new ArrayType(module.TypeSystem.Int32, 2), owner));
        Reject("Modifier requires explicit semantics", () => RuntimeSignatures.Close(new RequiredModifierType(module.TypeSystem.Int32, module.TypeSystem.String), owner));
        Reject("Nested managed reference", () => RuntimeSignatures.Close(new ByReferenceType(new ByReferenceType(module.TypeSystem.Int32)), owner));
        TypeReference deep = module.TypeSystem.Int32;
        for (var n = 0; n < 34; n++) deep = new ArrayType(deep);
        Reject("Nesting limit", () => RuntimeSignatures.Close(deep, owner));
        Check("CLI VOID return has no result", RuntimeSignatures.Map(module.TypeSystem.Void, _ => null, returns: true) == "noresult");
        var namedVoid = new TypeReference("System", "Void", module, module, true);
        Check("Named Void storage is a type", RuntimeSignatures.Map(namedVoid, _ => null) == "Void");
        Check("Named Void value return is a type", RuntimeSignatures.Map(namedVoid, _ => null, returns: true) == "Void");
        var voidOwner = new GenericInstanceType(result);
        voidOwner.GenericArguments.Add(namedVoid);
        voidOwner.GenericArguments.Add(module.GetType(FileBindings.WriteError));
        Check("Generic Void result remains a carrier", RuntimeSignatures.Map(voidOwner, FileBindings.Type, returns: true)
            == "System.Result<Void,System.IO.FileWriteError>");
        var text = JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(Path.Combine(output, "signature-checks.json"), text);
        Console.WriteLine(text);
    }

    static MethodReference Reference(MethodDefinition definition, TypeReference owner)
    {
        var reference = new MethodReference(definition.Name, definition.ReturnType, owner) { HasThis = definition.HasThis };
        foreach (var parameter in definition.Parameters) reference.Parameters.Add(new ParameterDefinition(parameter.ParameterType));
        return reference;
    }
}
