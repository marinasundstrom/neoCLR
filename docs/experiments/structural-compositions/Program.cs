using NeoCLR.Metadata.Experimental;

// Host-side semantic probe. This does not enable native signature admission.
var probe = new MembershipProbe();
var read = probe.Nominal(1);
var seek = probe.Nominal(2);
var both = probe.Nominal(3);
var readOnly = probe.Nominal(4);
var neither = probe.Nominal(5);
var union = new TypeExpression("union", [read, seek]);
var intersection = new TypeExpression("intersection", [read, seek]);
int checks = 0;
void Check(bool condition, string name)
{
    if (!condition) throw new Exception(name);
    checks++;
}
void Reject(Action action, string name)
{
    try { action(); }
    catch (InvalidDataException) { checks++; return; }
    throw new Exception(name);
}

foreach (var kind in new[] { "union", "intersection" })
{
    var original = new TypeExpression(kind, [read, seek]);
    var reordered = new TypeExpression(kind, [seek, read]);
    var nested = new TypeExpression(kind, [read, new TypeExpression(kind, [seek, read])]);
    Check(probe.Identity(original).Equals(probe.Identity(reordered)), $"{kind}: order");
    Check(probe.Identity(original).Equals(probe.Identity(nested)), $"{kind}: flatten/deduplicate");
    var bytes = StructuralSignature.Write(original, new(), allowReferences: true);
    var decoded = StructuralSignature.Read(bytes, allowReferences: true).Root;
    Check(probe.Identity(original).Equals(probe.Identity(decoded)), $"{kind}: wire roundtrip");
}
Check(!probe.Identity(union).Equals(probe.Identity(intersection)), "distinct connectives");
// Existing identity deliberately retains a singleton compound node after deduplication.
Check(!probe.Identity(new("union", [read, read])).Equals(probe.Identity(read)), "singleton identity boundary");
Check(probe.CanAssign(both, intersection), "nominal entry into intersection");
Check(!probe.CanAssign(readOnly, intersection), "partial membership rejected");
Check(probe.CanAssign(intersection, read), "intersection projection");
Check(probe.CanAssign(readOnly, union), "union injection");
Check(!probe.CanAssign(union, read), "union requires narrowing");
Check(probe.CanAssign(intersection, union), "intersection to union");
Check(!probe.CanAssign(neither, union), "unrelated nominal rejected");

var value = new ReadSeek();
Check(ReferenceEquals(value, probe.Narrow(value, intersection)), "same object on entry");
Check(ReferenceEquals(value, probe.Narrow(value, union)), "overlapping union membership");
Check(((IRead)probe.Narrow(value, read)).Read() == 42, "read dispatch");
((ISeek)probe.Narrow(value, seek)).Seek(7);
Check(value.Position == 7, "seek dispatch reaches same object");
Reject(() => probe.Narrow(new ReadOnly(), intersection), "checked narrowing fails");
Reject(() => probe.Narrow(null, union), "null requires future explicit nullable contract");
Reject(() => probe.Narrow(new ShapeOnly(), read), "matching members do not imply nominal conformance");
Reject(() => probe.Identity(new TypeExpression("int32")), "value storage excluded");
Reject(() => probe.Identity(new TypeExpression("nullable", [read])), "nullable excluded");
Reject(() => probe.Identity(probe.Nominal(99)), "unresolved nominal rejected");
Reject(() => probe.Identity(new TypeExpression("intersection", [read])), "invalid wire arity rejected");

// Independent finite-model oracle for soundness, not a completeness claim:
// every accepted static conversion must preserve membership for every witness.
TypeExpression[] expressions = [read, seek, both, readOnly, neither, union, intersection,
    new("union", [both, readOnly]), new("intersection", [union, read])];
object[] witnesses = [value, new ReadOnly(), new SeekOnly(), new Neither(), new ShapeOnly()];
foreach (var source in expressions)
foreach (var target in expressions)
{
    if (!probe.CanAssign(source, target)) continue;
    foreach (var witness in witnesses)
        Check(!probe.Contains(witness, source) || probe.Contains(witness, target), "conversion soundness");
}
// The same definitions may occupy different local reference-table positions.
var assemblyScope = Guid.Parse("a3333333-3333-3333-3333-333333333333");
var moduleScope = Guid.Parse("b4444444-4444-4444-4444-444444444444");
var first = new MetadataReference(assemblyScope, moduleScope, 0x02000001);
var second = new MetadataReference(assemblyScope, moduleScope, 0x02000002);
var foreign = new MetadataReference(Guid.Parse("c5555555-5555-5555-5555-555555555555"), moduleScope, 0x02000001);
var catalog = new Dictionary<MetadataReference, MetadataDefinition>
{
    [first] = new("interface", 0),
    [second] = new("interface", 0),
    [foreign] = new("interface", 0)
};
ResolvedTypeIdentity ResolveAt(TypeExpression expression, params MetadataReference[] references) =>
    StructuralIdentity.Resolve(expression, new(), new ReferenceBindings(references), catalog);
Check(ResolveAt(probe.Nominal(1), first, second).Equals(ResolveAt(probe.Nominal(2), second, first)), "nominal table relocation");
Check(!ResolveAt(probe.Nominal(1), first).Equals(ResolveAt(probe.Nominal(1), foreign)), "assembly scope separates same nominal row");
foreach (var kind in new[] { "union", "intersection" })
{
    var expression = new TypeExpression(kind, [probe.Nominal(1), probe.Nominal(2)]);
    Check(ResolveAt(expression, first, second).Equals(ResolveAt(expression, second, first)), $"{kind}: reference table relocation");
    Check(!ResolveAt(expression, first, second).Equals(ResolveAt(expression, foreign, second)), $"{kind}: scoped constituents");
}

Console.WriteLine($"PASS: {checks} metadata, membership, dispatch and finite-model soundness checks");
Console.WriteLine("Host .NET probe only; native neoCLR storage/verification/codegen remain unimplemented.");

internal sealed class MembershipProbe
{
    private readonly Type[] hostTypes = [typeof(IRead), typeof(ISeek), typeof(ReadSeek), typeof(ReadOnly), typeof(Neither)];
    private readonly ReferenceBindings bindings;
    private readonly Dictionary<MetadataReference, MetadataDefinition> catalog;

    internal MembershipProbe()
    {
        var assembly = Guid.Parse("a1111111-1111-1111-1111-111111111111");
        var module = Guid.Parse("b2222222-2222-2222-2222-222222222222");
        var references = Enumerable.Range(1, hostTypes.Length)
            .Select(i => new MetadataReference(assembly, module, (uint)(0x02000000 + i))).ToArray();
        bindings = new ReferenceBindings(references);
        catalog = references.Select((reference, index) => (reference, index)).ToDictionary(
            item => item.reference,
            item => new MetadataDefinition(hostTypes[item.index].IsInterface ? "interface" : "type", 0));
    }

    internal TypeExpression Nominal(int index) => new("nominal", index: index);

    internal ResolvedTypeIdentity Identity(TypeExpression expression)
    {
        var identity = StructuralIdentity.Resolve(expression, new(), bindings, catalog);
        ValidateSubset(expression);
        return identity;
    }

    private void ValidateSubset(TypeExpression expression)
    {
        if (expression.Kind == "nominal" && expression.Children.Count == 0) return;
        if (expression.Kind is not ("union" or "intersection"))
            throw new InvalidDataException("probe supports only closed nominal reference compositions");
        foreach (var child in expression.Children) ValidateSubset(child);
    }

    internal bool CanAssign(TypeExpression source, TypeExpression target)
    {
        // Validate once at the boundary, including codec depth/node limits.
        var sourceIdentity = Identity(source);
        var targetIdentity = Identity(target);
        return sourceIdentity.Equals(targetIdentity) || Proves(source, target);
    }

    private bool Proves(TypeExpression source, TypeExpression target)
    {
        if (source.Kind == "union") return source.Children.All(child => Proves(child, target));
        if (target.Kind == "intersection") return target.Children.All(child => Proves(source, child));
        if (target.Kind == "union") return target.Children.Any(child => Proves(source, child));
        if (source.Kind == "intersection") return source.Children.Any(child => Proves(child, target));
        return hostTypes[target.Index - 1].IsAssignableFrom(hostTypes[source.Index - 1]);
    }

    internal bool Contains(object? value, TypeExpression expression)
    {
        Identity(expression);
        return value is not null && Matches(value, expression);
    }

    private bool Matches(object value, TypeExpression expression) => expression.Kind switch
    {
        "union" => expression.Children.Any(child => Matches(value, child)),
        "intersection" => expression.Children.All(child => Matches(value, child)),
        _ => hostTypes[expression.Index - 1].IsInstanceOfType(value)
    };

    internal object Narrow(object? value, TypeExpression target) => Contains(value, target)
        ? value! : throw new InvalidDataException("value does not satisfy composition");
}

internal interface IRead { int Read(); }
internal interface ISeek { void Seek(int position); }
internal sealed class ReadSeek : IRead, ISeek
{
    public int Position { get; private set; }
    public int Read() => 42;
    public void Seek(int position) => Position = position;
}
internal sealed class ReadOnly : IRead { public int Read() => 1; }
internal sealed class SeekOnly : ISeek { public void Seek(int position) { } }
internal sealed class Neither { }
internal sealed class ShapeOnly { public int Read() => 42; }
