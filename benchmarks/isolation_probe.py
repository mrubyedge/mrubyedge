"""Fail the smoke check if the container exposes host capabilities."""
import errno
import os
from pathlib import Path
import socket

assert os.getuid() != 0
assert not any(key.startswith(('GITHUB_', 'ACTIONS_')) or key == 'GH_TOKEN' for key in os.environ)
assert not Path('/var/run/docker.sock').exists()
assert not Path('/home/runner').exists()
try:
    Path('/workspace/benchmarks/host-write-probe').write_text('forbidden')
except OSError as error:
    assert error.errno in (errno.EROFS, errno.EACCES)
else:
    raise AssertionError('source mount is writable')
with socket.socket() as connection:
    connection.settimeout(1)
    assert connection.connect_ex(('1.1.1.1', 443)) != 0
assert Path('/sys/fs/cgroup/memory.max').read_text().strip() != 'max'
assert Path('/sys/fs/cgroup/pids.max').read_text().strip() != 'max'
