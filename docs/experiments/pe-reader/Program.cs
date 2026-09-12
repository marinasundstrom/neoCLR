using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json;

var assembly = new PersistedAssemblyBuilder(new AssemblyName("PeReaderProbe"), typeof(object).Assembly);
var module = assembly.DefineDynamicModule("Probe");
var type = module.DefineType("Calls", TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed);
var answer = type.DefineMethod("Answer", MethodAttributes.Public | MethodAttributes.Static, typeof(int), Type.EmptyTypes);
var il = answer.GetILGenerator();
il.Emit(OpCodes.Ldc_I4_S, (sbyte)42);
il.Emit(OpCodes.Ret);
var local = type.DefineMethod("WithLocal", MethodAttributes.Public | MethodAttributes.Static, typeof(int), Type.EmptyTypes);
il = local.GetILGenerator();
il.DeclareLocal(typeof(int));
il.Emit(OpCodes.Call, answer);
il.Emit(OpCodes.Stloc_0);
il.Emit(OpCodes.Ldloc_0);
il.Emit(OpCodes.Ret);
type.CreateType();
using var stream = new MemoryStream();
assembly.Save(stream);
var image = stream.ToArray();
using var reader = new PEReader(new MemoryStream(image));
var metadata = reader.GetMetadataReader();
Console.WriteLine(JsonSerializer.Serialize(new {
    ImageHex = Convert.ToHexString(image),
    MetadataSize = reader.PEHeaders.CorHeader!.MetadataDirectory.Size,
    Methods = metadata.MethodDefinitions.Select(handle => {
        var method = metadata.GetMethodDefinition(handle);
        var body = reader.GetMethodBody(method.RelativeVirtualAddress);
        return new {
            Name = metadata.GetString(method.Name),
            Rva = method.RelativeVirtualAddress,
            body.MaxStack,
            InitLocals = body.LocalVariablesInitialized,
            LocalSignature = body.LocalSignature.IsNil ? 0 : System.Reflection.Metadata.Ecma335.MetadataTokens.GetToken(body.LocalSignature),
            CodeHex = Convert.ToHexString(body.GetILBytes()!)
        };
    })
}, new JsonSerializerOptions { WriteIndented = true }));
