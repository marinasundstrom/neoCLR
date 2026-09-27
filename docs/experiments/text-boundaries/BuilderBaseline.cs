using System;
using System.Globalization;
using System.Text;

// Semantic comparison only; this is not a .NET/neoCLR timing benchmark.
var builder = new StringBuilder();
builder.Append("name=").Append("e");
var first = builder.ToString();
builder.Append("\u0301").Append('\n').Append("status=ready").Append('\n');
var report = builder.ToString();
if (first != "name=e" || report != "name=e\u0301\nstatus=ready\n")
    throw new Exception("Unexpected snapshot or text");
if (builder.Length != 21 || Encoding.UTF8.GetByteCount(report) != 22
    || StringInfo.ParseCombiningCharacters(report).Length != 20)
    throw new Exception("Unexpected text units");
builder.Clear().Append("replacement");
if (report != "name=e\u0301\nstatus=ready\n" || builder.ToString() != "replacement")
    throw new Exception("Snapshot changed during reuse");
Console.WriteLine($".NET {Environment.Version}: snapshot/reuse passed; UTF-16=21, UTF-8=22, graphemes=20");
