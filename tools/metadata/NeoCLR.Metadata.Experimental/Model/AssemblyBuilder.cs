using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A controlled editable assembly graph for the bounded primitive and root-object compiler subset.</summary>
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
        => AddFunction(name, PrimitiveMethodSignature.Int32(parameterCount, returnsValue));
    /// <summary>Adds a static assembly-owned function with an explicit primitive or owned-class signature.</summary>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="signature">Primitive or owned root-class parameters/results; Void is allowed only as the result.</param>
    /// <returns>An assembly-owned method builder.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Invalid/duplicate name and parameter types or function limit exceeded.</exception>
    public MethodBuilder AddFunction(string name, MethodSignature signature)
        => AddFunction(name, signature, MethodVisibility.Public);
    /// <summary>Adds an assembly-owned function with public or internal access.</summary>
    /// <param name="name">Nonempty function name, unique by name and parameter types.</param>
    /// <param name="signature">Supported primitive/owned-class signature.</param>
    /// <param name="visibility">Public or Internal; private access requires a declaring type.</param>
    /// <returns>The owned function builder.</returns>
    /// <exception cref="ArgumentOutOfRangeException">Visibility is not Public or Internal.</exception>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Name is invalid/duplicate or the function limit is exceeded.</exception>
    public MethodBuilder AddFunction(string name, MethodSignature signature, MethodVisibility visibility)
        => AddFunction("", name, signature, visibility);
    /// <summary>Adds an ownerless function in an explicit namespace.</summary>
    /// <param name="namespace">Namespace, possibly empty; nonempty segments separated by dots.</param>
    /// <param name="name">Nonempty simple name. The CLI projection prefix &lt;NeoFunction&gt; is reserved.</param>
    /// <param name="signature">Supported primitive/owned-class signature.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An assembly-owned function preserving namespace and simple name.</returns>
    /// <exception cref="ArgumentNullException">Namespace or signature is null.</exception>
    /// <exception cref="ArgumentOutOfRangeException">Visibility is unsupported.</exception>
    /// <exception cref="ArgumentException">Invalid namespace/name, duplicate signature or exceeded limit.</exception>
    public MethodBuilder AddFunction(string @namespace, string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
    {
        if (visibility is not (MethodVisibility.Public or MethodVisibility.Internal)) throw new ArgumentOutOfRangeException(nameof(visibility));
        ArgumentNullException.ThrowIfNull(signature);
        signature.ValidateOwner(this);
        FunctionNamespaceEncoding.Validate(@namespace);
        if (string.IsNullOrEmpty(name) || name.StartsWith(FunctionNamespaceEncoding.Prefix, StringComparison.Ordinal) || @namespace.Length + name.Length > 1024 || functions.Count >= 256 ||
            functions.Any(m => m.Namespace == @namespace && m.Name == name && m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("invalid or duplicate function");
        var function = new MethodBuilder(this, null, name, signature, visibility, @namespace);
        functions.Add(function);
        return function;
    }
    internal MethodBuilder[] ValidateGraph(bool validateBodies = true)
    {
        var methods = functions.Concat(types.SelectMany(t => t.Methods)).ToArray();
        if (methods.Sum(method => (long)method.Instructions.Count) > 131072) throw new InvalidDataException("assembly instruction limit exceeded");
        if (methods.SelectMany(method => method.Instructions).Sum(instruction =>
            instruction.Text is null ? 0L : System.Text.Encoding.UTF8.GetByteCount(instruction.Text)) > MetadataArtifactReader.MaxImageSize)
            throw new InvalidDataException("assembly string literal limit exceeded");
        if (types.Sum(type => type.Properties.Count) > 4096) throw new InvalidDataException("too many properties");
        if (types.Sum(type => type.Fields.Count) > 4096) throw new InvalidDataException("too many fields");
        if (methods.Length > 4096) throw new InvalidDataException("too many methods");
        if (EntryPoint is not null && (!methods.Contains(EntryPoint) || !EntryPoint.IsStatic || EntryPoint.Signature.GenericParameterNames.Count != 0 || EntryPoint.DeclaringType?.GenericParameterNames.Count > 0 || EntryPoint.ParameterCount != 0 || EntryPoint.Signature.ReturnType.Primitive is not (PrimitiveType.Int32 or PrimitiveType.Void)))
            throw new InvalidDataException("entry point must be a local parameterless Int32 or no-result method");
        foreach (var target in methods.SelectMany(m => m.Instructions).Select(i => i.Target).OfType<MethodBuilder>())
            if (!ReferenceEquals(target.Assembly, this) && target.Signature.ParameterTypes.Append(target.Signature.ReturnType).Any(t => t.Primitive is null && t.ArrayElement?.Primitive is null && t.MethodParameterIndex is null && t.ArrayElement?.MethodParameterIndex is null))
                throw new InvalidDataException("external nominal method references require an import contract");
        try
        {
            foreach (var type in types.Where(t => !t.IsInterface))
                foreach (var contract in type.InterfaceMethods)
                    if (!type.Methods.Any(m => !m.IsStatic && m.Visibility == MethodVisibility.Public && m.Name == contract.Name &&
                        m.Signature.GenericParameterNames.Count == 0 && m.Signature.ReturnType == contract.Signature.ReturnType &&
                        m.Signature.ParameterTypes.SequenceEqual(contract.Signature.ParameterTypes)))
                        throw new InvalidDataException("missing public interface implementation: " + contract.Name);
            foreach (var type in types)
                foreach (var field in type.Fields) field.FieldType.ValidateOwner(this, typeArity: type.GenericParameterNames.Count, complete: true);
            foreach (var method in methods)
            {
                int arity = method.DeclaringType?.GenericParameterNames.Count ?? 0;
                method.Signature.ValidateOwner(this, arity, complete: true);
                foreach (var local in method.Locals) local.SignatureType.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                foreach (var instruction in method.Instructions)
                {
                    if ((instruction.Target ?? instruction.ConstructedTarget?.Definition ?? instruction.GenericTarget?.Definition)?.IsAbstract == true && instruction.Op != "call.virtual")
                        throw new InvalidDataException("interface dispatch requires a supported virtual-call contract");
                    if (instruction.ConstructedTarget is { } target)
                    {
                        target.Definition.DeclaringType!.ValidateTypeArguments(target.DeclaringTypeArguments, complete: true);
                        foreach (var argument in target.DeclaringTypeArguments.Concat(target.MethodArguments)) argument.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    }
                    if (instruction.GenericTarget is { } generic)
                        foreach (var argument in generic.TypeArguments) argument.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    if (instruction.ConstructedField is { } field) ((SignatureType)field.DeclaringType).ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    instruction.Type?.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                }
                if (method.IsAbstract)
                {
                    if (method.Instructions.Count != 0 || method.Locals.Count != 0) throw new InvalidDataException("abstract interface methods must have no body or locals");
                }
                else if (validateBodies) method.Validate();
            }
        }
        catch (ArgumentException error) { throw new InvalidDataException("invalid generic argument contract", error); }
        return methods;
    }
    /// <summary>Gets or sets a local parameterless Int32 or no-result entry point; null writes a library.</summary>
    public MethodBuilder? EntryPoint { get; set; }
    /// <summary>Adds a unique public static class.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <returns>A type owned by this builder.</returns>
    /// <exception cref="ArgumentException">Null/invalid names, duplicate type or more than 256 types.</exception>
    public TypeBuilder AddType(string @namespace, string name) => AddType(@namespace, name, TypeVisibility.Public);
    /// <summary>Adds a unique top-level static class with explicit public or assembly visibility.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="visibility">Public or internal visibility.</param>
    /// <returns>A type owned by this builder.</returns>
    /// <exception cref="ArgumentException">Invalid visibility/name, duplicate type or more than 256 types.</exception>
    public TypeBuilder AddType(string @namespace, string name, TypeVisibility visibility)
        => AddTypeCore(@namespace, name, visibility, isStatic: true);
    /// <summary>Adds a root reference class with mutable primitive instance fields.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty type name.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An owned, nonabstract, nonsealed class definition.</returns>
    /// <exception cref="ArgumentException">Invalid visibility/name, duplicate type or exceeded limit.</exception>
    /// <remarks>CLI base is System.Object. Native root classes have no declared base. Constructors are not synthesized.</remarks>
    public TypeBuilder AddClass(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
        => AddTypeCore(@namespace, name, visibility, isStatic: false);
    /// <summary>Adds an unconstrained static generic class, appending CLI arity to its name.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Simple name without an arity suffix.</param>
    /// <param name="genericParameterNames">One through 32 unique parameter names.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <exception cref="ArgumentException">Invalid name, parameters, duplicate type or limit exceeded.</exception>
    /// <exception cref="ArgumentNullException">Parameter names are null.</exception>
    public TypeBuilder AddGenericType(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public)
    {
        return AddGenericTypeCore(@namespace, name, genericParameterNames, visibility, true);
    }
    /// <summary>Adds an unconstrained generic reference class with explicit constructors.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Simple name without CLI arity.</param>
    /// <param name="genericParameterNames">One through 32 unique names, copied.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An owned root class; no constructor is synthesized.</returns>
    /// <exception cref="ArgumentNullException">Null parameter names.</exception>
    /// <exception cref="ArgumentException">Invalid names, visibility, duplicate type or exceeded limits.</exception>
    public TypeBuilder AddGenericClass(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public)
        => AddGenericTypeCore(@namespace, name, genericParameterNames, visibility, false);
    private TypeBuilder AddGenericTypeCore(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility, bool isStatic, bool isInterface = false)
    {
        ArgumentNullException.ThrowIfNull(genericParameterNames);
        var names = new MethodSignature(PrimitiveType.Void, [], genericParameterNames).GenericParameterNames;
        if (string.IsNullOrEmpty(name) || name.Contains('`') || names.Count == 0) throw new ArgumentException("generic type requires a simple name and parameters");
        return AddTypeCore(@namespace, name + "`" + names.Count, visibility, isStatic, names, isInterface);
    }
    private TypeBuilder AddTypeCore(string @namespace, string name, TypeVisibility visibility, bool isStatic, IReadOnlyList<string>? genericNames = null, bool isInterface = false)
    {
        if (visibility is not (TypeVisibility.Public or TypeVisibility.Internal)) throw new ArgumentOutOfRangeException(nameof(visibility));
        if (@namespace is null || string.IsNullOrEmpty(name) || name == "<Module>" || @namespace.Length + name.Length > 1024 ||
            types.Count >= 256 || types.Any(t => t.Namespace == @namespace && t.Name == name)) throw new ArgumentException("invalid or duplicate type");
        var type = new TypeBuilder(this, @namespace, name, visibility, isStatic, genericNames, isInterface); types.Add(type); return type;
    }
    /// <summary>Validates all bodies and emits a fresh unsigned managed PE32 image.</summary>
    /// <returns>Owned PE bytes suitable for conventional readers and the supported neoCLR CLI import bridge.</returns>
    /// <exception cref="InvalidDataException">Invalid body, entry point, dependency identity or resource limit.</exception>
    /// <remarks>No image is loaded. Cross-assembly calls import exact assembly/type/member references. Repeated writes have a fixed MVID and timestamp for reproducibility.</remarks>
    public byte[] Write() => WriteImage(referenceOnly: false);

    internal byte[] WriteReferenceImage() => WriteImage(referenceOnly: true);

    private byte[] WriteImage(bool referenceOnly)
    {
        var methods = ValidateGraph(validateBodies: !referenceOnly);
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
        var typeHandles = types.Select((type, index) => (type, handle: MetadataTokens.TypeDefinitionHandle(index + 2))).ToDictionary(p => p.type, p => p.handle);
        var externalTypeHandles = new Dictionary<(AssemblyIdentity, string, string), TypeReferenceHandle>();
        EntityHandle ImportedTypeHandle(ImportedTypeReference type)
        {
            var key = (type.AssemblyIdentity, type.Namespace, type.Name);
            if (!externalTypeHandles.TryGetValue(key, out var handle))
            {
                handle = metadata.AddTypeReference(ImportAssembly(type.AssemblyIdentity), metadata.GetOrAddString(type.Namespace), metadata.GetOrAddString(type.Name));
                externalTypeHandles.Add(key, handle);
            }
            return handle;
        }
        var primitiveTokens = new Dictionary<PrimitiveType, TypeReferenceHandle>();
        var elementSpecs = new Dictionary<SignatureType, TypeSpecificationHandle>();
        int ElementToken(SignatureType type)
        {
            if (type.ImportedType is { TypeArguments.Count: > 0 } || type.GenericInstance is not null || type.TypeParameterIndex is not null || type.MethodParameterIndex is not null || type.ArrayElement is not null)
            {
                if (!elementSpecs.TryGetValue(type, out var spec))
                {
                    var blob = new BlobBuilder(); EncodeType(new BlobEncoder(blob).TypeSpecificationSignature(), type);
                    spec = metadata.AddTypeSpecification(metadata.GetOrAddBlob(blob)); elementSpecs.Add(type, spec);
                }
                return MetadataTokens.GetToken(spec);
            }
            if (type.ImportedType is { } imported) return MetadataTokens.GetToken(ImportedTypeHandle(imported));
            if (type.ClassType is { } owner) return MetadataTokens.GetToken(typeHandles[owner]);
            var primitive = type.Primitive!.Value;
            if (!primitiveTokens.TryGetValue(primitive, out var handle))
            {
                handle = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString(primitive.ToString()));
                primitiveTokens.Add(primitive, handle);
            }
            return MetadataTokens.GetToken(handle);
        }
        void EncodeType(SignatureTypeEncoder encoder, SignatureType type)
        {
            if (type.ImportedType is { } imported)
            {
                var handle = ImportedTypeHandle(imported);
                if (imported.TypeArguments.Count == 0) encoder.Type(handle, false);
                else
                {
                    var arguments = encoder.GenericInstantiation(handle, imported.TypeArguments.Count, false);
                    foreach (var argument in imported.TypeArguments) EncodeType(arguments.AddArgument(), argument);
                }
                return;
            }
            if (type.GenericInstance is { } instance)
            {
                var arguments = encoder.GenericInstantiation(typeHandles[instance.Definition], instance.TypeArguments.Count, false);
                foreach (var argument in instance.TypeArguments) EncodeType(arguments.AddArgument(), argument);
                return;
            }
            if (type.TypeParameterIndex is { } ordinal) { encoder.GenericTypeParameter(ordinal); return; }
            if (type.MethodParameterIndex is { } index) { encoder.GenericMethodTypeParameter(index); return; }
            if (type.ArrayElement is { } element) { EncodeType(encoder.SZArray(), element); return; }
            if (type.ClassType is { } owner) { encoder.Type(typeHandles[owner], false); return; }
            switch (type.Primitive)
            {
                case PrimitiveType.Int32: encoder.Int32(); break;
                case PrimitiveType.Int64: encoder.Int64(); break;
                case PrimitiveType.Boolean: encoder.Boolean(); break;
                case PrimitiveType.String: encoder.String(); break;
                default: throw new InvalidDataException("unsupported value type");
            }
        }
        BlobHandle Signature(MethodBuilder method)
        {
            var signature = new BlobBuilder();
            new BlobEncoder(signature).MethodSignature(SignatureCallingConvention.Default, method.Signature.GenericParameterNames.Count, !method.IsStatic).Parameters(method.ParameterCount,
                result =>
                {
                    if (!method.ReturnsValue) result.Void();
                    else EncodeType(result.Type(), method.Signature.ReturnType);
                }, parameters =>
                {
                    foreach (var type in method.Signature.ParameterTypes) EncodeType(parameters.AddParameter().Type(), type);

                });
            return metadata.GetOrAddBlob(signature);
        }
        var accessors = types.SelectMany(t => t.Properties).SelectMany(p => new[] { p.GetMethod, p.SetMethod }).OfType<MethodBuilder>().ToHashSet();
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
                handle = metadata.AddMemberReference(type, metadata.GetOrAddString(method.CliName), Signature(method));
                importedMethods.Add(method, handle);
            }
            return MetadataTokens.GetToken(handle);
        }
        var methodSpecs = new Dictionary<(MethodBuilder Definition, BlobHandle Arguments), MethodSpecificationHandle>();
        int GenericCallToken(GenericMethodInstance method)
        {
            var blob = new BlobBuilder();
            var arguments = new BlobEncoder(blob).MethodSpecificationSignature(method.TypeArguments.Count);
            foreach (var argument in method.TypeArguments) EncodeType(arguments.AddArgument(), argument);
            var key = (method.Definition, metadata.GetOrAddBlob(blob));
            if (!methodSpecs.TryGetValue(key, out var handle))
            {
                handle = metadata.AddMethodSpecification(MetadataTokens.EntityHandle(ImportMethod(method.Definition)), key.Item2);
                methodSpecs.Add(key, handle);
            }
            return MetadataTokens.GetToken(handle);
        }
        var ownerSpecs = new Dictionary<BlobHandle, TypeSpecificationHandle>();
        var ownerMembers = new Dictionary<(MethodBuilder Method, TypeSpecificationHandle Owner), MemberReferenceHandle>();
        var ownerMethodSpecs = new Dictionary<(MemberReferenceHandle Member, BlobHandle Arguments), MethodSpecificationHandle>();
        int ConstructedCallToken(ConstructedMethodReference reference)
        {
            var ownerBlob = new BlobBuilder();
            var arguments = new BlobEncoder(ownerBlob).TypeSpecificationSignature().GenericInstantiation(typeHandles[reference.Definition.DeclaringType!], reference.DeclaringTypeArguments.Count, false);
            foreach (var type in reference.DeclaringTypeArguments) EncodeType(arguments.AddArgument(), type);
            var ownerKey = metadata.GetOrAddBlob(ownerBlob);
            if (!ownerSpecs.TryGetValue(ownerKey, out var owner)) { owner = metadata.AddTypeSpecification(ownerKey); ownerSpecs.Add(ownerKey, owner); }
            var memberKey = (reference.Definition, owner);
            if (!ownerMembers.TryGetValue(memberKey, out var member))
            {
                member = metadata.AddMemberReference(owner, metadata.GetOrAddString(reference.Definition.CliName), Signature(reference.Definition));
                ownerMembers.Add(memberKey, member);
            }
            if (reference.MethodArguments.Count == 0) return MetadataTokens.GetToken(member);
            var methodBlob = new BlobBuilder(); var methodArguments = new BlobEncoder(methodBlob).MethodSpecificationSignature(reference.MethodArguments.Count);
            foreach (var type in reference.MethodArguments) EncodeType(methodArguments.AddArgument(), type);
            var methodKey = (member, metadata.GetOrAddBlob(methodBlob));
            if (!ownerMethodSpecs.TryGetValue(methodKey, out var spec)) { spec = metadata.AddMethodSpecification(member, methodKey.Item2); ownerMethodSpecs.Add(methodKey, spec); }
            return MetadataTokens.GetToken(spec);
        }
        var constructedFields = new Dictionary<(FieldBuilder, TypeSpecificationHandle), MemberReferenceHandle>();
        int ConstructedFieldToken(ConstructedFieldReference reference)
        {
            var owner = (TypeSpecificationHandle)MetadataTokens.EntityHandle(ElementToken(reference.DeclaringType));
            var key = (reference.Definition, owner);
            if (!constructedFields.TryGetValue(key, out var member))
            {
                var signature = new BlobBuilder();
                EncodeType(new BlobEncoder(signature).FieldSignature(), reference.Definition.FieldType);
                member = metadata.AddMemberReference(owner, metadata.GetOrAddString(reference.Definition.Name), metadata.GetOrAddBlob(signature));
                constructedFields.Add(key, member);
            }
            return MetadataTokens.GetToken(member);
        }
        var fieldHandles = types.SelectMany(t => t.Fields).Select((field, index) => (field, handle: MetadataTokens.FieldDefinitionHandle(index + 1))).ToDictionary(p => p.field, p => p.handle);
        var objectConstructor = methods.Any(m => m.IsConstructor) ? metadata.AddMemberReference(objectType, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(new byte[] { 0x20, 0, 1 })) : default;
        var bodies = new BlobBuilder();
        var bodyEncoder = new MethodBodyStreamEncoder(bodies);
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        int nextField = 1;
        int nextMethod = 1;
        var genericRows = new List<(EntityHandle Owner, int Sort, IReadOnlyList<string> Names)>();
        void EmitMethod(MethodBuilder method)
        {
            if (method.IsAbstract)
            {
                metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot | MethodAttributes.HideBySig | (accessors.Contains(method) ? MethodAttributes.SpecialName : 0),
                    MethodImplAttributes.IL | MethodImplAttributes.Managed, metadata.GetOrAddString(method.CliName), Signature(method), -1, MetadataTokens.ParameterHandle(1));
                nextMethod++;
                return;
            }
            var code = new BlobBuilder();
            var offsets = new int[method.Instructions.Count + 1];
            offsets[0] = method.IsConstructor ? 6 : 0;
            for (int i = 0; i < method.Instructions.Count; i++)
                offsets[i + 1] = offsets[i] + (method.Instructions[i].Op switch
                {
                    "constant64" => 9,
                    "label" => 0,
                    "array.new" or "array.load" or "array.store" or "string" or "constant" or "call" or "call.virtual" or "call.generic" or "call.constructed" or "new.object" or "new.constructed" or "field.load" or "field.store" or "branch" or "branch.true" or "branch.false" => 5,
                    "argument" or "argument.store" or "local.load" or "local.store" or "local.address" => 4,
                    "local.initialize" => 6,
                    "equal" or "less" or "greater" => 2,
                    _ => 1
                });
            var labels = method.LabelPositions();
            if (referenceOnly) { code.WriteByte(0x14); code.WriteByte(0x7a); } // ldnull; throw: never substitute native behavior.
            else if (method.IsConstructor)
            {
                code.WriteByte(0x02); // ldarg.0: initialize the sole supported CLI root base.
                code.WriteByte(0x28); code.WriteInt32(MetadataTokens.GetToken(objectConstructor));
            }
            foreach (var instruction in referenceOnly ? [] : method.Instructions)
            {
                switch (instruction.Op)
                {
                    case "label": break;
                    case "array.length": code.WriteByte(0x8e); break;
                    case "array.new":
                    case "array.load":
                    case "array.store":
                        code.WriteByte(instruction.Op == "array.new" ? (byte)0x8d : instruction.Op == "array.load" ? (byte)0xa3 : (byte)0xa4);
                        code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "duplicate": code.WriteByte(0x25); break;
                    case "new.object": code.WriteByte(0x73); code.WriteInt32(ImportMethod(instruction.Target!)); break;
                    case "field.load":
                    case "field.store":
                        code.WriteByte(instruction.Op == "field.load" ? (byte)0x7b : (byte)0x7d);
                        code.WriteInt32(instruction.ConstructedField is { } fieldReference ? ConstructedFieldToken(fieldReference) : MetadataTokens.GetToken(fieldHandles[instruction.Field!])); break;
                    case "negate": code.WriteByte(0x65); break;
                    case "complement": code.WriteByte(0x66); break;
                    case "pop": code.WriteByte(0x26); break;
                    case "boolean": code.WriteByte(instruction.Value == 0 ? (byte)0x16 : (byte)0x17); break;
                    case "equal": code.WriteByte(0xfe); code.WriteByte(0x01); break;
                    case "less": code.WriteByte(0xfe); code.WriteByte(0x04); break;
                    case "greater": code.WriteByte(0xfe); code.WriteByte(0x02); break;
                    case "branch":
                    case "branch.true":
                    case "branch.false":
                        code.WriteByte(instruction.Op == "branch" ? (byte)0x38 : instruction.Op == "branch.true" ? (byte)0x3a : (byte)0x39);
                        code.WriteInt32(offsets[labels[instruction.Value]] - (code.Count + 4)); break;
                    case "string": code.WriteByte(0x72); code.WriteInt32(MetadataTokens.GetToken(metadata.GetOrAddUserString(instruction.Text!))); break;
                    case "constant64": code.WriteByte(0x21); code.WriteInt64(instruction.LongValue); break;
                    case "convert64": code.WriteByte(0x6a); break;
                    case "convert32": code.WriteByte(0x69); break;
                    case "constant": code.WriteByte(0x20); code.WriteInt32(instruction.Value); break;
                    case "argument.store": code.WriteByte(0xfe); code.WriteByte(0x0b); code.WriteUInt16((ushort)instruction.Value); break;
                    case "argument": code.WriteByte(0xfe); code.WriteByte(0x09); code.WriteUInt16((ushort)instruction.Value); break;
                    case "local.address": code.WriteByte(0xfe); code.WriteByte(0x0d); code.WriteUInt16((ushort)instruction.Value); break;
                    case "local.initialize": code.WriteByte(0xfe); code.WriteByte(0x15); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "local.load": code.WriteByte(0xfe); code.WriteByte(0x0c); code.WriteUInt16((ushort)instruction.Value); break;
                    case "local.store": code.WriteByte(0xfe); code.WriteByte(0x0e); code.WriteUInt16((ushort)instruction.Value); break;
                    case "add": code.WriteByte(0x58); break;
                    case "subtract": code.WriteByte(0x59); break;
                    case "multiply": code.WriteByte(0x5a); break;
                    case "divide": code.WriteByte(0x5b); break;
                    case "remainder": code.WriteByte(0x5d); break;
                    case "and": code.WriteByte(0x5f); break;
                    case "or": code.WriteByte(0x60); break;
                    case "xor": code.WriteByte(0x61); break;
                    case "shift.left": code.WriteByte(0x62); break;
                    case "shift.right": code.WriteByte(0x63); break;
                    case "new.constructed": case "call.constructed": code.WriteByte(instruction.Op == "new.constructed" ? (byte)0x73 : (byte)0x28); code.WriteInt32(ConstructedCallToken(instruction.ConstructedTarget!)); break;
                    case "call.generic": code.WriteByte(0x28); code.WriteInt32(GenericCallToken(instruction.GenericTarget!)); break;
                    case "call.virtual": case "call": code.WriteByte(instruction.Op == "call.virtual" ? (byte)0x6f : (byte)0x28); code.WriteInt32(ImportMethod(instruction.Target!)); break;
                    case "return": code.WriteByte(0x2a); break;
                    default: throw new InvalidDataException("operation requires native emission: " + instruction.Op);
                }
            }
            StandaloneSignatureHandle locals = default;
            if (!referenceOnly && method.Locals.Count != 0)
            {
                var signature = new BlobBuilder();
                var variables = new BlobEncoder(signature).LocalVariableSignature(method.Locals.Count);
                foreach (var local in method.Locals) EncodeType(variables.AddVariable().Type(), local.SignatureType);
                locals = metadata.AddStandaloneSignature(metadata.GetOrAddBlob(signature));
            }
            int body = bodyEncoder.AddMethodBody(new InstructionEncoder(code), maxStack: referenceOnly ? 1 : Math.Max(method.IsConstructor ? 1 : 0, method.MaxStack),
                localVariablesSignature: locals, attributes: MethodBodyAttributes.InitLocals);
            metadata.AddMethodDefinition((method.Visibility switch { MethodVisibility.Internal => MethodAttributes.Assembly, MethodVisibility.Private => MethodAttributes.Private, _ => MethodAttributes.Public }) | (method.IsStatic ? MethodAttributes.Static : 0) | (method.IsConstructor ? MethodAttributes.SpecialName | MethodAttributes.RTSpecialName : accessors.Contains(method) ? MethodAttributes.SpecialName : 0) | (method.DeclaringType?.Implements(method) == true ? MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot : 0) | MethodAttributes.HideBySig,
                MethodImplAttributes.IL | MethodImplAttributes.Managed, metadata.GetOrAddString(method.CliName), Signature(method), body, MetadataTokens.ParameterHandle(1));
            genericRows.Add((handles[method], MetadataTokens.GetRowNumber(handles[method]) * 2 + 1, method.Signature.GenericParameterNames));
            nextMethod++;
        }
        foreach (var function in functions) EmitMethod(function);
        foreach (var type in types)
        {
            var typeHandle = metadata.AddTypeDefinition((type.Visibility == TypeVisibility.Public ? TypeAttributes.Public : TypeAttributes.NotPublic) | (type.IsInterface ? TypeAttributes.Interface | TypeAttributes.Abstract : type.IsStatic ? TypeAttributes.Abstract | TypeAttributes.Sealed : 0),
                metadata.GetOrAddString(type.Namespace), metadata.GetOrAddString(type.Name), type.IsInterface ? default(EntityHandle) : objectType,
                MetadataTokens.FieldDefinitionHandle(nextField), MetadataTokens.MethodDefinitionHandle(nextMethod));
            genericRows.Add((typeHandle, MetadataTokens.GetRowNumber(typeHandle) * 2, type.GenericParameterNames));
            foreach (var field in type.Fields)
            {
                var signature = new BlobBuilder();
                var encoder = new BlobEncoder(signature).FieldSignature();
                EncodeType(encoder, field.FieldType);
                metadata.AddFieldDefinition((field.Visibility switch
                {
                    FieldVisibility.Public => FieldAttributes.Public,
                    FieldVisibility.Internal => FieldAttributes.Assembly,
                    _ => FieldAttributes.Private
                }) | (field.IsReadOnly ? FieldAttributes.InitOnly : 0), metadata.GetOrAddString(field.Name), metadata.GetOrAddBlob(signature));
                nextField++;
            }
            foreach (var inherited in type.InterfaceContracts) metadata.AddInterfaceImplementation(typeHandle, typeHandles[inherited]);
            foreach (var method in type.Methods) EmitMethod(method);
            bool firstProperty = true;
            foreach (var property in type.Properties)
            {
                var signature = new BlobBuilder();
                new BlobEncoder(signature).PropertySignature(isInstanceProperty: !property.IsStatic)
                    .Parameters(property.ParameterTypes.Count, result =>
                    {
                        EncodeType(result.Type(), property.PropertyType);
                    }, parameters =>
                    {
                        foreach (var type in property.ParameterTypes) EncodeType(parameters.AddParameter().Type(), type);
                    });
                var handle = metadata.AddProperty(PropertyAttributes.None, metadata.GetOrAddString(property.Name), metadata.GetOrAddBlob(signature));
                if (firstProperty) { metadata.AddPropertyMap(typeHandle, handle); firstProperty = false; }
                if (property.GetMethod is { } getter) metadata.AddMethodSemantics(handle, MethodSemanticsAttributes.Getter, handles[getter]);
                if (property.SetMethod is { } setter) metadata.AddMethodSemantics(handle, MethodSemanticsAttributes.Setter, handles[setter]);
            }
        }
        foreach (var row in genericRows.OrderBy(r => r.Sort))
            for (int i = 0; i < row.Names.Count; i++)
            {
                var attributes = row.Owner.Kind == HandleKind.TypeDefinition ? (GenericParameterAttributes)types[MetadataTokens.GetRowNumber(row.Owner) - 2].SpecialConstraints.GetValueOrDefault(i) : GenericParameterAttributes.None;
                var parameter = metadata.AddGenericParameter(row.Owner, attributes, metadata.GetOrAddString(row.Names[i]), i);
                if (row.Owner.Kind == HandleKind.TypeDefinition)
                {
                    var owner = types[MetadataTokens.GetRowNumber(row.Owner) - 2];
                    foreach (var constraint in owner.GenericConstraints.Where(c => c.ParameterIndex == i))
                        metadata.AddGenericParameterConstraint(parameter, typeHandles[constraint.BaseType]);
                }
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

/// <summary>Supported visibility for top-level types.</summary>
public enum TypeVisibility
{
    /// <summary>Accessible outside the defining assembly.</summary>
    Public,
    /// <summary>Accessible only within the defining assembly.</summary>
    Internal
}

/// <summary>An editable top-level static or root reference class owned by an AssemblyBuilder.</summary>
public sealed partial class TypeBuilder
{
    private readonly List<MethodBuilder> methods = [];
    internal TypeBuilder(AssemblyBuilder assembly, string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public, bool isStatic = true, IReadOnlyList<string>? genericNames = null, bool isInterface = false) { IsInterface = isInterface; GenericParameterNames = genericNames ?? Array.Empty<string>(); Assembly = assembly; Namespace = @namespace; Name = name; Visibility = visibility; IsStatic = isStatic; }
    /// <summary>Gets immutable declaring-type parameter names in ordinal order.</summary>
    public IReadOnlyList<string> GenericParameterNames { get; }
    /// <summary>Gets whether this is an abstract sealed static class.</summary>
    public bool IsStatic { get; }
    /// <summary>Gets the declared top-level visibility.</summary>
    public TypeVisibility Visibility { get; }
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
        => AddMethod(name, PrimitiveMethodSignature.Int32(parameterCount, returnsValue));
    /// <summary>Adds a public static method with an explicit primitive or owned-class signature.</summary>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="signature">Primitive or owned root-class parameters/results; Void is allowed only as the result.</param>
    /// <returns>A method builder owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Invalid/duplicate name and parameter types or method limit exceeded.</exception>
    public MethodBuilder AddMethod(string name, MethodSignature signature)
        => AddMethod(name, signature, MethodVisibility.Public);
    /// <summary>Adds a static primitive method with explicit visibility.</summary>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="signature">The primitive/owned-class parameter and result contract.</param>
    /// <param name="visibility">Public, internal or private access.</param>
    /// <returns>A method owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Invalid visibility/name, duplicate signature or method limit exceeded.</exception>
    public MethodBuilder AddMethod(string name, MethodSignature signature, MethodVisibility visibility)
        => AddMethodCore(name, signature, visibility, isStatic: true, constructor: false);
    /// <summary>Adds a nonvirtual instance method with primitive or owned-class parameters/results.</summary>
    /// <param name="name">Nonempty simple name; .ctor/.cctor are reserved.</param>
    /// <param name="signature">Primitive/owned-class declared parameters, excluding the receiver.</param>
    /// <param name="visibility">Public, Internal or Private.</param>
    /// <returns>An owned method whose argument zero is the declaring-class receiver.</returns>
    /// <exception cref="ArgumentException">Invalid or duplicate contract or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    public MethodBuilder AddInstanceMethod(string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
        => AddMethodCore(name, signature, visibility, isStatic: false, constructor: false);
    /// <summary>Adds a root-class constructor with primitive declared parameters and no result.</summary>
    /// <param name="parameterTypes">At most 256 primitive parameters, excluding the receiver.</param>
    /// <param name="visibility">Public, Internal or Private.</param>
    /// <returns>An owned .ctor body with receiver at argument zero.</returns>
    /// <exception cref="ArgumentException">Invalid/duplicate contract or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    /// <remarks>CLI emission initializes System.Object before this body. Native root construction requires no base call. Constructor chaining is unsupported.</remarks>
    public MethodBuilder AddConstructor(IEnumerable<PrimitiveType> parameterTypes, MethodVisibility visibility = MethodVisibility.Public)
        => AddMethodCore(".ctor", new(PrimitiveType.Void, parameterTypes), visibility, isStatic: false, constructor: true);
    /// <summary>Adds a constructor with an owned nominal/primitive signature whose result must be Void.</summary>
    /// <param name="signature">Void result and up to 256 primitive/owned-class declared parameters.</param>
    /// <param name="visibility">Public, Internal or Private.</param>
    /// <returns>A constructor owned by this root class.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Non-Void result, foreign class, duplicate signature or invalid visibility.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    public MethodBuilder AddConstructor(MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (signature.GenericParameterNames.Count != 0) throw new ArgumentException("generic constructors unsupported", nameof(signature));
        if (signature.ReturnType != PrimitiveType.Void) throw new ArgumentException("constructor must have no result", nameof(signature));
        return AddMethodCore(".ctor", signature, visibility, false, true);
    }
    private MethodBuilder AddMethodCore(string name, MethodSignature signature, MethodVisibility visibility, bool isStatic, bool constructor, bool abstractContract = false)
    {
        if (IsInterface != abstractContract) throw new InvalidOperationException("interface owners require abstract contract methods");
        if (!isStatic && IsStatic) throw new InvalidOperationException("instance methods require a reference class");
        if (!constructor && name is ".ctor" or ".cctor") throw new ArgumentException("reserved constructor name", nameof(name));
        if (visibility is not (MethodVisibility.Public or MethodVisibility.Internal or MethodVisibility.Private)) throw new ArgumentOutOfRangeException(nameof(visibility));
        ArgumentNullException.ThrowIfNull(signature);
        signature.ValidateOwner(Assembly, GenericParameterNames.Count);
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || methods.Count >= 256 ||
            methods.Any(m => m.Name == name && m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("invalid or duplicate method");
        var method = new MethodBuilder(Assembly, this, name, signature, visibility, isStatic: isStatic); methods.Add(method); return method;
    }
}

/// <summary>Supported method access scopes.</summary>
public enum MethodVisibility
{
    /// <summary>Accessible outside the declaring assembly, subject to owner visibility.</summary>
    Public,
    /// <summary>Accessible within the declaring assembly.</summary>
    Internal,
    /// <summary>Accessible only within the declaring type.</summary>
    Private
}

/// <summary>Typed Int32/Int64/Boolean/String body construction; invalid control-flow contracts fail before emission.</summary>
public sealed partial class MethodBuilder
{
    internal sealed record Operation(string Op, int Value = 0, MethodBuilder? Target = null, string? Text = null, NativeFunctionDefinition? NativeTarget = null, long LongValue = 0, FieldBuilder? Field = null, SignatureType? Type = null, GenericMethodInstance? GenericTarget = null, ConstructedMethodReference? ConstructedTarget = null, ConstructedFieldReference? ConstructedField = null);
    internal List<Operation> Instructions { get; } = [];
    internal int MaxStack { get; private set; }
    internal MethodBuilder(AssemblyBuilder assembly, TypeBuilder? owner, string name, int count, bool result)
        : this(assembly, owner, name, PrimitiveMethodSignature.Int32(count, result)) { }
    internal MethodBuilder(AssemblyBuilder assembly, TypeBuilder? owner, string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public, string @namespace = "", bool isStatic = true)
    { Assembly = assembly; DeclaringType = owner; Name = name; Signature = signature; Visibility = visibility; Namespace = owner?.Namespace ?? @namespace; IsStatic = isStatic; }
    /// <summary>Gets whether the signature excludes an instance receiver.</summary>
    public bool IsStatic { get; }
    /// <summary>Gets whether this is an instance .ctor with no result.</summary>
    public bool IsConstructor => !IsStatic && Name == ".ctor";
    internal int ArgumentCount => ParameterCount + (IsStatic ? 0 : 1);
    /// <summary>Gets the function namespace or the declaring type namespace; empty for the global namespace.</summary>
    public string Namespace { get; }
    internal string CliName => DeclaringType is null ? FunctionNamespaceEncoding.Encode(Namespace, Name) : Name;
    /// <summary>Gets declared method or assembly-function visibility.</summary>
    public MethodVisibility Visibility { get; }
    /// <summary>Gets the immutable primitive/owned-class method signature.</summary>
    public MethodSignature Signature { get; }
    /// <summary>Gets the owning assembly, including for top-level functions.</summary>
    public AssemblyBuilder Assembly { get; }
    /// <summary>Gets the declaring type, or null for a top-level function.</summary>
    public TypeBuilder? DeclaringType { get; }
    /// <summary>Gets the method name.</summary>
    public string Name { get; }
    /// <summary>Gets the parameter count.</summary>
    public int ParameterCount => Signature.ParameterTypes.Count;
    /// <summary>Gets whether the method has a primitive or owned-class result rather than no result.</summary>
    public bool ReturnsValue => Signature.ReturnType != PrimitiveType.Void;
    /// <summary>Appends an Int32 constant.</summary>
    /// <param name="value">Constant value.</param>
    public void LoadConstant(int value) => Emit(OpCode.Ldc_I4, value);
    /// <summary>Appends a native System.Console.WriteLine call with a constant UTF-8 string.</summary>
    /// <param name="text">Unicode text, at most 64 KiB when UTF-8 encoded.</param>
    /// <exception cref="ArgumentNullException">Text is null.</exception>
    /// <exception cref="ArgumentException">Invalid Unicode or text exceeds the limit.</exception>
    /// <remarks>Native emission only; ordinary CLI Write rejects this operation. Does not alter the surrounding primitive stack.</remarks>
    public void WriteConsoleLine(string text)
    {
        ValidateLiteral(text);
        Append(new("console.line", Text: text));
    }
    /// <summary>Consumes a String stack value and writes it through native System.Console.WriteLine.</summary>
    /// <remarks>Native-only bootstrap; discards bundled System's inhabited Void result. Ordinary CLI output rejects this operation.</remarks>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or stack mismatch when writing.</exception>
    public void WriteConsoleLine() => Append(new("console.write"));

    private static void ValidateLiteral(string text)
    {
        ArgumentNullException.ThrowIfNull(text);
        try
        {
            if (new System.Text.UTF8Encoding(false, true).GetByteCount(text) > 65536)
                throw new ArgumentException("string literal exceeds 64 KiB", nameof(text));
        }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid Unicode", nameof(text), error); }
    }
    /// <summary>Appends a parameter load; bounds are checked at Write.</summary>
    /// <param name="index">Argument slot index; instance receiver is zero and declared parameters start at one.</param>
    public void LoadArgument(int index) => Emit(OpCode.Ldarg, index);
    /// <summary>Stores a value into a by-value argument slot in this invocation.</summary>
    /// <param name="index">Argument slot index; instance declared parameters start at one. Bounds and exact type are checked when writing.</param>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or invalid index/stack type when writing.</exception>
    /// <remarks>Does not update caller storage. Receiver stores and by-reference parameters are unsupported.</remarks>
    public void StoreArgument(int index) => Emit(OpCode.Starg, index);
    /// <summary>Appends matching-width Int32/Int64 addition.</summary>
    public void Add() => Emit(OpCode.Add);
    /// <summary>Appends matching-width Int32/Int64 subtraction.</summary>
    public void Subtract() => Emit(OpCode.Sub);
    /// <summary>Appends matching-width Int32/Int64 multiplication.</summary>
    public void Multiply() => Emit(OpCode.Mul);
    /// <summary>Appends matching-width signed Int32/Int64 division, truncating toward zero.</summary>
    /// <remarks>Zero and minimum-value divided by -1 fault at execution, not when writing.</remarks>
    public void Divide() => Emit(OpCode.Div);
    /// <summary>Appends matching-width signed Int32/Int64 remainder, with the dividend's sign.</summary>
    /// <remarks>Zero faults at execution, not when writing. Native minimum/-1 faults; CLI follows the host CLR edge behavior.</remarks>
    public void Remainder() => Emit(OpCode.Rem);
    /// <summary>Appends bitwise AND of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseAnd() => Emit(OpCode.And);
    /// <summary>Appends bitwise OR of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseOr() => Emit(OpCode.Or);
    /// <summary>Appends bitwise XOR of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseXor() => Emit(OpCode.Xor);
    /// <summary>Appends an Int32/Int64 left shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    public void ShiftLeft() => Emit(OpCode.Shl);
    /// <summary>Appends a sign-extending Int32/Int64 right shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    public void ShiftRight() => Emit(OpCode.Shr);
    /// <summary>Appends a call; foreign methods are imported during Write.</summary>
    /// <param name="target">Local or external builder method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    public void Call(MethodBuilder target) => Emit(OpCode.Call, target);
    /// <summary>Appends a call to an imported read-only method contract.</summary>
    /// <param name="target">Reference imported by this method's assembly builder.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Reference belongs to another output builder.</exception>
    public void Call(ImportedMethodReference target) => Emit(OpCode.Call, target);
    /// <summary>Calls a static Int32 function selected from an explicitly loaded native System inventory.</summary>
    /// <param name="target">Owned System function whose parameters and result are Int32.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="InvalidDataException">Module is not System or the callable signature is unsupported.</exception>
    /// <remarks>Native-only bootstrap. The host must supply the matching System assembly to neoCLR;
    /// no assembly revision or image digest is encoded. Ordinary CLI output rejects this operation.</remarks>
    public void Call(NativeFunctionDefinition target) => Emit(OpCode.Call, target);
    /// <summary>Appends return with the declared stack shape; no values may remain afterward.</summary>
    public void Return() => Emit(OpCode.Ret);
    /// <summary>Clears instructions for editing before another Write; local declarations and handles are retained.</summary>
    public void ClearBody() => Instructions.Clear();
    private void Append(Operation operation)
    {
        if (Instructions.Count >= 4096) throw new InvalidDataException("method instruction limit exceeded");
        Instructions.Add(operation);
    }
}
