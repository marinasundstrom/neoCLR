namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, ImportedTypeReference> declaredClassBases = [];

    /// <summary>Declares an existing dependency's direct class base for output-side reference conversion validation.</summary>
    /// <param name="type">An output-owned nongeneric top-level external class reference.</param>
    /// <param name="baseType">Its direct base in the same exact dependency assembly.</param>
    /// <exception cref="ArgumentNullException">Either reference is null.</exception>
    /// <exception cref="ArgumentException">Foreign, value, interface, generic, nested, cross-assembly, cyclic or conflicting relationship.</exception>
    /// <exception cref="InvalidDataException">More than 4096 relationships are declared.</exception>
    /// <remarks>The host supplies truthful symbol facts; no reader is reopened. This does not author a new inherited definition or change dependency metadata. Native linking/verifying uses the actual dependency definitions. Generic and cross-assembly bases remain unsupported here.</remarks>
    public void DeclareClassBase(ImportedTypeReference type, ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(type); ArgumentNullException.ThrowIfNull(baseType);
        bool Supported(ImportedTypeReference value) => ReferenceEquals(value.Owner, this) && !value.IsValueType &&
            !IsAuthoredInterface(value) && value.GenericArity == 0 && value.TypeArguments.Count == 0 && value.DeclaringType is null;
        if (!Supported(type) || !Supported(baseType) || !type.AssemblyIdentity.Equals(baseType.AssemblyIdentity) ||
            type.Equals(baseType) || HasDeclaredClassBase(baseType, type))
            throw new ArgumentException("invalid or cyclic external class base");
        if (declaredClassBases.TryGetValue(type, out var existing))
        {
            if (!existing.Equals(baseType)) throw new ArgumentException("conflicting external class base");
            return;
        }
        if (declaredClassBases.Count >= 4096) throw new InvalidDataException("too many external class bases");
        declaredClassBases.Add(type, baseType);
    }

    internal bool HasDeclaredClassBase(ImportedTypeReference type, ImportedTypeReference target)
    {
        while (declaredClassBases.TryGetValue(type, out var parent))
        {
            if (parent.Equals(target)) return true;
            type = parent;
        }
        return false;
    }
}
