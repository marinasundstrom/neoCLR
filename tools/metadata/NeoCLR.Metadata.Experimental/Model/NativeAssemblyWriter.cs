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
        static string IdentityText(AssemblyIdentity identity) => JsonSerializer.Serialize(new[] {
            identity.Name, identity.Version.ToString(), identity.Culture, identity.PublicKeyToken, identity.Flags.ToString(System.Globalization.CultureInfo.InvariantCulture)
        });
        static string ModuleName(AssemblyBuilder assembly) => "NeoMetadata_" + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(IdentityText(assembly.Identity))));
        static string Encoded(string value) => Convert.ToHexString(Encoding.UTF8.GetBytes(value));
        static string TypeName(TypeBuilder type) => ModuleName(type.Assembly) + ".T_" + Encoded(type.Namespace) + "_" + Encoded(type.Name);
        static string FunctionName(MethodBuilder method) => method.IsConstructor ? TypeName(method.DeclaringType!) + "..ctor" : (method.DeclaringType is { } type ? TypeName(type) + ".M_" : ModuleName(method.Assembly) + ".F_") + Encoded(method.CliName);
        static object? Owner(MethodBuilder method) => method.DeclaringType is { } type ? new { Named = TypeName(type) } : null;
        static object SignatureValue(SignatureType type) => type.ClassType is { } c ? new { Named = TypeName(c) } : type.Primitive!.Value.ToString();
        static object[] Parameters(MethodBuilder method) => method.Signature.ParameterTypes.Select(SignatureValue).ToArray();
        object Origin(string name, int token, MethodBuilder? method = null, bool publiclyVisible = true) => method is null
            ? new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, publicly_visible = publiclyVisible }
            : new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, member_access = method.Visibility == MethodVisibility.Internal ? "Assembly" : method.Visibility.ToString(), parameter_tokens = new int[method.ParameterCount] };
        object Instruction(MethodBuilder.Operation instruction) => instruction.Op switch
        {
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
            "call" or "new.object" => new { op = instruction.Op == "call" ? "call" : "newobj.ctor", arg = (object)new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), instance = !instruction.Target!.IsStatic, parameters = Parameters(instruction.Target!) } },
            "duplicate" => new { op = "dup" },
            "field.load" or "field.store" => new { op = instruction.Op == "field.load" ? "ldfld" : "stfld", arg = (object)instruction.Field!.Index },
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
            _ => throw new InvalidDataException("unsupported instruction")
        };
        IEnumerable<object> NativeInstructions(MethodBuilder.Operation instruction)
        {
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
                offsets[i + 1] = offsets[i] + (method.Instructions[i].Op switch { "label" => 0, "console.line" => 3, "console.write" => 2, _ => 1 });
            var labels = method.LabelPositions();
            return method.Instructions.SelectMany(instruction => instruction.Op switch {
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
            if (type.Fields.Count == 0 && type.Properties.Count == 0)
                return Origin(name, 0x02000002 + index, publiclyVisible: type.Visibility == TypeVisibility.Public);
            var origin = new Dictionary<string, object> {
                ["assembly"] = IdentityText(Identity), ["module"] = Identity.Name + ".dll", ["name"] = name,
                ["token"] = 0x02000002 + index, ["publicly_visible"] = type.Visibility == TypeVisibility.Public,
                ["field_tokens"] = type.Fields.Select(f => fieldTokens[f]).ToArray(),
                ["field_access"] = type.Fields.Select(f => f.Visibility == FieldVisibility.Internal ? "Assembly" : f.Visibility.ToString()).ToArray(),
                ["field_readonly"] = type.Fields.Select(_ => false).ToArray()
            };
            if (type.Properties.Count != 0) origin["property_tokens"] = type.Properties.Select(p => propertyTokens[p]).ToArray();
            return origin;
        }
        object? Accessor(MethodBuilder? method) => method is null ? null : new { name = FunctionName(method), owner = Owner(method), instance = !method.IsStatic, parameters = Parameters(method) };
        var artifact = new
        {
            format = 5,
            name = ModuleName(this),
            revision = Identity.Version.ToString(),
            references = dependencies.Values.Select(d => new { name = ModuleName(d), revision = d.Identity.Version.ToString() }).ToArray(),
            entry = EntryPoint is null ? "" : FunctionName(EntryPoint),
            assemblies = new[] { new { name = Identity.Name, full_name = IdentityText(Identity), modules = new[] { Identity.Name + ".dll" }, references = dependencies.Keys.Select(IdentityText).ToArray() } },
            types = types.Select((type, index) => new NativeTypeRow(
                TypeName(type), type.Fields.Select(f => (object)new { name = f.Name, ty = SignatureValue(f.FieldType), visibility = f.Visibility.ToString().ToLowerInvariant() }).ToArray(), true, type.IsStatic, type.IsStatic,
                TypeOrigin(type, index),
                type.Visibility == TypeVisibility.Internal ? "internal" : null,
                type.Properties.Count == 0 ? null : type.Properties.Select(p => (object)new { name = p.Name, instance = !p.IsStatic, parameters = System.Array.Empty<string>(), ty = SignatureValue(p.PropertyType), getter = Accessor(p.GetMethod), setter = Accessor(p.SetMethod) }).ToArray())).ToArray(),
            functions = methods.Select((method, index) => new NativeMethodRow(
                FunctionName(method), Owner(method), Parameters(method),
                method.Locals.Select(local => local.ClassType is { } type ? (object)new { Named = TypeName(type) } : local.Type!.Value.ToString()).ToArray(),
                SignatureValue(method.Signature.ReturnType), !method.ReturnsValue,
                Origin(method.Name, 0x06000001 + index, method), NativeBody(method),
                method.Visibility == MethodVisibility.Public ? null : method.Visibility.ToString().ToLowerInvariant(),
                method.DeclaringType is null && method.Namespace.Length != 0 ? method.Namespace : null,
                method.IsStatic ? null : true)).ToArray()
        };
        var result = JsonSerializer.SerializeToUtf8Bytes(artifact);
        if (result.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("output image exceeds limit");
        return result;
    }
    private sealed record NativeMethodRow(string name, object? owner, object[] parameters, object[] locals,
        object returns, bool no_result, object origin, object[] body,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? visibility,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? @namespace,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        bool? instance);
    private sealed record NativeTypeRow(string name, object[] fields, bool is_reference_type,
        bool is_abstract, bool is_sealed, object origin,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        string? visibility,
        [property: System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
        object[]? properties);

}
