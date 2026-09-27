using Mono.Cecil;

// Exact development numeric surface; no general parsing interface or external math hierarchy.
static class NumberBindings
{
    public const string Contract = "System.Number`1";
    public static readonly string[] Types = ["SByte", "Byte", "Int16", "UInt16", "Int32", "UInt32", "Int64", "UInt64", "Single", "Double"];
    public static readonly string[] Operators = ["op_Addition", "op_Subtraction", "op_Multiply", "op_Division"];
    public static bool IsNumber(string name) => Types.Contains(name.Replace("System.", ""));
    public static bool HasNewParser(string name) => IsNumber(name) && name is not ("Int32" or "Int64" or "System.Int32" or "System.Int64") || name is "Boolean" or "System.Boolean";

    public static void Project(ModuleDefinition module)
    {
        var contract = new TypeDefinition("System", "Number`1", TypeAttributes.Public | TypeAttributes.Interface | TypeAttributes.Abstract);
        module.Types.Add(contract);
        var parameter = new GenericParameter("T", contract);
        contract.GenericParameters.Add(parameter);
        var comparable = new GenericInstanceType(module.GetType("System.ComparableTo`1"));
        comparable.GenericArguments.Add(parameter);
        contract.Interfaces.Add(new InterfaceImplementation(comparable));
        AddMembers(contract, parameter, true);
        foreach (var name in Types)
        {
            var type = module.GetType("System." + name);
            var scalar = type.Methods.Single(m => m.Name == "CompareTo").Parameters[0].ParameterType;
            var number = new GenericInstanceType(contract);
            number.GenericArguments.Add(scalar);
            type.Interfaces.Add(new InterfaceImplementation(number));
            AddMembers(type, scalar, false);
        }
    }

    static void AddMembers(TypeDefinition owner, TypeReference scalar, bool contract)
    {
        var attributes = MethodAttributes.Public | MethodAttributes.Static | MethodAttributes.HideBySig | MethodAttributes.SpecialName;
        if (contract) attributes |= MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot;
        foreach (var name in new[] { "Zero", "One" })
        {
            var method = new MethodDefinition("get_" + name, attributes, scalar);
            owner.Methods.Add(method);
            owner.Properties.Add(new PropertyDefinition(name, PropertyAttributes.None, scalar) { GetMethod = method });
        }
        foreach (var name in Operators)
        {
            var method = new MethodDefinition(name, attributes, scalar);
            method.Parameters.Add(new ParameterDefinition("left", ParameterAttributes.None, scalar));
            method.Parameters.Add(new ParameterDefinition("right", ParameterAttributes.None, scalar));
            owner.Methods.Add(method);
        }
    }

    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType(Contract);
        if (type is null || !type.IsPublic || !type.IsInterface || !type.IsAbstract || type.HasFields || type.HasEvents
            || type.GenericParameters.Count != 1 || type.GenericParameters[0].HasConstraints
            || type.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant
            || type.Interfaces.Count != 1 || type.Interfaces[0].InterfaceType.FullName != "System.ComparableTo`1<T>"
            || type.Methods.Count != 6 || type.Methods.Select(m => m.Name).Distinct().Count() != 6
            || type.Properties.Count != 2 || !type.Properties.Select(p => p.Name).Order().SequenceEqual(new[] { "One", "Zero" })
            || type.Properties.Any(p => p.SetMethod is not null || p.GetMethod != type.Methods.SingleOrDefault(m => m.Name == "get_" + p.Name) || p.PropertyType != type.GenericParameters[0]))
            throw new InvalidDataException("Invalid Number contract.");
        foreach (var method in type.Methods)
        {
            var arity = Operators.Contains(method.Name) ? 2 : method.Name is "get_Zero" or "get_One" ? 0 : -1;
            if (!method.IsStatic || !method.IsAbstract || !method.IsVirtual || !method.IsPublic || !method.IsNewSlot
                || method.HasBody || method.HasGenericParameters || method.HasOverrides || method.ExplicitThis
                || method.Parameters.Count != arity || method.ReturnType != type.GenericParameters[0]
                || method.Parameters.Any(p => p.ParameterType != type.GenericParameters[0] || p.IsOut))
                throw new InvalidDataException("Invalid Number member.");
        }
    }

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        var name = reference.DeclaringType.Name;
        var numericMember = IsNumber(name) && (Operators.Contains(reference.Name) || reference.Name is "get_Zero" or "get_One");
        var parser = HasNewParser(name) && reference.Name == "Parse";
        if (!numericMember && !parser) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => GenericUnionBindings.Type(t) ?? ErrorBindings.Type(t));
        var expectedResult = parser ? $"System.Result<{name},System.{(name == "Boolean" ? "Boolean" : "Number")}ParseError>" : name;
        var expectedArgs = parser ? new[] { "String" } : Operators.Contains(reference.Name) ? new[] { name, name } : [];
        if (reference.HasThis || definition.HasThis || reference.ExplicitThis || definition.HasGenericParameters
            || !definition.IsPublic || result != expectedResult || !args.SequenceEqual(expectedArgs))
            throw new InvalidDataException("Invalid concrete numeric signature: " + reference.FullName);
        return new("System." + name + "::" + reference.Name, args, result);
    }
}
