using Mono.Cecil;
using System.Text.RegularExpressions;

// A bounded implementation fragment for an existing reference contract. No name-based
// substitution of guest/application types: every exported signature must match the core.
static class LibraryImplementation
{
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner)
    {
        if (!Regex.IsMatch(owner, @"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+$"))
            throw new InvalidDataException("Invalid library owner.");
        var type = source.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t)) ?? source.Types.SingleOrDefault(t => t.FullName.Split('`')[0] == owner)
            ?? throw new InvalidDataException("Missing namespace implementation: " + owner);
        var contract = core.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t)) ?? core.Types.SingleOrDefault(t => t.FullName.Split('`')[0] == owner)
            ?? throw new InvalidDataException("Missing namespace reference contract: " + owner);
        if (!(type.IsAbstract && type.IsSealed))
            return InstanceRoots(type, contract, owner);
        if (!contract.IsPublic || contract.HasGenericParameters || contract.IsInterface)
            throw new InvalidDataException("Unsupported library reference owner.");
        if (!type.IsPublic || !type.IsAbstract || !type.IsSealed || type.HasGenericParameters || type.HasFields || type.HasInterfaces)
            throw new InvalidDataException("Library fragment requires a public nongeneric namespace container without fields.");
        var methods = type.Methods.ToArray();
        if (methods.Length == 0) throw new InvalidDataException("Empty library implementation.");
        foreach (var method in methods)
        {
            CheckMethod(method);
            if (!method.IsPublic || !method.IsStatic || method.IsConstructor || !method.HasBody
                || !Regex.IsMatch(method.Name, @"^[A-Za-z_][A-Za-z0-9_]*$")
                || method.Parameters.Any(p => p.IsOut || p.ParameterType.IsByReference)
                || method.ReturnType.MetadataType == MetadataType.Void)
                throw new InvalidDataException("Unsupported library export: " + method.FullName);
            var matches = contract.Methods.Where(m => m.IsPublic && m.IsStatic && m.Name == method.Name && m.Parameters.Count == method.Parameters.Count && m.GenericParameters.Count == method.GenericParameters.Count
                && SameType(m.ReturnType, method.ReturnType)
                && m.Parameters.Zip(method.Parameters).All(p => p.First.Name == p.Second.Name && SameType(p.First.ParameterType, p.Second.ParameterType))).ToArray();
            if (matches.Length == 1) CheckMethod(matches[0]);
            if (matches.Length != 1) throw new InvalidDataException("Library export does not match reference contract: " + method.FullName);
        }
        return methods;
    }

    static MethodDefinition[] InstanceRoots(TypeDefinition type, TypeDefinition contract, string owner)
    {
        // A single explicitly selected reference/implementation pair. Never alias arbitrary
        // guest types by namespace/name, and never execute reference-assembly stub bodies.
        foreach (var candidate in new[] { type, contract })
            if (!candidate.IsPublic || candidate.IsValueType || candidate.IsInterface || candidate.IsAbstract
                || candidate.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant) || candidate.HasNestedTypes || candidate.HasEvents
                || candidate.BaseType?.FullName != "System.Object" || candidate.IsExplicitLayout)
                throw new InvalidDataException("Unsupported instance library owner.");
        if (type.GenericParameters.Count != contract.GenericParameters.Count)
            throw new InvalidDataException("Instance library generic arity does not match reference contract.");
        if (type.IsSealed != contract.IsSealed) throw new InvalidDataException("Instance library sealing does not match reference contract.");
        bool MatchType(TypeReference left, TypeReference right)
        {
            if (left is GenericInstanceType l)
                return right is GenericInstanceType r && MatchType(l.ElementType, r.ElementType)
                    && l.GenericArguments.Count == r.GenericArguments.Count
                    && l.GenericArguments.Zip(r.GenericArguments).All(p => MatchType(p.First, p.Second));
            return (left.FullName == contract.FullName && right.FullName == type.FullName
                && left.Resolve() == contract && right.Resolve() == type) || SameType(left, right);
        }
        if (type.Interfaces.Count != contract.Interfaces.Count || type.Interfaces.Any(i =>
            contract.Interfaces.Count(c => MatchType(c.InterfaceType, i.InterfaceType)) != 1))
            throw new InvalidDataException("Instance library interfaces do not match reference contract.");
        bool MatchMethod(MethodDefinition left, MethodDefinition right) =>
            left.Name == right.Name && !left.HasGenericParameters && !left.ExplicitThis
            && left.CallingConvention == MethodCallingConvention.Default && left.IsStatic == right.IsStatic && left.IsConstructor == right.IsConstructor
            && left.IsVirtual == right.IsVirtual && left.IsFinal == right.IsFinal && left.IsNewSlot == right.IsNewSlot
            && MatchType(left.ReturnType, right.ReturnType) && left.Parameters.Count == right.Parameters.Count
            && left.Parameters.Zip(right.Parameters).All(p => p.First.Name == p.Second.Name && p.First.IsOut == p.Second.IsOut
                && MatchType(p.First.ParameterType, p.Second.ParameterType));
        if (type.Fields.Any(f => !f.IsPrivate || f.IsStatic || f.IsInitOnly || f.HasMarshalInfo)
            || contract.Fields.Any(f => !f.IsPrivate))
            throw new InvalidDataException("Instance library requires private mutable implementation fields.");
        var methods = type.Methods.ToArray();
        if (methods.Length == 0 || methods.Any(m => !m.IsPublic || !m.HasThis || !m.HasBody || m.HasGenericParameters
            || m.ExplicitThis || m.CallingConvention != MethodCallingConvention.Default
            || m.Parameters.Any(p => p.IsOut || p.ParameterType.IsByReference)))
            throw new InvalidDataException("Unsupported instance library export.");
        var expected = contract.Methods.Where(m => m.IsPublic).ToArray();
        if (expected.Length != methods.Length || methods.Any(m => expected.Count(e => MatchMethod(e, m)) != 1))
            throw new InvalidDataException("Instance library export does not match reference contract.");
        if (type.Properties.Count != contract.Properties.Count || type.Properties.Any(p =>
            contract.Properties.Count(c => c.Name == p.Name && MatchType(c.PropertyType, p.PropertyType)
                && c.Parameters.Count == p.Parameters.Count
                && c.Parameters.Zip(p.Parameters).All(a => MatchType(a.First.ParameterType, a.Second.ParameterType))
                && c.GetMethod?.Name == p.GetMethod?.Name && c.SetMethod?.Name == p.SetMethod?.Name) != 1))
            throw new InvalidDataException("Instance library property does not match reference contract.");
        ApplicationTypes.BindLibrary(type, owner);
        foreach (var method in methods) CheckMethod(method);
        return methods;
    }

    public static void CheckMethod(MethodDefinition method)
    {
        if (!method.HasGenericParameters) { ApplicationTypes.CheckMethod(method); return; }
        if (!method.IsStatic || method.ExplicitThis || method.DeclaringType.HasGenericParameters
            || method.CallingConvention != MethodCallingConvention.Generic
            || method.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported generic library signature: " + method.FullName);
    }

    public static string GenericName(MethodDefinition method) => method.Name + (method.HasGenericParameters
        ? "<" + string.Join(',', method.GenericParameters.Select(p => "T" + p.Position)) + ">" : "");

    public static bool SameType(TypeReference left, TypeReference right)
    {
        if (left is ArrayType la)
            return right is ArrayType ra && la.IsVector == ra.IsVector && la.Rank == ra.Rank && SameType(la.ElementType, ra.ElementType);
        if (left is GenericParameter lp)
            return right is GenericParameter rp && lp.Type == rp.Type && lp.Position == rp.Position;
        if (left is GenericInstanceType l)
            return right is GenericInstanceType r && l.IsValueType == r.IsValueType
                && SameType(l.ElementType, r.ElementType)
                && l.GenericArguments.Count == r.GenericArguments.Count
                && l.GenericArguments.Zip(r.GenericArguments).All(p => SameType(p.First, p.Second));
        return left.FullName == right.FullName && left.MetadataType == right.MetadataType && left.IsValueType == right.IsValueType
            && (left.MetadataType is not (MetadataType.Class or MetadataType.ValueType)
                || left.Resolve()?.Module.Assembly.Name.FullName == right.Resolve()?.Module.Assembly.Name.FullName);
    }

    public static TypeReference Close(TypeReference type, GenericInstanceMethod method, int depth = 0)
    {
        if (depth > 32) throw new InvalidDataException("Library signature nesting limit exceeded.");
        if (type is GenericParameter p && p.Type == GenericParameterType.Method)
            return method.GenericArguments[p.Position];
        if (type is GenericInstanceType generic)
        {
            var closed = new GenericInstanceType(generic.ElementType);
            foreach (var argument in generic.GenericArguments) closed.GenericArguments.Add(Close(argument, method, depth + 1));
            return closed;
        }
        if (type.ContainsGenericParameter) throw new InvalidDataException("Unsupported constructed library signature.");
        return type;
    }

    public static string QualifyHelpers(string text, string owner)
    {
        var helpers = Regex.Matches(text, @"(?m)^\.function ([^\(]+)\(").Select(m => m.Groups[1].Value)
            .Where(h => !h.StartsWith(owner + ".", StringComparison.Ordinal)).ToArray();
        // Adapters generated from open signatures must themselves declare the free
        // method parameters. Propagate through helper calls before qualifying names.
        var bodies = Regex.Matches(text, @"(?ms)^\.function ([^\(]+)\(.*?^\.end\r?$")
            .Where(m => helpers.Contains(m.Groups[1].Value))
            .ToDictionary(m => m.Groups[1].Value, m => m.Value);
        var parameters = helpers.ToDictionary(h => h, h => Regex.Matches(Regex.Replace(bodies[h], "\"(?:\\\\.|[^\"\\\\])*\"", ""), @"\bT[0-9]+\b")
            .Select(m => m.Value).ToHashSet());
        bool changed;
        do
        {
            changed = false;
            foreach (var helper in helpers)
                foreach (Match call in Regex.Matches(bodies[helper], @"(?m)^(?:call|ldftn) ([^\(]+)\("))
                    if (parameters.TryGetValue(call.Groups[1].Value, out var dependency))
                        foreach (var parameter in dependency)
                            changed |= parameters[helper].Add(parameter);
        } while (changed);
        foreach (var helper in helpers.Where(h => !h.StartsWith(owner + ".", StringComparison.Ordinal)))
        {
            var generic = parameters[helper].Count == 0 ? "" : "<" + string.Join(',',
                parameters[helper].OrderBy(p => int.Parse(p[1..]))) + ">";
            text = Regex.Replace(text, @"(?m)^(\.function |call |ldftn )" + Regex.Escape(helper) + @"(?=\()",
                m => m.Groups[1].Value + "neoCLR.Library." + owner + "." + helper + generic);
        }
        return text;
    }

    public static void Write(string source, string core, string owner, string output)
    {
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var projected = Path.Combine(output, "Implementation.dll");
        VoidProjection.Write(source, core, projected);
        UnionImport.WriteLibrary(projected, core, Path.Combine(output, "Implementation.neoil"), owner);
    }
}
