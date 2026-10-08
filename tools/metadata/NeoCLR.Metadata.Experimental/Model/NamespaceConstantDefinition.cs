using System.Globalization;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable namespace Double constant, with no runtime storage or initialization.</summary>
/// <remarks>Initial native-only contract: finite Double values and public/internal visibility.
/// Consumers inline the value and must be recompiled when it changes.</remarks>
public sealed class NamespaceConstantDefinition
{
    /// <summary>Creates a namespace constant with a finite Double value.</summary>
    /// <exception cref="ArgumentException">Invalid name, namespace, value or visibility.</exception>
    public NamespaceConstantDefinition(string @namespace, string name, double value, MethodVisibility visibility = MethodVisibility.Public)
    {
        if (!ValidNamespace(@namespace) || !ValidName(name) || !double.IsFinite(value) ||
            visibility is not (MethodVisibility.Public or MethodVisibility.Internal))
            throw new ArgumentException("namespace constants require bounded names, finite Double and public/internal visibility");
        Namespace = @namespace; Name = name; Value = value; Visibility = visibility;
    }
    internal static bool ValidName(string value) => !string.IsNullOrWhiteSpace(value) && value.Length <= 1024 && !value.Any(c => char.IsControl(c) || c == '.');
    internal static bool ValidNamespace(string value) => value is not null && value.Length <= 1024 && (value.Length == 0 || value.Split('.').All(ValidName));
    /// <summary>Gets the logical namespace, empty for the global namespace.</summary>
    public string Namespace { get; }
    /// <summary>Gets the simple name.</summary>
    public string Name { get; }
    /// <summary>Gets the exact finite binary64 value.</summary>
    public double Value { get; }
    /// <summary>Gets public or assembly-only visibility.</summary>
    public MethodVisibility Visibility { get; }
    internal string Bits => unchecked((ulong)BitConverter.DoubleToInt64Bits(Value)).ToString("x16", CultureInfo.InvariantCulture);
}

public sealed partial class AssemblyBuilder
{
    private readonly List<NamespaceConstantDefinition> namespaceConstants = [];
    /// <summary>Adds a native namespace Double constant without executable storage.</summary>
    /// <exception cref="ArgumentException">Duplicate name, invalid definition or more than 4096 constants.</exception>
    public void AddNamespaceConstant(NamespaceConstantDefinition constant)
    {
        ArgumentNullException.ThrowIfNull(constant);
        if (namespaceConstants.Count >= 4096 || namespaceConstants.Any(c => c.Namespace == constant.Namespace && c.Name == constant.Name))
            throw new ArgumentException("duplicate or excessive namespace constants");
        namespaceConstants.Add(constant);
    }
    internal IReadOnlyList<NamespaceConstantDefinition> NamespaceConstants => namespaceConstants.AsReadOnly();
}

public sealed partial class ModuleDefinition
{
    private IReadOnlyList<NamespaceConstantDefinition> loadedNamespaceConstants = Array.AsReadOnly(Array.Empty<NamespaceConstantDefinition>());
    /// <summary>Gets native namespace constants; ordinary CLI snapshots have none.</summary>
    public IReadOnlyList<NamespaceConstantDefinition> NamespaceConstants => Assembly.Producer?.NamespaceConstants ?? loadedNamespaceConstants;
    internal void SetNamespaceConstants(IEnumerable<NamespaceConstantDefinition> constants) => loadedNamespaceConstants = Array.AsReadOnly(constants.ToArray());
}
