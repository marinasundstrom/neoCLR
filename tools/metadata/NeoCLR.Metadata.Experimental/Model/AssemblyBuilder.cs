using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A controlled editable assembly graph for the initial static Int32 compiler subset.</summary>
/// <remarks>Produces ordinary unsigned CLI PE32 or native format-5 JSON. NEOX emission and arbitrary assembly rewriting are not supported.</remarks>
public sealed partial class AssemblyBuilder
{
    private readonly List<TypeBuilder> types = [];
    private readonly List<MethodBuilder> functions = [];
    private readonly Guid mvid = Guid.NewGuid();
    /// <summary>Creates an assembly builder with explicit core-library identity.</summary>
    /// <param name="identity">Unsigned assembly identity with no flags.</param>
    /// <param name="coreLibrary">Core assembly supplying System.Object; no host core library is inferred.</param>
    /// <exception cref="ArgumentNullException">Either identity is null.</exception>
    /// <exception cref="ArgumentException">Output identity requires signing or flags.</exception>
    public AssemblyBuilder(AssemblyIdentity identity, AssemblyIdentity coreLibrary)
    {
        ArgumentNullException.ThrowIfNull(identity); ArgumentNullException.ThrowIfNull(coreLibrary);
        if (identity.PublicKeyToken.Length != 0 || identity.Flags != 0) throw new ArgumentException("writer supports unsigned unflagged identities");
        Identity = identity; CoreLibrary = coreLibrary;
    }
    /// <summary>Gets the output identity.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Gets the explicit core-library identity.</summary>
    public AssemblyIdentity CoreLibrary { get; }
    /// <summary>Gets the current owned type definitions.</summary>
    public IReadOnlyList<TypeBuilder> Types => types.AsReadOnly();
    /// <summary>Gets assembly-owned functions, which have no declaring type.</summary>
    public IReadOnlyList<MethodBuilder> Functions => functions.AsReadOnly();
    /// <summary>Adds a top-level function with Int32 parameters and Int32 or absent result.</summary>
    /// <param name="name">Nonempty name, unique by name and parameter count among top-level functions.</param>
    /// <param name="parameterCount">Number of Int32 parameters, 0–256.</param>
    /// <param name="returnsValue">True for Int32, false for no result.</param>
    /// <returns>An assembly-owned function with a null declaring type.</returns>
    /// <exception cref="ArgumentException">Invalid/duplicate signature or more than 256 functions.</exception>
    public MethodBuilder AddFunction(string name, int parameterCount = 0, bool returnsValue = true)
    {
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || parameterCount is < 0 or > 256 || functions.Count >= 256 ||
            functions.Any(m => m.Name == name && m.ParameterCount == parameterCount)) throw new ArgumentException("invalid or duplicate function");
        var function = new MethodBuilder(this, null, name, parameterCount, returnsValue);
        functions.Add(function);
        return function;
    }
    internal MethodBuilder[] ValidateGraph()
    {
        var methods = functions.Concat(types.SelectMany(t => t.Methods)).ToArray();
        if (methods.Sum(method => (long)method.Instructions.Count) > 131072) throw new InvalidDataException("assembly instruction limit exceeded");
        if (methods.SelectMany(method => method.Instructions).Sum(instruction =>
            instruction.Text is null ? 0L : System.Text.Encoding.UTF8.GetByteCount(instruction.Text)) > MetadataArtifactReader.MaxImageSize)
            throw new InvalidDataException("assembly string literal limit exceeded");
        if (methods.Length > 4096) throw new InvalidDataException("too many methods");
        if (EntryPoint is not null && (!methods.Contains(EntryPoint) || EntryPoint.ParameterCount != 0))
            throw new InvalidDataException("entry point must be a local parameterless Int32 or no-result method");
        foreach (var method in methods) method.Validate();
        return methods;
    }
    /// <summary>Gets or sets a local parameterless Int32 or no-result entry point; null writes a library.</summary>
    public MethodBuilder? EntryPoint { get; set; }
    /// <summary>Adds a unique public static class.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <returns>A type owned by this builder.</returns>
    /// <exception cref="ArgumentException">Null/invalid names, duplicate type or more than 256 types.</exception>
    public TypeBuilder AddType(string @namespace, string name)
    {
        if (@namespace is null || string.IsNullOrEmpty(name) || name == "<Module>" || @namespace.Length + name.Length > 1024 ||
            types.Count >= 256 || types.Any(t => t.Namespace == @namespace && t.Name == name)) throw new ArgumentException("invalid or duplicate type");
        var type = new TypeBuilder(this, @namespace, name); types.Add(type); return type;
    }
    /// <summary>Validates all bodies and emits a fresh unsigned managed PE32 image.</summary>
    /// <returns>Owned PE bytes suitable for conventional readers and the supported neoCLR CLI import bridge.</returns>
    /// <exception cref="InvalidDataException">Invalid body, entry point, dependency identity or resource limit.</exception>
    /// <remarks>No image is loaded. Cross-assembly calls import exact assembly/type/member references. Repeated writes have a fixed MVID and timestamp for reproducibility.</remarks>
    public byte[] Write() => WriteImage(referenceOnly: false);

    internal byte[] WriteReferenceImage() => WriteImage(referenceOnly: true);

    private byte[] WriteImage(bool referenceOnly)
    {
        var methods = ValidateGraph();
        var metadata = new MetadataBuilder();
        // A stable per-builder MVID preserves snapshot scope; PE content IDs/timestamps are deterministic.
        metadata.AddModule(0, metadata.GetOrAddString(Identity.Name + ".dll"), metadata.GetOrAddGuid(mvid), default, default);
        metadata.AddAssembly(metadata.GetOrAddString(Identity.Name), Identity.Version, metadata.GetOrAddString(Identity.Culture), default, 0, AssemblyHashAlgorithm.None);
        var assemblyRefs = new Dictionary<AssemblyIdentity, AssemblyReferenceHandle>();
        AssemblyReferenceHandle ImportAssembly(AssemblyIdentity identity)
        {
            if (identity.Equals(Identity)) throw new InvalidDataException("external assembly identity collides with output identity");
            if (!assemblyRefs.TryGetValue(identity, out var handle))
            {
                if (assemblyRefs.Count >= 256) throw new InvalidDataException("too many imported assemblies");
                handle = metadata.AddAssemblyReference(metadata.GetOrAddString(identity.Name), identity.Version, metadata.GetOrAddString(identity.Culture),
                    metadata.GetOrAddBlob(Convert.FromHexString(identity.PublicKeyToken)), (AssemblyFlags)identity.Flags, default);
                assemblyRefs.Add(identity, handle);
            }
            return handle;
        }
        var objectType = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString("Object"));
        if (referenceOnly)
        {
            var attributeType = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System.Runtime.CompilerServices"), metadata.GetOrAddString("ReferenceAssemblyAttribute"));
            var constructor = metadata.AddMemberReference(attributeType, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(new byte[] { 0x20, 0, 1 }));
            metadata.AddCustomAttribute(MetadataTokens.EntityHandle(0x20000001), constructor, metadata.GetOrAddBlob(new byte[] { 1, 0, 0, 0 }));
        }
        var handles = methods.Select((method, index) => (method, handle: MetadataTokens.MethodDefinitionHandle(index + 1))).ToDictionary(pair => pair.method, pair => pair.handle);
        var importedTypes = new Dictionary<TypeBuilder, TypeReferenceHandle>();
        var importedMethods = new Dictionary<MethodBuilder, MemberReferenceHandle>();
        BlobHandle Signature(MethodBuilder method)
        {
            var signature = new BlobBuilder();
            new BlobEncoder(signature).MethodSignature().Parameters(method.ParameterCount,
                result => { if (method.ReturnsValue) result.Type().Int32(); else result.Void(); },
                parameters => { for (int i = 0; i < method.ParameterCount; i++) parameters.AddParameter().Type().Int32(); });
            return metadata.GetOrAddBlob(signature);
        }
        int ImportMethod(MethodBuilder method)
        {
            if (handles.TryGetValue(method, out var local)) return MetadataTokens.GetToken(local);
            if (!CoreLibrary.Equals(method.Assembly.CoreLibrary)) throw new InvalidDataException("cross-target call requires compatible core identity");
            if (!importedMethods.TryGetValue(method, out var handle))
            {
                var owner = method.DeclaringType ?? throw new InvalidDataException("cross-assembly global function calls require native emission");
                if (!importedTypes.TryGetValue(owner, out var type))
                {
                    type = metadata.AddTypeReference(ImportAssembly(owner.Assembly.Identity), metadata.GetOrAddString(owner.Namespace), metadata.GetOrAddString(owner.Name));
                    importedTypes.Add(owner, type);
                }
                handle = metadata.AddMemberReference(type, metadata.GetOrAddString(method.Name), Signature(method));
                importedMethods.Add(method, handle);
            }
            return MetadataTokens.GetToken(handle);
        }
        var bodies = new BlobBuilder();
        var bodyEncoder = new MethodBodyStreamEncoder(bodies);
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        int nextMethod = 1;
        void EmitMethod(MethodBuilder method)
        {
            var code = new BlobBuilder();
            if (referenceOnly) { code.WriteByte(0x14); code.WriteByte(0x7a); } // ldnull; throw: never substitute native behavior.
            foreach (var instruction in referenceOnly ? [] : method.Instructions)
            {
                switch (instruction.Op)
                {
                    case "constant": code.WriteByte(0x20); code.WriteInt32(instruction.Value); break;
                    case "argument": code.WriteByte(0xfe); code.WriteByte(0x09); code.WriteUInt16((ushort)instruction.Value); break;
                    case "add": code.WriteByte(0x58); break;
                    case "subtract": code.WriteByte(0x59); break;
                    case "multiply": code.WriteByte(0x5a); break;
                    case "call": code.WriteByte(0x28); code.WriteInt32(ImportMethod(instruction.Target!)); break;
                    case "return": code.WriteByte(0x2a); break;
                    default: throw new InvalidDataException("operation requires native emission: " + instruction.Op);
                }
            }
            int body = bodyEncoder.AddMethodBody(new InstructionEncoder(code), maxStack: referenceOnly ? 1 : method.MaxStack);
            metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.Static | MethodAttributes.HideBySig,
                MethodImplAttributes.IL | MethodImplAttributes.Managed, metadata.GetOrAddString(method.Name), Signature(method), body, MetadataTokens.ParameterHandle(1));
            nextMethod++;
        }
        foreach (var function in functions) EmitMethod(function);
        foreach (var type in types)
        {
            metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed,
                metadata.GetOrAddString(type.Namespace), metadata.GetOrAddString(type.Name), objectType,
                MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(nextMethod));
            foreach (var method in type.Methods) EmitMethod(method);
        }
        var builder = new ManagedPEBuilder(new PEHeaderBuilder(fileAlignment: 4096, sectionAlignment: 4096,
                imageCharacteristics: Characteristics.ExecutableImage | Characteristics.LargeAddressAware | (EntryPoint is null ? Characteristics.Dll : 0)),
            new MetadataRootBuilder(metadata), bodies, entryPoint: referenceOnly || EntryPoint is null ? default : handles[EntryPoint],
            strongNameSignatureSize: 0, deterministicIdProvider: blobs => BlobContentId.FromHash(System.Security.Cryptography.SHA256.HashData(blobs.SelectMany(blob => blob.GetBytes()).ToArray())));
        var image = new BlobBuilder(); builder.Serialize(image);
        if (image.Count > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("output image exceeds limit");
        return image.ToArray();
    }
}

/// <summary>An editable public static class owned by an AssemblyBuilder.</summary>
public sealed class TypeBuilder
{
    private readonly List<MethodBuilder> methods = [];
    internal TypeBuilder(AssemblyBuilder assembly, string @namespace, string name) { Assembly = assembly; Namespace = @namespace; Name = name; }
    /// <summary>Gets the owning assembly.</summary>
    public AssemblyBuilder Assembly { get; }
    /// <summary>Gets the declared namespace.</summary>
    public string Namespace { get; }
    /// <summary>Gets the declared name.</summary>
    public string Name { get; }
    /// <summary>Gets owned methods in declaration order.</summary>
    public IReadOnlyList<MethodBuilder> Methods => methods.AsReadOnly();
    /// <summary>Adds a public static method with Int32 parameters and Int32 or absent result.</summary>
    /// <param name="name">Nonempty method name, unique by name/parameter count.</param>
    /// <param name="parameterCount">Number of Int32 parameters, 0–256.</param>
    /// <param name="returnsValue">True for Int32, false for CLI void/no-result.</param>
    /// <returns>A mutable body builder.</returns>
    /// <exception cref="ArgumentException">Invalid or duplicate signature or more than 256 methods.</exception>
    public MethodBuilder AddMethod(string name, int parameterCount = 0, bool returnsValue = true)
    {
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || parameterCount is < 0 or > 256 || methods.Count >= 256 ||
            methods.Any(m => m.Name == name && m.ParameterCount == parameterCount)) throw new ArgumentException("invalid or duplicate method");
        var method = new MethodBuilder(Assembly, this, name, parameterCount, returnsValue); methods.Add(method); return method;
    }
}

/// <summary>Linear, typed Int32 body construction; invalid stack contracts fail before emission.</summary>
public sealed class MethodBuilder
{
    internal sealed record Operation(string Op, int Value = 0, MethodBuilder? Target = null, string? Text = null, NativeFunctionDefinition? NativeTarget = null);
    internal List<Operation> Instructions { get; } = [];
    internal int MaxStack { get; private set; }
    internal MethodBuilder(AssemblyBuilder assembly, TypeBuilder? owner, string name, int count, bool result) { Assembly = assembly; DeclaringType = owner; Name = name; ParameterCount = count; ReturnsValue = result; }
    /// <summary>Gets the owning assembly, including for top-level functions.</summary>
    public AssemblyBuilder Assembly { get; }
    /// <summary>Gets the declaring type, or null for a top-level function.</summary>
    public TypeBuilder? DeclaringType { get; }
    /// <summary>Gets the method name.</summary>
    public string Name { get; }
    /// <summary>Gets the Int32 parameter count.</summary>
    public int ParameterCount { get; }
    /// <summary>Gets whether the method returns Int32 rather than no result.</summary>
    public bool ReturnsValue { get; }
    /// <summary>Appends an Int32 constant.</summary>
    /// <param name="value">Constant value.</param>
    public void LoadConstant(int value) => Append(new("constant", value));
    /// <summary>Appends a native System.Console.WriteLine call with a constant UTF-8 string.</summary>
    /// <param name="text">Unicode text, at most 64 KiB when UTF-8 encoded.</param>
    /// <exception cref="ArgumentNullException">Text is null.</exception>
    /// <exception cref="ArgumentException">Invalid Unicode or text exceeds the limit.</exception>
    /// <remarks>Native emission only; ordinary CLI Write rejects this operation. Does not alter the Int32 stack.</remarks>
    public void WriteConsoleLine(string text)
    {
        ArgumentNullException.ThrowIfNull(text);
        try
        {
            if (new System.Text.UTF8Encoding(false, true).GetByteCount(text) > 65536)
                throw new ArgumentException("console literal exceeds 64 KiB", nameof(text));
        }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid Unicode", nameof(text), error); }
        Append(new("console.line", Text: text));
    }
    /// <summary>Appends a parameter load; bounds are checked at Write.</summary>
    /// <param name="index">Zero-based parameter index.</param>
    public void LoadArgument(int index) => Append(new("argument", index));
    /// <summary>Appends Int32 addition.</summary>
    public void Add() => Append(new("add"));
    /// <summary>Appends Int32 subtraction.</summary>
    public void Subtract() => Append(new("subtract"));
    /// <summary>Appends Int32 multiplication.</summary>
    public void Multiply() => Append(new("multiply"));
    /// <summary>Appends a call; foreign methods are imported during Write.</summary>
    /// <param name="target">Local or external builder method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    public void Call(MethodBuilder target) { ArgumentNullException.ThrowIfNull(target); Append(new("call", Target: target)); }
    /// <summary>Appends a call to an imported read-only method contract.</summary>
    /// <param name="target">Reference imported by this method's assembly builder.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Reference belongs to another output builder.</exception>
    public void Call(ImportedMethodReference target)
    {
        ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(target.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(target));
        Append(new("call", Target: target.Target));
    }
    /// <summary>Calls a static Int32 function selected from an explicitly loaded native System inventory.</summary>
    /// <param name="target">Owned System function whose parameters and result are Int32.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="InvalidDataException">Module is not System or the callable signature is unsupported.</exception>
    /// <remarks>Native-only bootstrap. The host must supply the matching System assembly to neoCLR;
    /// no assembly revision or image digest is encoded. Ordinary CLI output rejects this operation.</remarks>
    public void Call(NativeFunctionDefinition target)
    {
        ArgumentNullException.ThrowIfNull(target);
        if (target.Library.ModuleName != "System" || !target.TryGetStaticInt32Signature(out _))
            throw new InvalidDataException("native call requires a static Int32 System function");
        Append(new("native.call", NativeTarget: target));
    }
    /// <summary>Appends return; must be the last instruction with the declared stack shape.</summary>
    public void Return() => Append(new("return"));
    /// <summary>Clears the body for editing before another Write.</summary>
    public void ClearBody() => Instructions.Clear();
    private void Append(Operation operation)
    {
        if (Instructions.Count >= 4096) throw new InvalidDataException("method instruction limit exceeded");
        Instructions.Add(operation);
    }
    internal void Validate()
    {
        int stack = 0; MaxStack = 0;
        if (Instructions.Count == 0 || Instructions[^1].Op != "return") throw new InvalidDataException("method requires final return");
        for (int i = 0; i < Instructions.Count; i++)
        {
            var instruction = Instructions[i];
            int pop = 0, push = 0;
            switch (instruction.Op)
            {
                case "console.line": break; // Temporary native string operand is consumed by Console.WriteLine.
                case "constant": push = 1; break;
                case "argument":
                    if (instruction.Value < 0 || instruction.Value >= ParameterCount) throw new InvalidDataException("argument outside signature");
                    push = 1; break;
                case "add": case "subtract": case "multiply": pop = 2; push = 1; break;
                case "native.call":
                    if (!instruction.NativeTarget!.TryGetStaticInt32Signature(out pop)) throw new InvalidDataException("invalid native call");
                    push = 1; break;
                case "call": pop = instruction.Target!.ParameterCount; push = instruction.Target.ReturnsValue ? 1 : 0; break;
                case "return":
                    if (i != Instructions.Count - 1 || stack != (ReturnsValue ? 1 : 0)) throw new InvalidDataException("invalid return stack");
                    pop = stack; break;
            }
            if (stack < pop) throw new InvalidDataException("evaluation stack underflow");
            stack = stack - pop + push; MaxStack = Math.Max(MaxStack, stack);
        }
    }
}
