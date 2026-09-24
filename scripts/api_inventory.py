"""Inventory every externally visible type without executing the reference assembly."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def public_types(assembly):
    sources = ROOT / 'tools/api-inventory'
    fingerprint = hashlib.sha256(b''.join(p.read_bytes() for p in
        sorted(sources.glob('*.cs')) + sorted(sources.glob('*.csproj')))).hexdigest()
    output = ROOT / 'target/tools/api-inventory' / fingerprint
    if not (output / '.built').is_file():
        build = subprocess.run([
            'dotnet', 'build', str(sources / 'ApiInventory.csproj'),
            '--output', str(output), '--nologo', '--verbosity', 'quiet',
        ], capture_output=True, text=True)
        if build.returncode:
            raise ValueError('Cannot build public API inventory:\n' + build.stdout + build.stderr)
        (output / '.built').touch()
    return json.loads(subprocess.check_output([
        'dotnet', str(output / 'ApiInventory.dll'), str(assembly),
    ], text=True))
