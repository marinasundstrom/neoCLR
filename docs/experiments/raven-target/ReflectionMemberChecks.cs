using Mono.Cecil;

static class ReflectionMemberChecks
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var path = Path.Combine(output, "NeoCLR.CoreProbe.dll");
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path);
        var module = core.MainModule;
        ErrorBindings.Reset(module);
        var count = 0;
        void Check(bool valid) { if (!valid) throw new Exception("Reflection contract check failed"); count++; }
        void Reject(Action action) {
            try { action(); } catch (InvalidDataException) { count++; return; }
            throw new Exception("Reflection malformed signature admitted");
        }
        ReflectionBindings.Validate(module);
        var constructor = module.GetType("System.Introspection.ConstructorInfo");
        Check(constructor.IsInterface && constructor.Interfaces.Single().InterfaceType.FullName == "System.Introspection.MemberInfo");
        Check(constructor.Properties.All(p => p.Name != "ReturnType"));
        foreach (var name in new[] {"System.Introspection.ConstructorInfo", "System.Runtime.Reflection.ConstructorReflectionExtensions", "System.Runtime.Reflection.MethodReflectionExtensions", "System.Runtime.Reflection.FieldReflectionExtensions"})
            foreach (var method in module.GetType(name).Methods.Where(m => m.IsPublic)) {
                Check(ReflectionBindings.Bind(method,method) is not null);
                var wrong = new MethodReference(method.Name,module.TypeSystem.Int32,method.DeclaringType) {HasThis=method.HasThis};
                foreach (var p in method.Parameters) wrong.Parameters.Add(new ParameterDefinition(p.ParameterType));
                if (method.ReturnType.MetadataType != MetadataType.Int32) Reject(()=>ReflectionBindings.Bind(wrong,method));
            }
        foreach (var method in module.GetType("System.Introspection.TypeInfo").Methods.Where(m=>m.Name=="GetConstructors"))
            Check(ReflectionBindings.Bind(method,method)?.Result=="System.Collections.Sequence<System.Introspection.ConstructorInfo>");
        var activation=module.GetType("System.Runtime.Reflection.TypeReflectionExtensions").Methods.Single(m=>m.Name=="CreateInstance" && m.HasGenericParameters);
        Check(ReflectionGenericBindings.IsGeneric(activation));
        Check(activation.Parameters[1].CustomAttributes.Any(a=>a.AttributeType.FullName=="System.ParamArrayAttribute"));
        var generic=new GenericInstanceMethod(activation);generic.GenericArguments.Add(module.TypeSystem.Object);
        Check(ReflectionGenericBindings.Bind(generic,activation,false,false).Result=="System.Result<System.Object,System.Runtime.Reflection.ReflectionError>");
        var old=activation.ReturnType;activation.ReturnType=module.TypeSystem.Object;
        Check(!ReflectionGenericBindings.IsGeneric(activation));
        Reject(()=>ReflectionGenericBindings.Bind(generic,activation,false,false));activation.ReturnType=old;
        Console.WriteLine($"{count} reflection member signature checks passed");
    }
}
