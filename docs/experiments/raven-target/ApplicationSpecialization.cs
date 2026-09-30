using Mono.Cecil;
using Mono.Cecil.Cil;

// Close bounded static application helpers before the ordinary checked importer.
// Number algorithms additionally resolve their supported constrained calls.
sealed class ApplicationSpecialization(ModuleDefinition core, IEnumerable<ModuleDefinition> applications)
{
    readonly Dictionary<Instruction, string> nativeSelfCalls = new();
    public string? NativeSelfCall(Instruction instruction) => nativeSelfCalls.GetValueOrDefault(instruction);

    readonly HashSet<ModuleDefinition> modules = applications.ToHashSet();
    readonly Dictionary<string, MethodDefinition> copies = new();
    readonly Dictionary<MethodDefinition, MethodDefinition> origins = new();
    public bool IsSpecialized(MethodDefinition method) => origins.ContainsKey(method);
    public MethodDefinition Origin(MethodDefinition method) => origins.GetValueOrDefault(method, method);

    public void Rewrite(MethodDefinition method)
    {
        if (!method.HasBody) return;
        foreach (var instruction in method.Body.Instructions)
            if (instruction.Operand is GenericInstanceMethod call && modules.Contains(call.Resolve().Module))
                instruction.Operand = Specialize(call);
    }

    MethodDefinition Specialize(GenericInstanceMethod call)
    {
        var source = call.Resolve();
        // Cecil may classify the target's inhabited System.Void token as CLI VOID.
        // Normalize only closed generic storage from this exact empty target type;
        // literal no-result returns and foreign/malformed void arguments stay distinct.
        for (var index = 0; index < call.GenericArguments.Count; index++)
        {
            var argument = call.GenericArguments[index];
            if (argument.MetadataType == MetadataType.Void && argument.FullName == "System.Void"
                && argument.IsValueType && RuntimeSignatures.IsCore(argument.Scope)
                && argument.Resolve() is { IsValueType: true } definition
                && definition.Module.Assembly.Name.FullName == core.Assembly.Name.FullName
                && !definition.Fields.Any(field => !field.IsStatic))
                call.GenericArguments[index] = new TypeReference("System", "Void", source.Module, argument.Scope, true);
        }
        var key = source.Module.Mvid + ":" + call.FullName + ":" +
            string.Join(";", call.GenericArguments.Select(MetadataIdentity.TypeKey));
        if (copies.TryGetValue(key, out var previous)) return previous;
        if (copies.Count >= 128 || !source.IsStatic || source.DeclaringType.HasGenericParameters || source.ExplicitThis
            || !source.HasBody || source.Body.HasExceptionHandlers || source.IsPInvokeImpl || source.IsInternalCall
            || source.GenericParameters.Count == 0 || source.GenericParameters.Count > 4
            || call.GenericArguments.Count != source.GenericParameters.Count)
            throw new InvalidDataException("Unsupported application specialization: " + call.FullName);
        for (var index = 0; index < source.GenericParameters.Count; index++)
        {
            var parameter = source.GenericParameters[index];
            var actual = call.GenericArguments[index];
            if (parameter.Attributes == GenericParameterAttributes.NonVariant && parameter.Constraints.Count == 0)
            {
                // Validate that the argument is closed; normal import still owns its
                // supported representation and access checks, including reference types.
                if (actual is ByReferenceType || actual.MetadataType is MetadataType.Void or MetadataType.TypedByReference)
                    throw new InvalidDataException("Unsupported application specialization argument: " + actual.FullName);
                _ = RuntimeSignatures.Close(actual, source.DeclaringType);
                continue;
            }
            if (parameter.Attributes != GenericParameterAttributes.NonVariant || parameter.Constraints.Count != 1
                || parameter.Constraints[0].ConstraintType.FullName != NumberBindings.Contract
                || !RuntimeSignatures.IsCore(parameter.Constraints[0].ConstraintType.Scope)
                || !actual.IsValueType || !NumberBindings.IsNumber(actual.FullName)
                || actual.FullName != "System." + actual.MetadataType)
                throw new InvalidDataException("Numeric specialization requires Number and a supported concrete numeric argument: " + call.FullName);
            // CLI intrinsic signatures carry an element code, not a TypeRef scope.
            // Cecil may synthesize mscorlib for that code; bind it to supplied core storage.
            call.GenericArguments[index] = core.GetType(actual.FullName).Methods.Single(m => m.Name == "CompareTo").Parameters[0].ParameterType;
        }
        TypeReference Close(TypeReference type, bool memberSignature = false) => type switch {
            GenericParameter p when p.Type == GenericParameterType.Method && p.Owner == source => call.GenericArguments[p.Position],
            GenericParameter p when memberSignature && p.Type == GenericParameterType.Type => p,
            GenericParameter => throw new InvalidDataException("Unclosed application specialization parameter."),
            ByReferenceType b => new ByReferenceType(Close(b.ElementType, memberSignature)),
            ArrayType a when a.IsVector => new ArrayType(Close(a.ElementType, memberSignature)),
            GenericInstanceType g => Construct(g.ElementType, g.GenericArguments.Select(argument => Close(argument, memberSignature))),
            TypeSpecification => throw new InvalidDataException("Unsupported application specialization type."),
            _ => type
        };
        MethodReference CloseMethod(MethodReference method)
        {
            if (method is GenericInstanceMethod generic)
            {
                var result = new GenericInstanceMethod(generic.ElementMethod);
                foreach (var argument in generic.GenericArguments) result.GenericArguments.Add(Close(argument));
                return result;
            }
            // Generic type signatures retain their type parameters; the constructed
            // owner carries substitution for the ordinary signature checker.
            TypeReference SignatureType(TypeReference type) => Close(type, memberSignature: true);
            var closed = new MethodReference(method.Name, SignatureType(method.ReturnType), Close(method.DeclaringType)) {
                HasThis = method.HasThis, ExplicitThis = method.ExplicitThis, CallingConvention = method.CallingConvention
            };
            foreach (var parameter in method.Parameters)
                closed.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, SignatureType(parameter.ParameterType)));
            return closed;
        }
        var copy = new MethodDefinition("__Specialized" + copies.Count + "_" + source.Name,
            source.Attributes, Close(source.ReturnType));
        // Register before recursion, preserving the source's visibility and owner.
        copies.Add(key, copy);
        origins.Add(copy, source);
        source.DeclaringType.Methods.Add(copy);
        foreach (var parameter in source.Parameters)
            copy.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, Close(parameter.ParameterType)));
        copy.Body.InitLocals = source.Body.InitLocals;
        copy.Body.MaxStackSize = source.Body.MaxStackSize;
        foreach (var local in source.Body.Variables) copy.Body.Variables.Add(new VariableDefinition(Close(local.VariableType)));
        var instructions = source.Body.Instructions.ToDictionary(i => i, _ => Instruction.Create(OpCodes.Nop));
        foreach (var instruction in source.Body.Instructions)
        {
            var target = instructions[instruction];
            target.Offset = instruction.Offset;
            target.OpCode = instruction.OpCode;
            target.Operand = instruction.Operand switch {
                Instruction branch => instructions[branch],
                Instruction[] branches => branches.Select(b => instructions[b]).ToArray(),
                ParameterDefinition p => copy.Parameters[p.Index],
                VariableDefinition v => copy.Body.Variables[v.Index],
                MethodReference m => CloseMethod(m),
                FieldReference f => new FieldReference(f.Name, Close(f.FieldType, memberSignature: true), Close(f.DeclaringType)),
                TypeReference t => Close(t),
                _ => instruction.Operand
            };
            copy.Body.Instructions.Add(target);
        }
        for (var index = 0; index < copy.Body.Instructions.Count; index++)
        {
            var prefix = copy.Body.Instructions[index];
            if (prefix.OpCode.Code != Code.Constrained) continue;
            if (prefix.Operand is not TypeReference concrete || !NumberBindings.IsNumber(concrete.FullName)
                || index + 1 >= copy.Body.Instructions.Count)
                throw new InvalidDataException("Unsupported numeric constrained receiver.");
            var instruction = copy.Body.Instructions[index + 1];
            if (instruction.Operand is MethodReference inherited && inherited.DeclaringType is GenericInstanceType selfOwner
                && selfOwner.ElementType.FullName == "System.ComparableTo`1" && selfOwner.GenericArguments.Count == 1
                && selfOwner.GenericArguments[0].FullName == "System.Runtime.CompilerServices.Self")
                selfOwner.GenericArguments[0] = concrete;
            if (instruction.OpCode.Code is not (Code.Call or Code.Callvirt) || instruction.Operand is not MethodReference member
                || !RuntimeSignatures.IsCore(member.DeclaringType.Scope)
                || !(member.DeclaringType.FullName == NumberBindings.Contract
                    || member.DeclaringType is GenericInstanceType owner && owner.GenericArguments.Count == 1
                        && owner.GenericArguments[0].FullName == concrete.FullName && owner.ElementType.FullName == "System.ComparableTo`1"))
                throw new InvalidDataException("Unsupported numeric constrained member.");
            var expected = member.Resolve();
            if (!expected.IsAbstract || !expected.IsVirtual || !expected.IsPublic)
                throw new InvalidDataException("Numeric dispatch requires an abstract interface contract.");
            var nativeSelf = member.DeclaringType.FullName == NumberBindings.Contract;
            var shape = RuntimeSignatures.Match(member, expected, t => nativeSelf && t.FullName == "System.Runtime.CompilerServices.Self" && RuntimeSignatures.IsCore(t.Scope) ? concrete.Name : PrimitiveBindings.Type(t));
            var concreteName = concrete.Name;
            if (shape.Args.Any(argument => argument != concreteName)
                || shape.Result != (member.Name == "CompareTo" ? "Int32" : concreteName))
                throw new InvalidDataException("Invalid numeric constrained signature.");
            var candidates = core.GetType(concrete.FullName).Methods.Where(m => m.Name == member.Name && m.IsStatic == expected.IsStatic
                && m.IsPublic && m.Parameters.Count == member.Parameters.Count
                && m.Parameters.All(p => p.ParameterType.FullName == concrete.FullName)
                && m.ReturnType.FullName == (member.Name == "CompareTo" ? "System.Int32" : concrete.FullName)).ToArray();
            if (candidates.Length != 1) throw new InvalidDataException("Missing exact numeric implementation.");
            if (nativeSelf)
                nativeSelfCalls[instruction] = $"callself {concrete.Name} = System.Number::{member.Name}({string.Join(',', expected.Parameters.Select(_ => "Self"))})";
            prefix.OpCode = OpCodes.Nop;
            prefix.Operand = null;
            instruction.OpCode = OpCodes.Call;
            instruction.Operand = candidates[0];
        }
        Rewrite(copy);
        return copy;
    }

    static GenericInstanceType Construct(TypeReference owner, IEnumerable<TypeReference> arguments)
    {
        var result = new GenericInstanceType(owner);
        foreach (var argument in arguments) result.GenericArguments.Add(argument);
        return result;
    }
}
