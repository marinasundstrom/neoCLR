using System.Globalization;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable assembly-level Double constant, with no runtime storage or initialization.</summary>
/// <remarks>Initial native-only contract: finite Double values and public/internal visibility.
/// Consumers inline the value and must be recompiled when it changes.</remarks>
public sealed class AssemblyConstantDefinition
{
    /// <summary>Creates an assembly-level constant with a finite Double value.</summary>
    /// <exception cref="ArgumentException">Invalid name, namespace, value or visibility.</exception>
    public AssemblyConstantDefinition(string @namespace, string name, double value, MethodVisibility visibility = MethodVisibility.Public)
    {
        if (!ValidNamespace(@namespace) || !ValidName(name) || !double.IsFinite(value) ||
            visibility is not (MethodVisibility.Public or MethodVisibility.Internal))
            throw new ArgumentException("assembly-level constants require bounded names, finite Double and public/internal visibility");
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
    /// <summary>Gets the namespace-qualified member name.</summary>
    public string FullName => Namespace.Length == 0 ? Name : Namespace + "." + Name;
    /// <summary>Gets the declaring module after attachment, or null for an unattached definition.</summary>
    public ModuleDefinition? Module { get; private set; }
    /// <summary>Gets the declaring assembly after attachment, or null for an unattached definition.</summary>
    public AssemblyDefinition? Assembly => Module?.Assembly;
    internal void Attach(ModuleDefinition module)
    {
        if (Module is not null) throw new InvalidOperationException("constant already belongs to a module");
        Module = module;
    }
    internal string Bits => unchecked((ulong)BitConverter.DoubleToInt64Bits(Value)).ToString("x16", CultureInfo.InvariantCulture);
}

public sealed partial class AssemblyBuilder
{
    private readonly List<AssemblyConstantDefinition> assemblyConstants = [];
    /// <summary>Adds a native assembly-level Double constant without executable storage.</summary>
    /// <exception cref="ArgumentException">Duplicate name, invalid definition or more than 4096 constants.</exception>
    /// <exception cref="InvalidOperationException">The constant already belongs to a module.</exception>
    public void AddConstant(AssemblyConstantDefinition constant)
    {
        ArgumentNullException.ThrowIfNull(constant);
        if (assemblyConstants.Count >= 4096 || assemblyConstants.Any(c => c.Namespace == constant.Namespace && c.Name == constant.Name))
            throw new ArgumentException("duplicate or excessive assembly-level constants");
        constant.Attach(Definition.MainModule);
        assemblyConstants.Add(constant);
    }
    internal IReadOnlyList<AssemblyConstantDefinition> Constants => assemblyConstants.AsReadOnly();
}

public sealed partial class ModuleDefinition
{
    private IReadOnlyList<AssemblyConstantDefinition> loadedConstants = Array.AsReadOnly(Array.Empty<AssemblyConstantDefinition>());
    /// <summary>Gets native assembly-level constants; ordinary CLI snapshots have none.</summary>
    public IReadOnlyList<AssemblyConstantDefinition> Constants => Assembly.Producer?.Constants ?? loadedConstants;
    internal void SetConstants(IEnumerable<AssemblyConstantDefinition> constants)
    {
        var items = constants.ToArray();
        foreach (var constant in items) constant.Attach(this);
        loadedConstants = Array.AsReadOnly(items);
    }
}
