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
                type.AddField(dependency, builder.CreateTypeReference(Id(dependency), core, new string('a', 64), "Example", "Item"));
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
        var missing = new MetadataLoadContext([a]);
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
        var mixedView = closed.GetMethods().Single(m => m.Name == "Mixed");
        var mixedParameters = mixedView.GetParameters();
        Check(ReferenceEquals(mixedParameters[0].ParameterType, mixedView.GetGenericArguments()[0]) &&
            ReferenceEquals(mixedParameters[1].ParameterType, intView) && ReferenceEquals(mixedView.ReturnType, intView), "separate owner and method scopes");
        Check(mixedView.GetGenericArguments()[0] is MethodGenericParameterTypeInfo methodParameter &&
            ReferenceEquals(methodParameter.DeclaringMethod, mixedView) && methodParameter.Position == 0, "method scope provenance");
        var moduleView = genericContext.Resolve(genericSnapshot.Identity).GetModules().Single();
        var functionView = moduleView.GetFunctions().Single();
        Check(functionView.DeclaringType is null && functionView.ReturnType is ArrayTypeInfo functionArray &&
            ReferenceEquals(functionArray.ElementType, functionView.GetGenericArguments()[0]) &&
            ReferenceEquals(functionView.ReturnType, functionView.GetParameters()[0].ParameterType), "namespace generic vector signature");
        Check(ReferenceEquals(functionView, genericContext.Resolve(genericSnapshot.MainModule.Functions.Single())), "function resolution identity");
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
        Reject<ArgumentException>(() => new MetadataLoadContext([cli.Definition]));
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
