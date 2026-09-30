using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Emits a native neoCLR format-5 assembly directly from this graph.</summary>
    /// <returns>Owned UTF-8 JSON bytes accepted by the native assembly loader.</returns>
    /// <exception cref="InvalidDataException">Invalid graph, unsupported descriptive names, identity collision, incompatible core contract or output limit.</exception>
    /// <remarks>Supports linear Int32 bodies, cross-assembly top-level functions and native-only constant console output.
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
        static string[] Parameters(MethodBuilder method) => Enumerable.Repeat("Int32", method.ParameterCount).ToArray();
        object Origin(string name, int token, MethodBuilder? method = null) => method is null
            ? new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, publicly_visible = true }
            : new { assembly = IdentityText(Identity), module = Identity.Name + ".dll", name, token, member_access = "Public", parameter_tokens = new int[method.ParameterCount] };
        object Instruction(MethodBuilder.Operation instruction) => instruction.Op switch
        {
            "constant" => new { op = "ldc.i4", arg = (object)instruction.Value },
            "argument" => new { op = "ldarg", arg = (object)instruction.Value },
            "call" => new { op = "call", arg = (object)new { name = FunctionName(instruction.Target!), owner = Owner(instruction.Target!), parameters = Parameters(instruction.Target!) } },
            "add" => new { op = "add" },
            "subtract" => new { op = "sub" },
            "multiply" => new { op = "mul" },
            "return" => new { op = "ret" },
            _ => throw new InvalidDataException("unsupported instruction")
        };
        IEnumerable<object> NativeInstructions(MethodBuilder.Operation instruction)
        {
            if (instruction.Op == "console.line")
                return new object[] {
                    new { op = "ldstr", arg = instruction.Text! },
                    new { op = "call", arg = new { name = "System.Console.WriteLine", owner = new { Named = "System.Console" }, parameters = new[] { "String" } } },
                    new { op = "pop" } // Bundled System returns the Void value; statement discards it.
                };
            return new[] { Instruction(instruction) };
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
                returns = method.ReturnsValue ? "Int32" : "Void", no_result = !method.ReturnsValue,
                origin = Origin(method.Name, 0x06000001 + index, method), body = method.Instructions.SelectMany(NativeInstructions).ToArray()
            }).ToArray()
        };
        var result = JsonSerializer.SerializeToUtf8Bytes(artifact);
        if (result.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("output image exceeds limit");
        return result;
    }
}
