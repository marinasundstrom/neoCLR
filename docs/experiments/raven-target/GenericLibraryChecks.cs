using Mono.Cecil;
using Mono.Cecil.Cil;

// Test-only reference contract. This is not an addition to the System API catalog.
static class GenericLibraryChecks
{
    public static void WriteCore(string path)
    {
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path, new ReaderParameters { InMemory = true });
        var module = core.MainModule;
        TypeReference Primitive(string name) => module.Types.SelectMany(t => t.Methods)
            .SelectMany(m => m.Parameters.Select(p => p.ParameterType).Append(m.ReturnType))
            .First(t => t.FullName == "System." + name);
        var owner = new TypeDefinition("Probe.Generic", "NamespaceMembers",
            TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed, module.GetType("System.Object"));
        module.Types.Add(owner);
        owner.CustomAttributes.Add(new CustomAttribute(module.GetType("System.Runtime.CompilerServices.TopLevelAttribute").Methods.Single(m => m.IsConstructor && !m.HasParameters)));
        var choose = new MethodDefinition("Choose", MethodAttributes.Public | MethodAttributes.Static, Primitive("Int32"));
        owner.Methods.Add(choose);
        var parameter = new GenericParameter("Value", choose);
        choose.GenericParameters.Add(parameter);
        choose.CallingConvention = MethodCallingConvention.Generic;
        choose.ReturnType = parameter;
        choose.Parameters.Add(new ParameterDefinition("first", ParameterAttributes.None, Primitive("Boolean")));
        choose.Parameters.Add(new ParameterDefinition("left", ParameterAttributes.None, parameter));
        choose.Parameters.Add(new ParameterDefinition("right", ParameterAttributes.None, parameter));
        choose.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_1));
        choose.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
        foreach (var (name, type) in new[] { ("ChooseInt", Primitive("Int32")), ("ChooseText", Primitive("String")) })
        {
            var method = new MethodDefinition(name, MethodAttributes.Public | MethodAttributes.Static, type);
            owner.Methods.Add(method);
            method.Parameters.Add(new ParameterDefinition("first", ParameterAttributes.None, Primitive("Boolean")));
            method.Parameters.Add(new ParameterDefinition("left", ParameterAttributes.None, type));
            method.Parameters.Add(new ParameterDefinition("right", ParameterAttributes.None, type));
            method.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_1));
            method.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
        }
        var useUnit = new MethodDefinition("UseUnit", MethodAttributes.Public | MethodAttributes.Static, Primitive("Int32"));
        owner.Methods.Add(useUnit);
        useUnit.Body.Instructions.Add(Instruction.Create(OpCodes.Ldc_I4_0));
        useUnit.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
        foreach (var method in module.Types.SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in module.AssemblyReferences.ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Generic fixture introduced an external type scope.");
            module.AssemblyReferences.Remove(reference);
        }
        core.Write(path);
    }
}
