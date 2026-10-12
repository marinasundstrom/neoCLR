#!/usr/bin/env python3
"""Standalone experimental GC snapshot client; does not launch or rebuild the runtime."""
import argparse
import json
import socket
import sys
import time


def snapshot(endpoint):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
        connection.settimeout(2)
        connection.connect(endpoint)
        connection.sendall(b'NEOCLR/1 SNAPSHOT\n')
        data = bytearray()
        while not data.endswith(b'\n'):
            part = connection.recv(4096 - len(data))
            if not part:
                raise ValueError('incomplete diagnostic response')
            data.extend(part)
            if len(data) >= 4096:
                raise ValueError('oversized diagnostic response')
    row = json.loads(data)
    if row.get('version') != 1 or 'error' in row or row.get('capabilities') != ['gc-snapshot']:
        raise ValueError('unsupported diagnostic response')
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--socket', required=True, help='explicit runtime Unix socket path')
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--interval', type=float, default=1)
    args = parser.parse_args()
    if args.count < 1 or not 0 <= args.interval <= 3600:
        parser.error('count must be positive and interval must be between 0 and 3600 seconds')
    identity = None
    try:
        for index in range(args.count):
            if index:
                time.sleep(args.interval)
            row = snapshot(args.socket)
            current = (row['pid'], row['instance'])
            if identity is not None and current != identity:
                raise ValueError('runtime instance changed during capture')
            identity = current
            print(json.dumps(row, separators=(',', ':')), flush=True)
    except (OSError, ValueError, KeyError) as error:
        print(f'neoclr-profiler: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
