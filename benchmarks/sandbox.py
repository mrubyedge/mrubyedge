#!/usr/bin/env python3
"""Run dependency acquisition and offline benchmarks in restricted containers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import uuid

import ci
import report
import run

ROOT = Path(__file__).resolve().parents[1]
IMAGE = 'mrubyedge-benchmark-sandbox:local'


def expectation(options):
    available = {p.stem: p for p in sorted((ROOT / 'benchmarks/workloads').glob('*.rb'))}
    cases = list(dict.fromkeys(options.case or available))
    if not cases or set(cases) - available.keys() or len(cases) > 32:
        raise ValueError('unknown or excessive workloads')
    expected = {'versions': [], 'runner_sha256': hashlib.sha256((ROOT / 'benchmarks/runner.rs').read_bytes()).hexdigest(),
                'config': {'samples': options.samples, 'sample_ms': options.sample_ms, 'warmup_ms': options.warmup_ms,
                           'features': options.features, 'default_features': not options.no_default_features,
                           'profile': 'release', 'modes': ['cold', 'warm'] if options.mode == 'both' else [options.mode]},
                'workloads': {}}
    for case in cases:
        data = available[case].read_bytes()
        expected['workloads'][case] = {'expected': int(data.decode().splitlines()[0].removeprefix('# expected: ')),
                                       'sha256': hashlib.sha256(data).hexdigest()}
    for commit in options.ref:
        manifest = tomllib.loads(subprocess.check_output(['git', 'show', f'{commit}:mrubyedge/Cargo.toml'], cwd=ROOT, text=True))
        key = run.dependency_key(manifest, options)
        lock = ROOT / 'benchmarks/locks' / (key + '.lock')
        expected['versions'].append({'commit': commit, 'dependency_key': key,
                                     'lock_sha256': hashlib.sha256(lock.read_bytes()).hexdigest() if lock.is_file() else None})
    return expected


def container_command(name, target, cargo, arguments, online=False):
    return ['docker', 'run', '--rm', '--name', name, '--log-driver', 'none',
            '--user', f'{os.getuid()}:{os.getgid()}', '--read-only', '--cap-drop=ALL',
            '--security-opt=no-new-privileges', '--pids-limit=256', '--memory=6g', '--cpus=2',
            '--ulimit', 'fsize=134217728:134217728', '--network', 'bridge' if online else 'none',
            '--tmpfs', '/tmp:rw,nosuid,nodev,size=1g',
            '--mount', f'type=bind,src={ROOT},dst=/workspace,readonly',
            '--mount', f'type=bind,src={target},dst=/workspace/target',
            '--mount', f'type=bind,src={cargo},dst=/cargo',
            '--env', 'CARGO_HOME=/cargo', '--env', 'HOME=/tmp', '--env', 'PYTHONDONTWRITEBYTECODE=1',
            IMAGE, 'python3', '/workspace/benchmarks/run.py', *arguments]


def execute(command, log):
    # Do not print untrusted stdout as Actions workflow commands. Bound the Docker
    # client's output file too; container fsize limits do not cover its stdout.
    name = command[command.index('--name') + 1]
    try:
        with log.open('wb') as output:
            subprocess.run(['prlimit', '--fsize=8388608:8388608', '--', *command],
                           stdout=output, stderr=subprocess.STDOUT, check=True, timeout=1200)
    finally:
        subprocess.run(['docker', 'rm', '--force', name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--export', type=Path, required=True)
    parser.add_argument('--expected', type=Path, required=True)
    parser.add_argument('--logs', type=Path, required=True)
    parser.add_argument('--smoke', action='store_true')
    args = parser.parse_args()
    try:
        arguments = ci.arguments(os.environ, Path('/workspace/target/benchmark-report/results.json'))[2:]
        if args.smoke:
            arguments += ['--samples', '1', '--sample-ms', '10', '--warmup-ms', '0']
        options = run.parse_options(arguments)
        expected = expectation(options)
        args.expected.write_text(json.dumps(expected))
        reviewed = all(v['lock_sha256'] for v in expected['versions'])
        # These outputs are determined before any untrusted build executes, and
        # live outside the writable container mounts.
        if os.environ.get('GITHUB_OUTPUT'):
            with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
                output.write(f'locks-reviewed={str(reviewed).lower()}\n')
                output.write('expected=' + json.dumps(expected, separators=(',', ':')) + '\n')
        target = ROOT / 'target'
        target.mkdir(exist_ok=True)
        args.logs.mkdir(parents=True, exist_ok=True)
        subprocess.run(['docker', 'build', '--pull', '-t', IMAGE, '-f', str(ROOT / 'benchmarks/Dockerfile'),
                        str(ROOT / 'benchmarks')], check=True, timeout=600)
        with tempfile.TemporaryDirectory(prefix='benchmark-cargo-') as temporary:
            cargo = Path(temporary)
            if args.smoke:
                command = container_command('probe-' + uuid.uuid4().hex, target, cargo, [])
                command = command[:command.index(IMAGE) + 1] + ['python3', '/workspace/benchmarks/isolation_probe.py']
                execute(command, args.logs / 'isolation-probe.log')
            for online, flag in [(True, '--prepare-only'), (False, '--offline')]:
                name = 'benchmark-' + uuid.uuid4().hex
                command = container_command(name, target, cargo, [*arguments, flag], online)
                execute(command, args.logs / ('fetch.log' if online else 'measure.log'))
        report.export_report(target / 'benchmark-report', expected, args.export, images=False)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f'Isolated benchmark failed: {error}\nSee the bounded benchmark logs artifact.\n')
    finally:
        report.collect_logs(ROOT / 'target/benchmarks', args.logs)


if __name__ == '__main__':
    main()
