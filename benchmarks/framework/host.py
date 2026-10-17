"""Host selection for release runtime benchmarks; compilation stays untimed."""
import os
import platform
import subprocess


def target(system=None, machine=None):
    system = platform.system() if system is None else system
    machine = (platform.machine() if machine is None else machine).lower()
    if system == 'Darwin' and machine == 'arm64':
        return {'console': 'console', 'http': 'http', 'executable': 'app'}
    if system == 'Windows' and machine in ('amd64', 'x86_64'):
        return {'console': 'windows-console', 'http': 'windows-http', 'executable': 'app.exe'}
    raise ValueError('Runtime benchmarks require macOS ARM64 or Windows x64')


def cpu_description():
    if platform.system() == 'Darwin':
        return subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip()
    return platform.processor() or os.environ.get('PROCESSOR_IDENTIFIER') or 'unavailable'


def load_average():
    # Windows has no equivalent Unix load average; absence is not zero load.
    return os.getloadavg() if hasattr(os, 'getloadavg') else None


def process_snapshot():
    command = ['tasklist', '/FO', 'CSV'] if platform.system() == 'Windows' else ['ps', '-axo', '%cpu,comm']
    return {'command': command, 'output': subprocess.check_output(command, text=True, errors='replace')}
