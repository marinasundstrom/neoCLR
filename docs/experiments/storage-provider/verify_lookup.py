#!/usr/bin/env python3
"""Run host lookup evidence without modifying the checkout or neoCLR runtime."""
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix="neoclr-storage-lookup-") as folder:
    root = Path(folder)
    subprocess.run(["rustc", "--version"], check=True)
    subprocess.run(["dotnet", "--version"], check=True)
    subprocess.run(["rustc", "--edition=2021", "--test", str(HERE / "LookupProbe.rs"), "-o", str(root / "lookup")], check=True)
    subprocess.run([str(root / "lookup")], check=True)
    shutil.copyfile(HERE / "LookupProbe.cs", root / "Program.cs")
    # Match the installed SDK, while avoiding packages and external dependencies.
    sdk = subprocess.check_output(["dotnet", "--version"], text=True).strip()
    major = int(sdk.split(".")[0])
    (root / "Lookup.csproj").write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup>'
        f'<OutputType>Exe</OutputType><TargetFramework>net{major}.0</TargetFramework>'
        '</PropertyGroup></Project>\n'
    )
    subprocess.run(["dotnet", "run", "--project", str(root / "Lookup.csproj"), "--verbosity", "quiet"], check=True)
