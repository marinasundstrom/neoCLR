#!/usr/bin/env python3
"""Run the managed URI grammar/resolution contract and a recorded .NET comparison."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path)
args = parser.parse_args()
here = Path(__file__).resolve().parent
cases = json.loads((here / 'cases.json').read_text())
bundle = args.toolchain_root.resolve()
quote = lambda value: json.dumps(value, ensure_ascii=True)
source = '''import System.*
import System.Result.*
import System.Collections.*

func Check(value: bool) {
    if !value {
        System.Fault("URI assertion failed")
    }
}

func Parsed(text: string) -> Uri {
    if let Ok(uri) = Uri.Parse(text) {
        return uri
    }
    System.Fault("Expected valid URI")
    return Parsed("")
}

func Resolved(baseUri: Uri, reference: string, expected: string) {
    if let Ok(uri) = baseUri.Resolve(reference) {
        Check(uri.Text == expected)
        Check(uri.IsAbsolute && !uri.IsRelative)
        let same = Parsed(expected)
        Check(uri.Equals(same))
        Check(uri.GetHashCode() == same.GetHashCode())
        let values = ArrayList<Uri>()
        values.Add(uri)
        Check(values[0].Equals(same))
        let comparable: Equatable<Uri> = uri
        Check(comparable.Equals(same))
        let boxed: Object = uri
        Check(boxed.Equals(same))
        Check(boxed.ToString() == expected)
        if let Ok(typed) = baseUri.Resolve(Parsed(reference)) {
            Check(typed.Equals(uri))
        } else {
            Check(false)
        }
    } else {
        System.Fault("Expected resolved URI")
    }
}

func Invalid(text: string) {
    if let Error(error) = Uri.Parse(text) {
        Check(error is UriError.InvalidFormat)
    } else {
        Check(false)
    }
}

func Main() {
'''
source += f'    let baseUri = Parsed({quote(cases["base"])})\n'
for ref, expected in cases['resolution'].items():
    source += f'    Resolved(baseUri, {quote(ref)}, {quote(expected)})\n'
for value in cases['invalid']:
    source += f'    Invalid({quote(value)})\n'
source += '''    Resolved(Parsed("http://a"), "child", "http://a/child")
    Resolved(Parsed("http://a/b/"), "child", "http://a/b/child")
    Resolved(Parsed("http://a/b"), "child", "http://a/child")
    Check(Parsed("").IsRelative)
    Check(Parsed("mailto:a@example.test").IsAbsolute)
    Check(Parsed("//a/path").IsRelative)
    Check(!Parsed("http://A/").Equals(Parsed("http://a/")))
    let value: Object = Parsed("a")
    Check(!value.Equals("a"))
    if let Error(error) = Parsed("relative").Resolve("http://a/") {
        Check(error is UriError.BaseNotAbsolute)
    } else {
        Check(false)
    }
    if let Error(error) = Uri.Parse("http://[::1]/") {
        Check(error is UriError.UnsupportedAuthority)
    } else {
        Check(false)
    }
    Console.WriteLine("URI resolution and invalid grammar passed")
    var longText = "x"
    var index = 0
    while index < 12 {
        longText = String.Concat(longText, longText)
        index += 1
    }
    let maximum = Parsed(longText)
    Check(maximum.Text == longText)
    if let Error(error) = Uri.Parse(String.Concat(longText, "x")) {
        Check(error is UriError.TooLong)
    } else {
        Check(false)
    }
    if let Error(error) = baseUri.Resolve(maximum) {
        Check(error is UriError.TooLong)
    } else {
        Check(false)
    }
    Console.WriteLine("URI grammar, resolution and Object checks passed")
}
'''
with tempfile.TemporaryDirectory(prefix='neoclr-uri-') as folder:
    root = Path(folder)
    (root / 'Main.rvn').write_text(source)
    shutil.copyfile(here / 'Uri.rvnproj', root / 'Uri.rvnproj')
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'Uri.rvnproj'), '-nologo', '-v:minimal'], env=env, capture_output=True, text=True, timeout=150)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'), str(bundle / 'lib/System.neoil'), '256', '10000000'], capture_output=True, text=True, timeout=300)
    assert run.returncode == 0, run.stdout + run.stderr
    assert 'URI grammar, resolution and Object checks passed' in run.stdout, run.stdout
    assert 'live=0' in run.stderr, run.stderr
    print(run.stdout + run.stderr)
    comparison = root / 'reference'
    comparison.mkdir()
    shutil.copyfile(here / 'cases.json', comparison / 'cases.json')
    shutil.copyfile(here / 'Reference.cs', comparison / 'Program.cs')
    (comparison / 'Reference.csproj').write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    build = subprocess.run(['dotnet', 'build', str(comparison / 'Reference.csproj'), '-v:q'], capture_output=True, text=True, timeout=120)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run(['dotnet', str(comparison / 'bin/Debug/net10.0/Reference.dll'), str(here / 'cases.json')], capture_output=True, text=True, timeout=30)
    assert run.returncode == 0, run.stdout + run.stderr
    print(run.stdout)
