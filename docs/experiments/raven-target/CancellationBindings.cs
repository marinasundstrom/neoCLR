using Mono.Cecil;

// Application contracts over invocation-local request state; no scheduler handles.
static class CancellationBindings
{
    public const string Token = "System.Concurrency.CancellationToken";
    public const string Source = "System.Concurrency.CancellationTokenSource";
    public const string Registration = "System.Concurrency.CancellationRegistration";
    public static readonly string[] Names = [Token, Source, Registration];
    public static bool IsName(string name) => Names.Contains(name);
    public static bool IsReference(string name) => name is Source or Registration;
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && IsName(type.FullName) && type.IsValueType == (type.FullName == Token) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && left.IsValueType == right.IsValueType
        && (RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right)
            || ApplicationTypes.IsLibrary(left) && RuntimeSignatures.IsCore(right.Scope));
    public const string Declarations = "\n" + """
        #nullable enable annotations
        namespace Concurrency {
            public struct CancellationToken {
                private CancellationTokenSource? source;
                public CancellationToken() { source = null; }
                public CancellationToken(CancellationTokenSource source) { this.source = source; }
                public static CancellationToken None => default;
                public bool CanBeCancelled => default;
                public bool IsCancellationRequested => default;
                public CancellationRegistration Register(Func<PropagationUnit> callback) => default;
            }
            public sealed class CancellationTokenSource : Disposable {
                public CancellationTokenSource() { }
                public CancellationToken Token => default;
                public bool IsCancellationRequested => default;
                public bool IsDisposed => default;
                public void Cancel() { }
                public void Dispose() { }
                public CancellationRegistration Register(Func<PropagationUnit> callback) => default;
                public void Remove(CancellationRegistration registration) { }
            }
            public sealed class CancellationRegistration : Disposable {
                public CancellationRegistration(CancellationTokenSource? owner, Option<Func<PropagationUnit>> callback, CancellationRegistration? next) { }
                public void Dispose() { }
                public CancellationRegistration? Next => default;
                public CancellationRegistration? TakeNext() => default;
                public void SetNext(CancellationRegistration? value) { }
                public void Invoke() { }
            }
        }
        #nullable restore annotations
        """ + "\n";
    public static void Project(ModuleDefinition module)
    {
        var token = module.GetType(Token);
        token.PackingSize = -1;
        token.ClassSize = -1;
        if (!token.HasFields) token.Fields.Add(new FieldDefinition("source", FieldAttributes.Private, module.GetType(Source)));
        foreach (var name in Names)
            foreach (var method in module.GetType(name).Methods)
                if (name == Token && method.IsConstructor && method.Parameters.Count != 0
                    || name == Source && method.Name is "get_IsDisposed" or "Register" or "Remove"
                    || name == Registration && method.Name != "Dispose")
                    method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
    }
    public static bool IsTokenLayout(TypeDefinition type)
    {
        if (type.FullName != Token) return false;
        if (!type.IsValueType || !type.IsSequentialLayout || type.Fields.Count != 1
            || type.Fields[0].Name != "source" || !type.Fields[0].IsPrivate || type.Fields[0].IsStatic
            || type.Fields[0].FieldType.FullName != Source || type.Fields[0].FieldType.IsValueType)
            throw new InvalidDataException("CancellationToken must contain exactly one source reference.");
        return true;
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowInternal: library);
        var expected = (owner, definition.Name) switch {
            (Token, ".ctor") when args.Length == 0 => ("", "noresult", false),
            (Token, ".ctor") when library => (Source, "noresult", false),
            (Token, "get_None") => ("", Token, true),
            (Token, "get_CanBeCancelled" or "get_IsCancellationRequested") => ("", "Boolean", false),
            (Token, "Register") => ("System.Func<Void>", Registration, false),
            (Source, ".ctor") => ("", "noresult", false),
            (Source, "get_Token") => ("", Token, false),
            (Source, "get_IsCancellationRequested") => ("", "Boolean", false),
            (Source, "get_IsDisposed") when library => ("", "Boolean", false),
            (Source, "Register") when library => ("System.Func<Void>", Registration, false),
            (Source, "Remove") when library => (Registration, "noresult", false),
            (Source, "Cancel" or "Dispose") => ("", "noresult", false),
            (Registration, ".ctor") when library => ($"{Source},System.Option<System.Func<Void>>,{Registration}", "noresult", false),
            (Registration, "get_Next" or "TakeNext") when library => ("", Registration, false),
            (Registration, "SetNext") when library => (Registration, "noresult", false),
            (Registration, "Invoke") when library => ("", "noresult", false),
            (Registration, "Dispose") => ("", "noresult", false),
            _ => throw new InvalidDataException("Unsupported cancellation member.")
        };
        var disposable = definition.Name == "Dispose";
        if (definition.IsConstructor != construct || definition.IsStatic != expected.Item3 || reference.HasThis == expected.Item3
            || !(definition.IsPublic || library && definition.IsAssembly) || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || definition.IsVirtual != disposable || definition.IsFinal != disposable || definition.IsNewSlot != disposable
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported cancellation signature.");
        var signature = string.Join(',', args);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({signature})")
            : new(definition.IsStatic ? args : new[] { owner + (owner == Token ? "&" : "") }.Concat(args).ToArray(), result,
                $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({signature})");
    }
}
