namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    // Follow inline owned value storage only. References/vectors terminate layout recursion;
    // scoped parameters are checked when a concrete construction supplies their storage.
    internal void ValidateValueLayouts()
    {
        var active = new HashSet<SignatureType>();
        var complete = new HashSet<SignatureType>();
        void Visit(SignatureType signature, int depth)
        {
            var definition = signature.ClassType ?? signature.GenericInstance?.Definition;
            if (definition is not { IsValueType: true }) return;
            if (complete.Contains(signature)) return;
            if (depth >= 64 || active.Count + complete.Count >= 4096)
                throw new InvalidDataException("value layout expansion exceeds limit");
            if (!active.Add(signature)) throw new InvalidDataException("recursive value-type storage");
            var arguments = signature.GenericInstance?.TypeArguments;
            SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } position && arguments is not null
                ? arguments[position]
                : type.GenericInstance is { } instance ? instance.Definition.MakeGenericInstance(instance.TypeArguments.Select(Substitute).ToArray())
                : type;
            foreach (var field in definition.Fields) Visit(Substitute(field.FieldType), depth + 1);
            active.Remove(signature);
            complete.Add(signature);
        }
        foreach (var type in types.Where(t => t.IsValueType)) Visit(type.OpenSignature, 0);
        foreach (var field in types.SelectMany(t => t.Fields)) Visit(field.FieldType, 0);
    }
}
