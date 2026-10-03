using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class MetadataLoadContextChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        AssemblyIdentity Id(string name, int revision = 1) => new(name, new Version(revision, 0, 0, 0));
        AssemblyDefinition Snapshot(string name, params string[] dependencies)
        {
            var builder = new AssemblyBuilder(Id(name), core);
            var type = builder.AddClass("Example", "Item");
            foreach (var dependency in dependencies)
            {
                var reference = builder.CreateTypeReference(Id(dependency), core, new string('a', 64), "Example", "Item");
                var field = type.AddField(dependency, reference);
                var getter = type.AddInstanceMethod("get_" + dependency, new MethodSignature(reference, []));
                getter.LoadArgument(0); getter.LoadField(field); getter.Return();
                type.AddProperty(dependency, reference, getter);
            }
            return AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(builder.WriteNativeAssembly(), core));
        }
        var a = Snapshot("A", "B", "C");
        var b = Snapshot("B", "D");
        var c = Snapshot("C", "D");
        var d = Snapshot("D", "A"); // Legal assembly cycle, not a declaration cycle.
        var context = new MetadataLoadContext([a, b, c, d, a]);
        Check(context.Assemblies.Count == 4, "same snapshot registration is idempotent");
        var view = context.Resolve(a.Identity);
        var left = view.ReferencedAssemblies[0].ReferencedAssemblies.Single();
        var right = view.ReferencedAssemblies[1].ReferencedAssemblies.Single();
        Check(ReferenceEquals(left, right) && ReferenceEquals(left.ReferencedAssemblies.Single(), view), "diamond and cycle identity");
        var nominal = context.Resolve(b.MainModule.TypeReferences.Single());
        Check(ReferenceEquals(nominal, context.Resolve(d.Identity).GetTypes().Single()), "external type shares local facade");
        Check(ReferenceEquals(nominal.Module.Assembly, left) && nominal.FullName == "Example.Item" && nominal.IsNominalType, "provenance and nominal shape");
        Parallel.For(0, 32, _ => Check(ReferenceEquals(context.Resolve(b.MainModule.TypeReferences.Single()), nominal), "concurrent canonical identity"));
        Check(!ReferenceEquals(new MetadataLoadContext([a, b, c, d]).Resolve(d.Identity).GetTypes().Single(), nominal), "context isolation");
        Reject<InvalidDataException>(() => new MetadataLoadContext([a, Snapshot("A")]));
        Reject<InvalidDataException>(() => new MetadataLoadContext([b]).Resolve(b.MainModule.TypeReferences.Single()));
        Reject<InvalidDataException>(() => context.Resolve(Id("A", 2)));
        Reject<InvalidDataException>(() => context.Resolve(Snapshot("B", "D").MainModule.TypeReferences.Single()));
        Reject<NotSupportedException>(() => ((IList<NominalTypeInfo>)view.GetTypes()).Clear());
        Check(ReferenceEquals(context.Resolve(b.Identity).GetTypes().Single().GetProperties().Single().PropertyType, nominal), "external property dependency canonical identity");
        var missing = new MetadataLoadContext([a]);
        Reject<InvalidDataException>(() => missing.Resolve(a.Identity).GetTypes().Single().GetProperties());
        Reject<InvalidDataException>(() => _ = missing.Resolve(a.Identity).ReferencedAssemblies);
        Check(new MetadataLoadContext([a, b, c, d]).Resolve(a.Identity).ReferencedAssemblies.Count == 2, "failed context does not poison next context");
        var revisionBuilder = new AssemblyBuilder(Id("A", 2), core);
        revisionBuilder.AddClass("Example", "Item");
        var revision = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(revisionBuilder.WriteNativeAssembly(), core));
        var versions = new MetadataLoadContext([a, revision]);
        Check(!ReferenceEquals(versions.Resolve(a.Identity).GetTypes().Single(), versions.Resolve(revision.Identity).GetTypes().Single()), "same name different version remains distinct");
        var genericBuilder = new AssemblyBuilder(Id("GenericViews"), core);
        var box = genericBuilder.AddGenericClass("Example", "Box", ["T"]);
        var stored = box.AddField("Value", SignatureType.TypeParameter(0));
        box.AddField("Items", SignatureType.ArrayOf(SignatureType.TypeParameter(0)));
        box.AddField("Next", box.MakeGenericInstance(SignatureType.TypeParameter(0)));
        var get = box.AddInstanceMethod("Get", new MethodSignature(SignatureType.TypeParameter(0), []));
        get.LoadArgument(0); get.LoadField(stored); get.Return();
        var mixed = box.AddMethod("Mixed", new MethodSignature(SignatureType.TypeParameter(0),
            [SignatureType.MethodParameter(0), SignatureType.TypeParameter(0)], ["U"]));
        mixed.LoadArgument(1); mixed.Return();
        box.AddProperty("Current", SignatureType.TypeParameter(0), get);
        var empty = box.AddMethod("get_Empty", new MethodSignature(SignatureType.TypeParameter(0), []));
        empty.LoadDefault(SignatureType.TypeParameter(0)); empty.Return();
        box.AddProperty("Empty", SignatureType.TypeParameter(0), empty);
        var setIndex = box.AddInstanceMethod("set_Item", new MethodSignature(PrimitiveType.Void,
            [SignatureType.TypeParameter(0), SignatureType.TypeParameter(0)]));
        setIndex.LoadArgument(0); setIndex.LoadArgument(2); setIndex.StoreField(stored); setIndex.Return();
        box.AddProperty("Item", SignatureType.TypeParameter(0), null, setIndex);
        var root = genericBuilder.AddGenericInterface("Example", "Root", ["T"]);
        var middle = genericBuilder.AddGenericInterface("Example", "Middle", ["T"]);
        middle.AddBaseInterface(root.MakeGenericInstance(SignatureType.ArrayOf(SignatureType.TypeParameter(0))));
        box.AddInterfaceImplementation(middle.MakeGenericInstance(SignatureType.TypeParameter(0)));
        var sibling = genericBuilder.AddGenericInterface("Example", "Sibling", ["T"]);
        sibling.AddBaseInterface(root.MakeGenericInstance(SignatureType.ArrayOf(SignatureType.TypeParameter(0))));
        var diamond = genericBuilder.AddGenericInterface("Example", "Diamond", ["T"]);
        diamond.AddBaseInterface(middle.MakeGenericInstance(SignatureType.TypeParameter(0)));
        diamond.AddBaseInterface(sibling.MakeGenericInstance(SignatureType.TypeParameter(0)));
        diamond.AddBaseInterface(root.MakeGenericInstance(PrimitiveType.Boolean));

        var identity = genericBuilder.AddFunction("Identity", new MethodSignature(SignatureType.ArrayOf(SignatureType.MethodParameter(0)),
            [SignatureType.ArrayOf(SignatureType.MethodParameter(0))], ["T"]));
        identity.LoadArgument(0); identity.Return();
        genericBuilder.AddGenericClass("Example", "Other", ["T"]);
        var genericSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(genericBuilder.WriteNativeAssembly(), core));
        var genericContext = new MetadataLoadContext([genericSnapshot]);
        var genericTypes = genericContext.Resolve(genericSnapshot.Identity).GetTypes();
        var boxView = genericTypes.Single(t => t.Name == "Box`1");
        var otherView = genericTypes.Single(t => t.Name == "Other`1");
        var intView = genericContext.ResolveSignature(PrimitiveType.Int32);
        var args = new[] { intView };
        var closed = boxView.MakeGenericType(args);
        args[0] = genericContext.ResolveSignature(PrimitiveType.Boolean);
        Check(ReferenceEquals(closed, boxView.MakeGenericType(intView)), "canonical construction and copied arguments");
        Check(ReferenceEquals(closed.GetFields()[0].FieldType, intView), "field substitution");
        Check(closed.GetFields()[1].FieldType is ArrayTypeInfo array && ReferenceEquals(array.ElementType, intView), "vector field substitution");
        Check(ReferenceEquals(closed.GetFields()[2].FieldType, closed), "recursive nominal field resolves without member expansion");
        Check(ReferenceEquals(boxView.GetFields()[0].FieldType, boxView.GetGenericArguments()[0]), "open definition remains open");
        Check(!ReferenceEquals(boxView.GetGenericArguments()[0], otherView.GetGenericArguments()[0]), "parameter owner identity");
        var forwarded = boxView.MakeGenericType(otherView.GetGenericArguments()[0]);
        Check(ReferenceEquals(forwarded.GetFields()[0].FieldType, otherView.GetGenericArguments()[0]), "simultaneous caller-scope substitution");
        Reject<ArgumentException>(() => boxView.MakeGenericType(nominal));
        Reject<ArgumentException>(() => boxView.MakeGenericType(genericContext.ResolveSignature(PrimitiveType.Void)));
        Reject<ArgumentException>(() => boxView.MakeGenericType());
        Reject<ArgumentException>(() => boxView.MakeGenericType(otherView));
        Reject<InvalidDataException>(() => genericContext.ResolveSignature(SignatureType.TypeParameter(0)));
        Reject<InvalidDataException>(() => genericContext.ResolveSignature(SignatureType.MethodParameter(0)));
        Reject<NotSupportedException>(() => ((IList<TypeInfo>)closed.TypeArguments).Clear());
        var getView = closed.GetMethods().Single(m => m.Name == "Get");
        Check(ReferenceEquals(getView.ReturnType, intView) && getView.GetParameters().Count == 0, "constructed owner method result");
        Check(ReferenceEquals(getView, closed.GetMethods().Single(m => m.Name == "Get")), "canonical method view");
        var current = closed.GetProperties().Single(p => p.Name == "Current");
        Check(ReferenceEquals(current.PropertyType, intView) && ReferenceEquals(current.DeclaringType, closed) &&
            ReferenceEquals(current.GetMethod, getView) && current.SetMethod is null && !current.IsStatic, "closed property and canonical accessor");
        Check(ReferenceEquals(current, closed.GetProperties().Single(p => p.Name == "Current")), "stable property view");
        var emptyView = closed.GetProperties().Single(p => p.Name == "Empty");
        Check(emptyView.IsStatic && emptyView.GetMethod!.IsStatic && ReferenceEquals(emptyView.PropertyType, intView), "static generic property");
        Parallel.For(0, 32, _ => Check(ReferenceEquals(current, closed.GetProperties().Single(p => p.Name == "Current")), "concurrent property identity"));
        var indexed = closed.GetProperties().Single(p => p.Name == "Item");
        Check(indexed.GetMethod is null && indexed.SetMethod is not null && indexed.IndexParameterTypes.Count == 1 &&
            ReferenceEquals(indexed.IndexParameterTypes[0], intView) && ReferenceEquals(indexed.PropertyType, intView), "setter-only generic index excludes value");
        Check(ReferenceEquals(indexed.SetMethod, closed.GetMethods().Single(m => m.Name == "set_Item")), "canonical setter");
        Check(ReferenceEquals(forwarded.GetProperties()[0].PropertyType, otherView.GetGenericArguments()[0]), "caller-scoped property");
        Check(ReferenceEquals(boxView.GetProperties()[0].PropertyType, boxView.GetGenericArguments()[0]), "open property scope");
        var middleView = (ConstructedTypeInfo)closed.GetDeclaredInterfaces().Single();
        Check(ReferenceEquals(middleView.TypeArguments[0], intView), "closed direct interface substitution");
        var rootView = (ConstructedTypeInfo)middleView.GetDeclaredInterfaces().Single();
        Check(rootView.TypeArguments[0] is ArrayTypeInfo interfaceArray && ReferenceEquals(interfaceArray.ElementType, intView), "interface edge composes vector substitution");
        Check(ReferenceEquals(middleView, genericTypes.Single(t => t.Name == "Middle`1").MakeGenericType(intView)), "canonical interface construction");
        Check(ReferenceEquals(((ConstructedTypeInfo)boxView.GetDeclaredInterfaces().Single()).TypeArguments[0], boxView.GetGenericArguments()[0]), "open interface owner scope");
        Check(ReferenceEquals(((ConstructedTypeInfo)forwarded.GetDeclaredInterfaces().Single()).TypeArguments[0], otherView.GetGenericArguments()[0]), "caller-scoped interface argument");
        var closure = closed.GetInterfaces();
        Check(closure.Count == 2 && ReferenceEquals(closure[0], middleView) && ReferenceEquals(closure[1], rootView), "transitive interface order and identity");
        var diamondView = genericTypes.Single(t => t.Name == "Diamond`1").MakeGenericType(intView);
        var diamondClosure = diamondView.GetInterfaces();
        Check(diamondClosure.Count == 4 && ReferenceEquals(diamondClosure[0], middleView) &&
            ReferenceEquals(diamondClosure[1], rootView) && ((ConstructedTypeInfo)diamondClosure[2]).Definition.Name == "Sibling`1" &&
            ((ConstructedTypeInfo)diamondClosure[3]).TypeArguments[0] is PrimitiveTypeInfo { Kind: PrimitiveType.Boolean }, "diamond dedup retains different constructions");
        Check(otherView.GetInterfaces().Count == 0, "empty interface closure");
        Check(boxView.GetInterfaces().Count == 2 && forwarded.GetInterfaces().Count == 2, "open and caller-scoped closure");
        Parallel.For(0, 32, _ => Check(ReferenceEquals(diamondView.GetInterfaces(), diamondClosure), "stable concurrent interface closure"));
        Reject<NotSupportedException>(() => ((IList<TypeInfo>)diamondClosure).Clear());
        Reject<NotSupportedException>(() => ((IList<TypeInfo>)indexed.IndexParameterTypes).Clear());
        Reject<NotSupportedException>(() => ((IList<TypeInfo>)closed.GetDeclaredInterfaces()).Clear());
        Reject<NotSupportedException>(() => ((IList<NeoCLR.Metadata.Experimental.Introspection.PropertyInfo>)closed.GetProperties()).Clear());
        var mixedView = closed.GetMethods().Single(m => m.Name == "Mixed");
        var mixedParameters = mixedView.GetParameters();
        Check(ReferenceEquals(mixedParameters[0].ParameterType, mixedView.GetGenericArguments()[0]) &&
            ReferenceEquals(mixedParameters[1].ParameterType, intView) && ReferenceEquals(mixedView.ReturnType, intView), "separate owner and method scopes");
        Check(mixedView.GetGenericArguments()[0] is MethodGenericParameterTypeInfo methodParameter &&
            ReferenceEquals(methodParameter.DeclaringMethod, mixedView) && methodParameter.Position == 0, "method scope provenance");
        var boolView = genericContext.ResolveSignature(PrimitiveType.Boolean);
        var methodArgs = new[] { boolView };
        var mixedClosed = mixedView.MakeGenericMethod(methodArgs);
        methodArgs[0] = intView;
        Check(mixedView.IsGenericMethodDefinition && !mixedClosed.IsGenericMethodDefinition &&
            ReferenceEquals(mixedClosed.GetGenericMethodDefinition(), mixedView) &&
            ReferenceEquals(mixedView.GetGenericMethodDefinition(), mixedView), "generic method definition provenance");
        Check(ReferenceEquals(mixedClosed.ReturnType, intView) &&
            ReferenceEquals(mixedClosed.GetParameters()[0].ParameterType, boolView) &&
            ReferenceEquals(mixedClosed.GetParameters()[1].ParameterType, intView) &&
            ReferenceEquals(mixedClosed.GetParameters()[0].DeclaringMethod, mixedClosed), "simultaneous owner and method construction");
        Check(ReferenceEquals(mixedClosed, mixedView.MakeGenericMethod(boolView)), "canonical method construction and copied arguments");
        Check(!ReferenceEquals(mixedClosed, boxView.GetMethods().Single(m => m.Name == "Mixed").MakeGenericMethod(boolView)), "constructed method owner identity");
        Check(ReferenceEquals(mixedView.MakeGenericMethod(otherView.GetGenericArguments()[0]).GetParameters()[0].ParameterType,
            otherView.GetGenericArguments()[0]), "caller type parameter survives method construction");
        Check(ReferenceEquals(mixedView.MakeGenericMethod(mixedView.GetGenericArguments()[0]).GetParameters()[0].ParameterType,
            mixedView.GetGenericArguments()[0]), "simultaneous method scope does not rebind supplied parameters");
        Parallel.For(0, 32, _ => Check(ReferenceEquals(mixedClosed, mixedView.MakeGenericMethod(boolView)), "concurrent method construction identity"));
        var oracle = typeof(GenericMethodOracle<int>).GetMethod("Mixed")!.MakeGenericMethod(typeof(bool));
        Check(oracle.ReturnType == typeof(int) && oracle.GetParameters()[0].ParameterType == typeof(bool) &&
            oracle.GetParameters()[1].ParameterType == typeof(int), "CLR comparison for independent owner/method construction");
        TypeInfo deepArgument = intView;
        for (var i = 0; i < 16; i++) deepArgument = boxView.MakeGenericType(deepArgument);
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod(deepArgument));
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod());
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod(nominal));
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod(otherView));
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod(genericContext.ResolveSignature(PrimitiveType.Void)));
        Reject<ArgumentException>(() => mixedView.MakeGenericMethod(new TypeInfo[] { null! }));
        Reject<ArgumentNullException>(() => mixedView.MakeGenericMethod(null!));
        Reject<InvalidOperationException>(() => mixedClosed.MakeGenericMethod(intView));
        Reject<InvalidOperationException>(() => getView.MakeGenericMethod(intView));
        Reject<InvalidOperationException>(() => getView.GetGenericMethodDefinition());
        Reject<NotSupportedException>(() => ((IList<TypeInfo>)mixedClosed.GetGenericArguments()).Clear());
        var moduleView = genericContext.Resolve(genericSnapshot.Identity).GetModules().Single();
        var functionView = moduleView.GetFunctions().Single();
        Check(functionView.DeclaringType is null && functionView.ReturnType is ArrayTypeInfo functionArray &&
            ReferenceEquals(functionArray.ElementType, functionView.GetGenericArguments()[0]) &&
            ReferenceEquals(functionView.ReturnType, functionView.GetParameters()[0].ParameterType), "namespace generic vector signature");
        Check(ReferenceEquals(functionView, genericContext.Resolve(genericSnapshot.MainModule.Functions.Single())), "function resolution identity");
        var closedFunction = functionView.MakeGenericMethod(intView);
        Check(closedFunction.DeclaringType is null && closedFunction.ReturnType is ArrayTypeInfo closedVector &&
            ReferenceEquals(closedVector.ElementType, intView) && ReferenceEquals(closedFunction.ReturnType, closedFunction.GetParameters()[0].ParameterType), "constructed namespace function vector signature");
        Check(ReferenceEquals(genericContext.ResolveSignature(SignatureType.MethodParameter(0), boxView.GetGenericArguments(), [intView]), intView), "explicit method scope substitution");
        Reject<InvalidDataException>(() => genericContext.ResolveSignature(SignatureType.MethodParameter(1), null, [intView]));
        Reject<ArgumentException>(() => genericContext.ResolveSignature(SignatureType.MethodParameter(0), null, [nominal]));
        Reject<NotSupportedException>(() => ((IList<NeoCLR.Metadata.Experimental.Introspection.ParameterInfo>)mixedParameters).Clear());
        var cli = new AssemblyBuilder(Id("Cli"), core);
        cli.AddGenericClass("Example", "Box", ["T"]);
        var cliSnapshot = AssemblyDefinition.ReadAssembly(cli.Write(), expectedExtended: false);
        var cliContext = new MetadataLoadContext([cliSnapshot]);
        var cliType = cliContext.Resolve(cliSnapshot.Identity).GetTypes().Single();
        Check(cliType.Name == "Box`1" && cliType.GenericArity == 1 && cliType.DeclaringType is null, "CLI definition facade excludes module pseudo-type");
        Check(ReferenceEquals(cliContext.Resolve(cliSnapshot.MainModule.Types.Single(t => t.Name == "Box`1").ToReference()), cliType), "CLI local resolution");
        Reject<NotSupportedException>(() => cliType.GetDeclaredInterfaces());
        Reject<NotSupportedException>(() => cliType.GetInterfaces());
        Reject<ArgumentException>(() => new MetadataLoadContext([cli.Definition]));
    }
    private sealed class GenericMethodOracle<T>
    {
        public T Mixed<U>(U first, T second) => second;
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
