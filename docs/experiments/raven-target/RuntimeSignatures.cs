using Mono.Cecil;

// CLI signature mechanics shared by bounded runtime catalogs. This does not admit
// APIs: each catalog still selects its types, members and receiver conventions.
static class RuntimeSignatures
{
    public static bool IsCore(IMetadataScope? scope) => scope?.Name == CoreDeclarations.Identity
        || scope is ModuleDefinition module && module.Assembly.Name.Name == CoreDeclarations.Identity;

    public static TypeReference Close(TypeReference type, TypeReference owner, int depth = 0, GenericInstanceMethod? method = null, bool pointers = false, bool allowOpenMethodParameters = false)
    {
        if (depth > 32) throw new InvalidDataException("Signature nesting limit exceeded.");
        TypeReference Nested(TypeReference t) => Close(t, owner, depth + 1, method, pointers, allowOpenMethodParameters);
        if (type is GenericParameter parameter)
        {
            if (parameter.Type == GenericParameterType.Method)
            {
                if (method is null && allowOpenMethodParameters) return parameter;
                if (method is null || parameter.Position < 0 || parameter.Position >= method.GenericArguments.Count)
                    throw new InvalidDataException("Unsupported open method signature parameter.");
                return Nested(method.GenericArguments[parameter.Position]);
            }
            if (parameter.Type != GenericParameterType.Type || owner is not GenericInstanceType generic
                || parameter.Position < 0 || parameter.Position >= generic.GenericArguments.Count)
                throw new InvalidDataException("Unsupported open signature parameter.");
            return Nested(generic.GenericArguments[parameter.Position]);
        }
        if (type is GenericInstanceType instance)
        {
            var closed = new GenericInstanceType(instance.ElementType);
            foreach (var argument in instance.GenericArguments) closed.GenericArguments.Add(Nested(argument));
            return closed;
        }
        if (type is ByReferenceType byref)
        {
            var element = Nested(byref.ElementType);
            if (element is ByReferenceType) throw new InvalidDataException("Nested managed references are unsupported.");
            return new ByReferenceType(element);
        }
        if (pointers && type is PointerType pointer) return new PointerType(Nested(pointer.ElementType));
        if (type is ArrayType array && array.IsVector) return new ArrayType(Nested(array.ElementType));
        if (type is TypeSpecification || type.HasGenericParameters)
            throw new InvalidDataException("Unsupported signature shape: " + type.FullName);
        return type;
    }

    public static string Map(TypeReference type, Func<TypeReference, string?> catalog, bool returns = false)
    {
        if (returns && type.MetadataType == MetadataType.Void) return "noresult";
        if (type is ByReferenceType byref) return Map(byref.ElementType, catalog) + "&";
        return catalog(type) ?? PrimitiveBindings.Type(type) ?? (type.MetadataType switch {
            MetadataType.Double => "Double", MetadataType.Boolean => "Boolean", MetadataType.Int32 => "Int32", MetadataType.String => "String",
            _ when type.FullName == "System.Void" && type.IsValueType && IsCore(type.Scope) => "Void",
            _ => throw new InvalidDataException("Unsupported catalog signature type: " + type.FullName)
        });
    }

    public static (string[] Args, string Result) Match(MethodReference reference, MethodDefinition definition,
        Func<TypeReference, string?> catalog, bool pointers = false, bool allowOpenMethodParameters = false)
    {
        var method = reference as GenericInstanceMethod;
        var convention = method is null ? MethodCallingConvention.Default : MethodCallingConvention.Generic;
        if (!definition.IsPublic || reference.Name != definition.Name || reference.ExplicitThis || definition.ExplicitThis
            || reference.HasGenericParameters || definition.GenericParameters.Count != (method?.GenericArguments.Count ?? 0)
            || reference.CallingConvention != convention || definition.CallingConvention != convention
            || reference.HasThis != definition.HasThis
            || reference.DeclaringType.GetElementType().FullName != definition.DeclaringType.FullName
            || (reference.DeclaringType is GenericInstanceType owner ? owner.GenericArguments.Count : 0) != definition.DeclaringType.GenericParameters.Count)
            throw new InvalidDataException("Unsupported runtime member signature.");
        string Resolve(TypeReference t, bool returns = false) => Map(Close(t, reference.DeclaringType, method: method, pointers: pointers, allowOpenMethodParameters: allowOpenMethodParameters), catalog, returns && t.MetadataType == MetadataType.Void);
        var args = reference.Parameters.Select(p => Resolve(p.ParameterType)).ToArray();
        var result = Resolve(reference.ReturnType, true);
        if (!args.SequenceEqual(definition.Parameters.Select(p => Resolve(p.ParameterType)))
            || result != Resolve(definition.ReturnType, true))
            throw new InvalidDataException("Runtime reference/definition signature mismatch.");
        return (args, result);
    }
}
