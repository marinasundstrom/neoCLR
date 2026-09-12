"""Add explicit neoCLR build/run tasks without replacing unrelated VS Code tasks."""
import argparse
import json
from pathlib import Path
import sys


def configure(project, raven, runtime):
    directory = project.parent / '.vscode'
    directory.mkdir(exist_ok=True)
    path = directory / 'tasks.json'
    data = json.loads(path.read_text()) if path.exists() else {'version': '2.0.0', 'tasks': []}
    labels = ('neoCLR: Build saved project', 'neoCLR: Run saved project')
    retained = [task for task in data.get('tasks', []) if task.get('label') not in labels]
    for index, label in enumerate(labels):
        args = [str(Path(__file__).resolve().with_name('run_project.py')), str(project),
                '--raven', str(raven), '--runtime', str(runtime)]
        if index == 0:
            args.append('--build-only')
        retained.append({'label': label, 'type': 'process', 'command': sys.executable,
                        'args': args, 'problemMatcher': [], 'presentation': {'reveal': 'always', 'clear': True}})
    data['tasks'] = retained
    path.write_text(json.dumps(data, indent=2)+'\n')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('project', type=Path)
    parser.add_argument('--raven', required=True, type=Path)
    parser.add_argument('--runtime', required=True, type=Path)
    args = parser.parse_args()
    configure(args.project.resolve(), args.raven.resolve(), args.runtime.resolve())
