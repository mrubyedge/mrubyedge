#!/usr/bin/env python3
"""Publish validated files with Git plumbing; never check out the result branch."""
import argparse
import os
from pathlib import Path
import re
import subprocess

from report import FILES, IMAGE_LIMIT, read_regular

BRANCH = 'refs/heads/benchmark-results'
KEEP_RUNS = 30


def git(state, *args, data=None, env=None):
    return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', *args], cwd=state,
                                   input=data, env=env, stderr=subprocess.PIPE).decode().strip()


def prepare(state, repository, report, run_id, attempt):
    if not re.fullmatch(r'\d+', run_id) or not re.fullmatch(r'\d+', attempt):
        raise ValueError('invalid run identifier')
    state.mkdir(parents=True, exist_ok=False)
    git(state, 'init', '--quiet')
    git(state, 'config', 'user.name', 'github-actions[bot]')
    git(state, 'config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com')
    git(state, 'remote', 'add', 'origin', f'https://github.com/{repository}.git')
    remote = git(state, 'ls-remote', '--heads', 'origin', BRANCH)
    previous = remote.split()[0] if remote else ''
    if previous:
        git(state, 'fetch', '--depth=1', 'origin', BRANCH)
        if git(state, 'rev-parse', 'FETCH_HEAD') != previous:
            raise ValueError('result branch advanced; retry publication')
        git(state, 'read-tree', 'FETCH_HEAD')
    destination = f'runs/{run_id}-{attempt}'
    paths = git(state, 'ls-files', '-z').split('\0')
    pattern = re.compile(r'runs/(\d+)-(\d+)/(results\.json|results\.md|results-time\.png|results-speedup\.png)')
    retained = {destination}
    candidates = {path.rsplit('/', 1)[0] for path in paths if pattern.fullmatch(path)}
    retained.update(sorted(candidates - {destination}, key=lambda p: tuple(map(int, p.split('/')[-1].split('-'))),
                           reverse=True)[:KEEP_RUNS - 1])
    for path in paths:
        if path and (not pattern.fullmatch(path) or path.rsplit('/', 1)[0] not in retained):
            git(state, 'update-index', '--force-remove', '--', path)
    for name in FILES:
        content = read_regular(report / name, IMAGE_LIMIT)
        blob = git(state, 'hash-object', '-w', '--stdin', '--no-filters', data=content)
        git(state, 'update-index', '--add', '--cacheinfo', f'100644,{blob},{destination}/{name}')
    tree = git(state, 'write-tree')
    # A parentless snapshot bounds reachable history as well as the current tree.
    revision = git(state, 'commit-tree', tree,
                   data=(f'Save benchmark charts for run {run_id} ({attempt})\n\n'
                         'Co-authored-by: Codex <codex@openai.com>\n').encode())
    (state / 'revision').write_text(revision)
    (state / 'previous').write_text(previous)
    (state / 'destination').write_text(destination)
    return revision, destination


def push(state):
    revision = (state / 'revision').read_text()
    previous = (state / 'previous').read_text()
    # Credential helper reads the token from this step's environment. It is
    # neither persisted to Git config nor put on the command line.
    env = dict(os.environ, GIT_TERMINAL_PROMPT='0', GIT_CONFIG_COUNT='2',
               GIT_CONFIG_KEY_0='credential.helper', GIT_CONFIG_VALUE_0='',
               GIT_CONFIG_KEY_1='credential.helper',
               GIT_CONFIG_VALUE_1='!f() { if [ "$1" = get ]; then printf "username=x-access-token\\npassword=%s\\n" "$GH_TOKEN"; fi; }; f')
    git(state, 'push', f'--force-with-lease={BRANCH}:{previous}', 'origin', f'{revision}:{BRANCH}', env=env)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['prepare', 'push', 'summary'])
    parser.add_argument('--state', type=Path, required=True)
    parser.add_argument('--report', type=Path)
    parser.add_argument('--repository', required=True)
    args = parser.parse_args()
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', args.repository):
        parser.error('invalid repository')
    if args.operation == 'prepare':
        prepare(args.state, args.repository, args.report, os.environ['GITHUB_RUN_ID'], os.environ['GITHUB_RUN_ATTEMPT'])
    elif args.operation == 'push':
        push(args.state)
    else:
        revision = (args.state / 'revision').read_text()
        destination = (args.state / 'destination').read_text()
        base = f'https://raw.githubusercontent.com/{args.repository}/{revision}/{destination}'
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as summary:
            summary.write('## Native VM benchmark\n\n')
            summary.write(read_regular(args.report / 'results.md', 2 * 1024 * 1024).decode())
            summary.write(f'\n![Execution time]({base}/results-time.png)\n\n![Speedup]({base}/results-speedup.png)\n')
            summary.write(f'\n[Saved report and images](https://github.com/{args.repository}/tree/{revision}/{destination})\n')


if __name__ == '__main__':
    main()
