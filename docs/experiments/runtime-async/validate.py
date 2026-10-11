#!/usr/bin/env python3
"""Compile and execute the runtime-async POC with an explicit matched bundle/runtime."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', type=Path, required=True)
parser.add_argument('--compiler', type=Path, help='Override bundled rvnc.dll for compiler integration validation')
parser.add_argument('--runtime', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
mode = parser.add_mutually_exclusive_group()
mode.add_argument('--language-await', action='store_true', help='Validate ordinary await via the compatibility adapter')
mode.add_argument('--managed-task', action='store_true', help='Validate retained bodies producing ordinary managed Tasks')
args = parser.parse_args()
bundle, runtime, output = args.bundle.resolve(), args.runtime.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
source = Path(__file__).with_name('ManagedTask.rvn' if args.managed_task else
                                'AwaitLanguage.rvn' if args.language_await else 'Await.rvn').resolve()
expected = ('PASS managed Tasks through retained runtime bodies' if args.managed_task else
            'PASS ordinary Raven await through runtime suspension adapter' if args.language_await
            else 'PASS runtime-owned operation suspension and resumption')
sources = [source]
compiler = args.compiler.resolve() if args.compiler else bundle / 'sdk/tools/rvnc/rvnc.dll'
compiler_inputs = [compiler, *(compiler.parent / name for name in [
    'Raven.CodeAnalysis.dll', 'Raven.CodeAnalysis.NeoClr.dll', 'NeoCLR.Metadata.Experimental.dll'])]
lib = bundle / 'lib'
references = [lib / (name + '.dll') for name in ['System.Runtime', 'System.Data', 'System.Networking', 'System.Web']]
assembly = output / 'app.dll'
# This directory is dedicated to generated POC artifacts.
assembly.unlink(missing_ok=True)
compile_command = ['dotnet', str(compiler), 'neoclr', '--core-reference', str(lib / 'Core.dll'),
                   '--runtime-seed', str(lib / 'System.runtime.neox'), '--bootstrap-intrinsics',
                   '--bootstrap-ownership', str(lib / 'ownership.json'), '--object-library', 'System.Runtime',
                   '--async-library', 'System.Runtime']
for reference in references:
    compile_command += ['--reference', str(reference)]
compile_command += ['-o', str(assembly), *map(str, sources)]
run_command = [str(runtime), 'run', str(assembly), '--system', str(lib / 'System.runtime.neox'),
               '--instructions', '100000000', '--object-root', str(references[0])]
for reference in references:
    run_command += ['--module', str(reference)]
report = {'passed': False, 'commands': [], 'sha256': {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
          for path in [*sources, runtime, *compiler_inputs, lib / 'ownership.json', lib / 'Core.dll', lib / 'System.runtime.neox', *references]}}
try:
    for command in [compile_command, run_command]:
        result = subprocess.run(command, capture_output=True, text=True, timeout=120)
        report['commands'].append({'command': command, 'exitCode': result.returncode,
                                   'stdout': result.stdout, 'stderr': result.stderr})
        if result.returncode:
            break
    report['passed'] = (len(report['commands']) == 2 and result.returncode == 0
                        and result.stdout.strip() == expected
                        and not result.stderr)
finally:
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
raise SystemExit(0 if report['passed'] else 1)
