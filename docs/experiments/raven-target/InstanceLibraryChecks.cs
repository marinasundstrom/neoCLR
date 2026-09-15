using Mono.Cecil;
using Mono.Cecil.Cil;

// Independent reference contract: no implementation fields, no executable stubs.
static class InstanceLibraryChecks
{
    public static void WriteCore(string path, bool generic = false, bool value = false, bool wide = false)
    {
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true, libraryBootstrap: generic);
        using var core = AssemblyDefinition.ReadAssembly(path, new ReaderParameters { InMemory = true });
        var module = core.MainModule;
        var type = new TypeDefinition("Probe", generic ? "Cell`1" : "Counter", TypeAttributes.Public | TypeAttributes.Sealed | (value ? TypeAttributes.SequentialLayout : 0), module.GetType(value ? "System.ValueType" : "System.Object"));
        module.Types.Add(type);
        TypeReference Primitive(string name) => module.Types.SelectMany(t => t.Methods)
            .SelectMany(m => m.Parameters.Select(p => p.ParameterType).Append(m.ReturnType))
            .First(t => t.FullName == "System." + name);
        MethodDefinition Method(string name, TypeReference result, params (string, TypeReference)[] parameters)
        {
            var method = new MethodDefinition(name, MethodAttributes.Public | MethodAttributes.HideBySig, result);
            type.Methods.Add(method);
            foreach (var (parameter, shape) in parameters)
                method.Parameters.Add(new ParameterDefinition(parameter, ParameterAttributes.None, shape));
            // Deliberately unusable: accidentally executing this reference body must fault.
            method.Body.Instructions.Add(Instruction.Create(OpCodes.Ldnull));
            method.Body.Instructions.Add(Instruction.Create(OpCodes.Throw));
            return method;
        }
        TypeReference number = Primitive(wide ? "Int64" : "Int32");
        if (value) type.Fields.Add(new FieldDefinition("stored", FieldAttributes.Private, number));
        TypeReference self = type;
        if (generic)
        {
            var parameter = new GenericParameter("Item", type);
            type.GenericParameters.Add(parameter);
            number = parameter;
            var constructed = new GenericInstanceType(type);
            constructed.GenericArguments.Add(parameter);
            self = constructed;
            var iterable = new GenericInstanceType(module.GetType("System.Collections.Iterable`1"));
            iterable.GenericArguments.Add(parameter);
            type.Interfaces.Add(new InterfaceImplementation(iterable));
            Method("AsIterable", iterable);
            Method("Copy", self);
            var iterator = new GenericInstanceType(module.GetType("System.Collections.Iterator`1"));
            iterator.GenericArguments.Add(parameter);
            Method("GetIterator", iterator).Attributes |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
        }
        var noResult = Primitive("Void");
        Method(".ctor", noResult, ("value", number)).Attributes |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
        Method(generic ? "Set" : "Add", noResult, (generic ? "value" : "amount", number));
        Method("CopyFrom", noResult, ("other", self));
        Method("Self", self);
        if (value) Method("Copy", self);
        var getter = Method("get_Value", number);
        getter.Attributes |= MethodAttributes.SpecialName;
        type.Properties.Add(new PropertyDefinition("Value", PropertyAttributes.None, number) { GetMethod = getter });
        foreach (var method in module.Types.SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in module.AssemblyReferences.ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Instance fixture introduced an external type scope.");
            module.AssemblyReferences.Remove(reference);
        }
        core.Write(path);
    }
}
