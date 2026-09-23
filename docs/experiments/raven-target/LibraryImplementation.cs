using Mono.Cecil;
using System.Text.RegularExpressions;

// A bounded implementation fragment for an existing reference contract. No name-based
// substitution of guest/application types: every exported signature must match the core.
static class LibraryImplementation
{
    static readonly HashSet<MethodDefinition> ReadonlyReceivers = new();
    // Preserve the existing scalar neoIL formatting receiver while CIL uses a
    // managed-byref struct receiver. Intrinsic field reads accept either form.
    public static bool IsByValueReceiver(MethodReference method) =>
        EmptyLibrary.IsByValueReceiver(method) || GenericUnionLibrary.IsByValueReceiver(method) || ErrorCarrierLibrary.IsByValueReceiver(method) ||
        PrimitiveLibrary.IsMatched(method.DeclaringType.Resolve())
        && method.DeclaringType.FullName == "System.Int32" && method.Name == "ToString"
        && method.HasThis && !method.HasParameters && method.ReturnType.MetadataType == MetadataType.String;
    public static bool IsReadonlyReceiver(MethodDefinition method) => (GenericUnionLibrary.IsMatched(method.DeclaringType) && method.Name is "TryGetOutput" or "TryGetResidual" || ReadonlyReceivers.Contains(method) || OpaqueLibrary.IsByRefString(method))
        && !IsByValueReceiver(method);
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner)
    {
        ReadonlyReceivers.Clear();
        if (owner != "System" && !Regex.IsMatch(owner, @"^[A-Za-z_][A-Za-z0-9_]*(\.[A-Za-z_][A-Za-z0-9_]*)+$"))
            throw new InvalidDataException("Invalid library owner.");
        if (owner == "System.Storage.StorageItem")
        {
            var names = new[] { "System.Storage.StorageItem", "System.Storage.File", "System.Storage.Directory" };
            foreach (var name in names) ApplicationTypes.BindLibrary(source.GetType(name), name);
            return names.SelectMany(name => InterfaceRoots(source.GetType(name), core.GetType(name), name)).ToArray();
        }
        if (owner == "System.Concurrency.Thread")
        {
            var names = new[] { "System.Concurrency.Thread", "System.Concurrency.ThreadPool", "System.Concurrency.WorkerCompletion" };
            foreach (var name in names) ApplicationTypes.BindLibrary(source.GetType(name), name);
            return names.SelectMany(name => InstanceRoots(source.GetType(name), core.GetType(name), name)).ToArray();
        }
        if (owner == "System.Tasks.Task")
        {
            var names = new[] { "System.Tasks.TaskQueue", "System.Tasks.Task`1", "System.Tasks.Promise`1", "System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1", "System.Runtime.CompilerServices.IAsyncStateMachine", "System.Runtime.CompilerServices.ITaskAwaiter" };
            foreach (var name in names) ApplicationTypes.BindLibrary(source.GetType(name), name.Split('`')[0]);
            return names.SelectMany(name => source.GetType(name).IsInterface ? InterfaceRoots(source.GetType(name), core.GetType(name), name) : InstanceRoots(source.GetType(name), core.GetType(name), name.Split('`')[0])).Concat(Roots(source, core, "System.Tasks.TaskOperators")).Concat(Roots(source, core, "System.Tasks.TaskResultOperators")).ToArray();
        }
        if (owner == "System.Array") return InstanceRoots(source.GetType("System.Array`1") ?? throw new InvalidDataException("Missing Array implementation."), core.GetType("System.Array`1"), owner);
        if (MarkerLibrary.IsOwner(owner)) return MarkerLibrary.Roots(source, core, owner);
        if (EnumBindings.IsType(owner)) return FlagsLibrary.Roots(source, core, owner);
        if (owner == "System.Func") return DelegateLibrary.Roots(source, core);
        if (owner == "System.Introspection.MemberInfo")
            return DescriptorLibrary.Roots(source, core, InstanceRoots, InterfaceRoots);
        if (owner is "System.Introspection.TypeInfo" or "System.Introspection.ParameterInfo" or "System.Introspection.AssemblyInfo" or "System.Introspection.ModuleInfo")
        {
            InterfaceRoots(source.GetType(owner), core.GetType(owner), owner);
            var provider = owner.Replace(".Introspection.", ".Introspection.Runtime");
            return InstanceRoots(source.GetType(provider), core.GetType(provider), provider);
        }
        if (owner is "System.Option" or "System.Result" or "System.Tasks.TaskOutcome")
            return GenericUnionLibrary.Roots(source, core, owner, InstanceRoots);
        var consoleProviders = Array.Empty<MethodDefinition>();
        if (owner == "System.Console") {
            var names = new[] { "System.IO.ConsoleInputStream", "System.IO.ConsoleOutputStream" };
            foreach (var name in names) ApplicationTypes.BindLibrary(source.GetType(name), name);
            consoleProviders = names.SelectMany(name => InstanceRoots(source.GetType(name), core.GetType(name), name)).ToArray();
        }
        var type = source.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t)) ?? source.Types.SingleOrDefault(t => t.FullName.Split('`')[0] == owner)
            ?? throw new InvalidDataException("Missing namespace implementation: " + owner);
        var contract = core.Types.SingleOrDefault(t => t.Namespace == owner && NamespaceFunctions.IsContainer(t)) ?? core.Types.SingleOrDefault(t => t.FullName.Split('`')[0] == owner)
            ?? throw new InvalidDataException("Missing namespace reference contract: " + owner);
        if (type.IsInterface || contract.IsInterface)
            return InterfaceRoots(type, contract, owner);
        if (!(type.IsAbstract && type.IsSealed))
            return InstanceRoots(type, contract, owner);
        if (!contract.IsPublic || contract.HasGenericParameters || contract.IsInterface)
            throw new InvalidDataException("Unsupported library reference owner.");
        if (!type.IsPublic || !type.IsAbstract || !type.IsSealed || type.HasGenericParameters || type.HasFields || type.HasInterfaces)
            throw new InvalidDataException("Library fragment requires a public nongeneric namespace container without fields.");
        var methods = type.Methods.ToArray();
        if (methods.Length == 0) throw new InvalidDataException("Empty library implementation.");
        if (owner == NativeMemoryBindings.Owner && methods.Length != contract.Methods.Count(m => m.IsPublic && m.IsStatic))
            throw new InvalidDataException("NativeMemory requires its complete overload surface.");
        foreach (var method in methods)
        {
            CheckMethod(method);
            if (!method.IsPublic || !method.IsStatic || method.IsConstructor || !method.HasBody
                || !Regex.IsMatch(method.Name, @"^[A-Za-z_][A-Za-z0-9_]*$")
                || method.Parameters.Any(p => p.IsOut || p.ParameterType.IsByReference)
                || method.ReturnType.MetadataType == MetadataType.Void && owner is not ("System.Console" or "System.Runtime.InteropServices.NativeMemory" or "System"))
                throw new InvalidDataException("Unsupported library export: " + method.FullName);
            var matches = contract.Methods.Where(m => m.IsPublic && m.IsStatic && m.Name == method.Name && m.Parameters.Count == method.Parameters.Count && m.GenericParameters.Count == method.GenericParameters.Count
                && SameType(m.ReturnType, method.ReturnType)
                && m.Parameters.Zip(method.Parameters).All(p => p.First.Name == p.Second.Name && SameType(p.First.ParameterType, p.Second.ParameterType))).ToArray();
            if (matches.Length == 1) CheckMethod(matches[0]);
            if (matches.Length != 1) throw new InvalidDataException("Library export does not match reference contract: " + method.FullName);
            if (owner is "System.Linq.Operators" or "System.OptionOperators" or "System.OptionNestedOperators" or "System.ResultOperators" or "System.Tasks.TaskOperators" or "System.Tasks.TaskResultOperators")
                CheckExtensionContract(method, matches[0]);
        }
        return methods.Concat(consoleProviders).ToArray();
    }

    public static void CheckExtensionContract(MethodDefinition implementation, MethodDefinition declaration)
    {
        static bool Marked(MethodDefinition method) => method.CustomAttributes.Count(a =>
            a.AttributeType.FullName == "System.Runtime.CompilerServices.ExtensionAttribute"
            && RuntimeSignatures.IsCore(a.AttributeType.Scope)) == 1;
        if (!implementation.IsStatic || implementation.Parameters.Count == 0 || !Marked(implementation)
            || !declaration.IsStatic || declaration.Parameters.Count == 0 || !Marked(declaration))
            throw new InvalidDataException("Library extension metadata mismatch: " + implementation.FullName);
    }

    static MethodDefinition[] InterfaceRoots(TypeDefinition type, TypeDefinition contract, string owner)
    {
        IntrospectionHierarchy.Validate(type);
        StorageHierarchy.Validate(type);
        IntrospectionHierarchy.Validate(contract);
        StorageHierarchy.Validate(contract);
        // Bounded invariant declaration authoring, not permission to replace a class with an
        // interface or supply executable default/static interface members.
        foreach (var candidate in new[] { type, contract })
            if (!candidate.IsPublic || !candidate.IsInterface || !candidate.IsAbstract
                || candidate.IsSealed || candidate.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant) || candidate.HasFields
                || candidate.HasNestedTypes || candidate.HasEvents
                || candidate.BaseType is not null || candidate.IsExplicitLayout
                || candidate.Methods.Any(m => !m.IsPublic || !m.IsAbstract || !m.IsVirtual
                    || !m.IsNewSlot || m.IsFinal || m.IsStatic || m.IsConstructor || m.HasBody
                    || m.HasGenericParameters || m.ExplicitThis || m.HasOverrides
                    || m.CallingConvention != MethodCallingConvention.Default
                    || m.Parameters.Any(p => (p.IsOut || p.ParameterType.IsByReference) && !PropagationLibrary.IsConditionalOutput(m, p))))
                throw new InvalidDataException("Unsupported library interface contract.");
        // Self-referential interface signatures (for example Equatable<TypeInfo>)
        // use this explicitly selected source/reference pair during validation.
        ApplicationTypes.BindLibrary(type, owner);
        bool Match(MethodDefinition left, MethodDefinition right) =>
            left.Name == right.Name && SameType(left.ReturnType, right.ReturnType)
            && left.Parameters.Count == right.Parameters.Count
            && left.Parameters.Zip(right.Parameters).All(p => p.First.Name == p.Second.Name && p.First.IsOut == p.Second.IsOut
                && SameType(p.First.ParameterType, p.Second.ParameterType));
        if (type.GenericParameters.Count != contract.GenericParameters.Count
            || type.Interfaces.Count != contract.Interfaces.Count
            || type.Interfaces.Any(i => contract.Interfaces.Count(c => SameType(c.InterfaceType, i.InterfaceType)) != 1)
            || type.Methods.Count != contract.Methods.Count
            || type.Methods.Any(m => contract.Methods.Count(c => Match(c, m)) != 1)
            || type.Properties.Count != contract.Properties.Count
            || type.Properties.Any(p => contract.Properties.Count(c => c.Name == p.Name
                && SameType(c.PropertyType, p.PropertyType)
                && c.Parameters.Count == p.Parameters.Count
                && c.Parameters.Zip(p.Parameters).All(a => a.First.Name == a.Second.Name
                    && SameType(a.First.ParameterType, a.Second.ParameterType))
                && c.GetMethod?.Name == p.GetMethod?.Name && c.SetMethod?.Name == p.SetMethod?.Name) != 1))
            throw new InvalidDataException("Library interface does not match reference contract.");
        _ = ApplicationTypes.Type(type);
        return [];
    }

    static MethodDefinition[] InstanceRoots(TypeDefinition type, TypeDefinition contract, string owner)
    {
        IntrospectionHierarchy.Validate(type);
        StorageHierarchy.Validate(type);
        IntrospectionHierarchy.Validate(contract);
        StorageHierarchy.Validate(contract);
        if (DescriptorLibrary.IsProvider(type) && (!type.IsNotPublic || !contract.IsNotPublic))
            throw new InvalidDataException("Runtime descriptor providers must remain internal.");
        if (GenericUnionLibrary.IsFamily(type)) GenericUnionLibrary.Validate(type, contract);
        var caseMethods = ErrorCarrierLibrary.IsCarrier(type) ? ErrorCarrierLibrary.Validate(type, contract) : [];
        // A single explicitly selected reference/implementation pair. Never alias arbitrary
        // guest types by namespace/name, and never execute reference-assembly stub bodies.
        foreach (var candidate in new[] { type, contract })
            if (!(candidate.IsPublic || candidate.IsNotPublic && (DescriptorLibrary.IsProvider(candidate) || WorkerBindings.IsProvider(candidate) || ReaderBindings.IsProvider(candidate)) || candidate.IsNestedPublic && GenericUnionLibrary.IsCase(candidate)) || candidate.IsInterface || candidate.IsAbstract != (DescriptorLibrary.IsDescriptor(candidate) && candidate.Name == "RuntimeMemberInfo")
                || candidate.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant) || candidate.HasNestedTypes && !ErrorCarrierLibrary.IsCarrier(candidate) || candidate.HasEvents
                || candidate.BaseType?.FullName != (DescriptorLibrary.IsDescriptor(candidate) ? DescriptorLibrary.Base(candidate) : candidate.IsValueType ? "System.ValueType" : "System.Object") || candidate.IsExplicitLayout)
                throw new InvalidDataException($"Unsupported instance library owner: {candidate.FullName}, base={candidate.BaseType}, public={candidate.IsPublic}, abstract={candidate.IsAbstract}, nested={candidate.HasNestedTypes}, value={candidate.IsValueType}.");
        if (type.IsValueType != contract.IsValueType)
            throw new InvalidDataException("Library value/reference representation does not match reference contract.");
        // Native snapshot factories construct Type from precisely one opaque handle.
        if (owner is "System.Introspection.TypeInfo" or "System.Introspection.RuntimeTypeInfo" && (type.Fields.Count != 1 || type.Fields[0].Name != "Handle"
            || type.Fields[0].FieldType.FullName != "System.RuntimeTypeHandle"
            || !RuntimeSignatures.IsCore(type.Fields[0].FieldType.Scope)))
            throw new InvalidDataException("Type library layout must contain exactly one core RuntimeTypeHandle.");
        if (owner == "System.Introspection.RuntimeParameterInfo")
        {
            var layout = new (string Name, string Type)[] {
                ("StoredName", "System.String"), ("StoredPosition", "System.Int32"),
                ("StoredParameterType", "System.Introspection.TypeInfo"), ("StoredIsOut", "System.Boolean"),
                ("StoredIsOutWhenTrue", "System.Boolean"), ("StoredIsReadOnly", "System.Boolean"),
                ("StoredMetadataToken", "System.Int32"), ("StoredModule", "System.Introspection.ModuleInfo")
            };
            if (type.Fields.Count != layout.Length || type.Fields.Zip(layout).Any(p =>
                p.First.Name != p.Second.Name || p.First.FieldType.FullName != p.Second.Type
                || (p.Second.Type is "System.Introspection.TypeInfo" or "System.Introspection.ModuleInfo" ? !RuntimeSignatures.IsCore(p.First.FieldType.Scope)
                    : p.First.FieldType.MetadataType is not (MetadataType.String or MetadataType.Int32 or MetadataType.Boolean))))
                throw new InvalidDataException("ParameterInfo library layout must match the runtime snapshot fields: "
                    + string.Join(";", type.Fields.Select(f => f.Name + ":" + f.FieldType.FullName + "@" + f.FieldType.Scope)));
        }
        if (owner is "System.Introspection.RuntimeAssemblyInfo" or "System.Introspection.RuntimeModuleInfo")
        {
            var fields = owner.EndsWith("RuntimeModuleInfo") ? new[] { "StoredIdentity", "StoredName" } : new[] { "StoredIdentity" };
            if (!type.Fields.Select(f => f.Name).SequenceEqual(fields) || type.Fields.Any(f => f.FieldType.MetadataType != MetadataType.String))
                throw new InvalidDataException("Assembly/module descriptor storage does not match runtime contract.");
        }
        if (owner == "System.String") OpaqueLibrary.ValidateString(type, contract);
        if (owner == "System.Array") ArrayLibrary.Validate(type);
        if (PrimitiveLibrary.IsPrimitive(type)) PrimitiveLibrary.Validate(type, contract);
        else if (type.IsValueType)
        {
            // Unlike reference classes, a value's instance layout is an ABI contract.
            // Start with scalar, nongeneric sequential records; do not infer layout.
            if (type.HasGenericParameters && !GenericUnionLibrary.IsFamily(type)
                || !type.IsSequentialLayout || !contract.IsSequentialLayout
                || type.PackingSize != contract.PackingSize || type.ClassSize != contract.ClassSize
                || type.Fields.Count != contract.Fields.Count
                || type.Fields.Zip(contract.Fields).Any(p => p.First.Name != p.Second.Name
                    || !SameType(p.First.FieldType, p.Second.FieldType)
                    || (p.First.FieldType.MetadataType is not (MetadataType.Int32 or MetadataType.Int64 or MetadataType.Boolean)
                        && !GenericUnionLibrary.IsFamily(type)
                        && !(ErrorCarrierLibrary.IsCarrier(type) && p.First.FieldType.FullName == "System.Value" && RuntimeSignatures.IsCore(p.First.FieldType.Scope))
                        && !(owner == "System.LocalDateTime" && p.First.FieldType.FullName is "System.Date" or "System.Time"
                            && RuntimeSignatures.IsCore(p.First.FieldType.Scope)))))
                throw new InvalidDataException($"Unsupported or mismatched value library layout: source sequential={type.IsSequentialLayout}, pack={type.PackingSize}, size={type.ClassSize}, fields={string.Join(',', type.Fields.Select(f => f.Name + ":" + f.FieldType.FullName))}; reference sequential={contract.IsSequentialLayout}, pack={contract.PackingSize}, size={contract.ClassSize}, fields={string.Join(',', contract.Fields.Select(f => f.Name + ":" + f.FieldType.FullName))}.");
        }
        if (type.GenericParameters.Count != contract.GenericParameters.Count)
            throw new InvalidDataException("Instance library generic arity does not match reference contract.");
        if (type.IsSealed != contract.IsSealed) throw new InvalidDataException("Instance library sealing does not match reference contract.");
        bool MatchType(TypeReference left, TypeReference right)
        {
            if (left is ByReferenceType br)
                return right is ByReferenceType rr && MatchType(br.ElementType, rr.ElementType);
            if (left is ArrayType a)
                return right is ArrayType b && a.IsVector == b.IsVector && a.Rank == b.Rank
                    && MatchType(a.ElementType, b.ElementType);
            if (left is GenericInstanceType l)
                return right is GenericInstanceType r && MatchType(l.ElementType, r.ElementType)
                    && l.GenericArguments.Count == r.GenericArguments.Count
                    && l.GenericArguments.Zip(r.GenericArguments).All(p => MatchType(p.First, p.Second));
            if (left.FullName is "System.Option/None" or "System.Option/Some`1" or "System.Result/Ok`1" or "System.Result/Error`1" or "System.Tasks.TaskOutcome/Completed`1" or "System.Tasks.TaskOutcome/Cancelled"
                && left.FullName == right.FullName && GenericUnionLibrary.IsCase(left.Resolve()) && GenericUnionLibrary.IsCase(right.Resolve())
                && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right)) return true;
            return AsyncBindings.SameType(left, right) || TaskBindings.SameType(left, right) || DescriptorLibrary.SameType(left, right) || SameTypeArgument(left, right) || ErrorCarrierLibrary.SameCase(left, right, type, contract) || (left.FullName == contract.FullName && right.FullName == type.FullName
                && left.Resolve() == contract && right.Resolve() == type);
        }
        if (type.Interfaces.Count != contract.Interfaces.Count || type.Interfaces.Any(i =>
            contract.Interfaces.Count(c => MatchType(c.InterfaceType, i.InterfaceType)) != 1))
            throw new InvalidDataException("Instance library interfaces do not match reference contract.");
        bool MatchMethod(MethodDefinition left, MethodDefinition right) =>
            left.Name == right.Name && !left.HasGenericParameters && !left.ExplicitThis
            && left.CallingConvention == MethodCallingConvention.Default && left.IsStatic == right.IsStatic && left.IsConstructor == right.IsConstructor
            && left.IsVirtual == right.IsVirtual && left.IsFinal == right.IsFinal && left.IsNewSlot == right.IsNewSlot
            && (left.ReturnType.MetadataType == MetadataType.Void || right.ReturnType.MetadataType == MetadataType.Void
                ? left.ReturnType.MetadataType == right.ReturnType.MetadataType : MatchType(left.ReturnType, right.ReturnType))
            && left.Parameters.Count == right.Parameters.Count
            && left.Parameters.Zip(right.Parameters).All(p => p.First.Name == p.Second.Name && p.First.IsOut == p.Second.IsOut
                && MatchType(p.First.ParameterType, p.Second.ParameterType));
        if (type.Fields.Any(f => !f.IsPrivate || f.IsStatic || f.IsInitOnly || f.HasMarshalInfo)
            || contract.Fields.Any(f => !f.IsPrivate))
            throw new InvalidDataException("Instance library requires private mutable implementation fields.");
        var declarationOnly = type.IsValueType && !type.HasFields && !contract.HasFields
            && !contract.HasMethods && !type.HasProperties && !type.HasInterfaces
            && type.Methods.All(PrimitiveLibrary.IsDefaultConstructor);
        var methods = type.Methods.Where(m => !OpaqueLibrary.IsOmittedConstructor(m) && !ArrayLibrary.OmitConstructor(m) && !EmptyLibrary.OmitConstructor(m) && !((ErrorCarrierLibrary.IsCarrier(type) || GenericUnionLibrary.IsFamily(type) && type.HasFields) && PrimitiveLibrary.IsDefaultConstructor(m)) && (!(PrimitiveLibrary.IsPrimitive(type) || declarationOnly) || !PrimitiveLibrary.IsDefaultConstructor(m))).ToArray();
        if (methods.Length == 0 && !declarationOnly || methods.Any(m => !(m.IsPublic || DescriptorLibrary.IsDescriptor(type) && m.IsFamily && m.IsConstructor || m.IsPrivate && !m.IsVirtual
            || m.IsAssembly && !m.IsVirtual && contract.Methods.Count(c => c.IsAssembly && MatchMethod(c, m)) == 1) || !m.HasBody || m.HasGenericParameters
            || m.ExplicitThis || m.IsConstructor && m.IsStatic || m.CallingConvention != MethodCallingConvention.Default
            || m.Parameters.Any(p => (p.IsOut || p.ParameterType.IsByReference) && !GenericUnionLibrary.IsConditionalOutput(m, p))))
            throw new InvalidDataException("Unsupported instance library export: " + string.Join(";", methods.Select(m => m.FullName + " " + m.Attributes + " matches=" + contract.Methods.Count(c => c.IsAssembly && MatchMethod(c, m)))));
        // Private implementation helpers are not exports, but remain roots so even
        // unused bodies are checked and emitted with their original visibility.
        var expected = contract.Methods.Where(m => m.IsPublic && !OpaqueLibrary.IsOmittedConstructor(m) && !OpaqueLibrary.IsStringOperator(m) && !GenericUnionLibrary.IsProtocol(m)).ToArray();
        var exports = methods.Where(m => m.IsPublic).ToArray();
        if (contract.Methods.Where(m => m.IsAssembly && !m.IsConstructor).Any(c =>
            methods.Count(m => m.IsAssembly && MatchMethod(c, m)) != 1))
            throw new InvalidDataException("Internal library factory does not match reference contract.");
        if (expected.Length != exports.Length || exports.Any(m => expected.Count(e => MatchMethod(e, m)) != 1))
            throw new InvalidDataException("Instance library export does not match reference contract: missing=[" + string.Join(";", expected.Where(e => !exports.Any(m => MatchMethod(e, m))).Select(m => m.FullName + " " + m.Attributes)) + "]; unmatched=[" + string.Join(";", exports.Where(m => !expected.Any(e => MatchMethod(e, m))).Select(m => m.FullName + " " + m.Attributes)) + "]");
        if (type.Properties.Count(p => p.Name != "Value" || !GenericUnionLibrary.IsCarrier(type)) != contract.Properties.Count(p => p.Name != "Value" || !GenericUnionLibrary.IsCarrier(contract)) || type.Properties.Any(p =>
            contract.Properties.Count(c => c.Name == p.Name && MatchType(c.PropertyType, p.PropertyType)
                && c.Parameters.Count == p.Parameters.Count
                && c.Parameters.Zip(p.Parameters).All(a => MatchType(a.First.ParameterType, a.Second.ParameterType))
                && c.GetMethod?.Name == p.GetMethod?.Name && (GenericUnionLibrary.IsCase(type) && p.SetMethod is null || c.SetMethod?.Name == p.SetMethod?.Name)) != 1))
            throw new InvalidDataException("Instance library property does not match reference contract.");
        foreach (var method in exports.Where(m => m.HasThis && !m.IsConstructor && type.IsValueType))
            if (expected.Single(e => MatchMethod(e, method)).CustomAttributes.Any(a =>
                a.AttributeType.FullName == "System.Runtime.CompilerServices.IsReadOnlyAttribute"))
                ReadonlyReceivers.Add(method);
        ApplicationTypes.BindLibrary(type, owner);
        if (declarationOnly) _ = ApplicationTypes.Type(type);
        foreach (var method in methods.Concat(caseMethods)) CheckMethod(method);
        return methods.Concat(caseMethods).ToArray();
    }

    public static void CheckMethod(MethodDefinition method)
    {
        if (!method.HasGenericParameters) { ApplicationTypes.CheckMethod(method); return; }
        if (!method.IsStatic || method.ExplicitThis || method.DeclaringType.HasGenericParameters
            || method.CallingConvention != MethodCallingConvention.Generic
            || method.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported generic library signature: " + method.FullName);
    }

    public static string GenericName(MethodDefinition method) => method.Name + (method.HasGenericParameters
        ? "<" + string.Join(',', method.GenericParameters.Select(p => "T" + p.Position)) + ">" : "");

    public static bool SameType(TypeReference left, TypeReference right)
    {
        if (ReaderBindings.SameType(left, right) || FileSystemBindings.SameType(left, right) || StorageItemBindings.SameType(left, right) || StorageProviderBindings.SameType(left, right) || PathBindings.SameType(left, right) || StreamBindings.SameType(left, right) || WorkerBindings.SameType(left, right) || AsyncBindings.SameType(left, right) || TaskBindings.SameType(left, right) || DescriptorLibrary.SameType(left, right)) return true;
        if (left is ByReferenceType lb)
            return right is ByReferenceType rb && SameType(lb.ElementType, rb.ElementType);
        if (left is ArrayType la)
            return right is ArrayType ra && la.IsVector == ra.IsVector && la.Rank == ra.Rank && SameType(la.ElementType, ra.ElementType);
        if (left is GenericParameter lp)
            return right is GenericParameter rp && lp.Type == rp.Type && lp.Position == rp.Position;
        if (left is GenericInstanceType l)
            return right is GenericInstanceType r && l.IsValueType == r.IsValueType
                && SameType(l.ElementType, r.ElementType)
                && l.GenericArguments.Count == r.GenericArguments.Count
                && l.GenericArguments.Zip(r.GenericArguments).All(p => SameTypeArgument(p.First, p.Second));
        return left.FullName == right.FullName && left.MetadataType == right.MetadataType && left.IsValueType == right.IsValueType
            && (left.MetadataType is not (MetadataType.Class or MetadataType.ValueType)
                || left.Resolve()?.Module.Assembly.Name.FullName == right.Resolve()?.Module.Assembly.Name.FullName);
    }

    static bool SameTypeArgument(TypeReference left, TypeReference right)
    {
        if (SameType(left, right)) return true;
        // Cecil can read an external System.Void token as primitive Void while
        // the defining core retains ValueType. Only generic storage admits this
        // equivalence; ordinary no-result method returns remain distinct.
        if (left.FullName != "System.Void" || right.FullName != "System.Void"
            || !left.IsValueType || !right.IsValueType
            || !RuntimeSignatures.IsCore(left.Scope) || !RuntimeSignatures.IsCore(right.Scope)) return false;
        var l = left.Resolve();
        var r = right.Resolve();
        return l is not null && r is not null && l.IsValueType && r.IsValueType
            && !l.Fields.Any(f => !f.IsStatic) && !r.Fields.Any(f => !f.IsStatic)
            && l.Module.Assembly.Name.FullName == r.Module.Assembly.Name.FullName;
    }

    public static TypeReference Close(TypeReference type, GenericInstanceMethod method, int depth = 0)
    {
        if (depth > 32) throw new InvalidDataException("Library signature nesting limit exceeded.");
        if (type is GenericParameter p && p.Type == GenericParameterType.Method)
            return method.GenericArguments[p.Position];
        if (type is GenericInstanceType generic)
        {
            var closed = new GenericInstanceType(generic.ElementType);
            foreach (var argument in generic.GenericArguments) closed.GenericArguments.Add(Close(argument, method, depth + 1));
            return closed;
        }
        if (type.ContainsGenericParameter) throw new InvalidDataException("Unsupported constructed library signature.");
        return type;
    }

    public static string QualifyHelpers(string text, string owner)
    {
        var helpers = Regex.Matches(text, @"(?m)^\.function (?:internal )?([^\(]+)\(").Select(m => m.Groups[1].Value)
            .Where(h => !h.StartsWith(owner + ".", StringComparison.Ordinal)
                && !(owner == "System.Tasks.Task" && (h.StartsWith("System.Tasks.TaskOperators.", StringComparison.Ordinal) || h.StartsWith("System.Tasks.TaskResultOperators.", StringComparison.Ordinal)))).ToArray();
        // Adapters generated from open signatures must themselves declare the free
        // method parameters. Propagate through helper calls before qualifying names.
        var bodies = Regex.Matches(text, @"(?ms)^\.function (?:internal )?([^\(]+)\(.*?^\.end\r?$")
            .Where(m => helpers.Contains(m.Groups[1].Value))
            .ToDictionary(m => m.Groups[1].Value, m => m.Value);
        var parameters = helpers.ToDictionary(h => h, h => Regex.Matches(Regex.Replace(bodies[h], "\"(?:\\\\.|[^\"\\\\])*\"", ""), @"\bT[0-9]+\b")
            .Select(m => m.Value).ToHashSet());
        bool changed;
        do
        {
            changed = false;
            foreach (var helper in helpers)
                foreach (Match call in Regex.Matches(bodies[helper], @"(?m)^(?:call|ldftn) ([^\(]+)\("))
                    if (parameters.TryGetValue(call.Groups[1].Value, out var dependency))
                        foreach (var parameter in dependency)
                            changed |= parameters[helper].Add(parameter);
        } while (changed);
        foreach (var helper in helpers.Where(h => !h.StartsWith(owner + ".", StringComparison.Ordinal)
                && !(owner == "System.Tasks.Task" && (h.StartsWith("System.Tasks.TaskOperators.", StringComparison.Ordinal) || h.StartsWith("System.Tasks.TaskResultOperators.", StringComparison.Ordinal)))))
        {
            var generic = parameters[helper].Count == 0 ? "" : "<" + string.Join(',',
                parameters[helper].OrderBy(p => int.Parse(p[1..]))) + ">";
            text = Regex.Replace(text, @"(?m)^(\.function (?:internal )?|call |ldftn )" + Regex.Escape(helper) + @"(?=\()",
                m => m.Groups[1].Value + "neoCLR.Library." + owner + "." + helper + generic);
        }
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
