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
        static string FunctionName(MethodBuilder method) => (method.DeclaringType is { } type ? TypeName(type) + ".M_" : ModuleName(method.Assembly) + ".F_") + Encoded(method.Name);
        static object? Owner(MethodBuilder method) => method.DeclaringType is { } type ? new { Named = TypeName(type) } : null;
        static string[] Parameters(MethodBuilder method) => method.Signature.ParameterTypes.Select(t => t.ToString()).ToArray();
        object Origin(string name, int token, MethodBuilder? method = null) => method is null
            ? new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, publicly_visible = true }
            : new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, member_access = "Public", parameter_tokens = new int[method.ParameterCount] };
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
            "argument" => new { op = "ldarg", arg = (object)instruction.Value },
            "call" => new { op = "call", arg = (object)new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), parameters = Parameters(instruction.Target!) } },
            "local.load" => new { op = "ldloc", arg = (object)instruction.Value },
            "local.store" => new { op = "stloc", arg = (object)instruction.Value },
            "add" => new { op = "add" },
            "subtract" => new { op = "sub" },
            "multiply" => new { op = "mul" },
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
        var artifact = new
        {
            format = 5,
            name = ModuleName(this),
            revision = Identity.Version.ToString(),
            references = dependencies.Values.Select(d => new { name = ModuleName(d), revision = d.Identity.Version.ToString() }).ToArray(),
            entry = EntryPoint is null ? "" : FunctionName(EntryPoint),
            assemblies = new[] { new { name = Identity.Name, full_name = IdentityText(Identity), modules = new[] { Identity.Name + ".dll" }, references = dependencies.Keys.Select(IdentityText).ToArray() } },
            types = types.Select((type, index) => new {
                name = TypeName(type), fields = Array.Empty<object>(), is_reference_type = true, is_abstract = true, is_sealed = true,
                origin = Origin(type.Namespace.Length == 0 ? type.Name : type.Namespace + "." + type.Name, 0x02000002 + index)
            }).ToArray(),
            functions = methods.Select((method, index) => new {
                name = FunctionName(method), owner = Owner(method), parameters = Parameters(method),
                locals = method.Locals.Select(local => local.Type.ToString()).ToArray(),
                returns = method.Signature.ReturnType.ToString(), no_result = !method.ReturnsValue,
                origin = Origin(method.Name, 0x06000001 + index, method), body = NativeBody(method)
            }).ToArray()
        };
        var result = JsonSerializer.SerializeToUtf8Bytes(artifact);
        if (result.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("output image exceeds limit");
        return result;
    }
}
