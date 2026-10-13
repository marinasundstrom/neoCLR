#!/usr/bin/env python3
"""Verify native gzip output with independent Python/zlib and .NET decoders."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--bundle', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
def run(command):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=300)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result.stdout
run(['python3', 'scripts/build-native-project.py', '--project', Path(__file__).with_name('Interop.rvnproj'), '--bundle', a.bundle.resolve(), '--aot', a.bundle.resolve()/'bin/neoclr-codegen', '--output', out/'native', '--profile', 'http', '--http-heap-bytes', '33554432'])
encoded = bytes(int(line) for line in run([out/'native/app']).splitlines())
(out/'output.gz').write_bytes(encoded)
data = bytes((i*17+i//251)%256 for i in range(40000))
expected = data[7:39993] + data
assert gzip.decompress(encoded) == expected
(out/'expected.bin').write_bytes(expected)
comparison = out/'dotnet'
comparison.mkdir()
(comparison/'Compare.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
(comparison/'Program.cs').write_text('''using System.IO.Compression;
using var input = File.OpenRead(args[0]);
using var gzip = new GZipStream(input, CompressionMode.Decompress);
using var output = new MemoryStream();
gzip.CopyTo(output);
if (!output.ToArray().SequenceEqual(File.ReadAllBytes(args[1]))) throw new Exception("Gzip mismatch");
Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
''')
version = run(['dotnet', 'run', '--project', comparison/'Compare.csproj', '--', out/'output.gz', out/'expected.bin']).strip()
report = dict(passed=True, sourceBytes=len(expected), compressedBytes=len(encoded), gzipSha256=hashlib.sha256(encoded).hexdigest(), dotnet=version, pythonZlib=True, nativeBuild=json.loads((out/'native/build.json').read_text()))
(out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k != 'nativeBuild'},indent=2))
