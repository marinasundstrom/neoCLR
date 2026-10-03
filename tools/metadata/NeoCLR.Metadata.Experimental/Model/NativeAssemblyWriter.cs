using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Emits a native neoCLR format-5 assembly directly from this graph.</summary>
    /// <returns>Owned UTF-8 JSON bytes accepted by the native assembly loader.</returns>
    /// <exception cref="InvalidDataException">Invalid graph, unsupported descriptive names, identity collision, incompatible core contract or output limit.</exception>
    /// <remarks>Supports bounded built-in typed bodies, cross-assembly top-level functions and native-only console output.
    /// Dependencies must be emitted separately and supplied explicitly to the runtime. No PE conversion or external process runs.
    /// This is the current native JSON format, not the experimental NEOX PE transport.</remarks>
    public byte[] WriteNativeAssembly()
    {
        var methods = ValidateGraph();
        static void CheckText(string text)
        {
            if (string.IsNullOrWhiteSpace(text) || text.Any(char.IsControl)) throw new InvalidDataException("invalid native descriptive name");
            try { _ = new UTF8Encoding(false, true).GetByteCount(text); }
            catch (EncoderFallbackException error) { throw new InvalidDataException("invalid Unicode in native descriptive name", error); }
        }
        CheckText(Identity.Name);
        var dependencies = new Dictionary<AssemblyIdentity, AssemblyBuilder>();
        foreach (var method in methods)
        {
            CheckText(method.Name);
            foreach (var call in method.Instructions.Where(i => i.Target is not null))
            {
                var target = call.Target!.Assembly;
                if (ReferenceEquals(target, this)) continue;
                if (target.Identity.Equals(Identity)) throw new InvalidDataException("external assembly identity collides with output identity");
                if (!target.CoreLibrary.Equals(CoreLibrary)) throw new InvalidDataException("cross-target call requires compatible core identity");
                if (dependencies.TryGetValue(target.Identity, out var previous) && !ReferenceEquals(previous, target))
                    throw new InvalidDataException("different dependency graphs share an identity");
                dependencies[target.Identity] = target;
                if (dependencies.Count > 256) throw new InvalidDataException("too many imported assemblies");
            }
        }
        foreach (var type in importedNominalTypes.Values)
            if (!dependencies.ContainsKey(type.AssemblyIdentity))
                dependencies.Add(type.AssemblyIdentity, importedGraphs.TryGetValue(type.AssemblyIdentity, out var graph) ? graph.Graph : new AssemblyBuilder(type.AssemblyIdentity, CoreLibrary));
        if (dependencies.Count > 256) throw new InvalidDataException("too many imported assemblies");
        static string IdentityText(AssemblyIdentity identity) => JsonSerializer.Serialize(new[] {
            identity.Name, identity.Version.ToString(), identity.Culture, identity.PublicKeyToken, identity.Flags.ToString(System.Globalization.CultureInfo.InvariantCulture)
        });
        static string ModuleName(AssemblyBuilder assembly) => assembly.NativeBinding?.Library.ModuleName ?? ModuleIdentity(assembly.Identity);
        static string ModuleIdentity(AssemblyIdentity identity) => "NeoMetadata_" + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(IdentityText(identity))));
        static string Encoded(string value) => Convert.ToHexString(Encoding.UTF8.GetBytes(value));
        static string TypeName(TypeBuilder type) => type.Assembly.NativeBinding is { } binding ? binding.TypeName(type.Definition) : type.Definition.DeclaringType is { } parent ? TypeName(parent.Producer!) + ".N_" + Encoded(type.Name) : ModuleName(type.Assembly) + ".T_" + Encoded(type.Namespace) + "_" + Encoded(type.Name);
        static string FunctionName(MethodBuilder method) => method.NativeImportName ?? (method.IsConstructor ? TypeName(method.DeclaringType!) + "..ctor" : (method.DeclaringType is { } type ? TypeName(type) + ".M_" : ModuleName(method.Assembly) + ".F_") + Encoded(method.CliName));
        static object? Owner(MethodBuilder method) => method.NativeImportIsNamespaceFunction ? null : method.DeclaringType is { } type ? TypeOwner(type, type.GenericParameterNames.Select((_, i) => SignatureType.TypeParameter(i)).ToArray()) : null;
        static object TypeOwner(TypeBuilder type, IReadOnlyList<SignatureType> arguments) => arguments.Count == 0 ? new { Named = TypeName(type) } : new { Constructed = new { definition = TypeName(type), arguments = arguments.Select(SignatureValue).ToArray() } };
        static string ExternalName(ImportedTypeReference type) => type.Owner.NativeBindingFor(type.AssemblyIdentity) is { } binding ? binding.TypeName(type) : type.DeclaringType is { } parent ? ExternalName(parent) + ".N_" + Encoded(type.Name) : ModuleIdentity(type.AssemblyIdentity) + ".T_" + Encoded(type.Namespace) + "_" + Encoded(type.Name);
        static object ExternalValue(ImportedTypeReference type) => NativeImportBinding.IsInhabitedVoid(type) ? "Void" : type.TypeArguments.Count == 0 ? new { Named = ExternalName(type) } : new { Constructed = new { definition = ExternalName(type), arguments = type.TypeArguments.Select(SignatureValue).ToArray() } };
        static object SignatureValue(SignatureType type) => type.IsSelf ? "SelfType" : type.FunctionSignature is { } function ? new { Function = new { parameters = function.ParameterTypes.Select(SignatureValue).ToArray(), returns = SignatureValue(function.ReturnType), no_result = function.NoResult } } : type.ByReferenceElement is { } target ? new { ByRef = SignatureValue(target) } : type.ImportedType is { } imported ? ExternalValue(imported) : type.GenericInstance is { } instance ? TypeOwner(instance.Definition, instance.TypeArguments) : type.TypeParameterIndex is { } ordinal ? new { TypeParameter = ordinal } : type.MethodParameterIndex is { } index ? new { MethodTypeParameter = index } : type.ArrayElement is { } element ? new { ArrayRef = SignatureValue(element) } : type.ClassType is { } c ? new { Named = TypeName(c) } : type.Primitive!.Value.ToString();
        static object[] Parameters(MethodBuilder method) => method.Signature.ParameterTypes.Select(SignatureValue).ToArray();
        object Origin(string name, int token, MethodBuilder? method = null, bool publiclyVisible = true) => method is null
            ? new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, publicly_visible = publiclyVisible }
            : new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, member_access = method.Visibility == MethodVisibility.Internal ? "Assembly" : method.Visibility.ToString(), parameter_tokens = new int[method.ParameterCount] };
        object Instruction(MethodBuilder.Operation instruction) => instruction.Op switch
        {
            "function.bind" => new { op = "function.bind", arg = new { function_type = SignatureValue(instruction.Type!), target = new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), parameters = instruction.Target!.Signature.ParameterTypes.Select(SignatureValue).ToArray() } } },
            "function.invoke" => new { op = "call", arg = new { name = "$Function.Invoke", owner = SignatureValue(instruction.Type!), instance = true, parameters = instruction.Type!.FunctionSignature!.ParameterTypes.Select(SignatureValue).ToArray() } },
            "array.length" => new { op = "ldlen" },
            "array.reserve" => new { op = "array.reserve", arg = SignatureValue(instruction.Type!) },
            "array.new" or "array.load" or "array.store" => new { op = instruction.Op == "array.new" ? "newarr" : instruction.Op == "array.load" ? "ldelem" : "stelem", arg = SignatureValue(instruction.Type!) },
            "string" => new { op = "ldstr", arg = (object)instruction.Text! },
            "boolean" => new { op = "ldc.bool", arg = (object)(instruction.Value != 0) },
            "negate" => new { op = "neg" },
            "complement" => new { op = "not" },
            "constant64" => new { op = "ldc.i8", arg = (object)instruction.LongValue },
            "convert64" => new { op = "conv.i8" },
            "convert32" => new { op = "conv.i4" },
            "pop" => new { op = "pop" },
            "equal" => new { op = "ceq" },
            "less" => new { op = "clt" },
            "greater" => new { op = "cgt" },
            "constant" => new { op = "ldc.i4", arg = (object)instruction.Value },
            "argument.store" => new { op = "starg", arg = (object)instruction.Value },
            "argument" => new { op = "ldarg", arg = (object)instruction.Value },
            "new.constructed" or "call.constructed" or "call.virtual.constructed" => new
            {
                op = instruction.Op == "new.constructed" ? "newobj.ctor" : instruction.Op == "call.virtual.constructed" ? "callvirt" : "call",
                arg = (object)new
                {
                    name = FunctionName(instruction.Target!),
                    owner = TypeOwner(instruction.Target!.DeclaringType!, instruction.ConstructedTarget!.DeclaringTypeArguments),
                    instance = !instruction.Target.IsStatic,
                    generic_arguments = instruction.ConstructedTarget.MethodArguments.Select(SignatureValue).ToArray(),
                    parameters = instruction.ConstructedTarget.Signature.ParameterTypes.Select(SignatureValue).ToArray()
                }
            },
            "call.generic" => new
            {
                op = "call",
                arg = (object)new
                {
                    name = FunctionName(instruction.Target!),
                    owner = Owner(instruction.Target!),
                    instance = !instruction.Target!.IsStatic,
                    generic_arguments = instruction.GenericTarget!.TypeArguments.Select(SignatureValue).ToArray(),
                    parameters = instruction.GenericTarget.Signature.ParameterTypes.Select(SignatureValue).ToArray()
                }
            },
            "call.virtual" or "call" or "new.object" => new { op = instruction.Op == "call.virtual" ? "callvirt" : instruction.Op == "call" ? "call" : "newobj.ctor", arg = (object)new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), instance = !instruction.Target!.IsStatic, parameters = Parameters(instruction.Target!) } },
            "duplicate" => new { op = "dup" },
            "field.import.load" or "field.import.store" => new { op = instruction.Op == "field.import.load" ? "ldfld" : "stfld", arg = (object)(instruction.ImportedField!.NativeIndex ?? throw new InvalidDataException("native field emission requires a native layout ordinal")) },
            "field.load" or "field.store" => new { op = instruction.Op == "field.load" ? "ldfld" : "stfld", arg = (object)instruction.Field!.Index },
            "local.address" => new { op = "ldloca", arg = (object)instruction.Value },
            "reference.cast" => new { op = "castclass", arg = SignatureValue(instruction.Type!) },
            "object.load" or "object.store" => new { op = instruction.Op == "object.load" ? "ldobj" : "stobj", arg = SignatureValue(instruction.Type!) },
            "local.initialize" => new { op = "initobj", arg = SignatureValue(instruction.Type!) },
            "local.load" => new { op = "ldloc", arg = (object)instruction.Value },
            "local.store" => new { op = "stloc", arg = (object)instruction.Value },
            "add" => new { op = "add" },
            "subtract" => new { op = "sub" },
            "multiply" => new { op = "mul" },
            "divide" => new { op = "div" },
            "remainder" => new { op = "rem" },
            "and" => new { op = "and" },
            "or" => new { op = "or" },
            "xor" => new { op = "xor" },
            "shift.left" => new { op = "shl" },
            "shift.right" => new { op = "shr" },
            "return" => new { op = "ret" },
            "fail" => new { op = "fault", arg = instruction.Text! },
            _ => throw new InvalidDataException("unsupported instruction")
        };
        IEnumerable<object> NativeInstructions(MethodBuilder.Operation instruction)
        {
            if (instruction.Target is { DiscardNativeImportResult: true } && instruction.Op is "call" or "call.virtual" or "call.constructed" or "call.virtual.constructed" or "call.generic")
                return new[] { Instruction(instruction), new { op = "pop" } };
            // Legacy native stores through managed references return an inhabited Void.
            // The producer contract follows CLI stfld, which leaves no stack value.
            if (instruction.Op == "field.store" && instruction.Field!.DeclaringType.IsValueType)
                return new[] { Instruction(instruction), new { op = "pop" } };
            if (instruction.Op == "native.call")
            {
                var target = instruction.NativeTarget!;
                target.TryGetStaticInt32Signature(out var count);
                return new object[] { new { op = "call", arg = new {
                    name = target.Name, owner = new { Named = target.DeclaringTypeName! },
                    parameters = Enumerable.Repeat("Int32", count).ToArray()
                } } };
            }
            if (instruction.Op == "console.write")
                return new object[] {
                    new { op = "call", arg = new { name = "System.Console.WriteLine", owner = new { Named = "System.Console" }, parameters = new[] { "String" } } },
                    new { op = "pop" }
                };
            if (instruction.Op == "console.line")
                return new object[] {
                    new { op = "ldstr", arg = instruction.Text! },
                    new { op = "call", arg = new { name = "System.Console.WriteLine", owner = new { Named = "System.Console" }, parameters = new[] { "String" } } },
                    new { op = "pop" } // Bundled System returns the Void value; statement discards it.
                };
            return new[] { Instruction(instruction) };
        }
        object[] NativeBody(MethodBuilder method)
        {
            var offsets = new int[method.Instructions.Count + 1];
            for (int i = 0; i < method.Instructions.Count; i++)
                offsets[i + 1] = offsets[i] + (method.Instructions[i].Op switch { "label" => 0, "console.line" => 3, "console.write" => 2, "call" or "call.virtual" or "call.constructed" or "call.virtual.constructed" or "call.generic" when method.Instructions[i].Target?.DiscardNativeImportResult == true => 2, "field.store" when method.Instructions[i].Field!.DeclaringType.IsValueType => 2, _ => 1 });
            var labels = method.LabelPositions();
            return method.Instructions.SelectMany(instruction => instruction.Op switch
            {
                "label" => Array.Empty<object>(),
                "branch" or "branch.true" or "branch.false" => new object[] {
                    new { op = instruction.Op == "branch" ? "br" : instruction.Op == "branch.true" ? "brtrue" : "brfalse", arg = offsets[labels[instruction.Value]] }
                },
                _ => NativeInstructions(instruction)
            }).ToArray();
        }
        foreach (var type in types) CheckText(type.Namespace.Length == 0 ? type.Name : type.Namespace + "." + type.Name);
        var fieldTokens = types.SelectMany(t => t.Fields).Select((field, index) => (field, token: 0x04000001 + index)).ToDictionary(p => p.field, p => p.token);
        var propertyTokens = types.SelectMany(t => t.Properties).Select((property, index) => (property, token: 0x17000001 + index)).ToDictionary(p => p.property, p => p.token);
        object TypeOrigin(TypeBuilder type, int index)
        {
            var name = type.Namespace.Length == 0 ? type.Name : type.Namespace + "." + type.Name;
            if (type.Fields.Count == 0 && type.Properties.Count == 0 && type.Definition.DeclaringType is null)
                return Origin(name, 0x02000002 + index, publiclyVisible: type.Visibility == TypeVisibility.Public);
            var origin = new Dictionary<string, object>
            {
                ["assembly"] = IdentityText(Identity),
                ["module"] = Identity.Name + ".dll",
                ["name"] = name,
                ["token"] = 0x02000002 + index,
                ["publicly_visible"] = type.Visibility == TypeVisibility.Public,
                ["field_tokens"] = type.Fields.Select(f => fieldTokens[f]).ToArray(),
                ["field_access"] = type.Fields.Select(f => f.Visibility == FieldVisibility.Internal ? "Assembly" : f.Visibility.ToString()).ToArray(),
                ["field_readonly"] = type.Fields.Select(f => f.IsReadOnly).ToArray()
            };
            if (type.Definition.DeclaringType is { } parent) origin["declaring_type_token"] = 0x02000002 + types.IndexOf(parent.Producer!);
            if (type.Properties.Count != 0) origin["property_tokens"] = type.Properties.Select(p => propertyTokens[p]).ToArray();
            return origin;
        }
        object? Accessor(MethodBuilder? method) => method is null ? null : new { name = FunctionName(method), owner = Owner(method), instance = !method.IsStatic, parameters = Parameters(method) };
        object[]? Constraints(TypeBuilder type)
        {
            var result = type.GenericConstraints.Select(c => (object)new { parameter = c.ParameterIndex, kind = new { TypeBound = new { Named = TypeName(c.BaseType) } } }).ToList();
            foreach (var (parameter, flags) in type.SpecialConstraints.OrderBy(p => p.Key))
                foreach (var flag in new[] { TypeParameterConstraints.ReferenceType, TypeParameterConstraints.ValueType, TypeParameterConstraints.DefaultConstructor })
                    if (flags.HasFlag(flag)) result.Add(new { parameter, kind = flag.ToString() });
            return result.Count == 0 ? null : result.ToArray();
        }
        var manifest = new Dictionary<string, object> { ["name"] = Identity.Name, ["full_name"] = IdentityText(Identity), ["modules"] = new[] { Identity.Name + ".dll" }, ["references"] = dependencies.Keys.Select(IdentityText).ToArray() };
        var moduleBindings = dependencies.Values.Where(d => d.NativeBinding is not null).Select(d => new
        {
            assembly = IdentityText(d.Identity),
            module = ModuleName(d),
            revision = d.NativeBinding!.Revision
        }).ToArray();
        if (moduleBindings.Length != 0) manifest["native_module_bindings"] = moduleBindings;
        var typeBindings = importedNominalTypes.Values.Where(t => NativeBindingFor(t.AssemblyIdentity) is not null).Select(t => new
        {
            native_name = ExternalName(t),
            assembly = IdentityText(t.AssemblyIdentity),
            @namespace = t.Namespace,
            name = t.Name,
            arity = t.GenericArity,
            value_type = t.IsValueType,
            declaring = t.DeclaringType is null ? null : ExternalName(t.DeclaringType)
        }).ToArray();
        if (typeBindings.Length != 0) manifest["native_type_bindings"] = typeBindings;
        var importedValues = importedNominalTypes.Values.Where(t => t.IsValueType).Select(ExternalName).Order().ToArray();
        if (importedValues.Length != 0) manifest["value_type_references"] = importedValues;
        var artifact = new
        {
            format = 5,
            name = ModuleName(this),
            revision = Identity.Version.ToString(),
            references = dependencies.Values.Select(d => d.NativeBinding is { Revision: null } ? (object)ModuleName(d) : new { name = ModuleName(d), revision = d.NativeBinding is { } binding ? binding.Revision : d.Identity.Version.ToString() }).ToArray(),
            entry = EntryPoint is null ? "" : FunctionName(EntryPoint),
            assemblies = new[] { manifest },
            types = types.Select((type, index) => new NativeTypeRow(
                TypeName(type), type.Fields.Select(f => (object)new { name = f.Name, ty = SignatureValue(f.FieldType), visibility = f.Visibility.ToString().ToLowerInvariant() }).ToArray(), !type.IsInterface && !type.IsValueType, type.IsStatic, type.IsStatic || type.IsValueType,
                TypeOrigin(type, index), type.IsInterface ? "Interface" : null,
                !type.InterfaceSignatures.Any() ? null : type.InterfaceSignatures.Select(SignatureValue).ToArray(),
                type.Visibility == TypeVisibility.Internal ? "internal" : null,
                type.Properties.Count == 0 ? null : type.Properties.Select(p => (object)new { name = p.Name, instance = !p.IsStatic, parameters = p.ParameterTypes.Select(SignatureValue).ToArray(), ty = SignatureValue(p.PropertyType), getter = Accessor(p.GetMethod), setter = Accessor(p.SetMethod) }).ToArray(), type.GenericParameterNames.Count == 0 ? null : type.GenericParameterNames.ToArray(), Constraints(type), type.Definition.DeclaringType is { } parent ? new { module = ModuleName(this), revision = Identity.Version.ToString(), index = types.IndexOf(parent.Producer!) } : null)).ToArray(),
            functions = methods.Select((method, index) => new NativeMethodRow(
                FunctionName(method), Owner(method), Parameters(method),
                method.Locals.Select(local => SignatureValue(local.SignatureType)).ToArray(),
                SignatureValue(method.Signature.ReturnType), !method.ReturnsValue,
                Origin(method.Name, 0x06000001 + index, method), method.IsAbstract ? [] : NativeBody(method), method.IsAbstract ? true : null, method.IsAbstract ? true : null,
                method.Visibility == MethodVisibility.Public ? null : method.Visibility.ToString().ToLowerInvariant(),
                method.DeclaringType is null && method.Namespace.Length != 0 ? method.Namespace : null,
                method.IsStatic ? null : true, method.Signature.GenericParameterNames.Count == 0 ? null : method.Signature.GenericParameterNames.ToArray(), method.Signature.OutParameters.Count == 0 ? null : method.Signature.OutParameters.ToArray(), !method.IsStatic && method.DeclaringType!.IsValueType ? true : null)).ToArray()
        };
        var result = JsonSerializer.SerializeToUtf8Bytes(artifact);
        if (result.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("output image exceeds limit");
        return result;
    }
    private sealed record NativeMethodRow(string name, object? owner, object[] parameters, object[] locals,
        object returns, bool no_result, object origin, object[] body,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? is_abstract,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? is_virtual,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? visibility,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? @namespace,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? instance,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string[]? generic_parameters,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        int[]? out_parameters,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? receiver_byref);
    private sealed record NativeTypeRow(string name, object[] fields, bool is_reference_type,
        bool is_abstract, bool is_sealed, object origin,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? representation,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? implements,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? visibility,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? properties,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string[]? generic_parameters,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? generic_constraints,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object? declaring_type);

}
