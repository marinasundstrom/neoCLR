using Mono.Cecil;

// Selected immutable address family, not a general inheritance escape hatch.
static class IPAddressBindings
{
    public const string Root = "System.Networking.IPAddress";
    public static readonly string[] Names = [Root, "System.Networking.IPv4Address", "System.Networking.IPv6Address"];
    const string Marker = "System.Runtime.CompilerServices.ClosedHierarchyAttribute";
    public static bool IsName(string name) => Names.Contains(name);
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = "\n" + """
        #nullable enable annotations
        namespace Networking {
            public abstract class IPAddress : Equatable<IPAddress> {
                protected IPAddress(byte[] bytes) { }
                public static Result<IPAddress, IPAddressError> Parse(string text) => default;
                public bool Equals(IPAddress other) => default;
                public override bool Equals(object? other) => default;
                public override int GetHashCode() => default;
                public override string ToString() => default;
            }
            public sealed class IPv4Address : IPAddress {
                internal IPv4Address(byte[] bytes) : base(bytes) { }
            }
            public sealed class IPv6Address : IPAddress {
                internal IPv6Address(byte[] bytes) : base(bytes) { }
            }
        }
        #nullable restore annotations
        """ + "\n";
    public static void Project(ModuleDefinition module)
    {
        foreach (var name in Names.Skip(1)) {
            var leaf = module.GetType(name);
            if (!leaf.Methods.Any(m => m.IsConstructor)) {
                var factory = new MethodDefinition(".ctor", MethodAttributes.Assembly | MethodAttributes.HideBySig
                    | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName, module.TypeSystem.Void);
                factory.Parameters.Add(new ParameterDefinition("bytes", ParameterAttributes.None, new ArrayType(module.TypeSystem.Byte)));
                leaf.Methods.Add(factory);
            }
        }
        var constructor = module.GetType(Marker).Methods.Single(m => m.IsConstructor);
        var attribute = new CustomAttribute(constructor);
        attribute.ConstructorArguments.Add(new CustomAttributeArgument(constructor.Parameters[0].ParameterType,
            Names.Skip(1).Select(name => new CustomAttributeArgument(module.GetType("System.Type"), module.GetType(name))).ToArray()));
        module.GetType(Root).CustomAttributes.Add(attribute);
    }
    public static void Validate(TypeDefinition type)
    {
        if (!IsName(type.FullName)) return;
        var markers = type.CustomAttributes.Where(a => a.AttributeType.FullName == Marker).ToArray();
        if (type.FullName != Root) {
            if (!type.IsSealed || type.IsAbstract || type.BaseType?.Resolve() != type.Module.GetType(Root) || markers.Length != 0)
                throw new InvalidDataException("Invalid IPAddress leaf contract.");
            return;
        }
        if (!type.IsAbstract || type.IsSealed || type.BaseType?.FullName != "System.Object"
            || markers.Length != 1 || markers[0].ConstructorArguments.Count != 1
            || markers[0].ConstructorArguments[0].Value is not CustomAttributeArgument[] branches
            || branches.Length != 2
            || branches.Any(b => b.Value is not TypeReference r || type.Module.GetType(r.FullName) is null
                || (r.Scope is AssemblyNameReference assembly ? assembly.Name != type.Module.Assembly.Name.Name : r.Scope != type.Module))
            || !branches.Select(b => ((TypeReference)b.Value).FullName).Order().SequenceEqual(Names.Skip(1).Order()))
            throw new InvalidDataException("IPAddress must be closed to IPv4Address and IPv6Address.");
    }
    public static void RejectExternalBranches(IEnumerable<ModuleDefinition> modules)
    {
        foreach (var type in modules.SelectMany(m => m.GetTypes()))
            if (type.BaseType is { } parent && RuntimeSignatures.IsCore(parent.Scope) && IsName(parent.FullName))
                throw new InvalidDataException("External IPAddress branches are not permitted: " + type.FullName);
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (Type(reference.DeclaringType) is null) return null;
        Validate(definition.DeclaringType);
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            "Parse" => ("String", "System.Result<System.Networking.IPAddress,System.Networking.IPAddressError>", true),
            "ToString" => ("", "String", false),
            "Equals" => (args.Length == 1 && args[0] == "System.Object" ? "System.Object" : Root, "Boolean", false),
            "GetHashCode" => ("", "Int32", false),
            _ => throw new InvalidDataException("Unsupported IPAddress member: " + reference.FullName)
        };
        var typedEquality = definition.Name == "Equals" && expected.Item1 == Root;
        if (definition.DeclaringType.FullName != Root || !definition.IsPublic || definition.HasGenericParameters
            || definition.IsStatic != expected.Item3 || reference.HasThis == expected.Item3
            || definition.IsVirtual != !expected.Item3 || definition.IsNewSlot != typedEquality || definition.IsFinal != typedEquality
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported IPAddress signature.");
        return new(Root + "::" + reference.Name,
            definition.IsStatic ? args : new[] { Root }.Concat(args).ToArray(), result,
            Instruction: $"call {(definition.IsStatic ? "" : "instance ")}{Root}::{reference.Name}({string.Join(',', args)})");
    }
}
