using Mono.Cecil;

// Raven-specific development adapter, not a neoCLR union convention. These
// compiler-facing relationships are consumed here, never emitted as runtime dependencies.
static class RavenUnionMetadata
{
    public const string CaseAttribute = "Raven.Runtime.CompilerServices.RavenUnionCaseAttribute";
    public const string CompanionAttribute = "Raven.Runtime.CompilerServices.RavenUnionCompanionAttribute";
    public sealed record Case(string MetadataName, string Name, int Ordinal);

    public static Case[] Cases(TypeDefinition carrier)
    {
        var cases = carrier.CustomAttributes.Where(a => a.AttributeType.FullName == CaseAttribute).Select(attribute =>
        {
            if (attribute.ConstructorArguments.Count != 3 || attribute.HasFields || attribute.HasProperties
                || attribute.ConstructorArguments[0].Value is not string metadataName
                || attribute.ConstructorArguments[1].Value is not string name
                || attribute.ConstructorArguments[2].Value is not int ordinal || ordinal < 0)
                throw new InvalidDataException("Invalid Raven union case metadata.");
            return new Case(metadataName, name, ordinal);
        }).OrderBy(c => c.Ordinal).ToArray();
        if (cases.Select(c => c.Ordinal).Distinct().Count() != cases.Length
            || cases.Select(c => c.Name).Distinct().Count() != cases.Length
            || cases.Select(c => c.MetadataName).Distinct().Count() != cases.Length)
            throw new InvalidDataException("Duplicate Raven union case metadata.");
        return cases;
    }

    public static string? CompanionTarget(TypeDefinition companion)
    {
        var attributes = companion.CustomAttributes.Where(a => a.AttributeType.FullName == CompanionAttribute).ToArray();
        if (attributes.Length == 0) return null;
        if (attributes.Length != 1 || attributes[0].ConstructorArguments.Count != 1
            || attributes[0].HasFields || attributes[0].HasProperties
            || attributes[0].ConstructorArguments[0].Value is not string name || string.IsNullOrWhiteSpace(name))
            throw new InvalidDataException("Invalid Raven union companion metadata.");
        return name;
    }

    // Bounded parenthesized-union admission. Variants are existing types, not
    // nested generated cases; validate the ordinary constructor/extractor ABI.
    public static bool IsDateTimeUnion(TypeDefinition type)
    {
        if (type.FullName != "System.Time.DateTime" || !type.IsValueType || !type.IsPublic
            || !type.IsSealed || !type.IsSequentialLayout || type.HasNestedTypes
            || type.HasGenericParameters || type.HasEvents || Cases(type).Length != 0
            || !type.CustomAttributes.Any(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute"
                && RuntimeSignatures.IsCore(a.AttributeType.Scope))
            || type.Fields.Count != 3 || type.Fields.Any(f => !f.IsPrivate || f.IsStatic || f.HasMarshalInfo)) return false;
        var variants = new[] { "System.Time.LocalDateTime", "System.Time.ZonedDateTime" };
        var tag = type.Fields.SingleOrDefault(f => f.Name == "<Tag>");
        if (tag?.FieldType.MetadataType != MetadataType.Byte
            || !type.Fields.Where(f => f != tag).Select(f => f.FieldType.FullName).Order().SequenceEqual(variants.Order())
            || type.Fields.Where(f => f != tag).Any(f => !RuntimeSignatures.IsCore(f.FieldType.Scope))) return false;
        var constructors = type.Methods.Where(m => m.IsConstructor && m.IsPublic).ToArray();
        var extractors = type.Methods.Where(m => m.Name == "TryGetValue").ToArray();
        return constructors.Length == 2 && constructors.All(m => !m.IsStatic && m.Parameters.Count == 1
                && !m.Parameters[0].ParameterType.IsByReference && RuntimeSignatures.IsCore(m.Parameters[0].ParameterType.Scope))
            && constructors.Select(m => m.Parameters[0].ParameterType.FullName).Order().SequenceEqual(variants.Order())
            && extractors.Length == 2 && extractors.All(m => m.IsPublic && !m.IsStatic && m.Parameters.Count == 1
                && m.ReturnType.MetadataType == MetadataType.Boolean && m.Parameters[0].IsOut
                && m.Parameters[0].ParameterType is ByReferenceType byref && RuntimeSignatures.IsCore(byref.ElementType.Scope))
            && extractors.Select(m => ((ByReferenceType)m.Parameters[0].ParameterType).ElementType.FullName).Order().SequenceEqual(variants.Order())
            && type.Properties.Any(p => p.Name == "Value" && p.PropertyType.MetadataType == MetadataType.Object
                && p.GetMethod is { IsPublic: true, IsStatic: false } && p.SetMethod is null);
    }

    public static TypeDefinition[] Family(TypeDefinition carrier)
    {
        if (IsDateTimeUnion(carrier)) return [carrier];
        var cases = Cases(carrier);
        if (cases.Length == 0 || cases.Where((c, i) => c.Ordinal != i).Any())
            throw new InvalidDataException("Incomplete Raven union case metadata.");
        var companion = carrier.HasGenericParameters
            ? carrier.Module.Types.SingleOrDefault(t => CompanionTarget(t) == carrier.FullName)
                ?? throw new InvalidDataException("Missing generic union companion.")
            : carrier;
        if (companion.HasGenericParameters || companion != carrier &&
            (!companion.IsPublic || !companion.IsAbstract || !companion.IsSealed
                || companion.HasFields || companion.HasMethods || companion.HasInterfaces))
            throw new InvalidDataException("Unsupported generic union companion.");
        if (cases.Length != companion.NestedTypes.Count)
            throw new InvalidDataException("Union companion case count mismatch.");
        var members = cases.Select(c => companion.NestedTypes.SingleOrDefault(t =>
            t.FullName.Replace('/', '+') == c.MetadataName && t.Name.Split('`')[0] == c.Name)
            ?? throw new InvalidDataException("Unresolved Raven union case metadata.")).ToArray();
        return new[] { carrier }.Concat(companion == carrier ? [] : new[] { companion }).Concat(members).ToArray();
    }

    public static Case[] ValidateNestedCases(TypeDefinition carrier)
    {
        var cases = Cases(carrier);
        if (CompanionTarget(carrier) is not null || cases.Length != carrier.NestedTypes.Count
            || cases.Where((c, i) => c.Ordinal != i || !carrier.NestedTypes.Any(t =>
                t.FullName.Replace('/', '+') == c.MetadataName && t.Name == c.Name)).Any())
            throw new InvalidDataException("Raven union case metadata does not describe the nongeneric case family.");
        return cases;
    }
}

static class RavenUnionMetadataChecks
{
    public static void MutateCompanion(string path, string output, string mode)
    {
        using var image = AssemblyDefinition.ReadAssembly(path);
        var originalReferences = image.MainModule.AssemblyReferences.ToHashSet();
        var companion = image.MainModule.Types.Single(t => RavenUnionMetadata.CompanionTarget(t) is not null);
        var attribute = companion.CustomAttributes.Single(a => a.AttributeType.FullName == RavenUnionMetadata.CompanionAttribute);
        if (mode == "missing") companion.CustomAttributes.Remove(attribute);
        else if (mode == "wrong-target") attribute.ConstructorArguments[0] = new CustomAttributeArgument(
            attribute.ConstructorArguments[0].Type, "UnionMetadataProbe.Missing`1");
        else throw new InvalidDataException("Unknown companion metadata mutation.");
        foreach (var method in image.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in image.MainModule.AssemblyReferences.Where(r => !originalReferences.Contains(r)).ToArray())
        {
            if (image.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Companion mutation introduced an external scope.");
            image.MainModule.AssemblyReferences.Remove(reference);
        }
        image.Write(output);
    }

    public static void Run(string path)
    {
        using var image = AssemblyDefinition.ReadAssembly(path);
        var module = image.MainModule;
        var carriers = module.Types.Where(t => t.CustomAttributes.Any(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)).ToArray();
        if (carriers.Length == 0) throw new InvalidDataException("No Raven union case metadata found.");
        foreach (var carrier in carriers)
        {
            var cases = RavenUnionMetadata.Cases(carrier);
            foreach (var item in cases)
            {
                var caseType = module.GetType(item.MetadataName.Replace('+', '/'));
                if (caseType is null || caseType.Name.Split('`')[0] != item.Name || caseType.DeclaringType is null)
                    throw new InvalidDataException("Unresolved Raven union case metadata.");
                if (caseType.DeclaringType != carrier && RavenUnionMetadata.CompanionTarget(caseType.DeclaringType) != carrier.FullName)
                    throw new InvalidDataException("Raven union companion does not identify the case carrier.");
            }
            Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(new {
                Carrier = carrier.FullName,
                Cases = cases,
                Companions = module.Types.Where(t => RavenUnionMetadata.CompanionTarget(t) == carrier.FullName).Select(t => t.FullName).ToArray()
            }));
        }
        foreach (var companion in module.Types)
            if (RavenUnionMetadata.CompanionTarget(companion) is { } target && !carriers.Any(t => t.FullName == target))
                throw new InvalidDataException("Unresolved Raven union companion target.");
    }
}
