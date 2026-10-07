#!/usr/bin/env python3
"""Inventory candidate library boundaries in an API-decoded native module.

Read-only planning evidence, not a partitioner or proof of separate compilation.
Use RuntimeAssemblyContainer.Read(PE bytes) to obtain the --native-json input.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path


def cbor_size(value):
    def head(n):
        return 1 if n < 24 else 2 if n < 256 else 3 if n < 65536 else 5 if n < 4294967296 else 9
    if value is None or isinstance(value, bool):
        return 1
    if isinstance(value, int):
        return head(value if value >= 0 else -1 - value)
    if isinstance(value, str):
        length = len(value.encode('utf-8'))
        return head(length) + length
    if isinstance(value, list):
        return head(len(value)) + sum(map(cbor_size, value))
    if isinstance(value, dict):
        return head(len(value)) + sum(cbor_size(k) + cbor_size(v) for k, v in value.items())
    raise ValueError('Unsupported native CBOR value')


def inventory(model):
    types = {t['name']: t for t in model['types']}
    if len(types) != len(model['types']):
        raise ValueError('Expected unique native type identities')
    labels = {}

    def type_label(name, visiting=()):
        if name in visiting:
            raise ValueError('Cyclic declaring-type relationship')
        if name not in labels:
            item = types[name]
            label = item['origin']['name']
            if parent := item.get('declaring_type'):
                if parent['module'] != model['name'] or parent.get('revision') != model.get('revision'):
                    raise ValueError('Expected local declaring-type identity')
                enclosing = model['types'][parent['index']]['name']
                label = type_label(enclosing, (*visiting, name)) + '+' + label
            labels[name] = label
        return labels[name]

    def group(label):
        for namespace in ('System.Data', 'System.Networking', 'System.Web'):
            if label == namespace or label.startswith(namespace + '.'):
                return namespace
        return 'System.Runtime candidate'

    groups = {name: group(type_label(name)) for name in types}
    counts = Counter(groups.values())
    function_counts = Counter()
    bytes_by_group = Counter()
    for name, item in types.items():
        bytes_by_group[groups[name]] += cbor_size(item)
    for function in model['functions']:
        owner = function.get('owner')
        if isinstance(owner, dict):
            owner = owner.get('Named') or owner.get('Constructed', {}).get('definition')
        label = labels.get(owner, function.get('namespace', ''))
        assigned = group(label)
        name = function['name']
        if name in groups and groups[name] != assigned:
            raise ValueError('Overloads assigned to different candidates')
        groups[name] = assigned
        labels[name] = label + '.' + function['origin']['name']
        function_counts[assigned] += 1
        bytes_by_group[assigned] += cbor_size(function)

    edges = defaultdict(set)

    def reference(target, source):
        if target in groups and groups[source] != groups[target]:
            edges[(groups[source], groups[target])].add((labels[source], labels[target]))

    def scan(value, source):
        if isinstance(value, dict):
            # Exact native nominal/function identities; no source import guessing.
            for key in ('Named', 'definition', 'name'):
                if isinstance(value.get(key), str):
                    reference(value[key], source)
            for key, child in value.items():
                if key not in ('origin', 'parameter_names', 'local_names', 'generic_parameters'):
                    scan(child, source)
        elif isinstance(value, list):
            for child in value:
                scan(child, source)

    for item in [*model['types'], *model['functions']]:
        scan(item, item['name'])
    return {
        'scope': 'Candidate namespace partition; nested types follow their declaring owner. Local metadata/signature/body references only. Imported seed/bootstrap dependencies and final native-adapter ownership require separate review. Not a proven build DAG.',
        'types': dict(counts), 'functions': dict(function_counts),
        'declarationCborBytes': dict(bytes_by_group),
        'edges': [dict(source=a, target=b, relationships=len(pairs), examples=sorted(pairs)[:12])
                  for (a, b), pairs in sorted(edges.items())],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native-json', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    data = args.native_json.read_bytes()
    model = json.loads(data)
    report = inventory(model)
    report.update(input=str(args.native_json.resolve()), sha256=hashlib.sha256(data).hexdigest(),
                  nativeJsonBytes=len(data), nativeCborBytes=cbor_size(model))
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ('types', 'functions', 'declarationCborBytes')}, indent=2))


if __name__ == '__main__':
    main()
