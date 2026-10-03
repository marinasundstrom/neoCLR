namespace NeoCLR.Metadata.Experimental.Introspection;

public sealed partial class MetadataLoadContext
{
    internal IReadOnlyList<TypeInfo> ProjectInterfaceClosure(TypeInfo owner)
    {
        // Keep traversal iterative and local: lazy closures never depend on other closures.
        var result = new List<TypeInfo>();
        var seen = new HashSet<TypeInfo>();
        var active = new HashSet<NominalTypeInfo> { DefinitionOf(owner) };
        var pending = new Stack<(TypeInfo View, bool Exit)>();
        PushEdges(owner);
        var edges = 0;
        while (pending.TryPop(out var next))
        {
            var definition = DefinitionOf(next.View);
            if (next.Exit) { active.Remove(definition); continue; }
            if (++edges > 65536) throw new InvalidDataException("metadata interface edge limit exceeded");
            // Definition identity catches recursive inheritance even when arguments keep changing.
            if (active.Contains(definition)) throw new InvalidDataException("cyclic metadata interface inheritance");
            if (!seen.Add(next.View)) continue;
            if (seen.Count > 4096) throw new InvalidDataException("metadata interface expansion limit exceeded");
            result.Add(next.View);
            active.Add(definition);
            pending.Push((next.View, true));
            PushEdges(next.View);
        }
        return Array.AsReadOnly(result.ToArray());

        void PushEdges(TypeInfo view)
        {
            var direct = view switch
            {
                NominalTypeInfo nominal => nominal.GetDeclaredInterfaces(),
                ConstructedTypeInfo constructed => constructed.GetDeclaredInterfaces(),
                _ => throw new InvalidDataException("non-nominal metadata interface owner")
            };
            for (var i = direct.Count - 1; i >= 0; i--) pending.Push((direct[i], false));
        }
        static NominalTypeInfo DefinitionOf(TypeInfo view) => view switch
        {
            NominalTypeInfo nominal => nominal,
            ConstructedTypeInfo constructed => constructed.Definition,
            _ => throw new InvalidDataException("non-nominal metadata interface relationship")
        };
    }
}
