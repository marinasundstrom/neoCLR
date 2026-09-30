using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Text.Json;

if (args.Length < 2) throw new ArgumentException("generate/read/rewrite path [output]");
if (args[0] == "generate")
{
    var assembly = new PersistedAssemblyBuilder(new AssemblyName("NeoMetadataContainerProbe"), typeof(object).Assembly);
    var module = assembly.DefineDynamicModule("Probe");
    var type = module.DefineType("Probe.Contracts", TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed);
    type.DefineField("Label", typeof(string), FieldAttributes.Public | FieldAttributes.Static);
    var answer = type.DefineMethod("Answer", MethodAttributes.Public | MethodAttributes.Static, typeof(int), Type.EmptyTypes);
    var il = answer.GetILGenerator();
    il.Emit(OpCodes.Ldc_I4_S, (sbyte)42);
    il.Emit(OpCodes.Ret);
    var greeting = type.DefineMethod("Greeting", MethodAttributes.Public | MethodAttributes.Static, typeof(string), Type.EmptyTypes);
    il = greeting.GetILGenerator();
    il.Emit(OpCodes.Ldstr, "neoCLR metadata — UTF-8 identifiers, CLI user strings");
    il.Emit(OpCodes.Ret);
    var identity = type.DefineMethod("Identity", MethodAttributes.Public | MethodAttributes.Static);
    var parameter = identity.DefineGenericParameters("T")[0];
    identity.SetReturnType(parameter);
    identity.SetParameters(parameter);
    il = identity.GetILGenerator();
    il.Emit(OpCodes.Ldarg_0);
    il.Emit(OpCodes.Ret);
    type.CreateType();
    var metadata = assembly.GenerateMetadata(out var bodies, out var fields);
    var builder = new ManagedPEBuilder(
        new PEHeaderBuilder(fileAlignment: 4096, sectionAlignment: 4096),
        new MetadataRootBuilder(metadata), bodies, mappedFieldData: fields,
        strongNameSignatureSize: 0);
    var image = new BlobBuilder();
    builder.Serialize(image);
    File.WriteAllBytes(args[1], image.ToArray());
    return;
}
if (args[0] == "rewrite")
{
    using var input = Mono.Cecil.AssemblyDefinition.ReadAssembly(args[1]);
    input.Write(args[2]);
    return;
}
if (args[0] != "read") throw new ArgumentException("unknown operation");
using var pe = new PEReader(File.OpenRead(args[1]));
var reader = pe.GetMetadataReader();
using var cecil = Mono.Cecil.AssemblyDefinition.ReadAssembly(args[1]);
var srm = new {
    Assembly = reader.GetString(reader.GetAssemblyDefinition().Name),
    Mvid = reader.GetGuid(reader.GetModuleDefinition().Mvid),
    Types = reader.TypeDefinitions.Select(handle => {
        var type = reader.GetTypeDefinition(handle);
        return new {
            Token = MetadataTokens.GetToken(handle),
            Name = reader.GetString(type.Name), Namespace = reader.GetString(type.Namespace),
            Fields = type.GetFields().Select(fieldHandle => {
                var field = reader.GetFieldDefinition(fieldHandle);
                return new { Token = MetadataTokens.GetToken(fieldHandle), Name = reader.GetString(field.Name),
                    Signature = Convert.ToHexString(reader.GetBlobBytes(field.Signature)) };
            }).ToArray(),
            Methods = type.GetMethods().Select(methodHandle => {
                var method = reader.GetMethodDefinition(methodHandle);
                return new { Token = MetadataTokens.GetToken(methodHandle), Name = reader.GetString(method.Name),
                    Signature = Convert.ToHexString(reader.GetBlobBytes(method.Signature)),
                    Rva = method.RelativeVirtualAddress,
                    IL = Convert.ToHexString(pe.GetMethodBody(method.RelativeVirtualAddress).GetILBytes()!),
                    Generics = method.GetGenericParameters().Select(h => reader.GetString(reader.GetGenericParameter(h).Name)).ToArray() };
            }).ToArray()
        };
    }).ToArray()
};
var other = new {
    Assembly = cecil.Name.Name, Mvid = cecil.MainModule.Mvid,
    Types = cecil.MainModule.Types.Select(type => new {
        Token = type.MetadataToken.ToInt32(), type.FullName,
        Fields = type.Fields.Select(f => new { Token = f.MetadataToken.ToInt32(), f.FullName }).ToArray(),
        Methods = type.Methods.Select(m => new {
            Token = m.MetadataToken.ToInt32(), m.FullName,
            IL = m.Body.Instructions.Select(i => i.ToString()).ToArray()
        }).ToArray()
    }).ToArray()
};
Console.WriteLine(JsonSerializer.Serialize(new {
    Runtime = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription,
    SrmVersion = typeof(PEReader).Assembly.GetName().Version!.ToString(),
    CecilVersion = typeof(Mono.Cecil.AssemblyDefinition).Assembly.GetName().Version!.ToString(),
    Srm = srm, Cecil = other
}, new JsonSerializerOptions { WriteIndented = true }));
