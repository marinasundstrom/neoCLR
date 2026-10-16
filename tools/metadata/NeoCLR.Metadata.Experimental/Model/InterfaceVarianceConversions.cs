namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    // Reference views only. This is also used for nested argument conversion; it
    // deliberately excludes primitive materialization, boxing and writable aliases.
    internal bool HasVariantReferenceConversion(SignatureType source, SignatureType target, int depth = 0)
    {
        if (source == target) return true;
        if (depth >= 64 || !Reference(source) || !Reference(target)) return false;
        if (source.ClassType is { } derived && target.ClassType is { } ancestor && derived.DerivesFrom(ancestor)) return true;
        // Local subclasses retain an explicit dependency base contract. Reuse the
        // ordinary reference-conversion proof, including transitive local ancestry.
        if (source.ClassType is { } local && target.ImportedType is { } dependency && local.HasExternalBase(dependency)) return true;
        if (source.ImportedType is { } imported && target.ImportedType is { } external && HasDeclaredClassBase(imported, external)) return true;
        if (target.ClassType is { IsNativeObjectRoot: true } || target.ImportedType is { } root && Equals(root, ExternalObjectRoot)) return true;
        if (!Interface(target)) return false;
        var views = Views(source).Distinct().ToArray();
        if (views.Contains(target)) return true;
        return views.Count(view => Compatible(view, target)) == 1;

        bool Reference(SignatureType type) => type.ArrayElement is not null ||
            type.ClassType is { IsStatic: false, IsValueType: false, NativePrimitive: null } ||
            type.GenericInstance is { Definition.IsStatic: false, Definition.IsValueType: false } ||
            type.ImportedType is { IsValueType: false } imported &&
                !(nativeImportedDefinitions.TryGetValue(InterfaceDefinition(imported), out var definition) && definition.NativePrimitive is not null);
        bool Interface(SignatureType type) => type.ClassType?.IsInterface == true ||
            type.GenericInstance?.Definition.IsInterface == true || type.ImportedType is { } imported && IsAuthoredInterface(imported);
        bool Compatible(SignatureType actual, SignatureType expected)
        {
            IReadOnlyList<SignatureType> from, to;
            Func<int, GenericVariance> variance;
            if (actual.GenericInstance is { } a && expected.GenericInstance is { } b && ReferenceEquals(a.Definition, b.Definition))
            {
                from = a.TypeArguments; to = b.TypeArguments;
                variance = i => a.Definition.Definition.GenericParameterVariance?[i] ?? GenericVariance.Invariant;
            }
            else if (actual.ImportedType is { } aImport && expected.ImportedType is { } bImport && InterfaceDefinition(aImport).Equals(InterfaceDefinition(bImport)))
            {
                from = aImport.TypeArguments; to = bImport.TypeArguments;
                variance = i => ImportedVariance(aImport, i);
            }
            else return false;
            if (from.Count != to.Count) return false;
            for (int i = 0; i < from.Count; i++)
            {
                if (from[i] == to[i]) continue;
                if (variance(i) == GenericVariance.Invariant ||
                    !HasVariantReferenceConversion(variance(i) == GenericVariance.Covariant ? from[i] : to[i],
                        variance(i) == GenericVariance.Covariant ? to[i] : from[i], depth + 1)) return false;
            }
            return true;
        }
        IEnumerable<SignatureType> Views(SignatureType type)
        {
            var pending = new Stack<SignatureType>(); pending.Push(type);
            var seen = new HashSet<SignatureType>();
            while (pending.TryPop(out var current))
            {
                if (!seen.Add(current)) continue;
                if (seen.Count > 4096) throw new InvalidDataException("variant interface conversion graph exceeds limit");
                if (Interface(current)) yield return current;
                var owner = current.GenericInstance?.Definition ?? current.ClassType;
                var arguments = current.GenericInstance?.TypeArguments ?? current.ImportedType?.TypeArguments ?? [];
                SignatureType Substitute(SignatureType part) => part.TypeParameterIndex is { } i && arguments.Count != 0 ? arguments[i]
                    : part.GenericInstance is { } nested ? nested.Definition.MakeGenericInstance(nested.TypeArguments.Select(Substitute).ToArray())
                    : part.ImportedType is { } imported ? imported.Substitute(Substitute)
                    : part.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element)) : part;
                if (owner is not null)
                    foreach (var parent in owner.InheritedContracts()) pending.Push(Substitute(parent));
                if (current.ImportedType is { } authoredOwner && authoredInterfaceBases.TryGetValue(InterfaceDefinition(authoredOwner), out var edges))
                    foreach (var parent in edges) pending.Push(parent.Substitute(Substitute));
                if (current.ImportedType is { } importedOwner && nativeImportedDefinitions.TryGetValue(InterfaceDefinition(importedOwner), out var definition))
                    foreach (var relationship in definition.Interfaces)
                    {
                        var parent = ImportReference(relationship.InterfaceType.Resolve(), CoreLibrary);
                        if (relationship.TypeArguments.Count != 0)
                            parent = parent.MakeGenericInstance(relationship.TypeArguments.Select(t => Substitute(ImportNativeSignatureType(t, CoreLibrary, null))).ToArray());
                        pending.Push(parent);
                    }
            }
        }
    }
}
