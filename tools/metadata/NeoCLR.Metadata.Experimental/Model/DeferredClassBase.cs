namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeBuilder
{
    /// <summary>Binds an explicitly completed external class base after declaring this class identity and before its members.</summary>
    /// <param name="baseType">A consumer-owned complete class construction, registered with CompleteClassBaseReference.</param>
    /// <exception cref="ArgumentException">Missing contract, foreign owner or invalid generic scope.</exception>
    /// <exception cref="InvalidOperationException">The owner is unsupported, has members or already has an explicit base.</exception>
    public void SetBaseType(ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        ValidateDeferredBaseOwner();
        if (!ReferenceEquals(baseType.Owner, Assembly) || !Assembly.HasClassBaseContract(baseType))
            throw new ArgumentException("external base requires a complete explicit contract", nameof(baseType));
        ((SignatureType)baseType).ValidateOwner(Assembly, typeArity: GenericParameterNames.Count);
        Definition.BindExternalBase(baseType);
    }

    /// <summary>Binds the fieldless runtime System.ValueType base of a native value-record definition.</summary>
    /// <param name="baseType">An owned, complete nongeneric System.ValueType reference with no inherited storage.</param>
    /// <remarks>Preserves nominal ancestry without adding reference storage or a base-constructor call.</remarks>
    /// <exception cref="ArgumentException">The base is foreign, has storage or lacks a complete ValueType contract.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an ordinary value record or already has members/an explicit base.</exception>
    public void SetValueTypeBase(ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!IsValueType || Definition.IsEnum || NativePrimitive is not null || NativeGrapheme ||
            ExternalBase is not null || LocalBase is not null || Methods.Count != 0 || Fields.Count != 0 || StaticFields.Count != 0)
            throw new InvalidOperationException("value base binding requires a value record before its members");
        if (!ReferenceEquals(baseType.Owner, Assembly) || baseType.Namespace != "System" || baseType.Name != "ValueType" ||
            baseType.IsValueType || baseType.GenericArity != 0 || baseType.DeclaringType is not null ||
            !Assembly.HasClassBaseContract(baseType) || Assembly.ExternalClassFieldCount(baseType) != 0)
            throw new ArgumentException("value base requires a complete fieldless System.ValueType contract", nameof(baseType));
        Definition.BindExternalBase(baseType);
    }

    private void ValidateDeferredBaseOwner()
    {
        if (IsStatic || IsInterface || IsValueType || IsNativeObjectRoot || NativePrimitive is not null || NativeGrapheme ||
            Definition.DeclaringType is not null || IsClosedHierarchy || HasExplicitClassBase ||
            Methods.Count != 0 || Fields.Count != 0 || StaticFields.Count != 0)
            throw new InvalidOperationException("deferred base binding requires a top-level root class before its members");
    }

    /// <summary>Binds an owned constructed base after declaring this class identity, before authoring its members.</summary>
    /// <param name="baseType">A nonsealed top-level class construction in this assembly, with arguments scoped to this declaration.</param>
    /// <remarks>Allows recursive arguments such as Node : Base&lt;Node&gt;. The direct inheritance chain must remain acyclic.
    /// Only an ordinary top-level root class can be bound, once; an existing explicit base cannot be replaced.</remarks>
    /// <exception cref="ArgumentException">Foreign, sealed, nonclass, cyclic or incorrectly scoped base.</exception>
    /// <exception cref="InvalidOperationException">This owner is unsupported, already has an explicit base, or has members.</exception>
    public void SetBaseType(GenericTypeInstance baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        ValidateDeferredBaseOwner();
        var parent = baseType.Definition;
        if (!ReferenceEquals(parent.Assembly, Assembly) || parent.IsStatic || parent.IsInterface || parent.IsValueType ||
            parent.NativePrimitive is not null || parent.NativeGrapheme || parent.Definition.DeclaringType is not null ||
            (parent.Definition.Attributes & 0x100) != 0 || ReferenceEquals(parent, this) || parent.DerivesFrom(this))
            throw new ArgumentException("constructed base requires an owned nonsealed class and an acyclic inheritance chain", nameof(baseType));
        ((SignatureType)baseType).ValidateOwner(Assembly, typeArity: GenericParameterNames.Count);
        Definition.BindConstructedBase(baseType);
    }
}
