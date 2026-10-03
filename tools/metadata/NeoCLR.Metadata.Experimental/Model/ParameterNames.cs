using System.Collections.ObjectModel;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodDefinition
{
    private readonly Dictionary<int, string> parameterNames = [];
    /// <summary>Gets declared names keyed by zero-based position; unnamed parameters have no entry.</summary>
    public IReadOnlyDictionary<int, string> ParameterNames => parameterNameView ??= new ReadOnlyDictionary<int, string>(parameterNames);
    private IReadOnlyDictionary<int, string>? parameterNameView;
    /// <summary>Sets or clears a parameter name on an authored definition, without changing its signature.</summary>
    /// <exception cref="InvalidOperationException">This is a loaded snapshot.</exception>
    /// <exception cref="ArgumentOutOfRangeException">Position is outside the declared parameters.</exception>
    /// <exception cref="ArgumentException">Name is empty, too long, or contains control characters.</exception>
    public void SetParameterName(int position, string? name)
    {
        if (Producer is null) throw new InvalidOperationException("loaded parameter metadata is read-only");
        if (position < 0 || position >= Producer.ParameterCount) throw new ArgumentOutOfRangeException(nameof(position));
        if (name is not null && (name.Length == 0 || name.Length > 1024 || name.Any(char.IsControl))) throw new ArgumentException("invalid parameter name", nameof(name));
        if (name is null) parameterNames.Remove(position); else parameterNames[position] = name;
    }
    internal void LoadParameterNames(IReadOnlyDictionary<int, string>? names)
    {
        foreach (var pair in names ?? new Dictionary<int, string>()) parameterNames.Add(pair.Key, pair.Value);
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Sets or clears a declared parameter name through the owned definition.</summary>
    public void SetParameterName(int position, string? name) => Definition.SetParameterName(position, name);
}
