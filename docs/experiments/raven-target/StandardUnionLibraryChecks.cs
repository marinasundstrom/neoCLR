using Mono.Cecil;

// Negative fixtures wrap the same reference projector used by the bridge command.
static class StandardUnionLibraryChecks
{
    public static void WriteReference(string implementation, string corePath, string owner, string output, string? mutation = null)
    {
        StandardUnionReference.WriteReference(implementation, corePath, owner, output);
        if (mutation is null) return;
        using var core = AssemblyDefinition.ReadAssembly(new MemoryStream(File.ReadAllBytes(output)));
        var module = core.MainModule;
        var originals = module.AssemblyReferences.ToHashSet();
        var carrier = module.GetType(owner);
        switch (mutation)
        {
            case null: break;
            case "wrong-protocol":
                module.GetType(StandardUnionLibrary.ProtocolName).Methods.Single().ReturnType =
                    carrier.Methods.Single(m => m.Name == "get_IsHeader").ReturnType; break;
            case "wrong-case": carrier.NestedTypes[0].Name += "Changed"; break;
            case "wrong-return": carrier.Methods.Single(m => m.Name == "get_IsHeader").ReturnType = carrier.Methods.Single(m => m.Name == "ToString").ReturnType; break;
            case "wrong-output": carrier.Methods.First(m => m.Name == "TryGetValue").Parameters[0].IsOut = false; break;
            case "nonempty-case": carrier.NestedTypes[0].Fields.Add(new FieldDefinition("Extra", FieldAttributes.Private,
                carrier.Fields.Single(f => f.Name == "<Tag>").FieldType)); break;
            case "missing-case-metadata":
                carrier.CustomAttributes.Remove(carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)); break;
            case "wrong-case-ordinal":
                carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)
                    .ConstructorArguments[2] = new CustomAttributeArgument(carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute).ConstructorArguments[2].Type, 99); break;
            case "wrong-case-name":
                carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)
                    .ConstructorArguments[0] = new CustomAttributeArgument(carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute).ConstructorArguments[0].Type, "Probe.Missing+Headers"); break;
            case "unmarked": carrier.CustomAttributes.Clear(); break;
            default: throw new InvalidDataException("Unknown union reference mutation.");
        }
        foreach (var method in module.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in module.AssemblyReferences.Where(r => !originals.Contains(r)).ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Probe reference introduced an external scope.");
            module.AssemblyReferences.Remove(reference);
        }
        core.Write(output);
    }
}
