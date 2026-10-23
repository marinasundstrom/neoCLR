namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    // Constraint satisfaction admits value implementations without promising a
    // reference conversion. Reference variance must continue to exclude boxing.
    internal bool SatisfiesNominalTypeBound(SignatureType actual, SignatureType bound)
    {
        if (actual == bound || HasVariantReferenceConversion(actual, bound)) return true;
        if (bound.ClassType is { IsInterface: true } local)
            return actual.ClassType?.ConformsTo(local) == true || actual.GenericInstance?.ConformsTo(bound) == true;
        if (bound.ImportedType is { } external && nominalKinds.GetValueOrDefault(InterfaceDefinition(external)))
        {
            if (actual.ImportedType is { } imported) return HasNativeInterfaceConversion(imported, external);
            if (actual.GenericInstance is { } instance) return instance.ConformsTo(bound);
            return actual.ClassType?.InheritedContracts().Any(c => c == bound ||
                c.ImportedType is { } contract && HasNativeInterfaceConversion(contract, external)) == true;
        }
        return false;
    }
}
