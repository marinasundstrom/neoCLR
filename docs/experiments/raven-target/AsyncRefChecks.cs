using Mono.Cecil;

static class AsyncRefChecks
{
    public static void Run(string path)
    {
        using var resolver = new DefaultAssemblyResolver();
        resolver.AddSearchDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
        using var core = ModuleDefinition.ReadModule(path, new ReaderParameters { AssemblyResolver = resolver });
        using var app = ModuleDefinition.CreateModule("AsyncContractProbe", ModuleKind.Dll);
        var state = new TypeDefinition("", "State", TypeAttributes.Public | TypeAttributes.Sealed,
            app.ImportReference(core.GetType("System.ValueType")));
        state.Interfaces.Add(new InterfaceImplementation(app.ImportReference(core.GetType("System.Runtime.CompilerServices.IAsyncStateMachine"))));
        app.Types.Add(state);
        ApplicationTypes.Reset(app);
        var definition = core.GetType("System.Runtime.CompilerServices.AsyncTaskMethodBuilder`1");
        var owner = new GenericInstanceType(definition);
        owner.GenericArguments.Add(core.TypeSystem.Int32);
        var start = definition.Methods.Single(m => m.Name == "Start");
        GenericInstanceMethod Call(MethodDefinition method, params TypeReference[] arguments)
        {
            var reference = new MethodReference(method.Name, method.ReturnType, owner) {
                HasThis = true, CallingConvention = MethodCallingConvention.Generic
            };
            foreach (var parameter in method.GenericParameters)
                reference.GenericParameters.Add(new GenericParameter(parameter.Name, reference));
            foreach (var parameter in method.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.ParameterType));
            var call = new GenericInstanceMethod(reference);
            foreach (var argument in arguments) call.GenericArguments.Add(argument);
            return call;
        }
        string Map(TypeReference type) => type == state ? "State" : GenericUnionBindings.Type(type)
            ?? throw new InvalidDataException("Unsupported probe type.");
        void Reject(GenericInstanceMethod call, MethodDefinition method)
        {
            try { AsyncBindings.BindByRef(call, method, "test", Map); }
            catch (InvalidDataException) { return; }
            throw new Exception("Malformed ref protocol was admitted.");
        }
        var valid = Call(start, state);
        var binding = AsyncBindings.BindByRef(valid, start, "test", Map)!;
        if (binding.Result != "noresult" || binding.Arguments.Length != 2 || binding.Arguments[1] != "State&")
            throw new Exception("Ref startup contract changed.");
        var wrongReturn = Call(start, state);
        wrongReturn.ElementMethod.ReturnType = core.TypeSystem.Int32;
        Reject(wrongReturn, start);
        var copiedParameter = Call(start, state);
        copiedParameter.ElementMethod.Parameters[0].ParameterType = start.GenericParameters[0];
        Reject(copiedParameter, start);
        Reject(Call(start, core.TypeSystem.Int32), start);
        var task = new GenericInstanceType(core.GetType("System.Tasks.Task`1"));
        task.GenericArguments.Add(core.TypeSystem.Int32);
        var awaitMethod = definition.Methods.Single(m => m.Name == "AwaitOnCompleted");
        var awaitBinding = AsyncBindings.BindByRef(Call(awaitMethod, task, state), awaitMethod, "test", Map)!;
        if (awaitBinding.Arguments.Length != 3 || awaitBinding.Arguments[1] != "System.Tasks.Task<Int32>&"
            || awaitBinding.Arguments[2] != "State&") throw new Exception("Ref await contract changed.");
        Reject(Call(awaitMethod, core.GetType("System.Object"), state), awaitMethod);
        Console.WriteLine("Ref async metadata: 2 valid and 4 rejected contracts passed");
    }
}
