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
    public byte[] WriteNativeAssembly() => WriteNativeCore(MetadataArtifactReader.MaxImageSize);

    // Schema 2 is signed-integer-only; high binary64 bits require the existing schema 3 profile.
    internal bool RequiresWideNumericPayload => functions.Concat(types.SelectMany(t => t.Methods))
        .Any(m => m.Instructions.Any(i => i.Op == "constantDouble" && i.LongValue < 0));

    internal byte[] WriteNativeLibraryAssembly() => WriteNativeCore(32 * 1024 * 1024);

    private byte[] WriteNativeCore(int maxImageSize)
    {
        var methods = ValidateGraph();
        if (externalGrapheme is not null && types.Any(t => t.NativeGrapheme))
            throw new InvalidDataException("native Char cannot have both a local and external owner");
        int nextParameterToken = 0x08000001;
        var parameterTokens = new Dictionary<MethodBuilder, int[]>();
        foreach (var method in methods)
        {
            var tokens = new int[method.ParameterCount];
            if (method.Signature.OutParameters.Count > 0 || method.Definition.ParameterNames.Count > 0 || method.Definition.ParameterArrayIndex is not null)
                for (int position = 0; position < tokens.Length; position++) tokens[position] = nextParameterToken++;
            parameterTokens.Add(method, tokens);
        }
        var attributeOwners = new Dictionary<CustomAttributeDefinition, SignatureType>();
        var parameterAttributes = new Dictionary<MethodBuilder, CustomAttributeDefinition>();
        if (methods.Any(m => m.Definition.ParameterArrayIndex is not null))
        {
            (NativeBindingFor(CoreLibrary) ?? throw new InvalidDataException("native parameter arrays require an explicit core marker binding")).ValidateParameterArrayMarker();
            var marker = ImportTypeIdentity(CoreLibrary, "System", "ParamArrayAttribute", 0);
            foreach (var method in methods.Where(m => m.Definition.ParameterArrayIndex is not null))
            {
                var attribute = new CustomAttributeDefinition(Definition.MainModule.ImportReference(CoreLibrary, "System", "ParamArrayAttribute"), []);
                attributeOwners.Add(attribute, marker); parameterAttributes.Add(method, attribute);
            }
        }
        foreach (var type in types)
            foreach (var attribute in type.Definition.CustomAttributes.Where(a => !type.Definition.IsFlagsAttribute(a)))
            {
                attribute.ValidateOwner(type.Definition);
                var reference = attribute.AttributeType;
                attributeOwners[attribute] = reference.ExplicitScope is { } scope
                    ? ImportTypeIdentity(scope, reference.Namespace, reference.Name, 0)
                    : reference.Resolve().Producer ?? throw new InvalidDataException("detached attribute owner");
            }
        if (methods.Any(m => m.Instructions.Any(i => i.Op == "object.box" || i.Op == "reference.test" && !MethodBuilder.IsReferenceSignature(i.Type!))))
        {
            if (NativeObjectRoot is not null || ExternalObjectRoot is not null) ValidateNativeObjectSlots();
            else (NativeBindingFor(CoreLibrary) ?? throw new InvalidDataException("native boxing/value type tests require an explicit System core binding")).ValidateBoxingCore();
        }

        foreach (var target in methods.SelectMany(m => m.Instructions).Select(i => i.Target).OfType<MethodBuilder>().Where(m => m.IsCoreObjectToString).Distinct())
            (target.Assembly.NativeBinding ?? throw new InvalidDataException("native Object.ToString dispatch requires an explicit core slot binding")).ValidateObjectToStringSlot();

        foreach (var target in methods.SelectMany(m => m.Instructions).Select(i => i.Target).OfType<MethodBuilder>().Where(m => m.IsCoreObjectHash).Distinct())
            (target.Assembly.NativeBinding ?? throw new InvalidDataException("native Object.GetHashCode dispatch requires an explicit core slot binding")).ValidateObjectHashSlot();

        foreach (var target in methods.SelectMany(m => m.Instructions).Select(i => i.Target).OfType<MethodBuilder>().Where(m => m.IsCoreObjectEquals).Distinct())
            (target.Assembly.NativeBinding ?? throw new InvalidDataException("native Object.Equals dispatch requires an explicit core slot binding")).ValidateObjectOverride("Equals");

        static void CheckText(string text)
        {
            if (string.IsNullOrWhiteSpace(text) || text.Any(char.IsControl)) throw new InvalidDataException("invalid native descriptive name");
            try { _ = new UTF8Encoding(false, true).GetByteCount(text); }
            catch (EncoderFallbackException error) { throw new InvalidDataException("invalid Unicode in native descriptive name", error); }
        }
        CheckText(Identity.Name);
        var dependencies = new Dictionary<AssemblyIdentity, AssemblyBuilder>();
        if (methods.Any(method => method.IsImplicitObjectOverride))
        {
            if (NativeObjectRoot is not null) ValidateNativeObjectSlots();
            else if (ExternalObjectRoot is null)
            {
                var bindings = nativeBindings.Where(pair => pair.Value.Library.ModuleName == "System").ToArray();
                if (bindings.Length != 1) throw new InvalidDataException("native Object overrides require one explicit runtime slot binding for System");
                foreach (var name in methods.Where(m => m.IsImplicitObjectOverride).Select(m => m.Name).Distinct()) bindings[0].Value.ValidateObjectOverride(name);
                dependencies.Add(bindings[0].Key, importedGraphs[bindings[0].Key].Graph);
            }
        }
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
        static string TypeName(TypeBuilder type) => type.Definition.IsNativeObjectRoot ? "System.Object" : type.NativeGrapheme ? "System.Char" : type.NativePrimitive is { } primitive ? "System." + primitive : type.Assembly.NativeBinding is { } binding ? binding.TypeName(type.Definition) : type.Definition.DeclaringType is { } parent ? TypeName(parent.Producer!) + ".N_" + Encoded(type.Name) : ModuleName(type.Assembly) + ".T_" + Encoded(type.Namespace) + "_" + Encoded(type.Name);
        static string FunctionName(MethodBuilder method) => method.NativeImportName ?? (method.Definition.ImplementationAttributes == 0x1000 ? (method.Namespace.Length == 0 ? method.Name : method.Namespace + "." + method.Name) : ((method.IsVirtual && method.DeclaringType?.IsInterface == false || method.NativeValueOverride || method.DeclaringType?.NativePrimitive is not null || method.DeclaringType?.NativeGrapheme == true || method.DeclaringType?.Definition.IsNativeObjectRoot == true) ? TypeName(method.DeclaringType!) + "." + method.Name : method.IsConstructor ? TypeName(method.DeclaringType!) + "..ctor" : (method.DeclaringType is { } type ? TypeName(type) + ".M_" : ModuleName(method.Assembly) + ".F_") + Encoded(method.CliName)));
        static object? Owner(MethodBuilder method) => method.NativeImportObjectOwner ? new { Named = "System.Object" } : method.NativeImportCharOwner ? "Char" : method.NativeImportPrimitiveOwner is { } primitive ? primitive.ToString() : method.NativeImportIsNamespaceFunction ? null : method.DeclaringType is { } type ? TypeOwner(type, type.GenericParameterNames.Select((_, i) => SignatureType.TypeParameter(i)).ToArray()) : null;
        static object TypeOwner(TypeBuilder type, IReadOnlyList<SignatureType> arguments) => type.NativeGrapheme ? "Char" : type.NativePrimitive is { } primitive ? primitive.ToString() : arguments.Count == 0 ? new { Named = TypeName(type) } : new { Constructed = new { definition = TypeName(type), arguments = arguments.Select(SignatureValue).ToArray() } };
        static string ExternalName(ImportedTypeReference type) => Equals(type.Owner.ExternalObjectRoot, type) ? "System.Object" : type.Owner.AuthoredPrimitiveOwner(type) is PrimitiveType.Void or PrimitiveType.Value ? "System." + type.Owner.AuthoredPrimitiveOwner(type) : type.Owner.IsNativeGrapheme(type) ? "System.Char" : type.Owner.NativeBindingFor(type.AssemblyIdentity) is { } binding ? binding.TypeName(type) : type.DeclaringType is { } parent ? ExternalName(parent) + ".N_" + Encoded(type.Name) : ModuleIdentity(type.AssemblyIdentity) + ".T_" + Encoded(type.Namespace) + "_" + Encoded(type.Name);
        static object ExternalValue(ImportedTypeReference type) => type.Owner.AuthoredPrimitiveOwner(type) is { } primitive ? primitive.ToString() : NativeImportBinding.IsInhabitedVoid(type) ? "Void" : NativeImportBinding.IsIntrinsicChar(type) ? "Char" : NativeImportBinding.IsErasedValue(type) ? "Value" : type.TypeArguments.Count == 0 ? new { Named = ExternalName(type) } : new { Constructed = new { definition = ExternalName(type), arguments = type.TypeArguments.Select(SignatureValue).ToArray() } };
        static object SignatureValue(SignatureType type) => type.PointerElement is { } pointer ? new { Ptr = SignatureValue(pointer) } : type.IsSelf ? "SelfType" : type.FunctionSignature is { } function ? new { Function = new { parameters = function.ParameterTypes.Select(SignatureValue).ToArray(), returns = SignatureValue(function.ReturnType), no_result = function.NoResult } } : type.ByReferenceElement is { } target ? new { ByRef = SignatureValue(target) } : type.ImportedType is { } imported ? ExternalValue(imported) : type.GenericInstance is { } instance ? TypeOwner(instance.Definition, instance.TypeArguments) : type.TypeParameterIndex is { } ordinal ? new { TypeParameter = ordinal } : type.MethodParameterIndex is { } index ? new { MethodTypeParameter = index } : type.ArrayElement is { } element ? new { ArrayRef = SignatureValue(element) } : type.ClassType is { NativePrimitive: PrimitiveType.Void or PrimitiveType.Value } scalar ? (object)scalar.NativePrimitive.Value.ToString() : type.ClassType is { NativeGrapheme: true } ? (object)"Char" : type.ClassType is { } c ? new { Named = TypeName(c) } : type.Primitive!.Value.ToString();
        static object[] Parameters(MethodBuilder method) => method.Signature.ParameterTypes.Select(SignatureValue).ToArray();
        object Origin(string name, int token, MethodBuilder? method = null, bool publiclyVisible = true)
        {
            var origin = new Dictionary<string, object> { ["assembly"] = IdentityText(Identity), ["module"] = Identity.Name + ".dll", ["name"] = name, ["token"] = token };
            if (method is null) origin["publicly_visible"] = publiclyVisible;
            else
            {
                origin["member_access"] = method.Visibility == MethodVisibility.Internal ? "Assembly" : method.Visibility == MethodVisibility.Protected ? "Family" : method.Visibility.ToString();
                origin["parameter_tokens"] = parameterTokens[method];
                if (method.Definition.NullableAnnotations.Count != 0)
                    origin["nullable_annotations"] = method.Definition.NullableAnnotations.OrderBy(p => p.Key).Select(p => new { position = p.Key, flags = p.Value.Flags.Select(f => (int)f).ToArray(), uniform = p.Value.IsUniform }).ToArray();
            }
            return origin;
        }
        object Instruction(MethodBuilder.Operation instruction) => instruction.Op switch
        {
            "function.bind" => new { op = "function.bind", arg = new { function_type = SignatureValue(instruction.Type!), target = new { name = FunctionName(instruction.Target!), owner = instruction.ConstructedTarget is { } binding ? TypeOwner(binding.Definition.DeclaringType!, binding.DeclaringTypeArguments) : Owner(instruction.Target!), instance = !instruction.Target!.IsStatic, generic_arguments = (instruction.ConstructedTarget?.MethodArguments ?? instruction.GenericTarget?.TypeArguments ?? []).Select(SignatureValue).ToArray(), parameters = (instruction.ConstructedTarget?.Signature ?? instruction.GenericTarget?.Signature ?? instruction.Target!.Signature).ParameterTypes.Select(SignatureValue).ToArray() } } },
            "function.invoke" => new { op = "call", arg = new { name = "$Function.Invoke", owner = SignatureValue(instruction.Type!), instance = true, parameters = instruction.Type!.FunctionSignature!.ParameterTypes.Select(SignatureValue).ToArray() } },
            "array.length" => new { op = "ldlen" },
            "array.reserve" => new { op = "array.reserve", arg = SignatureValue(instruction.Type!) },
            "array.new" or "array.load" or "array.store" => new { op = instruction.Op == "array.new" ? "newarr" : instruction.Op == "array.load" ? "ldelem" : "stelem", arg = SignatureValue(instruction.Type!) },
            "type.token" => new { op = "ldtoken", arg = SignatureValue(instruction.Type!) },
            "string" => new { op = "ldstr", arg = (object)instruction.Text! },
            "boolean" => new { op = "ldc.bool", arg = (object)(instruction.Value != 0) },
            "negate" => new { op = "neg" },
            "complement" => new { op = "not" },
            "constantSingle" => new { op = "ldc.r4", arg = new { bits = unchecked((uint)instruction.Value) } },
            "constantDouble" => new { op = "ldc.r8", arg = new { bits = unchecked((ulong)instruction.LongValue) } },
            "convertSingle" => new { op = "conv.r4" },
            "convertDouble" => new { op = "conv.r8" },
            "constant64" => new { op = "ldc.i8", arg = (object)instruction.LongValue },
            "convert64" => new { op = "conv.i8" },
            "convert32" => new { op = "conv.i4" },
            "convertByte" => new { op = "conv.u1" },
            "convertSByte" => new { op = "conv.i1" },
            "convertInt16" => new { op = "conv.i2" },
            "convertUInt16" => new { op = "conv.u2" },
            "convertUInt32" => new { op = "conv.u4" },
            "convertUInt64" => new { op = "conv.u8" },
            "convertIntPtr" => new { op = "conv.i" },
            "convertUIntPtr" => new { op = "conv.u" },
            "divide.unsigned" => new { op = "div.un" },
            "remainder.unsigned" => new { op = "rem.un" },
            "shift.right.unsigned" => new { op = "shr.un" },
            "convertUnsignedDouble" => new { op = "conv.r.un" },

            "pop" => new { op = "pop" },
            "equal" => new { op = "ceq" },
            "less" => new { op = "clt" },
            "less.unordered" => new { op = "clt.un" },
            "greater.unordered" => new { op = "cgt.un" },
            "greater" => new { op = "cgt" },
            "constant" => new { op = "ldc.i4", arg = (object)instruction.Value },
            "enum.from" => new { op = "newobj", arg = SignatureValue(instruction.Type!) },
            "enum.to" => new { op = "conv.i4" },
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
            "call.constrained" => new { op = "callself", arg = new { borrowed = !instruction.Target!.IsStatic, self_type = SignatureValue(instruction.Type!), target = new { name = FunctionName(instruction.Target!), owner = instruction.ConstrainedConstructedReference is { } constrained ? SignatureValue(constrained.DeclaringType) : Owner(instruction.Target!), instance = !instruction.Target.IsStatic, parameters = instruction.ConstructedTarget is { } constrainedTarget ? constrainedTarget.Signature.ParameterTypes.Select(SignatureValue).ToArray() : Parameters(instruction.Target!) } } },
            "call.virtual" or "call" or "new.object" => new { op = instruction.Op == "call.virtual" ? "callvirt" : instruction.Op == "call" ? "call" : "newobj.ctor", arg = (object)new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), instance = !instruction.Target!.IsStatic, parameters = Parameters(instruction.Target!) } },
            "duplicate" => new { op = "dup" },
            "field.import.load" or "field.import.store" => new { op = instruction.Op == "field.import.load" ? "ldfld" : "stfld", arg = (object)(instruction.ImportedField!.NativeIndex ?? throw new InvalidDataException("native field emission requires a native layout ordinal")) },
            "field.address" or "field.load" or "field.store" => new { op = instruction.Op == "field.address" ? "ldflda" : instruction.Op == "field.load" ? "ldfld" : "stfld", arg = (object)(instruction.Field!.DeclaringType.InheritedFieldCount + instruction.Field.Index) },
            "argument.address" => new { op = "ldarga", arg = (object)instruction.Value },
            "local.address" => new { op = "ldloca", arg = (object)instruction.Value },
            "object.box" => new { op = "box", arg = SignatureValue(instruction.Type!) },
            "reference.isnull" => new { op = "ref.isnull" },
            "reference.test" => new { op = "isinst", arg = SignatureValue(instruction.Type!) },
            "object.unbox" => new { op = "unbox.any", arg = SignatureValue(instruction.Type!) },
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
        var fieldTokens = types.SelectMany(t => t.MetadataFields).Select((field, index) => (field, token: 0x04000001 + index)).ToDictionary(p => p.field, p => p.token);
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
        object Attribute(CustomAttributeDefinition attribute, int? targetToken = null)
        {
            var owner = attributeOwners[attribute];
            var name = owner.ClassType is { } local ? TypeName(local) : ExternalName(owner.ImportedType!);
            var arguments = attribute.GetArguments();
            var data = new Dictionary<string, object>
            {
                ["constructor"] = new { name = name + "..ctor", owner = SignatureValue(owner), instance = true, parameters = arguments.Select(a => a.Type.ToString()).ToArray() },
                ["arguments"] = arguments.Select(a => (object)new Dictionary<string, object?> { [a.Type.ToString()] = a.Value }).ToArray()
            };
            if (targetToken is { } token) data["target_token"] = token;
            return data;
        }
        object[]? ExplicitMappings(MethodBuilder method) => method.Definition.ExplicitInterfaceImplementations.Count == 0 ? null :
            method.Definition.ExplicitInterfaceImplementations.Select(mapping =>
            {
                var owner = method.ExplicitOwner(mapping);
                var contract = method.DeclaringType!.RequiredInterfaceMethods.Single(c => c.Name == mapping.MemberName && Equals(c.Owner, owner) && c.Signature.Matches(method.Signature));
                return (object)new { name = FunctionName(contract.Declaration), owner = SignatureValue(owner), instance = true, parameters = contract.Signature.ParameterTypes.Select(SignatureValue).ToArray() };
            }).ToArray();
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
        if (arrayBacking is not null)
        {
            ValidateArrayBacking(arrayBacking);
            manifest["array_backing"] = new { module = ModuleName(this), revision = Identity.Version.ToString(), index = types.IndexOf(arrayBacking) };
        }
        var moduleBindings = dependencies.Values.Where(d => ExternalObjectRoot?.AssemblyIdentity.Equals(d.Identity) == true || d.NativeBinding is not null || externalGrapheme?.AssemblyIdentity.Equals(d.Identity) == true || ExternalPrimitive(PrimitiveType.Void)?.AssemblyIdentity.Equals(d.Identity) == true || ExternalPrimitive(PrimitiveType.Value)?.AssemblyIdentity.Equals(d.Identity) == true).Select(d => new
        {
            assembly = IdentityText(d.Identity),
            module = ModuleName(d),
            revision = d.NativeBinding is null ? d.Identity.Version.ToString() : d.NativeBinding.Revision
        }).ToArray();
        if (moduleBindings.Length != 0) manifest["native_module_bindings"] = moduleBindings;
        var typeBindings = importedNominalTypes.Values.Where(t => Equals(ExternalObjectRoot, t) || NativeBindingFor(t.AssemblyIdentity) is not null || IsNativeGrapheme(t) || AuthoredPrimitiveOwner(t) is PrimitiveType.Void or PrimitiveType.Value).Select(t => new
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
        var importedValues = importedNominalTypes.Values.Where(t => t.IsValueType && AuthoredPrimitiveOwner(t) is null or PrimitiveType.Void or PrimitiveType.Value).Select(ExternalName).Order().ToArray();
        if (namespaceConstants.Count != 0) manifest["namespace_constants"] = namespaceConstants.Select(c => new
        {
            @namespace = c.Namespace,
            name = c.Name,
            type = "Double",
            bits = c.Bits,
            visibility = c.Visibility.ToString().ToLowerInvariant()
        }).ToArray();
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
                TypeName(type), type.Fields.Select(f => (object)new { name = f.Name, ty = SignatureValue(f.FieldType), visibility = type.IsEnum ? "private" : f.Visibility.ToString().ToLowerInvariant() }).ToArray(), !type.IsInterface && !type.IsValueType && type.NativePrimitive is null, !type.IsInterface && type.IsAbstract, (type.Definition.Attributes & 0x100) != 0, type.IsClosedHierarchy,
                TypeOrigin(type, index), type.LocalBase is { } baseType ? SignatureValue(baseType.OpenSignature) : null, type.IsInterface ? "Interface" : (type.NativePrimitive is not null || type.NativeGrapheme) ? "Runtime" : null,
                !type.InterfaceSignatures.Any() ? null : type.InterfaceSignatures.Select(SignatureValue).ToArray(),
                type.Visibility == TypeVisibility.Internal ? "internal" : null,
                type.Properties.Count == 0 ? null : type.Properties.Select(p => (object)new { name = p.Name, instance = !p.IsStatic, parameters = p.ParameterTypes.Select(SignatureValue).ToArray(), ty = SignatureValue(p.PropertyType), getter = Accessor(p.GetMethod), setter = Accessor(p.SetMethod) }).ToArray(), type.GenericParameterNames.Count == 0 ? null : type.GenericParameterNames.ToArray(), Constraints(type), type.Definition.DeclaringType is { } parent ? new { module = ModuleName(this), revision = Identity.Version.ToString(), index = types.IndexOf(parent.Producer!) } : null, !type.Definition.CustomAttributes.Any(a => !type.Definition.IsFlagsAttribute(a)) ? null : type.Definition.CustomAttributes.Where(a => !type.Definition.IsFlagsAttribute(a)).Select(a => Attribute(a)).ToArray(), type.IsEnum ? new { underlying = "Int32", flags = type.IsFlagsEnum, members = type.MetadataFields.Where(f => f.Definition.IsLiteral).Select(f => new { name = f.Name, value = f.Definition.Constant!.Value }).ToArray() } : null)).ToArray(),
            functions = methods.Select((method, index) => new NativeMethodRow(
                FunctionName(method), Owner(method), Parameters(method),
                method.Locals.Select(local => SignatureValue(local.SignatureType)).ToArray(),
                SignatureValue(method.Signature.ReturnType), !method.ReturnsValue,
                Origin(method.Name, 0x06000001 + index, method), method.IsAbstract ? [] : NativeBody(method), method.IsAbstract ? true : null, method.IsVirtual && !method.IsStatic ? true : null, method.IsOverride ? true : null,
                method.Visibility == MethodVisibility.Public ? null : method.Visibility.ToString().ToLowerInvariant(),
                method.DeclaringType is null && method.Namespace.Length != 0 ? method.Namespace : null,
                method.IsStatic ? null : true, method.Signature.GenericParameterNames.Count == 0 ? null : method.Signature.GenericParameterNames.ToArray(), method.Signature.OutParameters.Count == 0 ? null : method.Signature.OutParameters.ToArray(), !method.IsStatic && method.DeclaringType!.IsValueType ? true : null,
                method.Definition.ParameterNames.Count == 0 ? null : Enumerable.Range(0, method.ParameterCount).Select(i => method.Definition.ParameterNames.GetValueOrDefault(i)).ToArray(),
                method.InterfaceConstraints.Count == 0 ? null : method.InterfaceConstraints.Select(c => (object)new { parameter = c.ParameterIndex, kind = new { TypeBound = SignatureValue(method.Definition.ConstraintSignature(c.InterfaceType)) } }).ToArray(), ExplicitMappings(method), method.Definition.ImplementationAttributes == 0 ? null : method.Definition.ImplementationAttributes, parameterAttributes.TryGetValue(method, out var parameterAttribute) ? [Attribute(parameterAttribute, parameterTokens[method][method.Definition.ParameterArrayIndex!.Value])] : null)).ToArray()
        };
        var result = JsonSerializer.SerializeToUtf8Bytes(artifact);
        if (result.Length > maxImageSize) throw new InvalidDataException("output image exceeds limit");
        return result;
    }
    private sealed record NativeMethodRow(string name, object? owner, object[] parameters, object[] locals,
        object returns, bool no_result, object origin, object[] body,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? is_abstract,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? is_virtual,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? is_override,
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
        bool? receiver_byref,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string?[]? parameter_names,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? generic_constraints,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? interface_implementations,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        ushort? impl_flags,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? custom_attributes);
    private sealed record NativeTypeRow(string name, object[] fields, bool is_reference_type,
        bool is_abstract, bool is_sealed,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingDefault)]
        bool is_closed_hierarchy, object origin,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object? @base,
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
        object? declaring_type,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? custom_attributes,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object? enum_info);

}
