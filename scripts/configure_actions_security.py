#!/usr/bin/env python3
"""Apply repository controls for benchmark publication; preview unless --apply."""
import argparse
import base64
import json
import re
import subprocess


def api(path, method='GET', payload=None):
    command = ['gh', 'api', path, '--method', method]
    if payload is not None:
        command += ['--input', '-']
    data = subprocess.check_output(command, input=json.dumps(payload).encode() if payload is not None else None)
    return json.loads(data) if data else None


def require_pinned_default_workflows(repository):
    files = api(f'repos/{repository}/contents/.github/workflows?ref=master')
    for file in files:
        if not file['name'].endswith(('.yaml', '.yml')):
            continue
        content = api(f'repos/{repository}/contents/{file["path"]}?ref=master')
        text = base64.b64decode(content['content']).decode()
        for action in re.findall(r'uses:\s*([^\s#]+)', text):
            if not action.startswith('./') and not re.fullmatch(r'[^@]+@[0-9a-f]{40}', action):
                raise ValueError(f'merge SHA pins before enabling repository enforcement: {file["name"]}: {action}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repository', default='mrubyedge/mrubyedge')
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--enable-sha-pinning', action='store_true', help='requires all default-branch workflows already pinned')
    args = parser.parse_args()
    repository = args.repository
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        parser.error('invalid repository')
    rulesets = [
        {'name': 'Protect default branch', 'target': 'branch', 'enforcement': 'active',
         'bypass_actors': [{'actor_id': 5, 'actor_type': 'RepositoryRole', 'bypass_mode': 'pull_request'}],
         'conditions': {'ref_name': {'include': ['refs/heads/master'], 'exclude': []}},
         'rules': [{'type': 'deletion'}, {'type': 'non_fast_forward'},
                   {'type': 'pull_request', 'parameters': {
                       'required_approving_review_count': 1, 'dismiss_stale_reviews_on_push': True,
                       'require_code_owner_review': True, 'require_last_push_approval': False,
                       'required_review_thread_resolution': True}}]},
        {'name': 'Protect release tags', 'target': 'tag', 'enforcement': 'active',
         'bypass_actors': [{'actor_id': 5, 'actor_type': 'RepositoryRole', 'bypass_mode': 'always'}],
         'conditions': {'ref_name': {'include': ['refs/tags/v*'], 'exclude': []}},
         'rules': [{'type': 'deletion'}, {'type': 'update'}, {'type': 'non_fast_forward'}]},
    ]
    environment = {'deployment_branch_policy': {'protected_branches': False, 'custom_branch_policies': True}}
    print(json.dumps({'rulesets': rulesets, 'benchmark-publication': environment,
                      'sha_pinning_required': args.enable_sha_pinning}, indent=2))
    if not args.apply:
        return
    if args.enable_sha_pinning:
        require_pinned_default_workflows(repository)
    existing = api(f'repos/{repository}/rulesets')
    for rule in rulesets:
        match = next((r for r in existing if r['name'] == rule['name']), None)
        endpoint = f'repos/{repository}/rulesets' + (f'/{match["id"]}' if match else '')
        saved = api(endpoint, 'PUT' if match else 'POST', rule)
        print(f'Applied ruleset: {saved["name"]} ({saved["id"]})')
    endpoint = f'repos/{repository}/environments/benchmark-publication'
    api(endpoint, 'PUT', environment)
    policies = api(endpoint + '/deployment-branch-policies')['branch_policies']
    for policy in policies:
        if policy['name'] != 'master' or policy.get('type', 'branch') != 'branch':
            api(endpoint + f'/deployment-branch-policies/{policy["id"]}', 'DELETE')
    if not any(p['name'] == 'master' and p.get('type', 'branch') == 'branch' for p in policies):
        api(endpoint + '/deployment-branch-policies', 'POST', {'name': 'master', 'type': 'branch'})
    print('Applied Environment branch restriction: master only')
    if args.enable_sha_pinning:
        current = api(f'repos/{repository}/actions/permissions')
        payload = {key: current[key] for key in ('enabled', 'allowed_actions')}
        payload['sha_pinning_required'] = True
        api(f'repos/{repository}/actions/permissions', 'PUT', payload)
        print('Enabled repository-wide SHA pinning')
    else:
        print('SHA enforcement deferred until the pinned workflows are merged into master.')


if __name__ == '__main__':
    main()
