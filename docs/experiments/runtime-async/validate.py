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
mode.add_argument('--runtime-await', action='store_true', help='Validate compiler-native runtime async lowering')
mode.add_argument('--managed-task', action='store_true', help='Validate retained bodies producing ordinary managed Tasks')
parser.add_argument('--runtime-sample', choices=['language', 'gc', 'fault'], default='language')
args = parser.parse_args()
if args.runtime_sample != 'language' and not args.runtime_await:
    parser.error('--runtime-sample requires --runtime-await')
bundle, runtime, output = args.bundle.resolve(), args.runtime.resolve(), args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
source = Path(__file__).with_name({'language': 'RuntimeAwait.rvn', 'gc': 'RuntimeAwaitGc.rvn', 'fault': 'RuntimeAwaitFault.rvn'}[args.runtime_sample] if args.runtime_await else 'ManagedTask.rvn' if args.managed_task else
                                'AwaitLanguage.rvn' if args.language_await else 'Await.rvn').resolve()
expected = ({'language': 'PASS Raven runtime async language support', 'gc': 'PASS Raven runtime async GC reclamation', 'fault': 'runtime async sample terminal fault'}[args.runtime_sample] if args.runtime_await else
            'PASS managed Tasks through retained runtime bodies' if args.managed_task else
            'PASS ordinary Raven await through runtime suspension adapter' if args.language_await
            else 'PASS runtime-owned operation suspension and resumption')
sources = [source]
if args.runtime_await:
    sources.append(source.with_name("RuntimeAsyncBridge.rvn"))
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
if args.runtime_await:
    compile_command += ["--runtime-async-poc", "RuntimeAwaitProvider.Bridge"]
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
    if args.runtime_await and args.runtime_sample == 'fault':
        report['passed'] = (len(report['commands']) == 2 and result.returncode != 0
                            and 'UserFault: ' + expected in result.stderr
                            and 'UNREACHABLE' not in result.stdout)
    if report['passed'] and args.runtime_await:
        listing = output / 'app.neoil'
        inspection = subprocess.run([str(runtime), 'disassemble', str(assembly), str(listing)],
                                    capture_output=True, text=True, timeout=120)
        report['metadataInspection'] = {'exitCode': inspection.returncode, 'stderr': inspection.stderr}
        if inspection.returncode == 0:
            report['metadataInspection']['stateMachineMentions'] = listing.read_text().count('AsyncStateMachine')
        report['passed'] = (inspection.returncode == 0 and
                            report['metadataInspection']['stateMachineMentions'] == 0)
    if report['passed'] and args.runtime_await and args.runtime_sample == 'language':
        report['rejectedContracts'] = []
        for case in ['missing-helper', 'missing-async-library']:
            rejected_output = output / (case + '.dll')
            rejected_output.unlink(missing_ok=True)
            command = compile_command.copy()
            command[command.index('-o') + 1] = str(rejected_output)
            if case == 'missing-helper':
                command[command.index('--runtime-async-poc') + 1] = 'Missing.RuntimeHelper'
                diagnostic = 'helper type is unavailable'
            else:
                index = command.index('--async-library')
                del command[index:index + 2]
                diagnostic = '--runtime-async-poc requires --async-library'
            rejected = subprocess.run(command, capture_output=True, text=True, timeout=120)
            passed = rejected.returncode != 0 and diagnostic in rejected.stderr and not rejected_output.exists()
            report['rejectedContracts'].append({'case': case, 'passed': passed, 'command': command,
                                                'exitCode': rejected.returncode, 'stderr': rejected.stderr})
            report['passed'] = report['passed'] and passed
finally:
    (output / 'validation.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
raise SystemExit(0 if report['passed'] else 1)
