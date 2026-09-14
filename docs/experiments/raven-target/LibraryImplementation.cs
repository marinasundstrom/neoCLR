using Mono.Cecil;
using System.Text.RegularExpressions;

// A bounded implementation fragment for an existing reference contract. No name-based
// substitution of guest/application types: every exported signature must match the core.
static class LibraryImplementation
{
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner)
    {
        if (!Regex.IsMatch(owner, @"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+$"))
            throw new InvalidDataException("Invalid library owner.");
        var type = source.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t))
            ?? throw new InvalidDataException("Missing namespace implementation: " + owner);
        var contract = core.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t))
            ?? throw new InvalidDataException("Missing namespace reference contract: " + owner);
        if (!type.IsPublic || !type.IsAbstract || !type.IsSealed || type.HasGenericParameters || type.HasFields || type.HasInterfaces)
            throw new InvalidDataException("Library fragment requires a public nongeneric namespace container without fields.");
        var methods = type.Methods.ToArray();
        if (methods.Length == 0) throw new InvalidDataException("Empty library implementation.");
        foreach (var method in methods)
        {
            ApplicationTypes.CheckMethod(method);
            if (!method.IsPublic || !method.IsStatic || method.IsConstructor || !method.HasBody
                || !Regex.IsMatch(method.Name, @"^[A-Za-z_][A-Za-z0-9_]*$")
                || method.Parameters.Any(p => p.IsOut || p.ParameterType.IsByReference)
                || method.ReturnType.MetadataType == MetadataType.Void)
                throw new InvalidDataException("Unsupported library export: " + method.FullName);
            var matches = contract.Methods.Where(m => m.IsPublic && m.IsStatic && m.Name == method.Name && m.Parameters.Count == method.Parameters.Count
                && SameType(m.ReturnType, method.ReturnType)
                && m.Parameters.Zip(method.Parameters).All(p => p.First.Name == p.Second.Name && SameType(p.First.ParameterType, p.Second.ParameterType))).ToArray();
            if (matches.Length != 1) throw new InvalidDataException("Library export does not match reference contract: " + method.FullName);
        }
        return methods;
    }

    static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && left.MetadataType == right.MetadataType && left.IsValueType == right.IsValueType
        && (left.MetadataType is not (MetadataType.Class or MetadataType.ValueType or MetadataType.GenericInstance)
            || left.Resolve()?.Module.Assembly.Name.FullName == right.Resolve()?.Module.Assembly.Name.FullName)
        && (left is not GenericInstanceType l || right is GenericInstanceType r
            && l.GenericArguments.Zip(r.GenericArguments).All(p => SameType(p.First, p.Second)));

    public static string QualifyHelpers(string text, string owner)
    {
        var helpers = Regex.Matches(text, @"(?m)^\.function ([^\(]+)\(").Select(m => m.Groups[1].Value).ToArray();
        foreach (var helper in helpers.Where(h => !h.StartsWith(owner + ".", StringComparison.Ordinal)))
            text = Regex.Replace(text, @"(?m)^(\.function |call |ldftn )" + Regex.Escape(helper) + @"(?=\()",
                m => m.Groups[1].Value + "neoCLR.Library." + owner + "." + helper);
        return text;
    }

    public static void Write(string source, string core, string owner, string output)
    {
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var projected = Path.Combine(output, "Implementation.dll");
        VoidProjection.Write(source, core, projected);
        UnionImport.WriteLibrary(projected, core, Path.Combine(output, "Implementation.neoil"), owner);
    }
}
