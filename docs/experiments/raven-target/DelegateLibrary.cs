using Mono.Cecil;

// Ordinary CLI delegates are declarations. Their Invoke execution belongs to the
// runtime; this admits only the existing invariant Func family, not delegate bodies.
static class DelegateLibrary
{
    public static bool IsMatched(TypeDefinition type) => type.Namespace == "System"
        && type.Name.StartsWith("Func`", StringComparison.Ordinal) && ApplicationTypes.IsLibrary(type)
        && type.BaseType?.FullName == "System.MulticastDelegate";
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core)
    {
        for (var arity = 1; arity <= 5; arity++)
        {
            var name = "System.Func`" + arity;
            var type = source.GetType(name) ?? throw new InvalidDataException("Missing Func arity " + arity);
            var contract = core.GetType(name) ?? throw new InvalidDataException("Missing Func contract.");
            foreach (var candidate in new[] { type, contract })
            {
                if (!candidate.IsPublic || !candidate.IsSealed || candidate.IsValueType || candidate.IsAbstract
                    || candidate.IsInterface || candidate.HasFields || candidate.HasInterfaces || candidate.HasEvents
                    || candidate.HasNestedTypes || candidate.IsExplicitLayout
                    || candidate.BaseType?.FullName != "System.MulticastDelegate"
                    || !RuntimeSignatures.IsCore(candidate.BaseType.Scope)
                    || candidate.GenericParameters.Count != arity
                    || candidate.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
                    || candidate.Methods.Count != 2)
                    throw new InvalidDataException("Unsupported Func declaration: " + name);
                var ctor = candidate.Methods.SingleOrDefault(m => m.IsConstructor);
                var invoke = candidate.Methods.SingleOrDefault(m => m.Name == "Invoke");
                bool RuntimeMethod(MethodDefinition m) => m.IsPublic && m.HasThis && !m.ExplicitThis
                    && m.IsRuntime && !m.HasBody && !m.HasGenericParameters && !m.HasOverrides
                    && m.CallingConvention == MethodCallingConvention.Default;
                bool Parameter(TypeReference t, int position) => t is GenericParameter p
                    && p.Owner == candidate && p.Position == position && p.Type == GenericParameterType.Type;
                if (ctor is null || invoke is null || !RuntimeMethod(ctor) || !RuntimeMethod(invoke)
                    || ctor.IsVirtual || ctor.ReturnType.MetadataType != MetadataType.Void || ctor.Parameters.Count != 2
                    || ctor.Parameters[0].ParameterType.MetadataType != MetadataType.Object
                    || ctor.Parameters[1].ParameterType.MetadataType != MetadataType.IntPtr
                    || !invoke.IsVirtual || !invoke.IsNewSlot || invoke.IsFinal || invoke.IsAbstract
                    || !Parameter(invoke.ReturnType, arity - 1) || invoke.Parameters.Count != arity - 1
                    || invoke.Parameters.Where((p, i) => p.IsOut || !Parameter(p.ParameterType, i)).Any())
                    throw new InvalidDataException("Unsupported Func invocation contract: " + name);
            }
            ApplicationTypes.BindLibrary(type, "System.Func");
            _ = ApplicationTypes.Type(type);
        }
        if (source.Types.Any(t => t.Namespace == "System" && t.Name.StartsWith("Func`", StringComparison.Ordinal)
            && !IsMatched(t))) throw new InvalidDataException("Unsupported Func arity.");
        return [];
    }
    public static string Declaration(TypeDefinition type, string name, Func<TypeReference, bool, string> map)
    {
        var invoke = type.Methods.Single(m => m.Name == "Invoke");
        return $".delegate {name}\n.method instance Invoke({string.Join(',', invoke.Parameters.Select((p, i) => map(p.ParameterType, false) + " arg" + (i + 1)))}) -> {map(invoke.ReturnType, true)}\n.end\n.end\n";
    }
}
