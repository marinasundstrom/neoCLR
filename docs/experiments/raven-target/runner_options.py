"""Select the source bridge or a published bridge, shared by integration checks."""
from pathlib import Path


def add_toolchain_arguments(parser):
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument('--raven', type=Path, help='Built Raven source checkout')
    choice.add_argument('--bridge', type=Path, help='Published Probe.dll; no source checkout required')
    parser.add_argument('--system', type=Path, help='Matching prebuilt target System library')


def runner_arguments(args):
    result = ['--bridge', str(args.bridge.resolve())] if args.bridge else ['--raven', str(args.raven.resolve())]
    if args.system:
        result += ['--system', str(args.system.resolve())]
    return result
