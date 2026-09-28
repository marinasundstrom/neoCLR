using Mono.Cecil;

// neoCLR's value-tuple identity. This is not the CLR reference Tuple family.
static class TupleBindings
{
    public static bool IsName(string type) => type.StartsWith("System.Tuple<", StringComparison.Ordinal);

    public static bool IsDefinition(TypeDefinition type) => type.Namespace == "System"
        && type.GenericParameters.Count is >= 1 and <= 7
        && type.Name == "Tuple`" + type.GenericParameters.Count;

    public static string Declarations => string.Join("\n", Enumerable.Range(1, 7).Select(n =>
    {
        var types = Enumerable.Range(1, n).Select(i => "T" + i).ToArray();
        return "public struct Tuple<" + string.Join(',', types) + "> { "
            + string.Join(' ', types.Select((t, i) => $"public {t} Item{i + 1};"))
            + " public Tuple(" + string.Join(',', types.Select((t, i) => $"{t} item{i + 1}")) + ") { } }";
    }));

    public static void Validate(TypeDefinition type)
    {
        if (!IsDefinition(type) || !type.IsPublic || !type.IsValueType || !type.IsSealed || !type.IsSequentialLayout
            || type.PackingSize is not (-1 or 0) || type.ClassSize > 0
            || type.HasInterfaces || type.HasProperties || type.HasEvents || type.HasNestedTypes
            || type.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || type.Fields.Count != type.GenericParameters.Count
            || type.Fields.Where((f, i) => !f.IsPublic || f.IsStatic || f.IsInitOnly || f.HasMarshalInfo
                || f.Name != "Item" + (i + 1) || f.FieldType != type.GenericParameters[i]).Any())
            throw new InvalidDataException("Invalid tuple layout: " + type.FullName);
    }

    public static string? Type(TypeReference type)
    {
        if (type is not GenericInstanceType g || !RuntimeSignatures.IsCore(type.Scope)
            || !g.ElementType.FullName.StartsWith("System.Tuple`", StringComparison.Ordinal)) return null;
        var definition = g.ElementType.Resolve();
        Validate(definition);
        if (!type.IsValueType || g.GenericArguments.Count != definition.GenericParameters.Count)
            throw new InvalidDataException("Invalid tuple construction.");
        var args = g.GenericArguments.Select(GenericUnionBindings.Type).ToArray();
        return args.All(a => a is not null) ? "System.Tuple<" + string.Join(',', args) + ">" : null;
    }

    public static CollectionBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = ((GenericInstanceType)reference.DeclaringType).GenericArguments.Select(GenericUnionBindings.Type);
        if (!definition.IsConstructor || definition.IsStatic || definition.IsVirtual || result != "noresult"
            || !args.SequenceEqual(expected)) throw new InvalidDataException("Invalid tuple constructor.");
        return new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})");
    }

    public static ApplicationTypes.FieldShape? Field(FieldReference reference)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var definition = reference.Resolve();
        if (definition is null || !definition.IsPublic || definition.IsStatic
            || !LibraryImplementation.SameType(reference.FieldType, definition.FieldType))
            throw new InvalidDataException("Invalid tuple field signature.");
        var fieldType = GenericUnionBindings.Type(RuntimeSignatures.Close(definition.FieldType, reference.DeclaringType))
            ?? throw new InvalidDataException("Unsupported tuple component.");
        return new(owner, fieldType, definition.Name, true);
    }
}
