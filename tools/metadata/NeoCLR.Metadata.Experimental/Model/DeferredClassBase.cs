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
        ValidateDeferredBaseOwner(IsClosedHierarchy && ReferenceEquals(baseType, Assembly.ExternalObjectRoot));
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

    /// <summary>Binds an enum's exact imported, fieldless System.Enum foundation.</summary>
    /// <param name="baseType">An owned complete nongeneric System.Enum class contract.</param>
    /// <remarks>Preserves nominal enum identity and its existing Int32 value__ storage.</remarks>
    /// <exception cref="ArgumentException">The reference is foreign, incomplete, storage-bearing or not System.Enum.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an enum or already has an explicit base.</exception>
    public void SetEnumBase(ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!IsEnum || ExternalBase is not null || LocalBase is not null)
            throw new InvalidOperationException("enum base binding requires an enum without an explicit base");
        if (!ReferenceEquals(baseType.Owner, Assembly) || baseType.Namespace != "System" || baseType.Name != "Enum" ||
            baseType.IsValueType || baseType.GenericArity != 0 || baseType.DeclaringType is not null ||
            !Assembly.HasClassBaseContract(baseType) || Assembly.ExternalClassFieldCount(baseType) != 0)
            throw new ArgumentException("enum base requires a complete fieldless System.Enum contract", nameof(baseType));
        Definition.BindExternalBase(baseType);
    }

    /// <summary>Binds an intrinsic value declaration to its owned System.ValueType foundation.</summary>
    /// <param name="baseType">The fieldless abstract ValueType over this assembly's native Object root.</param>
    /// <exception cref="ArgumentException">The foundation is foreign or has an incompatible identity or layout.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an intrinsic value or already has a base or members.</exception>
    public void SetValueTypeBase(TypeBuilder baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!IsValueType || NativePrimitive is null && !NativeGrapheme || IsEnum ||
            ExternalBase is not null || LocalBase is not null || Methods.Count != 0 || Fields.Count != 0 || StaticFields.Count != 0)
            throw new InvalidOperationException("local value base requires an intrinsic value before its members");
        if (!ValidLocalValueTypeBase(baseType))
            throw new ArgumentException("local value base requires the owned fieldless ValueType foundation", nameof(baseType));
        Definition.BindLocalValueBase(baseType);
    }

    internal bool HasLocalValueTypeBase => IsValueType && !IsEnum && (NativePrimitive is not null || NativeGrapheme) &&
        LocalBase is { } parent && ValidLocalValueTypeBase(parent);

    private bool ValidLocalValueTypeBase(TypeBuilder parent) => ReferenceEquals(parent.Assembly, Assembly) &&
        parent.Namespace == "System" && parent.Name == "ValueType" && !parent.IsValueType && !parent.IsInterface &&
        parent.GenericParameterNames.Count == 0 && parent.Definition.DeclaringType is null &&
        (parent.Definition.Attributes & 0x80) != 0 && parent.Fields.Count == 0 && parent.InheritedFieldCount == 0 &&
        parent.LocalBase is { IsNativeObjectRoot: true };

    /// <summary>Binds a core-library enum to its owned System.Enum foundation.</summary>
    /// <param name="baseType">The fieldless abstract Enum over the owned ValueType/Object chain.</param>
    /// <exception cref="ArgumentException">The foundation identity, ownership or layout is invalid.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an enum or already has a base.</exception>
    public void SetEnumBase(TypeBuilder baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!IsEnum || LocalBase is not null || ExternalBase is not null)
            throw new InvalidOperationException("local enum base requires an enum without an explicit base");
        if (!ValidLocalEnumBase(baseType))
            throw new ArgumentException("local enum base requires the owned Enum foundation", nameof(baseType));
        Definition.BindLocalValueBase(baseType);
    }

    internal bool HasLocalEnumBase => IsEnum && LocalBase is { } parent && ValidLocalEnumBase(parent);
    private bool ValidLocalEnumBase(TypeBuilder parent) => ReferenceEquals(parent.Assembly, Assembly) &&
        parent.Namespace == "System" && parent.Name == "Enum" && !parent.IsValueType && !parent.IsInterface &&
        parent.GenericParameterNames.Count == 0 && parent.Definition.DeclaringType is null &&
        (parent.Definition.Attributes & 0x80) != 0 && parent.Fields.Count == 0 &&
        parent.LocalBase is { } valueType && ValidLocalValueTypeBase(valueType);

    private void ValidateDeferredBaseOwner(bool allowClosedObjectBase = false)
    {
        if (IsStatic || IsInterface || IsValueType || IsNativeObjectRoot || NativePrimitive is not null || NativeGrapheme ||
            Definition.DeclaringType is not null || IsClosedHierarchy && !allowClosedObjectBase || HasExplicitClassBase ||
            Methods.Count != 0 || Fields.Count != 0 || StaticFields.Count != 0)
            throw new InvalidOperationException($"deferred base binding requires a top-level root class before its members: {Namespace}.{Name}");
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

public sealed partial class TypeDefinition
{
    internal void BindLocalValueBase(TypeBuilder baseType) => authoredBaseType = baseType.Definition.ToReference();
}
