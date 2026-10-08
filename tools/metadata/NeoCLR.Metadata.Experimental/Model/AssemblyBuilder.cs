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
        Identity = identity; CoreLibrary = coreLibrary; Definition = AssemblyDefinition.ForProducer(this, mvid);
    }
    /// <summary>Gets the canonical authored assembly definition for type/field declarations.</summary>
    public AssemblyDefinition Definition { get; }
    /// <summary>Gets the output identity.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Gets the explicit core-library identity.</summary>
    public AssemblyIdentity CoreLibrary { get; }
    /// <summary>Gets the output-owned System.Object reference in the explicit core assembly.</summary>
    /// <remarks>No assembly is loaded. Native boxing additionally requires an explicit matching System bootstrap binding.</remarks>
    public ImportedTypeReference CoreObjectType => ImportTypeIdentity(CoreLibrary, "System", "Object", 0);
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
        var definition = new MethodDefinition(name, signature, visibility, @namespace);
        Definition.MainModule.Functions.Add(definition);
        return definition.Producer!;
    }

    internal MethodBuilder[] ValidateGraph(bool validateBodies = true)
    {
        var methods = functions.Concat(types.SelectMany(t => t.Methods)).ToArray();
        if (methods.Sum(method => (long)method.Instructions.Count) > 131072) throw new InvalidDataException("assembly instruction limit exceeded");
        if (methods.SelectMany(method => method.Instructions).Sum(instruction =>
            instruction.Text is null ? 0L : System.Text.Encoding.UTF8.GetByteCount(instruction.Text)) > MetadataArtifactReader.MaxImageSize)
            throw new InvalidDataException("assembly string literal limit exceeded");
        if (types.Sum(type => type.Properties.Count) > 4096) throw new InvalidDataException("too many properties");
        if (types.Sum(type => type.MetadataFields.Count()) > 4096) throw new InvalidDataException("too many fields");
        if (methods.Length > 4096) throw new InvalidDataException("too many methods");
        if (EntryPoint is not null && (!methods.Contains(EntryPoint) || EntryPoint.Definition.ImplementationAttributes != 0 || !EntryPoint.IsStatic || EntryPoint.Signature.GenericParameterNames.Count != 0 || EntryPoint.DeclaringType?.GenericParameterNames.Count > 0 || (EntryPoint.ParameterCount != 0 && EntryPoint.Signature.ParameterTypes is not [{ ArrayElement.Primitive: PrimitiveType.String }]) || EntryPoint.Signature.ReturnType.Primitive is not (PrimitiveType.Int32 or PrimitiveType.Void)))
            throw new InvalidDataException("entry point must be a local Int32 or no-result method with no parameters or one String vector");
        var importedTargets = importedReferences.Values.Concat(authoredCallableReferences).Select(reference => reference.Target).ToHashSet();
        foreach (var target in methods.SelectMany(m => m.Instructions).Select(i => i.Target).OfType<MethodBuilder>())
            if (!ReferenceEquals(target.Assembly, this) && !importedTargets.Contains(target) && target.Signature.ParameterTypes.Append(target.Signature.ReturnType).Any(t => t.Primitive is null && t.ArrayElement?.Primitive is null && t.MethodParameterIndex is null && t.ArrayElement?.MethodParameterIndex is null))
                throw new InvalidDataException("external nominal method references require an import contract");
        try
        {
            foreach (var type in types)
            {
                type.ValidateEnum(); type.ValidatePrimitiveRepresentation(); type.ValidateClassSlots();
                if (type.Definition.IsNativeObjectRoot) type.Definition.ValidateNativeObjectRoot();
                if (type.IsClosedHierarchy && (!(IsOrdinaryBase(type) || type.IsInterface && type.GenericParameterNames.Count == 0 && type.Definition.DeclaringType is null) || !type.IsAbstract))
                    throw new InvalidDataException("closed families require nongeneric top-level abstract class or interface owners");
                if (type.LocalBase is { } parent && (!(IsOrdinaryBase(type) || parent.IsNativeObjectRoot && !type.IsStatic && !type.IsInterface && !type.IsValueType && type.Definition.DeclaringType is null) || !IsOrdinaryBase(parent) || (parent.Definition.Attributes & 0x100) != 0))
                    throw new InvalidDataException("derived classes require ordinary nongeneric reference owners");
            }
            ValidateValueLayouts();
            foreach (var type in types)
                foreach (var attribute in type.Definition.CustomAttributes) attribute.ValidateContract(type.Definition);
            foreach (var type in types)
                foreach (var contract in type.InheritedContracts()) { _ = contract; }
            foreach (var type in types.Where(t => !t.IsInterface))
                foreach (var contract in type.RequiredInterfaceMethods)
                    if (type.FindInterfaceImplementation(contract.Name, contract.Signature, contract.IsStatic, contract.Owner) is null)
                        throw new InvalidDataException("missing public interface implementation: " + type.Namespace + "." + type.Name + "." + contract.Name);
            foreach (var type in types) type.ValidateExplicitImplementations();
            foreach (var type in types)
                foreach (var contract in type.InterfaceSignatures)
                {
                    contract.ValidateOwner(this, typeArity: type.GenericParameterNames.Count, complete: true, allowSelf: type.IsInterface);
                }
            foreach (var type in types)
                foreach (var field in type.Fields) field.FieldType.ValidateOwner(this, typeArity: type.GenericParameterNames.Count, complete: true);
            foreach (var method in methods)
            {
                int arity = method.DeclaringType?.GenericParameterNames.Count ?? 0;
                if (method.IsImplicitObjectOverride && !MethodDefinition.IsObjectOverride(method.Name, method.Signature, CoreLibrary, NativeObjectRoot, ExternalObjectRoot))
                    throw new InvalidDataException("Object override does not match the selected root signature");
                method.Signature.ValidateOwner(this, arity, complete: true, allowSelf: method.DeclaringType?.IsInterface == true);
                foreach (var bound in method.InterfaceConstraints)
                {
                    var signature = method.Definition.ConstraintSignature(bound.InterfaceType);
                    if (signature.ImportedType is { } external) _ = ExternalInterfaceMethods(external).ToArray();
                }
                foreach (var local in method.Locals) local.SignatureType.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                foreach (var instruction in method.Instructions)
                {
                    if (instruction.Op is "new.object" or "new.constructed" && instruction.Target?.DeclaringType?.IsAbstract == true)
                        throw new InvalidDataException("abstract classes cannot be constructed");
                    if ((instruction.Target ?? instruction.ConstructedTarget?.Definition ?? instruction.GenericTarget?.Definition) is { Visibility: MethodVisibility.Protected } familyTarget &&
                        (method.DeclaringType is not { } callerType ||
                         !ReferenceEquals(callerType, familyTarget.DeclaringType) && !callerType.DerivesFrom(familyTarget.DeclaringType!)))
                        throw new InvalidDataException("protected constructor requires a declaring-family caller");
                    if ((instruction.Target ?? instruction.ConstructedTarget?.Definition ?? instruction.GenericTarget?.Definition)?.IsAbstract == true && instruction.Op is not ("call.virtual" or "call.virtual.constructed" or "call.constrained" or "function.bind"))
                        throw new InvalidDataException("interface dispatch requires a supported virtual-call contract");
                    if (instruction.ConstrainedOwner is { } implementingType)
                        MethodILGenerator.ValidateConstrainedOperands(this, implementingType, instruction.Target!);
                    else if (instruction.ConstrainedConstructedReference is { } constructedConstrained)
                        MethodILGenerator.ValidateExternalConstructedConstrainedOperands(method, instruction.Type!, constructedConstrained);
                    else if (instruction.ConstrainedReference is { } externalConstrained)
                        MethodILGenerator.ValidateExternalConstrainedOperands(method, instruction.Type!, externalConstrained);
                    else if (instruction.Op == "call.constrained")
                        MethodILGenerator.ValidateOpenConstrainedOperands(method, instruction.Type!, instruction.Target!);
                    if (instruction.ConstructedTarget is { } target)
                    {
                        target.Definition.DeclaringType!.ValidateTypeArguments(target.DeclaringTypeArguments, complete: true);
                        target.Definition.ValidateMethodArguments(target.MethodArguments, method);
                        foreach (var argument in target.DeclaringTypeArguments.Concat(target.MethodArguments)) argument.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    }
                    if (instruction.GenericTarget is { } generic)
                    {
                        generic.Definition.ValidateMethodArguments(generic.TypeArguments, method);
                        foreach (var argument in generic.TypeArguments) argument.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    }
                    if (instruction.ConstructedField is { } field) ((SignatureType)field.DeclaringType).ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                    instruction.Type?.ValidateOwner(this, method.Signature.GenericParameterNames.Count, arity, complete: true);
                }
                if (method.Definition.ImplementationAttributes == 0x1000)
                {
                    if (method.DeclaringType is not null || method.Signature.GenericParameterNames.Count != 0 ||
                        method.Instructions.Count != 0 || method.Locals.Count != 0 || method.Definition.Body.Labels.Count != 0)
                        throw new InvalidDataException("internal calls require bodyless nongeneric assembly functions");
                }
                else if (method.IsAbstract)
                {
                    if (method.Instructions.Count != 0 || method.Locals.Count != 0) throw new InvalidDataException("abstract interface methods must have no body or locals");
                }
                else if (validateBodies)
                {
                    try { method.Validate(); }
                    catch (InvalidDataException error)
                    {
                        throw new InvalidDataException($"{method.DeclaringType?.Name ?? method.Namespace}.{method.Name}: {error.Message}", error);
                    }
                }
            }
        }
        catch (ArgumentException error) { throw new InvalidDataException("invalid generic argument contract", error); }
        return methods;
    }
    /// <summary>Gets or sets a local Int32 or no-result entry point with no parameters or one String vector; null writes a library.</summary>
    public MethodBuilder? EntryPoint { get; set; }
    /// <summary>Adds a unique public static class.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <returns>A type owned by this builder.</returns>
    /// <exception cref="ArgumentException">Null/invalid names, duplicate type or more than 4095 declared types.</exception>
    public TypeBuilder AddType(string @namespace, string name) => AddType(@namespace, name, TypeVisibility.Public);
    /// <summary>Adds a unique top-level static class with explicit public or assembly visibility.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="visibility">Public or internal visibility.</param>
    /// <returns>A type owned by this builder.</returns>
    /// <exception cref="ArgumentException">Invalid visibility/name, duplicate type or more than 4095 declared types.</exception>
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
    /// <summary>Adds an abstract root class whose direct family belongs to this output.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Nonempty unique metadata name.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An attached native closed-family definition.</returns>
    /// <exception cref="ArgumentException">Invalid visibility, name, duplicate type or exceeded limit.</exception>
    /// <remarks>Use protected constructors for derived initialization. Native PE emission preserves closure; ordinary CLI executable emission rejects it explicitly.</remarks>
    public TypeBuilder AddClosedClass(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
    {
        if (!Enum.IsDefined(visibility)) throw new ArgumentException("invalid visibility", nameof(visibility));
        var definition = new TypeDefinition(@namespace, name, (visibility == TypeVisibility.Public ? 1u : 0u) | 0x80,
            Definition.MainModule.ImportReference(CoreLibrary, "System", "Object"), true);
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
    /// <summary>Adds an abstract closed family derived from an owned ordinary class.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Nonempty unique metadata name.</param>
    /// <param name="baseType">Already attached nongeneric reference base, including the native Object root.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>The attached closed-family definition.</returns>
    /// <exception cref="ArgumentNullException">The base is null.</exception>
    /// <exception cref="ArgumentException">Invalid base, visibility, name or duplicate type.</exception>
    /// <remarks>Uses the same validation as manually attached definitions. Constructors must initialize the direct base; closure constrains this family's direct children, not its base.</remarks>
    public TypeBuilder AddClosedClass(string @namespace, string name, TypeBuilder baseType, TypeVisibility visibility = TypeVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!Enum.IsDefined(visibility)) throw new ArgumentException("invalid visibility", nameof(visibility));
        var definition = new TypeDefinition(@namespace, name, (visibility == TypeVisibility.Public ? 1u : 0u) | 0x80,
            baseType.Definition.ToReference(), true);
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
    /// <summary>Adds a nongeneric interface whose direct implementations and derived interfaces belong to this output.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Nonempty unique metadata name.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An attached native closed-family interface.</returns>
    /// <exception cref="ArgumentException">Invalid visibility/name, duplicate type or exceeded limit.</exception>
    /// <remarks>CLI flags remain Interface and Abstract. Closure is native metadata, enforced when dependencies are linked; executable CLI emission rejects it.</remarks>
    public TypeBuilder AddClosedInterface(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
    {
        if (!Enum.IsDefined(visibility)) throw new ArgumentException("invalid visibility", nameof(visibility));
        var definition = new TypeDefinition(@namespace, name, (visibility == TypeVisibility.Public ? 1u : 0u) | 0xa0, null, true);
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
    /// <summary>Adds a nongeneric reference class derived from an owned ordinary class.</summary>
    /// <param name="namespace">Metadata namespace, possibly empty.</param>
    /// <param name="name">Nonempty unique metadata name.</param>
    /// <param name="baseType">Already attached ordinary nongeneric base in this output.</param>
    /// <param name="visibility">Public or Internal visibility.</param>
    /// <returns>The attached derived type builder.</returns>
    /// <exception cref="ArgumentNullException">The base is null.</exception>
    /// <remarks>The base must already be attached. Constructors explicitly call a direct base constructor through their IL generator.</remarks>
    /// <exception cref="ArgumentException">The base is foreign, generic, nested, static, an interface or a value type.</exception>
    public TypeBuilder AddClass(string @namespace, string name, TypeBuilder baseType, TypeVisibility visibility = TypeVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!Enum.IsDefined(visibility)) throw new ArgumentException("invalid visibility", nameof(visibility));
        var definition = new TypeDefinition(@namespace, name, visibility == TypeVisibility.Public ? 1u : 0u, baseType.Definition.ToReference());
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
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
    /// <summary>Adds a generic reference class deriving from this output's explicit native Object root.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Simple name without CLI arity.</param>
    /// <param name="genericParameterNames">One through 32 unique names.</param>
    /// <param name="baseType">The attached native Object root in this output.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An attached generic class; constructors must explicitly initialize the base.</returns>
    /// <exception cref="ArgumentException">Invalid owner, visibility, names or duplicate declaration.</exception>
    /// <exception cref="ArgumentNullException">Base or parameter sequence is null.</exception>
    public TypeBuilder AddGenericClass(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeBuilder baseType, TypeVisibility visibility = TypeVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!ReferenceEquals(baseType.Assembly, this) || !baseType.IsNativeObjectRoot || !Enum.IsDefined(visibility))
            throw new ArgumentException("generic local bases require this output's native Object root");
        var definition = new TypeDefinition(@namespace, name, visibility == TypeVisibility.Public ? 1u : 0u, baseType.Definition.ToReference(), genericParameterNames);
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }

    private TypeBuilder AddGenericTypeCore(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility, bool isStatic, bool isInterface = false, bool isValueType = false)
    {
        ArgumentNullException.ThrowIfNull(genericParameterNames);
        var names = new MethodSignature(PrimitiveType.Void, [], genericParameterNames).GenericParameterNames;
        if (string.IsNullOrEmpty(name) || name.Contains('`') || names.Count == 0) throw new ArgumentException("generic type requires a simple name and parameters");
        return AddTypeCore(@namespace, name + "`" + names.Count, visibility, isStatic, names, isInterface, isValueType);
    }
    private TypeBuilder AddTypeCore(string @namespace, string name, TypeVisibility visibility, bool isStatic, IReadOnlyList<string>? genericNames = null, bool isInterface = false, bool isValueType = false)
    {
        if (visibility is not (TypeVisibility.Public or TypeVisibility.Internal)) throw new ArgumentOutOfRangeException(nameof(visibility));
        if (@namespace is null || string.IsNullOrEmpty(name) || name == "<Module>" || @namespace.Length + name.Length > 1024 ||
            types.Count >= DefinitionLimits.AuthoredTypes || types.Any(t => t.Definition.DeclaringType is null && t.Namespace == @namespace && t.Name == name)) throw new ArgumentException("invalid or duplicate type");
        var type = new TypeBuilder(this, @namespace, name, visibility, isStatic, genericNames, isInterface, isValueType); Definition.MainModule.Types.Add(type.Definition); return type;
    }
    /// <summary>Validates all bodies and emits a fresh unsigned managed PE32 image.</summary>
    /// <returns>Owned PE bytes suitable for conventional readers and the supported neoCLR CLI import bridge.</returns>
    /// <exception cref="InvalidDataException">Invalid body, entry point, dependency identity or resource limit.</exception>
    /// <remarks>No image is loaded. Cross-assembly calls import exact assembly/type/member references. Repeated writes have a fixed MVID and timestamp for reproducibility.</remarks>
    public byte[] Write() => WriteImage(referenceOnly: false);

    internal byte[] WriteReferenceImage() => WriteImage(referenceOnly: true);

    private byte[] WriteImage(bool referenceOnly)
    {
        if (!referenceOnly && assemblyConstants.Count != 0)
            throw new InvalidDataException("assembly-level constants require native emission");
        var methods = ValidateGraph(validateBodies: !referenceOnly);
        if (!referenceOnly && (ExternalObjectRoot is not null || types.Any(t => t.Definition.IsNativeObjectRoot)))
            throw new InvalidDataException("native Object root requires native emission; CLI projection is reference-only");
        if (!referenceOnly && types.Any(t => t.IsClosedHierarchy))
            throw new InvalidDataException("closed class families require native emission; CLI closed-family attributes are not authored");
        if (!referenceOnly && (authoredPrimitiveOwners.Count != 0 || externalGrapheme is not null || types.Any(t => t.NativePrimitive is not null || t.NativeGrapheme) ||
            methods.SelectMany(m => m.Instructions).Any(i => i.Target?.DeclaringType?.NativePrimitive is not null)))
            throw new InvalidDataException("native primitive implementations require native emission");
        if (!referenceOnly && methods.Any(m => m.Instructions.Any(i => i.Op == "array.reserve")))
            throw new InvalidDataException("checked uninitialized array reservation requires native emission; executable CLI has no equivalent operation");
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
        var enumBase = types.Any(t => t.IsEnum) ? metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString("Enum")) : default;
        var valueBase = types.Any(t => t.IsValueType) ? metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString("ValueType")) : default;
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
        var externalTypeHandles = new Dictionary<(AssemblyIdentity, string, string, ImportedTypeReference?), TypeReferenceHandle>();
        EntityHandle ImportedTypeHandle(ImportedTypeReference type)
        {
            var key = (type.AssemblyIdentity, type.Namespace, type.Name, type.DeclaringType);
            if (!externalTypeHandles.TryGetValue(key, out var handle))
            {
                handle = metadata.AddTypeReference(type.DeclaringType is { } parent ? ImportedTypeHandle(parent) : ImportAssembly(type.AssemblyIdentity), metadata.GetOrAddString(type.Namespace), metadata.GetOrAddString(type.Name));
                externalTypeHandles.Add(key, handle);
            }
            return handle;
        }
        TypeReferenceHandle ImportOwner(TypeBuilder owner)
        {
            if (!importedTypes.TryGetValue(owner, out var handle))
            {
                handle = metadata.AddTypeReference(owner.Definition.DeclaringType is { } parent ? ImportOwner(parent.Producer!) : ImportAssembly(owner.Assembly.Identity),
                    metadata.GetOrAddString(owner.Namespace), metadata.GetOrAddString(owner.Name));
                importedTypes.Add(owner, handle);
            }
            return handle;
        }
        var functionCarriers = new Dictionary<(int, bool), TypeReferenceHandle>();
        TypeReferenceHandle FunctionCarrier(FunctionSignature shape)
        {
            int arity = shape.ParameterTypes.Count + (shape.NoResult ? 0 : 1);
            var key = (arity, shape.NoResult);
            if (!functionCarriers.TryGetValue(key, out var handle))
            {
                var name = (shape.NoResult ? "Action" : "Func") + (arity == 0 ? "" : "`" + arity);
                handle = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString(name));
                functionCarriers.Add(key, handle);
            }
            return handle;
        }
        var primitiveTokens = new Dictionary<PrimitiveType, TypeReferenceHandle>();
        var elementSpecs = new Dictionary<SignatureType, TypeSpecificationHandle>();
        int ElementToken(SignatureType type)
        {
            if (type.FunctionSignature is not null || type.ImportedType is { TypeArguments.Count: > 0 } || type.GenericInstance is not null || type.TypeParameterIndex is not null || type.MethodParameterIndex is not null || type.ArrayElement is not null)
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
        TypeReferenceHandle selfMarker = default;
        void EncodeType(SignatureTypeEncoder encoder, SignatureType type)
        {
            if (type.PointerElement is { } pointer)
            {
                if (pointer.Primitive == PrimitiveType.Void) encoder.VoidPointer();
                else EncodeType(encoder.Pointer(), pointer);
                return;
            }
            if (type.IsSelf)
            {
                if (!referenceOnly) throw new InvalidDataException("Self requires native emission; CLI projection is reference-only");
                if (selfMarker.IsNil) selfMarker = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System.Runtime.CompilerServices"), metadata.GetOrAddString("Self"));
                encoder.Type(selfMarker, isValueType: true);
                return;
            }
            if (type.FunctionSignature is { } function)
            {
                var arguments = function.NoResult ? function.ParameterTypes.ToArray() : function.ParameterTypes.Append(function.ReturnType).ToArray();
                if (arguments.Length == 0) encoder.Type(FunctionCarrier(function), isValueType: false);
                else
                {
                    var encoded = encoder.GenericInstantiation(FunctionCarrier(function), arguments.Length, isValueType: false);
                    foreach (var argument in arguments) EncodeType(encoded.AddArgument(), argument);
                }
                return;
            }
            if (type.ImportedType is { } imported)
            {
                if (imported.AssemblyIdentity.Equals(CoreLibrary) && imported is { Namespace: "System", Name: "Char", IsValueType: true, GenericArity: 0, DeclaringType: null })
                {
                    encoder.Char(); return;
                }
                if (Equals(imported, ExternalObjectRoot) || imported.AssemblyIdentity.Equals(CoreLibrary) && imported is { Namespace: "System", Name: "Object", IsValueType: false, GenericArity: 0, DeclaringType: null })
                {
                    encoder.Object(); return;
                }
                var handle = ImportedTypeHandle(imported);
                if (imported.TypeArguments.Count == 0) encoder.Type(handle, imported.IsValueType);
                else
                {
                    var arguments = encoder.GenericInstantiation(handle, imported.TypeArguments.Count, imported.IsValueType);
                    foreach (var argument in imported.TypeArguments) EncodeType(arguments.AddArgument(), argument);
                }
                return;
            }
            if (type.GenericInstance is { } instance)
            {
                var arguments = encoder.GenericInstantiation(typeHandles[instance.Definition], instance.TypeArguments.Count, instance.Definition.IsValueType);
                foreach (var argument in instance.TypeArguments) EncodeType(arguments.AddArgument(), argument);
                return;
            }
            if (type.TypeParameterIndex is { } ordinal) { encoder.GenericTypeParameter(ordinal); return; }
            if (type.MethodParameterIndex is { } index) { encoder.GenericMethodTypeParameter(index); return; }
            if (type.ArrayElement is { } element) { EncodeType(encoder.SZArray(), element); return; }
            if (type.ClassType is { } owner)
            {
                if (owner.Definition.IsNativeObjectRoot) encoder.Object();
                else encoder.Type(typeHandles[owner], owner.IsValueType);
                return;
            }
            switch (type.Primitive)
            {
                case PrimitiveType.Int32: encoder.Int32(); break;
                case PrimitiveType.Int64: encoder.Int64(); break;
                case PrimitiveType.Byte: encoder.Byte(); break;
                case PrimitiveType.SByte: encoder.SByte(); break;
                case PrimitiveType.Int16: encoder.Int16(); break;
                case PrimitiveType.UInt16: encoder.UInt16(); break;
                case PrimitiveType.UInt32: encoder.UInt32(); break;
                case PrimitiveType.UInt64: encoder.UInt64(); break;
                case PrimitiveType.IntPtr: encoder.IntPtr(); break;
                case PrimitiveType.UIntPtr: encoder.UIntPtr(); break;
                case PrimitiveType.RuntimeTypeHandle: encoder.Type(MetadataTokens.EntityHandle(ElementToken(type)), isValueType: true); break;

                case PrimitiveType.Single: encoder.Single(); break;
                case PrimitiveType.Double: encoder.Double(); break;
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
                    foreach (var type in method.Signature.ParameterTypes) EncodeType(parameters.AddParameter().Type(isByRef: type.ByReferenceElement is not null), type.ByReferenceElement ?? type);

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
                var type = ImportOwner(owner);
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
            var declaring = reference.Definition.DeclaringType!;
            EntityHandle declaringHandle;
            if (typeHandles.TryGetValue(declaring, out var ownedHandle)) declaringHandle = ownedHandle;
            else
            {
                if (!CoreLibrary.Equals(declaring.Assembly.CoreLibrary)) throw new InvalidDataException("incompatible core identity");
                declaringHandle = ImportOwner(declaring);
            }
            var arguments = new BlobEncoder(ownerBlob).TypeSpecificationSignature().GenericInstantiation(declaringHandle, reference.DeclaringTypeArguments.Count, declaring.IsValueType);
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
        var importedFieldMembers = new Dictionary<(ImportedFieldReference, ImportedTypeReference), MemberReferenceHandle>();
        int ImportedFieldToken(ImportedFieldReference reference, ImportedConstructedFieldReference? construction)
        {
            var declaring = construction?.DeclaringType ?? reference.DeclaringType;
            var key = (reference, declaring);
            if (!importedFieldMembers.TryGetValue(key, out var member))
            {
                var signature = new BlobBuilder();
                EncodeType(new BlobEncoder(signature).FieldSignature(), reference.FieldType);
                member = metadata.AddMemberReference(MetadataTokens.EntityHandle(ElementToken(declaring)), metadata.GetOrAddString(reference.Name), metadata.GetOrAddBlob(signature));
                importedFieldMembers.Add(key, member);
            }
            return MetadataTokens.GetToken(member);
        }
        var functionMembers = new Dictionary<(SignatureType, bool), MemberReferenceHandle>();
        int FunctionMember(SignatureType type, bool constructor)
        {
            var key = (type, constructor);
            if (!functionMembers.TryGetValue(key, out var handle))
            {
                var blob = new BlobBuilder();
                if (constructor) blob.WriteBytes(new byte[] { 0x20, 2, 1, 0x1c, 0x18 });
                else
                {
                    var shape = type.FunctionSignature!;
                    new BlobEncoder(blob).MethodSignature(SignatureCallingConvention.Default, 0, true).Parameters(shape.ParameterTypes.Count,
                        result => { if (shape.NoResult) result.Void(); else result.Type().GenericTypeParameter(shape.ParameterTypes.Count); },
                        parameters => { for (int i = 0; i < shape.ParameterTypes.Count; i++) parameters.AddParameter().Type().GenericTypeParameter(i); });
                }
                handle = metadata.AddMemberReference(MetadataTokens.EntityHandle(ElementToken(type)), metadata.GetOrAddString(constructor ? ".ctor" : "Invoke"), metadata.GetOrAddBlob(blob));
                functionMembers.Add(key, handle);
            }
            return MetadataTokens.GetToken(handle);
        }
        var fieldHandles = types.SelectMany(t => t.MetadataFields).Select((field, index) => (field, handle: MetadataTokens.FieldDefinitionHandle(index + 1))).ToDictionary(p => p.field, p => p.handle);
        var failureConstructor = default(MemberReferenceHandle);
        if (!referenceOnly && methods.Any(m => m.Instructions.Any(i => i.Op == "fail")))
        {
            var failureType = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString("InvalidOperationException"));
            failureConstructor = metadata.AddMemberReference(failureType, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(new byte[] { 0x20, 1, 1, 0x0e }));
        }
        var objectConstructor = methods.Any(m => m.IsConstructor && !m.DeclaringType!.IsValueType) ? metadata.AddMemberReference(objectType, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(new byte[] { 0x20, 0, 1 })) : default;
        var bodies = new BlobBuilder();
        var bodyEncoder = new MethodBodyStreamEncoder(bodies);
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        int nextField = 1;
        int nextMethod = 1;
        int nextParameter = 1;
        var genericRows = new List<(EntityHandle Owner, int Sort, IReadOnlyList<string> Names)>();
        void EmitMethod(MethodBuilder method)
        {
            var firstParameter = MetadataTokens.ParameterHandle(nextParameter);
            void AnnotateNullable(ParameterHandle parameter, NullableAnnotation annotation)
            {
                var marker = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System.Runtime.CompilerServices"), metadata.GetOrAddString("NullableAttribute"));
                var constructor = metadata.AddMemberReference(marker, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(annotation.IsUniform ? new byte[] { 0x20, 1, 1, 5 } : new byte[] { 0x20, 1, 1, 0x1d, 5 }));
                var blob = new BlobBuilder(); blob.WriteUInt16(1);
                if (!annotation.IsUniform) blob.WriteInt32(annotation.Flags.Count);
                foreach (var flag in annotation.Flags) blob.WriteByte(flag);
                blob.WriteUInt16(0);
                metadata.AddCustomAttribute(parameter, constructor, metadata.GetOrAddBlob(blob));
            }
            if (method.Definition.NullableAnnotations.TryGetValue(-1, out var returnFlags))
            {
                AnnotateNullable(metadata.AddParameter(ParameterAttributes.None, default, 0), returnFlags);
                nextParameter++;
            }
            if (method.Signature.OutParameters.Count > 0 || method.Definition.ParameterNames.Count > 0 || method.Definition.ParameterArrayIndex is not null || method.Definition.NullableAnnotations.Keys.Any(i => i >= 0))
                for (int i = 0; i < method.ParameterCount; i++)
                {
                    var parameter = metadata.AddParameter(method.Signature.OutParameters.Contains(i) ? ParameterAttributes.Out : ParameterAttributes.None, method.Definition.ParameterNames.TryGetValue(i, out var parameterName) ? metadata.GetOrAddString(parameterName) : default, i + 1);
                    if (method.Definition.NullableAnnotations.TryGetValue(i, out var flags)) AnnotateNullable(parameter, flags);
                    if (method.Definition.ParameterArrayIndex == i)
                    {
                        var marker = metadata.AddTypeReference(ImportAssembly(CoreLibrary), metadata.GetOrAddString("System"), metadata.GetOrAddString("ParamArrayAttribute"));
                        var constructor = metadata.AddMemberReference(marker, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(new byte[] { 0x20, 0, 1 }));
                        metadata.AddCustomAttribute(parameter, constructor, metadata.GetOrAddBlob(new byte[] { 1, 0, 0, 0 }));
                    }
                    nextParameter++;
                }
            if (method.IsAbstract || method.Definition.ImplementationAttributes == 0x1000)
            {
                metadata.AddMethodDefinition((MethodAttributes)method.GetAttributes(accessors.Contains(method)),
                    (MethodImplAttributes)method.Definition.ImplementationAttributes, metadata.GetOrAddString(method.CliName), Signature(method), -1, firstParameter);
                nextMethod++;
                return;
            }
            var code = new BlobBuilder();
            var offsets = new int[method.Instructions.Count + 1];
            offsets[0] = method.IsConstructor && !method.DeclaringType!.IsValueType && method.DeclaringType.LocalBase is null ? 6 : 0;
            for (int i = 0; i < method.Instructions.Count; i++)
                offsets[i + 1] = offsets[i] + (method.Instructions[i].Op switch
                {
                    "enum.from" or "enum.to" => 0,
                    "constant64" or "constantDouble" => 9,
                    "constantSingle" => 5,
                    "call.constrained" => 11,
                    "fail" => 11,
                    "function.bind" => method.Instructions[i].Target!.IsStatic || method.Instructions[i].Target!.IsAbstract ? 12 : 11,
                    "function.invoke" => 5,
                    "label" => 0,
                    "type.token" or "object.unbox" or "reference.test" or "object.box" or "reference.cast" or "object.load" or "object.store" or "array.new" or "array.load" or "array.store" or "string" or "constant" or "call" or "call.virtual" or "call.generic" or "call.constructed" or "call.virtual.constructed" or "new.object" or "new.constructed" or "field.address" or "field.load" or "field.store" or "field.import.load" or "field.import.store" or "branch" or "branch.true" or "branch.false" => 5,
                    "argument" or "argument.store" or "argument.address" or "local.load" or "local.store" or "local.address" => 4,
                    "local.initialize" => 6,
                    "equal" or "less" or "greater" or "less.unordered" or "greater.unordered" => 2,
                    "reference.isnull" => 3,
                    _ => 1
                });
            var labels = method.LabelPositions();
            if (referenceOnly) { code.WriteByte(0x14); code.WriteByte(0x7a); } // ldnull; throw: never substitute native behavior.
            else if (method.IsConstructor && !method.DeclaringType!.IsValueType && method.DeclaringType.LocalBase is null)
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
                    case "field.import.load":
                    case "field.import.store":
                        code.WriteByte(instruction.Op == "field.import.load" ? (byte)0x7b : (byte)0x7d);
                        code.WriteInt32(ImportedFieldToken(instruction.ImportedField!, instruction.ImportedConstructedField)); break;
                    case "field.address":
                    case "field.load":
                    case "field.store":
                        code.WriteByte(instruction.Op == "field.address" ? (byte)0x7c : instruction.Op == "field.load" ? (byte)0x7b : (byte)0x7d);
                        code.WriteInt32(instruction.ConstructedField is { } fieldReference ? ConstructedFieldToken(fieldReference) : MetadataTokens.GetToken(fieldHandles[instruction.Field!])); break;
                    case "negate": code.WriteByte(0x65); break;
                    case "complement": code.WriteByte(0x66); break;
                    case "pop": code.WriteByte(0x26); break;
                    case "boolean": code.WriteByte(instruction.Value == 0 ? (byte)0x16 : (byte)0x17); break;
                    case "equal": code.WriteByte(0xfe); code.WriteByte(0x01); break;
                    case "less": code.WriteByte(0xfe); code.WriteByte(0x04); break;
                    case "less.unordered": code.WriteByte(0xfe); code.WriteByte(0x05); break;
                    case "greater.unordered": code.WriteByte(0xfe); code.WriteByte(0x03); break;
                    case "greater": code.WriteByte(0xfe); code.WriteByte(0x02); break;
                    case "branch":
                    case "branch.true":
                    case "branch.false":
                        code.WriteByte(instruction.Op == "branch" ? (byte)0x38 : instruction.Op == "branch.true" ? (byte)0x3a : (byte)0x39);
                        code.WriteInt32(offsets[labels[instruction.Value]] - (code.Count + 4)); break;
                    case "type.token": code.WriteByte(0xd0); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "string": code.WriteByte(0x72); code.WriteInt32(MetadataTokens.GetToken(metadata.GetOrAddUserString(instruction.Text!))); break;
                    case "constant64": code.WriteByte(0x21); code.WriteInt64(instruction.LongValue); break;
                    case "constantSingle": code.WriteByte(0x22); code.WriteInt32(instruction.Value); break;
                    case "constantDouble": code.WriteByte(0x23); code.WriteInt64(instruction.LongValue); break;
                    case "convertSingle": code.WriteByte(0x6b); break;
                    case "convertDouble": code.WriteByte(0x6c); break;
                    case "convert64": code.WriteByte(0x6a); break;
                    case "convert32": code.WriteByte(0x69); break;
                    case "convertByte": code.WriteByte(0xd2); break;
                    case "convertSByte": code.WriteByte(0x67); break;
                    case "convertInt16": code.WriteByte(0x68); break;
                    case "convertUInt16": code.WriteByte(0xd1); break;
                    case "convertUInt32": code.WriteByte(0x6d); break;
                    case "convertUInt64": code.WriteByte(0x6e); break;
                    case "convertIntPtr": code.WriteByte(0xd3); break;
                    case "convertUIntPtr": code.WriteByte(0xe0); break;
                    case "divide.unsigned": code.WriteByte(0x5c); break;
                    case "remainder.unsigned": code.WriteByte(0x5e); break;
                    case "shift.right.unsigned": code.WriteByte(0x64); break;
                    case "convertUnsignedDouble": code.WriteByte(0x76); break;

                    case "enum.from": case "enum.to": break;
                    case "constant": code.WriteByte(0x20); code.WriteInt32(instruction.Value); break;
                    case "argument.store": code.WriteByte(0xfe); code.WriteByte(0x0b); code.WriteUInt16((ushort)instruction.Value); break;
                    case "argument": code.WriteByte(0xfe); code.WriteByte(0x09); code.WriteUInt16((ushort)instruction.Value); break;
                    case "argument.address": code.WriteByte(0xfe); code.WriteByte(0x0a); code.WriteUInt16((ushort)instruction.Value); break;
                    case "local.address": code.WriteByte(0xfe); code.WriteByte(0x0d); code.WriteUInt16((ushort)instruction.Value); break;
                    case "object.box": code.WriteByte(0x8c); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "reference.isnull": code.WriteByte(0x14); code.WriteByte(0xfe); code.WriteByte(0x01); break;
                    case "reference.test": code.WriteByte(0x75); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "object.unbox": code.WriteByte(0xa5); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "reference.cast": code.WriteByte(0x74); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "object.load": code.WriteByte(0x71); code.WriteInt32(ElementToken(instruction.Type!)); break;
                    case "object.store": code.WriteByte(0x81); code.WriteInt32(ElementToken(instruction.Type!)); break;
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
                    case "new.constructed": case "call.constructed": case "call.virtual.constructed": code.WriteByte(instruction.Op == "new.constructed" ? (byte)0x73 : instruction.Op == "call.virtual.constructed" ? (byte)0x6f : (byte)0x28); code.WriteInt32(ConstructedCallToken(instruction.ConstructedTarget!)); break;
                    case "call.constrained":
                        code.WriteByte(0xfe); code.WriteByte(0x16); code.WriteInt32(ElementToken(instruction.Type!));
                        code.WriteByte(instruction.Target!.IsStatic ? (byte)0x28 : (byte)0x6f); code.WriteInt32(instruction.ConstructedTarget is { } constrainedCall ? ConstructedCallToken(constrainedCall) : ImportMethod(instruction.Target)); break;
                    case "call.generic": code.WriteByte(0x28); code.WriteInt32(GenericCallToken(instruction.GenericTarget!)); break;
                    case "call.virtual": case "call": code.WriteByte(instruction.Op == "call.virtual" ? (byte)0x6f : (byte)0x28); code.WriteInt32(ImportMethod(instruction.Target!)); break;
                    case "function.bind":
                        if (instruction.Target!.IsStatic) code.WriteByte(0x14);
                        if (instruction.Target.IsAbstract) code.WriteByte(0x25);
                        code.WriteByte(0xfe); code.WriteByte(instruction.Target.IsAbstract ? (byte)0x07 : (byte)0x06);
                        code.WriteInt32(instruction.ConstructedTarget is { } binding ? ConstructedCallToken(binding) : instruction.GenericTarget is { } genericBinding ? GenericCallToken(genericBinding) : ImportMethod(instruction.Target!));
                        code.WriteByte(0x73); code.WriteInt32(FunctionMember(instruction.Type!, true)); break;
                    case "function.invoke":
                        code.WriteByte(0x6f); code.WriteInt32(FunctionMember(instruction.Type!, false)); break;
                    case "fail":
                        code.WriteByte(0x72); code.WriteInt32(MetadataTokens.GetToken(metadata.GetOrAddUserString(instruction.Text!)));
                        code.WriteByte(0x73); code.WriteInt32(MetadataTokens.GetToken(failureConstructor));
                        code.WriteByte(0x7a); break;
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
            metadata.AddMethodDefinition((MethodAttributes)method.GetAttributes(accessors.Contains(method)),
                MethodImplAttributes.IL | MethodImplAttributes.Managed, metadata.GetOrAddString(method.CliName), Signature(method), body, firstParameter);
            genericRows.Add((handles[method], MetadataTokens.GetRowNumber(handles[method]) * 2 + 1, method.Signature.GenericParameterNames));
            nextMethod++;
        }
        foreach (var function in functions) EmitMethod(function);
        foreach (var type in types)
        {
            var typeHandle = metadata.AddTypeDefinition((TypeAttributes)type.Definition.Attributes,
                metadata.GetOrAddString(type.Namespace), metadata.GetOrAddString(type.Name), type.IsInterface || type.Definition.IsNativeObjectRoot ? default(EntityHandle) : type.IsEnum ? enumBase : type.IsValueType ? valueBase : type.LocalBase is { } parentType ? typeHandles[parentType] : objectType,
                MetadataTokens.FieldDefinitionHandle(nextField), MetadataTokens.MethodDefinitionHandle(nextMethod));
            if (type.Definition.DeclaringType is { } parent)
                metadata.AddNestedType(typeHandle, MetadataTokens.TypeDefinitionHandle(types.IndexOf(parent.Producer!) + 2));
            genericRows.Add((typeHandle, MetadataTokens.GetRowNumber(typeHandle) * 2, type.GenericParameterNames));
            foreach (var attribute in type.Definition.CustomAttributes)
            {
                attribute.ValidateOwner(type.Definition);
                var reference = attribute.AttributeType;
                EntityHandle attributeOwner = reference.ExplicitScope is { } scope
                    ? metadata.AddTypeReference(ImportAssembly(scope), metadata.GetOrAddString(reference.Namespace), metadata.GetOrAddString(reference.Name))
                    : typeHandles[reference.Resolve().Producer ?? throw new InvalidDataException("detached attribute owner")];
                var constructor = metadata.AddMemberReference(attributeOwner, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(attribute.GetConstructorSignature()));
                metadata.AddCustomAttribute(typeHandle, constructor, metadata.GetOrAddBlob(attribute.GetValue()));
            }
            foreach (var field in type.MetadataFields)
            {
                var signature = new BlobBuilder();
                var encoder = new BlobEncoder(signature).FieldSignature();
                EncodeType(encoder, field.FieldType);
                var fieldHandle = metadata.AddFieldDefinition((FieldAttributes)field.Definition.Attributes,
                    metadata.GetOrAddString(field.Name), metadata.GetOrAddBlob(signature));
                if (field.Definition.Constant is { } constant) metadata.AddConstant(fieldHandle, constant);
                nextField++;
            }
            foreach (var inherited in type.InterfaceSignatures) metadata.AddInterfaceImplementation(typeHandle, MetadataTokens.EntityHandle(ElementToken(inherited)));
            foreach (var method in type.Methods) EmitMethod(method);
            if (!type.IsInterface)
                foreach (var contract in type.RequiredInterfaceMethods)
                {
                    var implementation = type.FindInterfaceImplementation(contract.Name, contract.Signature, contract.IsStatic, contract.Owner)!;
                    if (!contract.IsStatic && implementation.Definition.ExplicitInterfaceImplementations.Count == 0) continue;
                    var declaration = metadata.AddMemberReference(MetadataTokens.EntityHandle(ElementToken(contract.Owner)),
                        metadata.GetOrAddString(contract.Name), Signature(contract.Declaration));
                    metadata.AddMethodImplementation(typeHandle, handles[implementation], declaration);
                }

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
                else
                    foreach (var constraint in methods[MetadataTokens.GetRowNumber(row.Owner) - 1].InterfaceConstraints.Where(c => c.ParameterIndex == i))
                        metadata.AddGenericParameterConstraint(parameter, MetadataTokens.EntityHandle(ElementToken(methods[MetadataTokens.GetRowNumber(row.Owner) - 1].Definition.ConstraintSignature(constraint.InterfaceType))));
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

/// <summary>An editable top-level class, interface or value type owned by an AssemblyBuilder.</summary>
public sealed partial class TypeBuilder
{
    private readonly List<MethodBuilder> methods = [];
    internal TypeBuilder(AssemblyBuilder assembly, string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public, bool isStatic = true, IReadOnlyList<string>? genericNames = null, bool isInterface = false, bool isValueType = false)
    {
        Assembly = assembly; var parameterNames = genericNames ?? Array.Empty<string>();
        var attributes = (visibility == TypeVisibility.Public ? TypeAttributes.Public : 0) |
            (isInterface ? TypeAttributes.Interface | TypeAttributes.Abstract : isStatic ? TypeAttributes.Abstract | TypeAttributes.Sealed : isValueType ? TypeAttributes.Sealed | TypeAttributes.SequentialLayout : 0);
        Definition = new TypeDefinition(@namespace, name, (uint)attributes, isInterface ? null : assembly.Definition.MainModule.ImportReference(assembly.CoreLibrary, "System", isValueType ? "ValueType" : "Object"));
        Definition.Producer = this; Definition.Module = assembly.Definition.MainModule; Definition.GenericParameterNames = parameterNames; Definition.GenericArity = parameterNames.Count;
    }
    internal TypeBuilder(AssemblyBuilder assembly, TypeDefinition definition)
    { Assembly = assembly; Definition = definition; definition.Producer = this; }
    /// <summary>Gets the same authored definition stored in the module's Types collection.</summary>
    public TypeDefinition Definition { get; }

    /// <summary>Gets immutable declaring-type parameter names in ordinal order.</summary>
    public IReadOnlyList<string> GenericParameterNames => Definition.GenericParameterNames!;
    /// <summary>Gets whether this declaration has the CLI Abstract flag.</summary>
    public bool IsAbstract => (Definition.Attributes & 0x80) != 0;
    /// <summary>Gets whether this class declares a native closed direct family.</summary>
    public bool IsClosedHierarchy => Definition.IsClosedHierarchy;
    /// <summary>Gets whether this is an abstract sealed static class.</summary>
    public bool IsStatic => (Definition.Attributes & 0x180) == 0x180 && !IsInterface;
    /// <summary>Gets whether this declaration is a CLI value type rather than a reference type.</summary>
    public bool IsValueType => Definition.IsValueType;
    /// <summary>Gets declared visibility within the assembly or enclosing type.</summary>
    public TypeVisibility Visibility => (Definition.Attributes & 7) is 1 or 2 ? TypeVisibility.Public : TypeVisibility.Internal;
    /// <summary>Gets the owning assembly.</summary>
    public AssemblyBuilder Assembly { get; }
    /// <summary>Gets the declared namespace.</summary>
    public string Namespace => Definition.Namespace;
    /// <summary>Gets the declared name.</summary>
    public string Name => Definition.Name;
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
    /// <summary>Adds a nonvirtual instance method on a reference class or value type.</summary>
    /// <param name="name">Nonempty simple name; .ctor/.cctor are reserved.</param>
    /// <param name="signature">Primitive/owned-class declared parameters, excluding the receiver.</param>
    /// <param name="visibility">Public, Internal or Private.</param>
    /// <returns>An owned method whose argument zero is a class receiver or initialized managed value receiver.</returns>
    /// <exception cref="ArgumentException">Invalid or duplicate contract or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    public MethodBuilder AddInstanceMethod(string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
        => AddMethodCore(name, signature, visibility, isStatic: false, constructor: false);
    /// <summary>Adds a class or value constructor with primitive declared parameters and no result.</summary>
    /// <param name="parameterTypes">At most 256 primitive parameters, excluding the receiver.</param>
    /// <param name="visibility">Public, Internal, Private or Protected (CLI Family).</param>
    /// <returns>An owned .ctor body with receiver at argument zero.</returns>
    /// <exception cref="ArgumentException">Invalid/duplicate contract or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    /// <remarks>CLI emission initializes System.Object before reference-class bodies; value bodies assign their own fields. Native root construction requires no base call. Derived classes explicitly call their direct base constructor.</remarks>
    public MethodBuilder AddConstructor(IEnumerable<PrimitiveType> parameterTypes, MethodVisibility visibility = MethodVisibility.Public)
        => AddMethodCore(".ctor", new(PrimitiveType.Void, parameterTypes), visibility, isStatic: false, constructor: true);
    /// <summary>Adds a constructor with an owned nominal/primitive signature whose result must be Void.</summary>
    /// <param name="signature">Void result and up to 256 primitive/owned-class declared parameters.</param>
    /// <param name="visibility">Public, Internal, Private or Protected (CLI Family).</param>
    /// <returns>A constructor owned by this class or value type.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Non-Void result, foreign class, duplicate signature or invalid visibility.</exception>
    /// <exception cref="InvalidOperationException">The declaring type is static.</exception>
    public MethodBuilder AddConstructor(MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (signature.GenericParameterNames.Count != 0 || signature.ParameterTypes.Any(p => p.ByReferenceElement is not null)) throw new ArgumentException("generic or byref constructor parameters unsupported", nameof(signature));
        if (signature.ReturnType != PrimitiveType.Void) throw new ArgumentException("constructor must have no result", nameof(signature));
        return AddMethodCore(".ctor", signature, visibility, false, true);
    }
    private MethodBuilder AddMethodCore(string name, MethodSignature signature, MethodVisibility visibility, bool isStatic, bool constructor, bool abstractContract = false)
    {
        if (IsInterface != abstractContract) throw new InvalidOperationException("interface owners require abstract contract methods");
        if (!isStatic && IsStatic) throw new InvalidOperationException("instance methods require a nonstatic nominal type");
        if (!constructor && name is ".ctor" or ".cctor") throw new ArgumentException("reserved constructor name", nameof(name));
        if (visibility is not (MethodVisibility.Public or MethodVisibility.Internal or MethodVisibility.Private or MethodVisibility.Protected)) throw new ArgumentOutOfRangeException(nameof(visibility));
        if (visibility == MethodVisibility.Protected && !constructor) throw new ArgumentException("protected visibility currently requires a constructor", nameof(visibility));
        ArgumentNullException.ThrowIfNull(signature);
        signature.ValidateOwner(Assembly, GenericParameterNames.Count, allowSelf: IsInterface);
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || methods.Count >= 256 ||
            methods.Any(m => m.Name == name && m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("invalid or duplicate method");
        var method = new MethodBuilder(Assembly, this, name, signature, visibility, isStatic: isStatic); Definition.Methods.Add(method.Definition); return method;
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
    Private,
    /// <summary>Accessible from the declaring type or a derived type; currently constructors only.</summary>
    Protected
}

/// <summary>Typed Int32/Int64/Boolean/String body construction; invalid control-flow contracts fail before emission.</summary>
public sealed partial class MethodBuilder
{
    internal sealed record Operation(string Op, int Value = 0, MethodBuilder? Target = null, string? Text = null, NativeFunctionDefinition? NativeTarget = null, long LongValue = 0, FieldBuilder? Field = null, SignatureType? Type = null, GenericMethodInstance? GenericTarget = null, ConstructedMethodReference? ConstructedTarget = null, ConstructedFieldReference? ConstructedField = null, ImportedFieldReference? ImportedField = null, ImportedConstructedFieldReference? ImportedConstructedField = null, TypeBuilder? ConstrainedOwner = null, ImportedMethodReference? ConstrainedReference = null, ImportedConstructedMethodReference? ConstrainedConstructedReference = null);
    internal List<Operation> Instructions => Definition.Body.Instructions;
    internal int MaxStack { get; private set; }
    internal MethodBuilder(AssemblyBuilder assembly, TypeBuilder? owner, string name, int count, bool result)
        : this(assembly, owner, name, PrimitiveMethodSignature.Int32(count, result)) { }
    internal MethodBuilder(AssemblyBuilder assembly, TypeBuilder? owner, string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public, string @namespace = "", bool isStatic = true)
    { Assembly = assembly; DeclaringType = owner; Definition = new MethodDefinition(this, name, owner?.Namespace ?? @namespace, signature, visibility, isStatic); }
    /// <summary>Gets whether the signature excludes an instance receiver.</summary>
    public bool IsStatic => Definition.IsStatic;
    /// <summary>Gets whether this is an instance .ctor with no result.</summary>
    public bool IsConstructor => !IsStatic && Name == ".ctor";
    internal int ArgumentCount => ParameterCount + (IsStatic ? 0 : 1);
    /// <summary>Gets the function namespace or the declaring type namespace; empty for the global namespace.</summary>
    public string Namespace => Definition.Namespace!;
    internal string CliName => DeclaringType is null ? FunctionNamespaceEncoding.Encode(Namespace, Name) : Name;
    /// <summary>Gets declared method or assembly-function visibility.</summary>
    public MethodVisibility Visibility => (Definition.DeclarationAttributes & 7) switch { 6 => MethodVisibility.Public, 3 => MethodVisibility.Internal, 4 => MethodVisibility.Protected, _ => MethodVisibility.Private };
    /// <summary>Gets the immutable primitive/owned-class method signature.</summary>
    public MethodSignature Signature => Definition.AuthoredSignature!;
    /// <summary>Gets the owning assembly, including for top-level functions.</summary>
    public AssemblyBuilder Assembly { get; }
    /// <summary>Gets the declaring type, or null for a top-level function.</summary>
    public TypeBuilder? DeclaringType { get; }
    /// <summary>Gets the method name.</summary>
    public string Name => Definition.Name;
    /// <summary>Gets the parameter count.</summary>
    public int ParameterCount => Signature.ParameterTypes.Count;
    /// <summary>Gets whether the method has a primitive or owned-class result rather than no result.</summary>
    public bool ReturnsValue => Signature.ReturnType != PrimitiveType.Void;
    /// <summary>Appends an Int32 constant.</summary>
    /// <param name="value">Constant value.</param>
    public void LoadConstant(int value) => GetILGenerator().LoadConstant(value);
    /// <summary>Appends a native System.Console.WriteLine call with a constant UTF-8 string.</summary>
    /// <param name="text">Unicode text, at most 64 KiB when UTF-8 encoded.</param>
    /// <exception cref="ArgumentNullException">Text is null.</exception>
    /// <exception cref="ArgumentException">Invalid Unicode or text exceeds the limit.</exception>
    /// <remarks>Native emission only; ordinary CLI Write rejects this operation. Does not alter the surrounding primitive stack.</remarks>
    public void WriteConsoleLine(string text) => GetILGenerator().WriteConsoleLine(text);
    /// <summary>Consumes a String stack value and writes it through native System.Console.WriteLine.</summary>
    /// <remarks>Native-only bootstrap; discards bundled System's inhabited Void result. Ordinary CLI output rejects this operation.</remarks>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or stack mismatch when writing.</exception>
    public void WriteConsoleLine() => GetILGenerator().WriteConsoleLine();


    /// <summary>Appends a parameter load; bounds are checked at Write.</summary>
    /// <param name="index">Argument slot index; instance receiver is zero and declared parameters start at one.</param>
    public void LoadArgument(int index) => GetILGenerator().LoadArgument(index);
    /// <summary>Stores a value into a by-value argument slot in this invocation.</summary>
    /// <param name="index">Argument slot index; instance declared parameters start at one. Bounds and exact type are checked when writing.</param>
    /// <exception cref="InvalidDataException">Instruction limit exceeded, or invalid index/stack type when writing.</exception>
    /// <remarks>Does not update caller storage. Receiver stores and by-reference parameters are unsupported.</remarks>
    public void StoreArgument(int index) => GetILGenerator().StoreArgument(index);
    /// <summary>Appends matching-width Int32/Int64 addition.</summary>
    public void Add() => GetILGenerator().Add();
    /// <summary>Appends matching-width Int32/Int64 subtraction.</summary>
    public void Subtract() => GetILGenerator().Subtract();
    /// <summary>Appends matching-width Int32/Int64 multiplication.</summary>
    public void Multiply() => GetILGenerator().Multiply();
    /// <summary>Appends matching-width signed Int32/Int64 division, truncating toward zero.</summary>
    /// <remarks>Zero and minimum-value divided by -1 fault at execution, not when writing.</remarks>
    public void Divide() => GetILGenerator().Divide();
    /// <summary>Appends matching-width signed Int32/Int64 remainder, with the dividend's sign.</summary>
    /// <remarks>Zero faults at execution, not when writing. Native minimum/-1 faults; CLI follows the host CLR edge behavior.</remarks>
    public void Remainder() => GetILGenerator().Remainder();
    /// <summary>Appends bitwise AND of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseAnd() => GetILGenerator().BitwiseAnd();
    /// <summary>Appends bitwise OR of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseOr() => GetILGenerator().BitwiseOr();
    /// <summary>Appends bitwise XOR of matching Int32/Int64 or Boolean operands.</summary>
    public void BitwiseXor() => GetILGenerator().BitwiseXor();
    /// <summary>Appends an Int32/Int64 left shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    public void ShiftLeft() => GetILGenerator().ShiftLeft();
    /// <summary>Appends a sign-extending Int32/Int64 right shift with an Int32 count.</summary>
    /// <remarks>CLI out-of-range counts are unspecified; native counts are masked to 5 or 6 bits.</remarks>
    public void ShiftRight() => GetILGenerator().ShiftRight();
    /// <summary>Appends a call; foreign methods are imported during Write.</summary>
    /// <param name="target">Local or external builder method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    public void Call(MethodBuilder target) => GetILGenerator().Call(target);
    /// <summary>Appends a call to an imported read-only method contract.</summary>
    /// <param name="target">Reference imported by this method's assembly builder.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Foreign reference, open generic definition, or a contract requiring Callvirt.</exception>
    public void Call(ImportedMethodReference target) => GetILGenerator().Call(target);
    /// <summary>Calls a static Int32 function selected from an explicitly loaded native System inventory.</summary>
    /// <param name="target">Owned System function whose parameters and result are Int32.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="InvalidDataException">Module is not System or the callable signature is unsupported.</exception>
    /// <remarks>Native-only bootstrap. The host must supply the matching System assembly to neoCLR;
    /// no assembly revision or image digest is encoded. Ordinary CLI output rejects this operation.</remarks>
    public void Call(NativeFunctionDefinition target) => GetILGenerator().Call(target);
    /// <summary>Appends return with the declared stack shape; no values may remain afterward.</summary>
    public void Return() => GetILGenerator().Return();
    /// <summary>Clears instructions for editing before another Write; local declarations and handles are retained.</summary>
    public void ClearBody() => GetILGenerator().ClearBody();

}
