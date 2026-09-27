using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using Mono.Cecil;

// This bounded experiment consumes emitted metadata, never source-text attributes.
// It generates ordinary Raven construction; the VM does not discover attributes.
if (args.Length != 3)
    throw new ArgumentException("Supply schema assembly, core reference and output Raven source.");
var outputPath = Path.GetFullPath(args[2]);
// Do not leave a previously generated parser available after a rejected schema.
if (File.Exists(outputPath)) File.Delete(outputPath);
using var resolver = new DefaultAssemblyResolver();
resolver.AddSearchDirectory(Path.GetDirectoryName(Path.GetFullPath(args[0]))!);
resolver.AddSearchDirectory(Path.GetDirectoryName(Path.GetFullPath(args[1]))!);
using var module = ModuleDefinition.ReadModule(args[0], new ReaderParameters { AssemblyResolver = resolver });
var unions = module.Types.Where(t => t.CustomAttributes.Any(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute")).ToArray();
if (unions.Length != 1) throw new InvalidDataException("Exactly one route union is required.");
var union = unions[0];
if (!union.IsPublic || union.HasGenericParameters || union.IsNested || union.Namespace != "" || !Identifier(union.Name))
    throw new InvalidDataException("Use one public nongeneric route union in the global namespace.");
var declarations = union.CustomAttributes.Where(a => a.AttributeType.FullName == "Raven.Runtime.CompilerServices.RavenUnionCaseAttribute")
    .Select(a => (Name: (string)a.ConstructorArguments[1].Value, Ordinal: (int)a.ConstructorArguments[2].Value))
    .OrderBy(c => c.Ordinal).ToArray();
if (declarations.Length is < 1 or > 16 || declarations.Select(c => c.Ordinal).Where((n, i) => n != i).Any())
    throw new InvalidDataException("Expected 1–16 complete union case declarations.");
var routes = new List<Route>();
foreach (var declaration in declarations)
{
    if (!Identifier(declaration.Name)) throw new InvalidDataException("Unsupported case name.");
    var type = union.NestedTypes.Single(t => t.Name == declaration.Name);
    var markers = type.CustomAttributes.Where(a => a.AttributeType.FullName == "RoutePatternAttribute").ToArray();
    if (markers.Length != 1 || markers[0].ConstructorArguments.Count != 1 || markers[0].HasProperties || markers[0].HasFields
        || markers[0].ConstructorArguments[0].Value is not string pattern)
        throw new InvalidDataException($"{type.Name}: exactly one RoutePattern string attribute is required.");
    var constructors = type.Methods.Where(m => m.IsPublic && m.IsConstructor && !m.IsStatic).ToArray();
    if (constructors.Length != 1) throw new InvalidDataException("Expected one public case constructor.");
    var parameters = constructors[0].Parameters.Select(p => new Parameter(p.Name, p.ParameterType.FullName)).ToArray();
    if (parameters.Any(p => !Identifier(p.Name) || p.Type is not ("System.Int32" or "System.String")))
        throw new InvalidDataException($"{type.Name}: only String and Int32 payloads are supported.");
    var segments = Segments(pattern);
    var names = segments.Where(s => s.Name is not null).Select(s => s.Name!).ToArray();
    if (names.Distinct().Count() != names.Length || !names.Order().SequenceEqual(parameters.Select(p => p.Name).Order()))
        throw new InvalidDataException($"{type.Name}: pattern names must exactly match case payload names.");
    var route = new Route(type.Name, pattern, parameters, segments);
    if (routes.Any(other => Overlap(other.Segments, segments)))
        throw new InvalidDataException($"{type.Name}: ambiguous route patterns are not supported.");
    routes.Add(route);
}
var code = new StringBuilder("// Generated from validated union-case metadata. Do not edit.\nimport System.*\nimport System.Option.*\nimport System.Result.*\nimport System.Web.Http.*\n\n");
code.AppendLine("public union RouteMappingError {\n    case NoMatch\n    case InvalidTarget(reason: string)\n    case InvalidParameter(name: string, expectedType: string)\n    case InvalidConfiguration(reason: string)\n}\n");
var parser = union.Name + "Parser";
code.AppendLine($"public class {parser} {{");
for (var i = 0; i < routes.Count; i++) code.AppendLine($"    private var route{i}: RoutePattern");
code.AppendLine($"\n    private init({string.Join(", ", routes.Select((_, i) => $"pattern{i}: RoutePattern"))}) {{");
for (var i = 0; i < routes.Count; i++) code.AppendLine($"        route{i} = pattern{i}");
code.AppendLine("    }\n");
code.AppendLine($"    static func Create() -> Result<{parser}, RouteMappingError> {{");
for (var i = 0; i < routes.Count; i++)
{
    code.AppendLine($"        let pattern{i} = match RoutePattern.Parse({Quote(routes[i].Pattern)}) {{");
    code.AppendLine("            Ok(let pattern) => pattern\n            Error(let reason) => return Error(RouteMappingError.InvalidConfiguration(reason))\n        }");
}
code.AppendLine($"        return Ok({parser}({string.Join(", ", routes.Select((_, i) => $"pattern{i}"))}))\n    }}\n");
code.AppendLine($"    func Parse(target: string) -> Result<{union.Name}, RouteMappingError> {{");
for (var i = 0; i < routes.Count; i++)
{
    var route = routes[i];
    code.AppendLine($"        let matched{i} = match route{i}.Match(target) {{\n            Ok(let matched) => matched\n            Error(let reason) => return Error(RouteMappingError.InvalidTarget(reason))\n        }}");
    code.AppendLine($"        if let Some(values{i}) = matched{i} {{");
    for (var j = 0; j < route.Parameters.Length; j++)
    {
        var parameter = route.Parameters[j];
        var read = parameter.Type == "System.Int32" ? "GetInt32" : "Get";
        code.AppendLine($"            let value{j} = match values{i}.{read}({Quote(parameter.Name)}) {{\n                Ok(let value) => value\n                Error(_) => return Error(RouteMappingError.InvalidParameter({Quote(parameter.Name)}, {Quote(parameter.Type)}))\n            }}");
    }
    code.AppendLine($"            return Ok({union.Name}.{route.Name}({string.Join(", ", route.Parameters.Select((_, j) => $"value{j}"))}))\n        }}");
}
code.AppendLine("        return Error(RouteMappingError.NoMatch())\n    }\n}");
Directory.CreateDirectory(Path.GetDirectoryName(outputPath)!);
File.WriteAllText(outputPath, code.ToString());
Console.WriteLine($"Generated {parser} from {routes.Count} attributed cases.");

static bool Identifier(string name) => Regex.IsMatch(name, "^[A-Za-z_][A-Za-z0-9_]*$");
static string Quote(string text) => JsonSerializer.Serialize(text, new JsonSerializerOptions { Encoder = System.Text.Encodings.Web.JavaScriptEncoder.UnsafeRelaxedJsonEscaping });
static Segment[] Segments(string pattern)
{
    if (new UTF8Encoding(false, true).GetByteCount(pattern) > 1024 || !pattern.StartsWith('/') || pattern.Contains('?') || pattern.Contains('#'))
        throw new InvalidDataException("Invalid route pattern.");
    var segments = pattern == "/" ? [] : pattern[1..].Split('/');
    if (segments.Length > 16) throw new InvalidDataException("Too many route segments.");
    return segments.Select(segment =>
    {
        if (segment.StartsWith('{'))
        {
            if (!segment.EndsWith('}') || !Identifier(segment[1..^1])) throw new InvalidDataException("Invalid parameter syntax.");
            return new Segment(null, segment[1..^1]);
        }
        if (segment.Contains('{') || segment.Contains('}')) throw new InvalidDataException("Embedded parameters are unsupported.");
        var bytes = new UTF8Encoding(false, true).GetBytes(segment);
        var decoded = new List<byte>();
        for (var i = 0; i < bytes.Length; i++)
        {
            var value = bytes[i];
            if (value == '%')
            {
                if (i + 2 >= bytes.Length || !byte.TryParse(Encoding.ASCII.GetString(bytes, i + 1, 2), System.Globalization.NumberStyles.AllowHexSpecifier, System.Globalization.CultureInfo.InvariantCulture, out value))
                    throw new InvalidDataException("Invalid pattern escape.");
                i += 2;
            }
            if (value is 47 or 92 or < 32 or 127) throw new InvalidDataException("Invalid pattern segment.");
            decoded.Add(value);
        }
        return new Segment(new UTF8Encoding(false, true).GetString(decoded.ToArray()), null);
    }).ToArray();
}
static bool Overlap(Segment[] left, Segment[] right) => left.Length == right.Length && left.Zip(right).All(pair =>
    pair.First.Name is not null ? pair.Second.Name is not null || pair.Second.Literal != "" :
    pair.Second.Name is not null ? pair.First.Literal != "" : pair.First.Literal == pair.Second.Literal);
record Parameter(string Name, string Type);
record Segment(string? Literal, string? Name);
record Route(string Name, string Pattern, Parameter[] Parameters, Segment[] Segments);
