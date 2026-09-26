#!/usr/bin/env python3
"""Focused OS/ABI checks; complements, never replaces, canonical full validation."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import time

# Integration tests that exercise OS resources, CLI/process boundaries or native layout.
HOST_TESTS = (
    'cancellation', 'workers', 'environment', 'instant_clock', 'date_time',
    'file_input', 'file_output', 'file_streams', 'path', 'io_errors',
    'console', 'console_streams', 'cli', 'cli_modules', 'source_files',
    'native', 'pinvoke', 'native_integers', 'native_memory_api', 'target_layout',
)
# Private providers and their VM/root/acknowledgement integration live in the library.
HOST_UNITS = (
    'file_io::', 'file_streams::', 'workers::', 'socket_io::',
    'name_resolution::', 'socket_vm_probe::', 'external_io_gc_probe::',
    'scheduler::',
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain', default='stable')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--plan', action='store_true', help='Print selected commands without executing')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    cargo = ['cargo', '+' + args.toolchain]
    commands = [cargo + ['test', '--locked', '--release', '--no-fail-fast'] +
                [item for name in HOST_TESTS for item in ('--test', name)]]
    commands += [cargo + ['test', '--locked', '--release', '--lib', name] for name in HOST_UNITS]
    commands += [cargo + ['run', '--locked', '--release', '--example', name]
                 for name in ('invoke', 'build_native')]
    commands += [cargo + ['run', '--locked', '--release', '--', 'run', 'examples/pinvoke.neoil']]
    if args.plan:
        print(json.dumps(commands, indent=2))
        return
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {'status': 'running', 'scope': 'focused host contracts; not full suite or packaged SDK qualification',
              'platform': platform.platform(), 'toolchain': args.toolchain,
              'createdUtc': datetime.now(timezone.utc).isoformat(),
              'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
              'validator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'integration_tests': list(HOST_TESTS), 'unit_filters': list(HOST_UNITS), 'commands': []}
    try:
        for command in commands:
            print('+ ' + ' '.join(command), flush=True)
            started = time.monotonic()
            result = subprocess.run(command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    text=True, encoding='utf-8')
            (output / ('command-%02d.log' % len(report['commands']))).write_text(result.stdout, encoding='utf-8')
            print(result.stdout, end='', flush=True)
            entry = {'argv': command, 'exit_code': result.returncode,
                     'seconds': round(time.monotonic() - started, 3)}
            report['commands'].append(entry)
            if result.returncode:
                raise RuntimeError('Host check failed: ' + ' '.join(command))
            if '--lib' in command and 'running 0 tests' in result.stdout:
                raise RuntimeError('Host unit filter selected no tests: ' + command[-1])
        report['status'] = 'passed'
    except Exception as error:
        report.update(status='failed', error=str(error))
        raise
    finally:
        (output / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    main()
